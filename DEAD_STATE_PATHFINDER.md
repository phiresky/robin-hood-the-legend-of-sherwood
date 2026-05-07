# Dead-state field audit (pathfinder / movement / sectors)

Scope: `crates/robin_engine/src/{pathfinder,fast_find_grid,sector,level_data,position_interface,geo2d,markers,sequence,order,element}.rs` and `engine/{level_loading,movement,jump,door_pass,refresh_seek,sector_motion,anti_collision,tick,…}.rs`.

**Three-stage audit:**
1. Rust-only — find fields written-only or never-written-but-read.
2. Cross-check with C++ original (`./original-code/`).
3. For each surviving "port gap" — investigate whether the C++ semantics are needed in the Rust architecture, or whether the Rust port replaced them with equivalent machinery.

---

## Phase 1 — Confirmed dead (DELETED on this branch)

5 fields, ~129 lines removed across 9 files:

| # | Field | Why dead |
|---|---|---|
| 1 | `GridLine::sound_material_sector_idx` | Rust-only field, "currently unused; reserved for future"; sound-line back-pointer is also unused in C++ |
| 2 | `GridSector::lift_motion_area` | No C++ counterpart — lifts inherit motion area in C++ |
| 3 | `Order::fly` / `AiOrderIntent::fly` | C++ `bHeyICanFly` only init/copy/serialize, never read — dead in C++ too |
| 4 | `LiftRuntimeState::occupants_pc` (+ `LiftData::occupants_pc`, `is_occupied_by_pc`, `is_pc` on `ActiveLiftClimb`) | C++ `IsOccupiedByPC()` defined but **zero callers** in C++ |
| 5 | `PendingDoorPass` struct + `ActorData::pending_door_pass` | No C++ counterpart; replaced in Rust by `ActiveDoorPass` |

---

## Phase 2 — Port-gap investigation

Each of the following looked dead Rust-only but had a live C++ counterpart. After detailed C++ comparison the question is: **do we need to port the missing machinery, or has the Rust architecture already replaced it?**

### NPG1. `PathFinderRuntime::ignore_next_path`  →  **REMOVE**

**C++ semantics:** flag for an asynchronous, multi-threaded pathfinder. `mbIgnoreNextPath` is set by `RHPathFinder::CancelPathRequest` / `RestartPathRequests` (RHpathfinder.cpp:551, 618 etc.) when a request must be cancelled while the worker thread is mid-compute. It's read at `ProcessPathRequests` (RHpathfinder.cpp:733) when the worker returns and decides whether to discard the result. The cancel callers are `RHSequenceElementMovement::MaybeCancelPathRequest` (RHSequenceElementMovement.cpp:489) — fires when a sequence element transitions out of `RHCOMMAND_MOVE_WAITING`.

**Rust architecture:** **pathfinding is synchronous.** From `tick.rs:1077-1081`:

> *"Pathfinding is synchronous now — Move sequence elements call `find_path` directly when their `InstructOwner` action dispatches. The legacy async `ProcessPathRequests` drain had no remaining producers post-refactor and was deleted."*

There is no async queue, no worker thread, no "next path" to ignore. A path is computed synchronously inside the dispatching call and returned. A halted move never has a queued result that needs discarding because it never queued one in the first place.

**Verdict:** the field has no architectural place in the Rust port. Remove it and the unreachable early-return in `find_path_nodes` (pathfinder.rs:1208-1211).

---

### NPG2. `GridSector::gate_directions`  →  **REMOVE**

**C++ semantics:** parallel `Vec<bool>` to `mlistGates` on `RHSectorMotionArea`. Read by `GetGate(uwIndex, bDirection)` (RHSector.h:198) which returns the direction via out-param. Used in `RHFastFindGrid::FindPathGates` (RHfastfindgrid.cpp:9212) — the cross-sector A* graph search. The `bDirect` boolean disambiguates whether the source sector is on the gate's "in" or "out" side, which the search needs because gate distance/score asymmetrically uses `GetPointIn()` vs `GetPointOut()` (RHfastfindgrid.cpp:9218-9235).

**Rust architecture:** the cross-sector search **is ported** as `gate::find_path_gates` (gate.rs:1129). But it computes `direct` on-the-fly from the door geometry rather than a cached array:

```rust
// gate.rs:1175-1187
if door.sector_out == source_sector {
    direct = true;  from_pt = door.point_out;  to_pt = door.point_in;
} else if door.sector_in == source_sector {
    direct = false; from_pt = door.point_in;   to_pt = door.point_out;
}
```

This is **architecturally equivalent** to the C++ — both produce the same `direct` value — but the Rust port iterates the door table directly rather than caching directions per-sector. The `gate_directions` Vec on `GridSector` has no consumer because the equivalent semantics live in `find_path_gates` itself.

**Verdict:** the cached array is redundant; the equivalent is already implemented inline in the consumer. Remove the field, the push at level_loading.rs:4780, and the 9 `Vec::new()` initializers.

---

### NPG3. `GridSector::highest_door_index`  →  **REMOVE (demote to local)**

