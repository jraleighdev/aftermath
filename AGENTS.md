# AGENTS.md

Top-down shooter in Rust + Bevy, built slice by slice — architecture is decided per-slice with the user, not designed up front. Don't introduce abstractions the current slice doesn't need.

## Commands

- `cargo run` — play the game (window "Aftermath"; WASD or arrow keys)
- `cargo test` — unit tests; only this crate recompiles, takes seconds
- A first clean build compiles all of Bevy (~10+ min). Don't be surprised; later builds are fast.

## Toolchain

- Bevy is pinned at 0.20, which requires rustc 1.97.1+. Toolchain is rustup-managed; a `rust-version` build error means running `rustup update stable`.
- Bevy changes API between minor versions. When unsure of an API, trust the examples at the matching tag — https://github.com/bevyengine/bevy/tree/v0.20.0/examples — not `main`, not older blog posts.

## Conventions

- `src/main.rs` is app wiring only (plugins, system registration). Each game domain is its own module: component + systems + a `#[cfg(test)] mod tests` at the bottom (`src/player.rs` is the template; shooting/enemies should follow it).
- Systems are tested headlessly: build a `World`, insert `Time` (use `advance_by`) and `ButtonInput<KeyCode>` resources, run the system through a `Schedule`, assert on components. No app, no window.
- Gameplay tuning values are consts at the top of their module (`player::SPEED`).

## Gotchas

- OpenCode server restarts kill background shells mid-build; compiled artifacts in `target/` survive, so just rerun the command.
