# 1.9a frozen contact damage gate

Freeze before the first live capture. The integrator runs the capture on disposable rigs. The primary reference is the vanilla 20 TPS clock.

## Pass line

Use three fresh PathBot survival saves on copies of the same 26.2 world. Start at full health and zero damage cooldown. Use normal difficulty, natural regeneration off, fire damage on, zero absorption, no effects and empty equipment slots. Use vanilla damage data. On the stone pad, settle at `4112.5 -60.0 4100.5`. Change only its center block at `4112 -61 4100` to the case's block while the player stays put.

| Case | Support after the swap | Actual pose and input | Requests | Accepted relative ticks |
|---|---|---|---|---|
| `magma-standing` | magma | STANDING, keys released | one `minecraft:hot_floor` with raw `1.0F` each tick | 0, 10, 20, 30 |
| `magma-sneaking` | magma | CROUCHING, sneak held | zero | none |
| `netherrack-standing` | netherrack | STANDING, keys released | zero | none |

Start at the first server `ServerPlayer.doTick` HEAD at the pad with the target block. Take exactly 32 consecutive ticks through the last `doTick` RETURN. Include the immediately preceding stone tick's end as a guard. There is one capture per JVM. Check both the world and entity clocks. Every selected tick must show the exact stationary coordinates, `onGround`, actual pose and support pad at HEAD, at the block dispatch and at RETURN. Require the expected client input, the matching `isSteppingCarefully` value, and both the actual supporting block position and the legacy contact position at the pad. The first tick and its stone predecessor must show a settled player at full health.

For each tick, check a single grounded `stepOn` dispatch between HEAD and RETURN. Order any damage rows within that dispatch and check their source, raw amount and vanilla return value. Separately compare the authoritative `getHealth()` samples at HEAD, dispatch and RETURN and the before/after samples in every damage call. Each accepted request debits exactly one health. All other requests leave health unchanged. In the active case there are 4 accepted and 28 refused requests, with end health 19 for ticks 0–9, 18 for 10–19, 17 for 20–29 and 16 for 30–31. The controls stay at 20. Check the cooldown at all samples. Reject other damage, a missing or additional event, any wrong phase or tick, any invalid observation, and a missing seal. Retain the raw TSV and verdict at the first difference.

## Build and offline checks

Run from `wt/law`. Node.js runs the verifier. The probe has no Fabric API dependency. `-CheckHooks` applies the required Mixins through Fabric and links the target classes without starting Minecraft.

```powershell
$env:PATHTRACING_DEPS = 'D:/path-tracing-deps/data'
$lawTree = $PWD.Path
$job = Join-Path $lawTree 'oracles/1.9a-contact-damage-witness'
$build = Join-Path 'D:/path-tracing-target' ('1.9a-' + [guid]::NewGuid().ToString('N'))
& "$job/build.ps1" -Server 'D:/path-tracing-lab/server' -Output $build -CheckHooks
$probe = Join-Path $build 'contact-witness.jar'
node "$job/verify.mjs" --self-test
if ($LASTEXITCODE) { throw 'Verifier self-test failed' }
```

## Capture at 20 TPS

Prepare three stopped disposable copies using the isolation and fresh-save procedure in [the resource probe gate](../1.5h-resource-order/gate.md). Use the lab world containing the full-blocks pad, omit PathBot's playerdata save and backup in every copy, and keep the source rig stopped and locked while copying. Give every copy an unused localhost-only server port and RCON at port + 10. Keep the game-logic mods excluded by the rig convention. Use the vanilla clock with no PathTracing client jar in these three copies. Use a focused bot client, hold-to-sneak and no auto-jump. Keep other clients out. In each copy create `server/contact-witness.disposable` and keep only the rig's approved server mods before the script adds the probe. Run the block below once per case with the corresponding copy, case and port. The setup edits happen in the copy. The probe reads game state and writes the trace.

