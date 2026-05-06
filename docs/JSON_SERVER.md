# JSON-RPC HTTP server

The game binary exposes a local HTTP endpoint for external tools — debug
shells, test harnesses, AI drivers, screenshot pipelines. Source lives
in [crates/robin_rs/src/http_server.rs](crates/robin_rs/src/http_server.rs).

## Starting / stopping

Bind address is always `127.0.0.1` (no authentication, no TLS, no LAN
exposure). Default port is `17640`.

```
robin                        # starts on :17640
robin --http-server 9999     # custom port
robin --http-server 0        # disabled
robin --start-paused         # launch with sim paused (HUD/cursor live)
```

`--start-paused` freezes the engine tick from frame 0 — the HUD,
cursor, and input are still live, but the simulation does not advance
until a `/step-forward` request drives it. Useful for automated test
drivers that want to observe-then-advance without the default 25 fps
wall-clock pace.

The listener runs in a dedicated `robin-http-server` thread; the game
loop drains queued requests once per tick.

## Threading model

```
HTTP client ──► listener thread ──► queue ──► game tick ──► reply
                (tiny_http)                   (drain_global)
```

Each request waits on a one-shot channel with a 60 s timeout. While the
game is paused, loading a level, rewinding, or displaying a modal
dialog / briefing, the per-tick drain is suspended — requests block
until the game resumes (or time out). Clients that want to fail fast
instead of waiting out a blocked main loop should pass a shorter HTTP
timeout themselves, e.g. `curl --max-time 2`.

## Endpoints

### `GET /`

Returns a listing of every endpoint with a short description.

### `GET /natives`

Lists every `NativeFn` the script VM recognises: index, name, return
type, parameter types and names (parsed from the embedded
`RHScriptAPI.scs`).

```json
{ "natives": [
    { "index": 12, "name": "SetMissionWon", "return_type": "void",
      "params": [] },
    ...
]}
```

### `GET /state`

Engine snapshot: frame counter, map name, mission flags, camera,
selected PCs, full entity list.

```json
{
  "frame": 1842,
  "map": "Dem_Lei_MP",
  "mission": { "won": false, "quit_won": false, "quit_lost": false },
  "camera": { "x": 3200.0, "y": 1800.0 },
  "selected_pc_handles": [1],
  "pcs": [...],
  "entity_count": 127,
  "entities": [...]
}
```

### `GET /script`

Mission script class and function listing (class names, function names,
member variables, quad count per class, total counts of instances).

### `GET /script/decompile[?class=Foo]`

Decompiles the loaded script bytecode to TypeScript-like pseudocode.
Pass `?class=Foo` to scope to one class; omit for the whole file.

```json
{ "source": "class Foo {\n  function bar(...) { ... }\n}\n..." }
```

### `POST /native`

Invoke a single script-VM native with integer arguments. `this` is an
optional script-side `script_this` override.

```
POST /native
{ "op": "SetMissionWon", "args": [], "this": null }
```

Returns `{ "return": <i32> }` or `{ "error": "..." }`.

### `POST /batch`

Run many natives on the same tick. Each call's `this` and arguments
are handled independently; results are returned in order.

```
POST /batch
{ "calls": [
    { "op": "GetFrame", "args": [] },
    { "op": "SetMissionWon", "args": [] }
]}
```

Returns `{ "results": [ {...}, {...} ] }`.

### `POST /console`

Run a debug-console command (the same strings you'd type into the
in-game `~` console).

```
POST /console
{ "command": "give all" }
```

Response `kind` tells you how the command was handled:

- `ok` — executed inline, optional `message` is the console output
- `unknown` — unrecognised command
- `not_implemented` — recognised but stubbed
- `host_followup` — sim-state command that needs host-side dispatch
  (CAMPAIGN load, ARES advance with side effects, …) — the variant
  name is returned in `variant`

### `POST /command`

Apply a `PlayerCommand` directly to the engine. Body is the externally
tagged JSON form of the [`PlayerCommand`](crates/robin_engine/src/player_command.rs)
enum. Returns `{ "ok": true }` on success.

```
POST /command
{ "SelectPc": { "id": 1 } }
```

### `GET /screenshot`

Returns a PNG image of the next rendered frame.
**Response body is raw `image/png` bytes**, not JSON.

Query parameters (all optional):