**C++ semantics:** `RHSectorLift::mpHighestDoor` / `mpLowestDoor` are pointers to the highest/lowest door of a lift sector by Y-coordinate (screen coords). Set during proto-load (RHsector.cpp:1517). Accessors:
- `GetHighEntryPoint()` / `GetHighExitPoint()` / `GetHighExitDirection()` / `GetHighSector()` / `GetHighLayer()` (RHSector.h:317-327)

Production callers:
- `RHelementactorhuman.cpp:12875-12879` — `ForecastDestinationForIA` reads `GetHighSector / GetHighLayer / GetHighEntryPoint / GetHighExitDirection` to predict where a lift-traversing actor is heading.
- `RHelementactor.cpp:4195` — `GetHighExitPoint()` for door-pass post-handling.

The only `mpHighestDoor` direct read in the production C++ source is wrapped in `#ifdef _DEBUG` *inside* a `/* … */` comment block (RHsector.cpp:1533-1558) — a debug assertion that's been commented out.

**Rust architecture:**
- `ForecastDestinationForIA` is ported as `ai::forecast_destination_for_ia` (ai.rs:632), which uses `find_lift_exit_door(lift_sector, moving_upwards, doors)` (ai.rs:742) to scan the door table by `DoorType::LiftHigh | LiftHighCrenel` for "high" or `DoorType::LiftLow` for "low". This **replaces** the cached pointer with a door-type scan and is the architectural equivalent.
- `GetHighExitPoint` is ported as the `high_exit_point: Option<Point2D>` cache on `GridSector` (fast_find_grid.rs), pre-resolved at level load. Same for `low_exit_point`.
- `highest_door_index` itself is **only read inside its own build loop** at level_loading.rs:5099-5106 as the "current best" accumulator while computing `high_exit_point`.

**Verdict:** the runtime consumers have all been re-implemented via `find_lift_exit_door` and the pre-resolved exit points. The remaining `highest_door_index` field is just a build-time accumulator that should be a local variable.

(Asymmetry note: `lowest_door_index` IS read externally at engine/melee/effects.rs:33-43 to grab `door.point_out` and `door.layer_out` for ladder-wall fall destinations. That use is also replaceable with `find_lift_exit_door(lift_sector, false, doors)` in a follow-up — but for this audit we leave `lowest_door_index` alone since it has a real consumer.)

Cleanup: refactor the `for (door_idx, door) in game_host.doors.iter().enumerate()` loop at level_loading.rs:5077-5108 to track `current_highest: Option<(u32, f32)>` as a local instead of writing to `gs.highest_door_index`. Then drop the field.

---

### NPG4. `Order::done` / `AiOrderIntent::done`  →  **PORT THE WRITE**

**C++ semantics:** real flag with both writer and reader.
- **Set** at `RHelementactor.cpp:691` — `mpOrder->bDone = true` when `mmotionState == RHMOTION_DONE`. Fires from the per-actor `Hourglass()` once the sprite advances past the action's hit/done frame.
- **Read** at `RHsequenceelement.cpp:590` — guard at the top of `Postpone()`: if `mlistOrders.GetLast()->bDone`, skip the postpone and short-circuit the element to `RHSEQ_TERMINATED`. This handles the 1-frame race where the action's done-frame fires on the same tick a postpone request arrives.

**Rust architecture:**
- The **read** site is **already ported** at engine/mod.rs:2249:
  ```rust
  e.command != Command::MoveOk && e.orders.back().is_some_and(|o| o.done)
  ```
  — exactly the same shape as the C++ guard, inside `engine_postpone`.
- The **write** site is **missing**. No production code sets `order.done = true`.

The Rust sprite tick observes `MotionState::Done` events in many places (engine/movement.rs:3843, 6632, 6650, etc. — for door-pass, melee, transition completion side-effects), but the propagation to the actor's current `Order::done` is not wired.

**Practical impact:** small. The guard catches a 1-tick window where:
- An actor's current action has reached its hit/done frame this tick.
- A new sequence element arbitrates `Postpone` against the actor's current element on the same tick.
- The Rust port currently postpones; the parity behaviour would short-circuit-terminate.

This race is rare and the symptom is subtle (an extra cross-postpone link instead of a clean termination), but is a real C++ → Rust parity gap.

**Verdict:** keep the field; port the writer. Implementation sketch:

1. Find the per-actor sprite-advance site that reports `MotionState::Done` for the actor's current movement order. Likely candidate: the inner loop of `tick_movement` / `tick_per_actor_sprite` in `engine/tick.rs` or `engine/movement.rs` — wherever the per-actor sprite tick is consumed and the resulting `motion_state` is matched.
2. Where Rust currently does effect dispatch on `MotionState::Done` (e.g. movement.rs:3843 for the door-pass climb-up done frame), additionally write `order.done = true` on the actor's active sequence element's front order (the one driving the current animation).
3. Confirm the read site at engine/mod.rs:2249 fires correctly under a synthetic test that arbitrates a postpone in the same tick the done-frame plays.

Alternative if the writer is hard to land cleanly: leave the field with a `// TODO(parity): not yet wired — see RHelementactor.cpp:691 / RHsequenceelement.cpp:590` next to the read at engine/mod.rs:2249 so the gap is documented.