```powershell
# Set these three values for each copy.
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_ROOT = 'D:/path-tracing-evidence/1.9a/20/magma-standing'
$env:PATHTRACING_SERVER_PORT = '25567'
$case = 'magma-standing'
$rate = 20
$clientTree = $lawTree

. "$lawTree/scripts/common.ps1"
function Contact-Rcon([string[]]$Commands) { & "$lawTree/scripts/rcon.ps1" -Commands $Commands }
$serverDir = Join-Path $Rig 'server'
$trace = Join-Path $serverDir "$case.tsv"
$server = $client = $null
Enter-Rig "1.9a-$rate-$case"
try {
    if (!(Test-Path "$serverDir/contact-witness.disposable" -PathType Leaf) -or (Test-Path $trace)) {
        throw 'Expected a fresh marked disposable server'
    }
    if ((Test-Port $ServerHost $ServerPort) -or (Test-Port $ServerHost $RconPort)) {
        throw 'Expected unused server and RCON ports'
    }
    Copy-Item -LiteralPath $probe -Destination "$serverDir/mods/"
    $server = Start-Process -FilePath "$Jdk/java.exe" -ArgumentList @(
        '-Xms2G', '-Xmx4G', "-Dpathtracing.contact.case=$case", '-jar', 'fabric-server-launch.jar', 'nogui'
    ) -WorkingDirectory $serverDir -WindowStyle Hidden -PassThru
    Read-Host 'Check that the copied server is ready in server/logs/latest.log, then press Enter'
    Contact-Rcon @('difficulty normal', 'gamerule natural_health_regeneration false', 'gamerule fire_damage true',
        'gamerule spawn_mobs false', 'weather clear', 'tick rate 20')
    $flags = if ($rate -eq 2000) { @{FollowTickRate = 'true'; FullTickDemand = 'true'} } else { @{} }
    $client = & "$clientTree/scripts/launch-client.ps1" -Role bot @flags
    Read-Host 'Check PathBot has joined this copy, then press Enter'
    Contact-Rcon @('gamemode survival PathBot',
        'fill 4111 -60 4099 4113 -57 4101 minecraft:air',
        'setblock 4112 -61 4100 minecraft:stone', 'tp PathBot 4112.5 -59.75 4100.5 0 0',
        "tick rate $rate", 'tick query')
    Read-Host 'Press Enter, focus the bot and hold sneak only for magma-sneaking until the capture ends'
    Start-Sleep -Seconds 5
    $block = if ($case -eq 'netherrack-standing') { 'netherrack' } else { 'magma_block' }
    Contact-Rcon @("setblock 4112 -61 4100 minecraft:$block")
    $deadline = [Diagnostics.Stopwatch]::StartNew()
    while (!(Test-Path -LiteralPath $trace) -and $deadline.Elapsed.TotalSeconds -lt 10) {
        Start-Sleep -Milliseconds 25
    }
    if (!(Test-Path -LiteralPath $trace)) { throw 'Missing contact trace' }
} finally {
    try {
        if ($server -and !$server.HasExited) {
            try { Contact-Rcon @('tick rate 20', 'stop') }
            finally { if (!$server.WaitForExit(15000)) { $server.Kill(); $server.WaitForExit() } }
        }
    } finally {
        if ($client -and !$client.HasExited) { $client.Kill(); $client.WaitForExit() }
        Exit-Rig
    }
}
```

Verify all three 20 TPS TSVs together. Keep the server log and RCON output from each copy beside the TSV. A capture refusal is a gate failure.

```powershell
$evidence = 'D:/path-tracing-evidence/1.9a/20'
node "$job/verify.mjs" 20 "$evidence/magma-standing/server/magma-standing.tsv" `
    "$evidence/magma-sneaking/server/magma-sneaking.tsv" "$evidence/netherrack-standing/server/netherrack-standing.tsv"
if ($LASTEXITCODE) { throw 'Contact witness gate failed' }
```

## Separate 2,000 TPS recheck

Run after the 20 TPS reference passes. Prepare three more fresh disposable copies with distinct port pairs and the same fixture, profile and probe. Build the qualified `wt/clock` client at `388a1bc` or a qualified descendant. Copy its `bot/build/pathtracing.jar` into each disposable bot's `mods` directory and use that worktree as `$clientTree`. Use the [qualified full-demand clock recipe](../../../clock/lab/gates/clock-full-demand.md) for the focused client profile. In the capture block set `$rate = 2000`, `$clientTree = 'D:/path-tracing-wt/clock'` and the new case-specific root and port. The server starts at 20, switches before the stone predecessor, and restores 20 on exit. Save the bot's `launch.args` with both clock flags enabled and the `tick query` reply. Verify the three new files with the same command, changing the root to `D:/path-tracing-evidence/1.9a/2000` and passing `2000`.

The two batches must independently satisfy the same relative tick, order, cooldown and health criteria. Absolute clocks, configured rate and measured capture span are kept separately. The sealed wall span covers the server capture from first HEAD through last RETURN. Use the clock gate's sustained measurement for throughput claims.

## Trace format

TSV uses one header, then globally numbered rows in emission order. `P`, `B`, `C`, `E` are the stone predecessor, tick start, block dispatch and tick end. Their columns after sequence and tag are world tick, entity tick, x/y/z, ground, pose, careful, sneak key, other movement key, supporting-position match, legacy-position match, block key, game mode, difficulty, regeneration, fire damage, effect count, equipment empty, max health, absorption, base or ability invulnerability or dimension transition, client loaded, overworld, configured rate, frozen, health and cooldown. Floats for health, absorption and damage use raw IEEE-754 hex.

`R` records world tick, entity tick, whether the call was within the block dispatch, source key, raw amount, boolean return, health before and after, and cooldown before and after. The probe pairs call entry and return by source identity and refuses nested ordering through `X`. `Z` seals the fixed count and measured wall span in nanoseconds. See [the server resource timeline](../1.5h-resource-order/gate.md) for the clock and sequence convention.
