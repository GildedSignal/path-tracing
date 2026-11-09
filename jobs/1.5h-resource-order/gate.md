# 1.5h resource-order pair

Frozen before the first live run. Both cases use the existing full-blocks course and the independent client recorder. Each starts from a fresh PathBot save on a separate copy of the same lab. Natural health regeneration is on and mob spawning is off. The one case setting that varies is peaceful versus normal difficulty. Player and world clocks are observed independently in each case.

The initial player stands at `4112.5,-60,4100.5` facing yaw and pitch zero. The fresh player has full health and a food timer of zero. Vanilla saturation and hunger effects prepare full food and saturation and exhaustion bits `40799996`. The latter comes from twenty additions of `0.005f * (38 + 1)` to the fresh player's zero exhaustion. Both effects expire before the probe starts. The input has a one-tick stationary jump at ticks 0, 20 and 40, with every other key released and angles held at zero. There are 80 input ticks plus the recorder's 20-tick rest tail. The driver owns this input sequence and the Rust gate independently checks the applied sequence.

The driver arms the server probe at the settled replay boundary, before it sends `rec continue`, and seals it after the recorder's completed reply. The CSV and client replay share the same run name. Every CSV row has a sequence number, server world tick and server player entity tick. The event rows record calls to player regeneration and FoodData, and raw exhaustion charges and effective health damage at the points where vanilla applies them. FoodData's own internal changes are part of its tick. The first FoodData debit has a post-tick comparison snapshot. The first subsequent saturation refill has a comparison clock. Start and end carry all five resource fields, with float values as eight-digit raw IEEE bits. The Rust replay reads one initial state and the event inputs. It computes its own checkpoints and compares them afterwards.

## Pass line

- Both recordings pass the existing recorder and fixture checks. The Rust comparison is bit-exact on every movement tick, with no anomalies. The two fixture starts agree apart from their independent clocks.
- The server timeline is complete from arm to seal. It supplies three charges of `0.05f`. The third pushes exhaustion above four. Peaceful shows the first saturation refill following the debit. Normal shows the debit and no refill.
- Health, food, saturation, exhaustion and the food timer agree at the first depleted tick and end. The regular RCON end snapshot independently agrees with the sealed state on this resting pair.
- In peaceful, the gate also runs a one-tick phase shift and separately moves the threshold-crossing charge to immediately after the distinguishing refill. The first must fail at the refill comparison and the second at the first depletion comparison.
- The existing L0 replay, refusal rules and movement mutants still apply. This pair is a resource-order gate. Damaging landing classification is 1.6e.

## Integrator run

Use the staged JDK and Rust toolchain. Build the existing client jar with `bot\build.ps1` and build the server probe from the 26.2 reference server's staged jars. The probe builder reads the reference jars.

```powershell
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
& .\bot\build.ps1
$probe = & .\lab\gates\probes\server-phase\build.ps1 -Server 'D:/path-tracing-lab/server' -Output (Join-Path $PWD 'lab/gates/results/1.5h-probe')
$bot = (Resolve-Path .\bot\build\pathtracing.jar).Path
```

Prepare two disposable copies of the same stopped reference rig. In each copy omit PathBot's playerdata save and backup from `server/world/players/data`. Keep the existing offline-mode setup, bot options and server mods, and install the client rig's matching Fabric API in the copied server's mods. Use `server-ip=127.0.0.1`, `enable-rcon=true`, `level-name=world`, and set `server-port` and `rcon.port` to the case's unused pair, ten ports apart. Create a file named `resource-pair.disposable` at each copied rig root. Keep other bot instances closed. The existing fixture driver takes the selected copy's rig lock before installing the opt-in probe, launching the pair of processes and recording. It stops its own processes and removes its probe jar.

```powershell
$env:PATHTRACING_ROOT = 'D:/path-tracing-evidence/1.5h/peaceful'
$env:PATHTRACING_SERVER_PORT = '25567'
& .\lab\gates\record-fixtures.ps1 -Courses full-blocks -Runs 1 -Ticks 80 -Jar $bot -ResourceDifficulty peaceful -ResourceProbeJar $probe

$env:PATHTRACING_ROOT = 'D:/path-tracing-evidence/1.5h/normal'
$env:PATHTRACING_SERVER_PORT = '25568'
& .\lab\gates\record-fixtures.ps1 -Courses full-blocks -Runs 1 -Ticks 80 -Jar $bot -ResourceDifficulty normal -ResourceProbeJar $probe
```

Each report's sibling `pair/full-blocks` directory contains the recording, its world and the raw ordered server trace. Set the two variables to the corresponding **recording** paths.

```powershell
$env:L0_RESOURCE_PEACEFUL = '<absolute path to peaceful recording .jsonl>'
$env:L0_RESOURCE_NORMAL = '<absolute path to normal recording .jsonl>'
$env:CARGO_HOME = 'D:/rust/cargo'
$env:RUSTUP_HOME = 'D:/rust/rustup'
Push-Location engine
cargo test --offline --release --test replay server_resources::authoritative_resource_pair -- --ignored --exact --nocapture
Pop-Location
```

Keep the reports, raw pairs and test output together. At the first mismatch, keep the raw clock and sequence around it. Trace its cause against the 26.2 jar before adding any event to the capture. A refused or missing checkpoint is a failed pair.
