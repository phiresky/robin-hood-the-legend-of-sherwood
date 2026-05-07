# Dead / unfilled state in the script & sequence subsystem

Audit scope: `crates/robin_engine/src/sequence.rs`,
`engine/script.rs`, `engine/sequence_validity.rs`, `script_manager.rs`,
`sprite_script.rs`, `replay.rs`, `order.rs` (script/sequence-related fields
only — `Order::done`/`fly`/`fast`/`target_z` are already on the cleanup
list and not re-listed here).

Status: deletion of all pure-dead items below has been applied to the
worktree (`cargo build --bin robin` clean, `cargo test -p robin_engine
--lib` 1389/1389 pass).

---

## Latent regression (one)

After deeper investigation, only one of the original three "latent
regression" candidates is a real port-time data-flow loss. The other
two are dead in Rust because the *consuming* feature was rearchitected
to use a different path (or, in the swordfight case, collapsed to a
single dispatch that no longer needs a gate).

### `Field::ShieldDangerPointLayer` — danger-point titbit drawn on wrong layer

- **Field**: `crates/robin_engine/src/sequence.rs:552`
  (post-cleanup index — was `:557`).
- **C++ counterpart**: `RHFIELD_SHIELD_DANGER_POINT_LAYER`
  (`RHSequenceElementGeneric.h:62`). Set:
  `RHengine.cpp:15358,15370,15399,15410` (player-click handler stamps
  `muwSelectedLayer` — i.e. the layer the player picked when raising
  the shield, which can differ from the PC's own layer). Read:
  ```cpp
  // RHelementactorpc.cpp:3022 (RHCOMMAND_RAISE_SHIELD)
  SWORD swLayer = pSequenceElementGeneric->GetPropertyAsInteger( RHFIELD_SHIELD_DANGER_POINT_LAYER );
  SBGeoPoint3D ptNewShieldDangerPoint = pSequenceElementGeneric->GetPropertyAs3DPoint( RHFIELD_SHIELD_DANGER_POINT );
  if( ptNewShieldDangerPoint.IsZero() ) swLayer = -1;
  else { …; AddTitbit( mptShieldDangerPoint, swLayer, RHTITBIT_DANGER_POINT, … ); }
  ```
  (The `RHCOMMAND_LOWER_SHIELD` path that also used the field at
  `RHelementactorpc.cpp:5408` is commented out in the C++ source.)
- **Rust producer**: `engine/commands.rs:3011`
  (`set_property(Field::ShieldDangerPointLayer, …)`) — set correctly.
- **Rust consumer flow that *should* read it**:
  - `engine/melee/dispatch.rs:548-589` reads
    `Field::ShieldDangerPoint` and stamps it onto
    `PcData::shield_danger_point: Point3D`. **Layer is dropped here.**
  - `engine/titbit_sync.rs:1010` then adds the danger-point titbit
    using `pc.element.layer()` — **the PC's own layer**, not the
    layer the player picked.
  ```rust
  // engine/titbit_sync.rs:996-1012 (today, post-cleanup):
  states.push(DangerState {
      id: pc_id,
      position: Point3D { x: pc.pc.shield_danger_point.x, … },
      layer: pc.element.layer(),                // ← wrong, should be ShieldDangerPointLayer
      is_protecting: pc.pc.shield_protected.is_some(),
  });
  ```
- **Symptom**: when the player raises a shield with the danger-point
  pick on a different map layer (looking at a threat across a chasm,
  off a balcony, etc.), the danger-point titbit indicator is drawn on
  the PC's layer rather than the picked layer. Layered rendering may
  cull or mis-Z-order it; the indicator is misleading or invisible.
- **Fix recipe**: in `engine/melee/dispatch.rs:548-589`, also read
  `Field::ShieldDangerPointLayer`, add a
  `pc.shield_danger_point_layer: u16` field on `PcData`, and have
  `engine/titbit_sync.rs:1010` use that field instead of
  `pc.element.layer()`. Mirror the C++ "if point is zero, layer=-1"
  semantics — the existing zero-skip in `sync_danger_point_titbits`
  already handles the zero-point case so a `u16` (no `-1`) is fine
  if paired with a "danger-point set" predicate.
- **Verification**: this regression was confirmed by reading the
  whole consumer path (`Field::ShieldDangerPoint` → `pc.shield_danger_point`
  → titbit add) — the layer field never enters that path.

---

## Reclassified: were latent regressions, now dead state

### `SequenceElement::script_driven` — feature replaced architecturally

- C++ reads `IsScriptDriven()` at `RHelementactorhuman.cpp:3280,3311`
  to gate `MSG_SELECT_ACTION` / `MSG_UNSELECT_ACTION` broadcasts on
  bow `MOTION_START`. The broadcast updated the UI button highlight.
- The Rust port has **no equivalent broadcast** at bow MOTION_START.
  UI button highlight is driven by `pc.current_action`, written only
  in player-click flows (`engine/selection.rs:562,600`,
  `inventory.rs:253`) and reset in `engine/commands.rs:642` /
  `engine/selection.rs:852`. Scripts launching `Command::EquipBow` or
  `Command::UnequipBow` (e.g. `natives/mod.rs:4632,4726`) leave
  `pc.current_action` untouched, so the C++ gate's purpose has no
  Rust analogue.
- The 8 write sites of `script_driven` (notably in
  `RecordingSession::add_element*` for every Record* native and
  inconsistently in `natives/mod.rs:4648,4672,4677,4703,4727`) all
  feed a field with no readers.
- **Reclassified: dead state**. Safe to delete the field, all 8
  write sites, and the `mbScriptDriven` mention in
  `RHsequenceelement.h`'s parity notes.
- *Was* listed under "Latent regressions" in the v1 report. The v1
  call was wrong: I conflated "C++ gates a broadcast that doesn't
  exist in Rust" with "Rust read site sees a default". There is no
  Rust read site.
- **Not deleted in this pass** (left for a follow-up since
  it crosses 8 files; deletion is mechanical but noisy).

### `Field::SwordfightPrepared` — gate collapsed by single-dispatch architecture

- C++ ENTER_SWORDFIGHT execute (`RHelementactorhuman.cpp:1239`)
  reads `SWORDFIGHT_PREPARED`; if false, walks the actor to a
  table-fight slot, sets it `true`, returns. Re-dispatch picks up
  with `true` and proceeds to actual swordfight entry.
- Rust ENTER_SWORDFIGHT execute (`engine/melee/dispatch.rs:140-266`)
  runs **both halves on the same dispatch** —
  `try_launch_table_swordfight_move()` enqueues the slot-walk Move as
  a *parallel* sequence element, then immediately calls
  `enter_swordfight_with_jump_line()` and terminates the
  EnterSwordfight element. There is no second dispatch, so no gate
  is needed.
- **Behaviour delta vs C++**: Rust enters swordfight (sword raised,
  opponents linked, cursor flips) before the actor reaches the slot;
  C++ waits until the slot is reached. Out of scope for this audit;
  if it matters, file under `NEW_FEATURES.md` or a parity note.
- All 7 Rust write sites set the field to `Bool(false)`, never
  `true`. Never read.
- **Reclassified: dead state**. Safe to delete the variant and the
  7 writes.
- **Not deleted in this pass** (same rationale as `script_driven`).

---

## Dead state — DELETED in this pass

The following items were removed from the worktree:

| Item | Where it was | Why dead |
| --- | --- | --- |
| `Order::priority` | `order.rs:523` | No reader. C++ has no `priority` on `RHOrder`. |
| `AiOrderIntent::priority` | `order.rs:667` | Same. Only ever `0`; the `with_priority` builder + tests were the only writes. |
| `Order::with_priority` builder | `order.rs:594` | Removed with the field. |
| `Order::no_halt` | `order.rs:542` | Stamped from `AiOrderIntent` but never read off `Order`. (`AiOrderIntent::no_halt` kept — real reads in `engine/movement.rs:5181,5188`.) |
| `Field::Event` | `sequence.rs:531` | Animal-AI leftover; no Rust producer or consumer. |
| `Field::ConcussionLevel` | `sequence.rs:544` | No setter or reader in either language. |
| `Field::BowTargetGuy` | `sequence.rs:536` | Bow targeting in Rust uses `engine::input::BowTarget`, not this field. |
| `Field::BowTargetPoint` | `sequence.rs:537` | Same. |
| `Field::DialogSource` | `sequence.rs:549` + `natives/mod.rs:3602` | Always-zero ceremony field; dead in C++ too. |
| `Field::RollPoint` | `sequence.rs:559` | `Command::Roll` is not in the Rust `Command` enum at all — Rolling lives at the `OrderType::Rolling` level (`engine/melee/effects.rs:1152`). |
| `Field::Gate` | `sequence.rs:565` | No setter, no reader. Movement gate refs live on `SequenceElementData::Movement::gate_id`. |

Also: the three `Order::priority` test asserts in `order.rs::tests`
were trimmed (`order_builder`, `order_defaults`, `serde_roundtrip_order`)
since the field they were checking is gone.

Build + tests: clean.

---

## Borderline (not dead, not regressions — kept)

- **`Sequence::started`** (`sequence.rs:1413`) — sole purpose is to
  debug-assert that `launch()` is called at most once. Real invariant
  guard.
- **`RecordingSession::has_elements_at_current_level`**
  (`sequence.rs:413`) — gates `Then()`'s level-bump so empty levels
  aren't advanced.
- **`Field::Amount`** — set in `engine/commands.rs:494` (DropAmmo
  count), read in `engine/tick.rs:3242`. Real, matches C++
  (`RHengine.cpp:12827` setter, `RHelementactorpc.cpp:2943,5095,5151`
  readers).

---

## False positives (ruled out at audit time)

| Field | Why it looked dead | Why it's live |
| --- | --- | --- |
| `posture_after_transition` / `action_state_after_transition` | "tracking" comment | 8+ sites set, multiple readers in `engine/transitions.rs`, `engine/movement.rs` |
| `num_transition_orders` | name suggests bookkeeping | Decremented in `sequence.rs:2575,3037,3077`; consumed by transition replay |
| `cross_postponed` | only `take()` calls visible | Real cross-sequence successor used by `engine/mod.rs:2228+` |
| `SequenceManager::halt_pending` | one assignment + one read | Tags every condolation queued during `stop_owner(Preference)` for downstream Think-event suppression |
| `PendingCondolation::from_halt` | only one explicit `card.from_halt = true` | Consumed by `engine/soldier_helpers.rs:307,395` |
| `Movement::direction` (Movement variant) | most sites set 0 | Set non-zero at `engine/movement.rs:1780,2133`; read at `engine/tick.rs:1881` for `Command::ChangePosition` (rotates actor instantly) |
| `Field::PurseTarget` / `NetTarget` / `WaspNestTarget` | grep showed zero direct sets | Set indirectly via `engine/commands.rs:357` (`*target_field`), originating in `crates/robin_rs/src/game_input.rs:609,628,649` |
| `Field::ShieldDangerPoint` | grep regex showed read-only | Set at `engine/commands.rs:3003` *and* `ai_enemy/event_handlers.rs:1329`; read at `engine/melee/dispatch.rs:550` |
| `Field::SpeakFlags` | grep showed set=0 | Set at `natives/mod.rs:4365`; read at `engine/tick.rs:6889` |
| `Order::priority` (in test asserts) | tests assert it round-trips | The round-trip is real; the *value* is never consumed in production. Listed under deleted dead state above. |

---

## Verification commands

```bash
# Field-variant tally
for f in PurseTarget NetTarget WaspNestTarget Amount …; do
    total=$(grep -rn "Field::$f\b" crates/robin_engine/src --include='*.rs' | wc -l)
    sets=$(grep -rnE "(set_property|update_property|properties\.insert)\(.*Field::$f\b" \
                 crates/robin_engine/src --include='*.rs' | wc -l)
    reads=$(grep -rnE "(get_property|properties\.get)\(.*Field::$f\b" \
                 crates/robin_engine/src --include='*.rs' | wc -l)
    echo "$f total=$total set=$sets read=$reads"
done

# Per-field assignment scan (ast-grep, structural)
ast-grep run --pattern '$X.script_driven = $$$' --lang rust crates/robin_engine/src
ast-grep run --pattern '$X.shield_danger_point = $$$' --lang rust crates/robin_engine/src

# Movement struct-literal direction values (Python regex, multiline)
python3 -c "
import re, os
for root, _, fs in os.walk('crates/robin_engine/src'):
    for fn in fs:
        if not fn.endswith('.rs'): continue
        p=os.path.join(root,fn); txt=open(p).read()
        for m in re.finditer(r'SequenceElementData::Movement\s*\{[^}]*direction:[^,}\n]*', txt, re.DOTALL):
            v=re.search(r'direction:\s*([^,\n}]+)', m.group(0))
            if v and v.group(1).strip()!='0':
                ln=txt[:m.start()].count('\n')+1
                print(f'{p}:{ln}: {v.group(1).strip()}')"

# C++ ground truth
grep -rn 'IsScriptDriven\|mbScriptDriven'  original-code/
grep -rn 'SWORDFIGHT_PREPARED'             original-code/
grep -rn 'SHIELD_DANGER_POINT_LAYER'       original-code/
```