---

### NPG5. `Order::target_z` / `AiOrderIntent::target_z`  →  **REMOVE**

**C++ semantics:** `pointDestination3D.Z` on `RHOrder`. Set in jump and projectile setup paths (RHelementactorpc.cpp:7847, 7887, 7968, 8020 — jump-trajectory math). Read in the look-ahead orders loop at RHsprite.cpp:1819 (`pNextOrder->pointDestination3D.IsNonZero() → return RHMOTION_TERMINATED`) — terminates the current animation when the next queued order is a flight (3D destination) so a fresh flight order can launch.

**Rust architecture:** jumps and flights are **re-architected onto separate state machines** that don't use `Order` at all:
- Jumps: `engine::jump::ActiveJump` + `JumpStep` (engine/jump.rs:60, 97). Each step has its own `Option<Point3D>` for the 3D destination.
- Flights (push, ladder-wall fall, hit fall): `element::ActiveFlight` (element.rs:609) — its own per-frame increment state.

`Order` in Rust is 2D-only by design — `target_x` / `target_y`. There is no production path that ever writes `Order::target_z`. The look-ahead at engine/movement.rs:4824 (`else if k == 1 && nxt.target_z != 0.0 { early_terminate = true; }`) was preserved structurally during the port but its branch is unreachable under the new architecture.

**Verdict:** remove `target_z` from both `Order` and `AiOrderIntent`, and drop the early-terminate branch in the look-ahead loop (engine/movement.rs:4824-4827) plus its corresponding bullet in the comment at engine/movement.rs:4798. Update the comment to reflect the 2D-only invariant.

If a future port of legacy projectile/jump-via-Order code is contemplated, the field can be re-added with its writers at that time. Right now it's a hook for code that doesn't exist and a parity-with-C++ argument for a field whose semantics live elsewhere.

---

### NPG6. `Order::fast` / `AiOrderIntent::fast`  →  **REMOVE**

**C++ semantics:** not a flag — a separate command discriminant. C++ has both `RHCOMMAND_TURN` and `RHCOMMAND_TURN_FAST` (RHCommand.h:42). The AI selects which to emit (`RHartificialintelligence.cpp:2728`: `pSequenceElement = new RHSequenceElementGeneric(1, bFast ? RHCOMMAND_TURN_FAST : RHCOMMAND_TURN, mpMe);`). Soldier/PC dispatch matches on the command (RHelementactor.cpp:5375). The behavioural difference: `Turn()` advances 1 sector per frame; `TurnFast()` (RHpositioninterface.cpp:136) advances 2 per frame. Both are multi-frame.

**Rust architecture:** turns are **instant**. Both `RHCOMMAND_TURN` and `RHCOMMAND_TURN_FAST` collapse to `Command::Turn` (engine/movement.rs:5173-5191) which uses `set_direction_instantly` (element.rs:308). The `fast` flag was added as a hedge so AI callers could still express the "wanted a fast turn" intent for a future engine port, but no consumer ever reads it. The field's own doc-comment says so explicitly:

> *"The current engine snaps direction instantaneously in `process_turn_orders`, so this flag is observationally a no-op — it exists so AI callers can express the intent without losing it when a future engine […]"*

**Verdict:** the Rust port made a deliberate architectural simplification (instant turns); the `fast` flag is a parity hedge for a feature that wasn't ported and has no roadmap. Remove the field and the `order.fast = fast;` assignment in ai.rs:7421. If multi-frame turns are ever re-introduced, the right place to express them is a fresh field at that time, alongside the consumer in `process_turn_orders`.

(The animation-fidelity loss vs C++ is real but separate from this audit — multi-frame turns are a known parity gap regardless of whether the `fast` flag exists.)

---

## Summary of recommendations

| # | Field | Recommendation | Rationale |
|---|---|---|---|
| NPG1 | `PathFinderRuntime::ignore_next_path` | **Remove** | Async-cancel machinery doesn't exist in Rust; pathfinding is synchronous |
| NPG2 | `GridSector::gate_directions` | **Remove** | `find_path_gates` computes `direct` inline; the cached array is redundant |
| NPG3 | `GridSector::highest_door_index` | **Remove (demote loop accumulator to local)** | Runtime consumers re-implemented via `find_lift_exit_door`; field is now build-loop-internal |
| NPG4 | `Order::done` / `AiOrderIntent::done` | **Port the writer** | Real parity gap; read site already in place; writer (`order.done = true` on `MotionState::Done`) needs landing |
| NPG5 | `Order::target_z` / `AiOrderIntent::target_z` | **Remove** | Jumps/flights re-architected to separate state machines; Order is 2D-only by design |
| NPG6 | `Order::fast` / `AiOrderIntent::fast` | **Remove** | Deliberate Rust simplification (instant turns); doc-confessed no-op with no roadmap |

**5 of 6 port gaps are removable** — the Rust architecture has already replaced the C++ semantics via different (and often cleaner) machinery. Only **NPG4 (`done`)** is a true port gap that warrants new code: the read site is correctly stubbed in but the writer is missing.