| Param              | Meaning                                                               |
|--------------------|-----------------------------------------------------------------------|
| `w`, `h`           | Output size. If both set, image is nearest-neighbour resized. Native render-target size if omitted. |
| `hide_ui`          | Crop off the bottom HUD panel (80 px) before encoding.                |
| `view_cones`       | Force the "show all NPC view cones" debug overlay on/off.             |
| `pc_sight`         | Force the PC-sight overlay.                                           |
| `motion_graph`     | Motion-graph debug lines.                                             |
| `all_obstacles`    | Render every active sight obstacle.                                   |
| `elevation`        | Elevation grid.                                                       |
| `noise`            | Noise display.                                                        |
| `sound_source`     | Sound sources.                                                        |
| `actor_info`       | Per-actor info text.                                                  |
| `script_zones`     | Script zone outlines.                                                 |
| `door`             | Door overlay.                                                         |
| `projection_areas` | Projection area outlines.                                             |
| `railroad`         | Railroad debug overlay.                                               |
| `probability`      | Probability display.                                                  |
| `company_number`   | Company number overlay.                                               |
| `combat_energy`    | Combat energy bars.                                                   |
| `light_zones`      | Light zone outlines.                                                  |
| `animation_lines`  | Animation lines.                                                      |
| `seek_points`      | Seek points.                                                          |
| `fps`              | FPS counter.                                                          |

Flag values accept `1`/`0`, `true`/`false`, `yes`/`no`, `on`/`off`
(case-insensitive), or just a bare `?view_cones` (treated as `true`).
Flags only affect *this* screenshot — the live `DevState` is untouched.

```
# Native resolution, no UI, all view cones:
curl -o shot.png 'http://127.0.0.1:17640/screenshot?hide_ui=1&view_cones=1'

# Scaled to 640x480:
curl -o thumb.png 'http://127.0.0.1:17640/screenshot?w=640&h=480'

# Multiple debug overlays at once:
curl -o debug.png 'http://127.0.0.1:17640/screenshot?view_cones&pc_sight&noise'
```

The render pipeline is explicitly set up so screenshot flags **do not**
mutate the live engine or dev state — `render_frame` takes `&Engine` +
`&DevState`, and a per-screenshot throwaway frame is rendered with a
cloned `DevState` that has the requested flag overrides merged in.

### `POST /step-forward`

Run `n` engine ticks synchronously. Body `{"n": N}` — `N` defaults to
`1` if the body is empty. Each tick goes through the same bookkeeping
as a normal unpaused frame (rollback checker, rewind-buffer commit,
`sim_frame++`).

```json
{"direction": "forward", "from_frame": 0, "frame": 10, "advanced": 10}
```

The endpoint bypasses the `--start-paused` / pause-menu gate so it
works from a paused game. **Fails organically** when the engine has
queued a modal dialog / briefing / debriefing — advancing the sim past
one would skip it, so the request responds:

```json
{"error": "modal dialog/briefing pending — dismiss before stepping the sim"}
```

`advanced` may be less than `n` if the sim hit a modal partway through
the batch — the step returns early with the frames it did manage to
run.

### `POST /step-back`

Rewind `n` frames through the engine's rollback buffer. Body
`{"n": N}` (defaults to 1). Replaces the live engine/game/assets/dev
state with the reconstructed state at `sim_frame - N`.

```json
{"direction": "back", "from_frame": 10, "frame": 7, "rewound": 3}
```

Fails with 400 if:
- `n` exceeds the current frame number, or
- the target frame is older than the oldest retained snapshot
  (the buffer's exponential retention means you can't always reach
  arbitrarily far back — typical coverage is a few hundred frames).

## Error responses

Any failure returns a 4xx/5xx with a JSON body:

```json
{ "error": "bad json: expected value at line 1 column 3" }
```

- `400` — malformed payload, bad native, or handler-returned error
- `404` — unknown path
- `504` — game loop didn't process the request within 60 s
- `500` — response channel dropped (listener thread is gone)

## Examples

```
# Snapshot the current state:
curl -s http://127.0.0.1:17640/state | jq .frame

# Invoke a native:
curl -s -X POST http://127.0.0.1:17640/native \
     -H 'Content-Type: application/json' \
     -d '{"op":"SetMissionWon","args":[]}'

# Run a console cheat:
curl -s -X POST http://127.0.0.1:17640/console \
     -H 'Content-Type: application/json' \
     -d '{"command":"give all"}'

# Decompile one class:
curl -s 'http://127.0.0.1:17640/script/decompile?class=MissionMain' | jq -r .source

# Save a clean screenshot of just the map:
curl -s -o map.png \
     'http://127.0.0.1:17640/screenshot?hide_ui=1'

# Drive a --start-paused game one tick at a time:
curl -s -X POST -d '{"n":1}' \
     -H 'Content-Type: application/json' \
     http://127.0.0.1:17640/step-forward

# Rewind 30 frames:
curl -s -X POST -d '{"n":30}' \
     -H 'Content-Type: application/json' \
     http://127.0.0.1:17640/step-back
```
