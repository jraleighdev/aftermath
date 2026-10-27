# Player Sprite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the player's green square with a 16×16 pixel-art player (3 facings: down, up, side) drawn in a dark gritty GB-Zelda style, with mouse aim quantized to choose the facing.

**Architecture:** Draw 3 idle facings as separate exported PNGs (`player_down.png`, `player_up.png`, `player_side.png`) from one aseprite source. A new `player::Facing` component + `player::update_facing` system quantizes the mouse direction to 4 directions and swaps the `Sprite`'s image (side-left mirrors side-right). `player::aim`'s rotation drops per-facings.

**Tech Stack:** Rust + Bevy 0.20, aseprite MCP tools for art.

**Spec:** docs/superpowers/plans/2026-10-26-player-sprite.md (approved short design inline: 16×16 @ ×6 scale, 3 facings, dark muted palette with near-black outline, no animation, no enemy/bullet/background sprites).

## Global Constraints

- Sprites live at `assets/sprites/<name>.aseprite` with exported `<name>.png` beside them; Bevy loads the PNG.
- All aseprite tool calls use absolute paths.
- 16×16 pixel canvas per facing, ×6 scale on-screen.
- Dark gritty palette: 3–4 ramp steps per material + near-black outline; no brights, no dithering on 16px.
- Headless tests build a `World`, insert `Time`/`ButtonInput`, run systems through a `Schedule`.

## Review Focus

1. Side-left facing must use flipped UVX, not a rotation or the up/down image. Test: `update_facing` test asserting the flipped variant for left cursor / mirrored check.
2. Diagonal cursor positions pick *some* correct facing (e.g. upper-right cursor → up or side, never down). Test: corner-cursor test.
3. No cursor on screen (window without cursor position) → facing unchanged, no panic. Test: `None` cursor leaves prior facing.
4. Missing `assets/sprites/player_*.png` at startup → game fails loudly at asset load, not silently blank. Check: run `cargo run` once with assets present (manual smoke).
5. Sprite scale must keep the player's on-screen size ≈96–100px; the collision radius consts stay unchanged this pass. Check in wiring task.

---

### Task 1: Draw the three player facings

**Files:**
- Create: `assets/sprites/player.aseprite` (3 frames: down, up, side)
- Create: `assets/sprites/player_down.png`, `player_up.png`, `player_side.png` (16×16 each)

**Interfaces:**
- Produces: three 16×16 PNGs on disk, readable by Bevy in Task 3.

- [ ] **Step 1: Plan the palette and silhouette** — build a dark muted ramp (e.g. 4 steps from near-black outline to a muted highlight) + one mid-tone skin/cloth accent; 8–16 colors total. Character centered with 1–2px margin, standing, feet on the bottom row.
- [ ] **Step 2: Draw facing frames** in Aseprite via aseprite MCP tools: silhouette first (draw_rows for a 16px figure is per-pixel work; do not build humans from ellipse primitives), then flats, then top-left light shading, then near-black outline. Three frames in one sprite: down, up, side (side = facing right; left is code-mirrored).
- [ ] **Step 3: Verify** with `view_sprite` + `lint_sprite` + `get_color_stats` (strays, detached parts, palette discipline). Fix what's flagged.
- [ ] **Step 4: Export** `export_frame` each frame to its PNG at 1× (16×16) beside the source. Confirm files exist on disk.

### Task 2: Facing component + system (TDD)

**Files:**
- Modify: `src/player.rs` (add `Facing` enum, `Facing` component, `update_facing` system; delete rotation from `aim`)

**Interfaces:**
- Produces: `pub enum Facing` with `Down|Up|Side` variants; a `#[derive(Component)] pub struct PlayerFacing(pub Facing)` component; system `pub fn update_facing(mut player: Single<(&mut PlayerFacing, &Transform), With<Player>>, camera: Single<(&Camera, &GlobalTransform)>, window: Single<&Window>)` reusing `aim`'s cursor→world math, setting `facing.0 = Down|Up|Side` for cursor below/above/beside the player (larger of `abs(x)` vs `abs(y)` on the cursor→player vector wins). Flip/image-swap wiring happens only in Task 3; Task 2 tests assert the enum value only.
- Consumes: `aim`'s existing cursor→world math (reuse it; extract nothing new unless the compiler demands).

- [ ] **Step 1: Write failing tests** in `player.rs` tests module, reusing `aim_world`'s window/camera fixture: (a) cursor below player → `Facing(Down)`; (b) cursor above → `Facing(Up)`; (c) cursor right → `Facing(Side)`; (d) cursor left → `Facing(Side)`; (e) no cursor → facing unchanged (pre-set `Facing(Up)` stays `Up`).
- [ ] **Step 2: Run** `cargo test player` → expect FAIL (Facing not defined).
- [ ] **Step 3: Implement** enum, component, `update_facing`; quantize the cursor→player vector by comparing `abs(x)` vs `abs(y)` (larger wins; y>0 = down screen-space); remove `aim` (replaced by `update_facing`).
- [ ] **Step 4: Run** `cargo test player` → PASS.
- [ ] **Step 5: Update `src/main.rs`**: replace `player::aim` registration with `player::update_facing`.
- [ ] **Step 6: Commit** `git commit -m "feat: player facing system with quantized mouse aim"`.

### Task 3: Wire the sprite

**Files:**
- Modify: `src/main.rs` (`setup`: load three PNGs, spawn player with `Sprite` image + `Transform::from_scale(Vec3::splat(6.0))` + `Facing`)
- Modify: `src/player.rs` (`update_facing`: also set flip / image swap)

**Interfaces:**
- Consumes: Task 1's PNGs, Task 2's `Facing` enum/component.

- [ ] **Step 1: Implement wiring.** Exact mechanics the implementer writes: extend `update_facing` to (a) on reaching `Side` with cursor left of the player, set `sprite.flip_x = true`, `false` otherwise; (b) exchange the `Sprite`'s image between the three loaded `Handle<Image>`s. Player spawn: `Sprite::from_image(down_handle)` scaled ×6, plus `Facing(PlayerFacing::Down)`. Anchor center, camera-centered start as today.
- [ ] **Step 2: Manual smoke — `cargo run`**: player shows the down sprite scaled ×6, mouse around the player flips between down/up/side(+mirror), no stretch except uniform ×6. Capture findings as new beads if wrong.
- [ ] **Step 3: Full `cargo test`** — all suites pass.
- [ ] **Step 4: Commit** `git commit -m "feat: swap player square for gritty GB-scale sprite"`.

---

## Self-Review notes

- Spec coverage: facings ✅ (Tasks 1–2), aim quantization ✅ (Task 2), sprite swap ✅ (Task 3), palette/style ✅ (Task 1). No walk animation — intentionally out (file a bead if wanted later).
- Exact names pinned: `Facing`, `update_facing`, three PNG filenames.
- Proportion: plan ≈ spec size; art steps are checkable but leave drawing judgment to the implementer (aseprite MCP guidance covers quality).
