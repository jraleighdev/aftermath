# Collision design: bullet-enemy and player-enemy (2026-10-25)

## Intent

Bullets and enemies currently ignore each other and the player. This spec
covers the two interaction slices that make combat real: shooting enemies,
and enemies reaching the player. Written up because we're moving from
independent systems into cross-entity state, where slice boundaries matter.

## Approach

Physics-free distance checks, circles only. Each entity gets a collision
radius const in its module (`player::RADIUS = 50.0` square half-size,
`enemies::RADIUS = 30.0` for the 60px sprite, `shoot::RADIUS = 6.0` for the
12px bullet). A per-frame pairwise distance system flags events; no
collider components, no collision crate (bevy_xpbd etc. would also drag in
broadphase we don't need for a handful of entities).

Chosen because: entity counts are small (<100), the game is 2D circles-in-
spite-of-squares, and repo convention is headless-testable systems, which
manual distance math gives us trivially (spawn entities at known positions,
advance time, assert).

Rejected: an HP component now (nothing needs multi-hit yet — add when a
second enemy type or weapon exists), an external physics crate (big dep for
`translation.distance() < a + b`).

## Slice 1 — bullet hits enemy (bead: bullet-enemy collision)

System in `shoot.rs` (or a small `collision.rs` if player-enemy wants to
live elsewhere): for each bullet, if any enemy exists within `RADIUS_BULLET
+ RADIUS_ENEMY` of it, despawn both. One test: bullet on enemy path at the
right distance kills both; one test: bullet 100px away kills neither;
one test: bullets to enemies ratio hygiene (a bullet only dies once).

## Slice 2 — enemy pushes player (bead: enemy-player contact)

System: if a player and enemy are within `RADIUS_PLAYER + RADIUS_ENEMY`,
the player is knocked back along the (player − enemy) direction, e.g.
`CONTACT_SPEED = 400` px/s impulse *per frame of contact* — continuous
shove, not a one-shot impulse, so overlap resolves naturally and being
surrounded hurts without any damage model. Tests: contact shoves the
player away, no contact no shove, shove direction respects geometry
(enemy right of player → player moves left).

## Non-goals

- No HP, damage numbers, or death for the player — later slice, filed when
  these feel wrong in play.
- No enemy repulsion/crowd dynamics; enemies may stack.
- No collision group filtering; entities are the groups.
- No effects: hit flashes, particles, sound — later polish slices.

## Left to the builder (the per-slice TDD cycle owns these)

- Exact system ordering vs existing systems (moved after chase, despawn
  after collision).
- Whether the collision system lives in `shoot.rs`, `enemies.rs`, or a new
  `collision.rs` (lean: player-enemy in `enemies.rs`, bullet-enemy next to
  `move_bullets` in `shoot.rs`; split into `collision.rs` only if shared
  helpers appear).
- Knockback strength tuning values (consts, tune during playtest).
