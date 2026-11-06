# Isolated phase witness. Observes the server at tick end without changing its tick implementation.
$ErrorActionPreference = 'Stop'
$env:PATHTRACING_ROOT = 'D:/path-tracing-lab'
$env:PATHTRACING_SERVER_PORT = '25566'
$PSDefaultParameterValues['Start-Process:WindowStyle'] = 'Hidden'
$PSDefaultParameterValues['Start-Process:PassThru'] = $true
$tree = (Resolve-Path -LiteralPath "$PSScriptRoot/../../../..").Path
. "$tree/scripts/common.ps1"
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$result = Join-Path $tree "lab/gates/results/$stamp-server-phase"
$classes = Join-Path $result 'classes'
$null = New-Item -ItemType Directory -Path $classes -Force
$loader = Join-Path $Libraries "net/fabricmc/fabric-loader/$LoaderVersion/fabric-loader-$LoaderVersion.jar"
$dependencies = @(
    "$Rig/server/versions/$McVersion/server-$McVersion.jar"
    $loader
    (Get-ChildItem "$Rig/server/libraries" -Recurse -Filter '*.jar').FullName
    (Get-ChildItem "$tree/bot/lib" -Filter '*.jar').FullName
)
& "$Jdk/javac.exe" --release 25 -proc:none -cp ($dependencies -join ';') -d $classes "$PSScriptRoot/PhaseProbe.java"
if ($LASTEXITCODE) { throw 'Probe compilation failed. Set up the reference rig and build the bot first.' }
Copy-Item -LiteralPath "$PSScriptRoot/fabric.mod.json" -Destination $classes
$probe = Join-Path $result 'phase-probe.jar'
& "$Jdk/jar.exe" --create --file $probe -C $classes .
if ($LASTEXITCODE) { throw 'Cannot package the passive probe.' }

function Rcon([string]$Command) { & "$tree/scripts/rcon.ps1" -Commands @($Command) }
function Wait-For([scriptblock]$Condition, [string]$Name) {
    $clock = [Diagnostics.Stopwatch]::StartNew()
    while ($clock.Elapsed.TotalSeconds -lt 120) {
        if (& $Condition) { return }
        Start-Sleep -Milliseconds 125
    }
    throw "Timed out waiting for $Name"
}
function Game-Time {
    $reply = Rcon 'time query gametime'
    if ($reply -notmatch '^The game time is (\d+) tick\(s\)$') { throw "Bad game time reply: $reply" }
    [long]$Matches[1]
}
function Saturation([string]$Bits) {
    [BitConverter]::ToSingle([BitConverter]::GetBytes([Convert]::ToUInt32($Bits, 16)), 0)
}
function Set-Regeneration([string]$Value) {
    $reply = Rcon "gamerule natural_health_regeneration $Value"
    if ($reply -notmatch "is now set to: $Value") { throw "Gamerule refused: $reply" }
}

$server = $null
$client = $null
$installed = @()
Enter-Rig 'peaceful-phase-probe'
try {
    if ((Test-Port $ServerHost $ServerPort) -or (Test-Port $ServerHost $RconPort)) { throw 'Reference rig is already running.' }
    $api = @(Get-ChildItem -LiteralPath "$Rig/clients/bot/mods" -Filter 'fabric-api-*.jar')
    if ($api.Count -ne 1) { throw 'Expected one Fabric API jar in the reference client.' }
    foreach ($source in @($api[0].FullName, $probe)) {
        $target = Join-Path "$Rig/server/mods" ([IO.Path]::GetFileName($source))
        if (Test-Path -LiteralPath $target) { throw "Will not overwrite $target" }
        Copy-Item -LiteralPath $source -Destination $target
        $installed += $target
    }
    $server = & "$tree/scripts/start-server.ps1"
    Wait-For { Test-Port $ServerHost $RconPort } 'server'
    $client = & "$tree/scripts/launch-client.ps1" -Role bot
    Wait-For { (Rcon 'list') -match "online:.*\b$BotName\b" } 'bot'
    Rcon "gamemode survival $BotName"
    Rcon "tp $BotName 0.5 -60 0.5 0 0"
    Rcon 'difficulty peaceful'
    Set-Regeneration 'false'
    Rcon "effect give $BotName minecraft:saturation 1 20 true"
    $until = (Game-Time) + 25
    Wait-For { (Game-Time) -ge $until } 'saturation effect end'
    Rcon "effect give $BotName minecraft:hunger 1 255 true"
    $until = (Game-Time) + 30
    Wait-For { (Game-Time) -ge $until } 'hunger effect end'
    $before = (& "$tree/scripts/server-state.ps1" $BotName) | ConvertFrom-Json
    if ($before.Health -ne 20 -or $before.foodSaturationLevel -gt 16 -or $before.foodExhaustionLevel -gt 4) {
        throw 'Expected full health and at least four units of saturation deficit after the hunger effect.'
    }
    foreach ($case in @(
        @{ name = 'peaceful-off'; difficulty = 'peaceful'; regeneration = 'false'; refills = 0 },
        @{ name = 'normal-on'; difficulty = 'normal'; regeneration = 'true'; refills = 0 },
        @{ name = 'peaceful-on'; difficulty = 'peaceful'; regeneration = 'true'; refills = 2 }
    )) {
        Rcon ('difficulty ' + $case.difficulty)
        Set-Regeneration $case.regeneration
        if ((Rcon "ptphase start $BotName") -ne 'Phase capture started') { throw 'Phase command did not start.' }
        Wait-For { @( (Rcon 'ptphase read') -split ';' ).Count -eq 45 } $case.name
        $raw = (Rcon 'ptphase read') -split ';'
        $raw | Set-Content -LiteralPath (Join-Path $result ($case.name + '.csv')) -Encoding Ascii
        $previous = $null
        $refills = 0
        $phases = @()
        foreach ($line in $raw) {
            $values = $line.Split(',')
            if ($values.Count -ne 4) { throw "Malformed sample: $line" }
            $current = @{ game = [long]$values[0]; player = [int]$values[1]; food = [int]$values[2]; saturation = Saturation $values[3] }
            if ($null -ne $previous) {
                if ($current.game -ne $previous.game + 1 -or $current.player -ne $previous.player + 1) { throw "Discontinuous ticks in $($case.name)" }
                if ($current.food -ne $previous.food) { throw "Unexpected food change in $($case.name)" }
                $delta = $current.saturation - $previous.saturation
                if ($delta -ne 0) {
                    if ($delta -ne 1 -or $current.player % 20 -ne 0) { throw "Unexpected saturation change: $line" }
                    $refills++
                    $phases += "$($current.game)/$($current.player)"
                }
            }
            $previous = $current
        }
        if ($case.refills -eq 0 -and $refills -ne 0 -or $case.refills -gt 0 -and $refills -lt $case.refills) {
            throw "Expected cadence not observed in $($case.name)"
        }
        "PASS $($case.name): 45 paired world/player ticks, $refills saturation refills (world/player $($phases -join ', '))"
    }
    "Evidence: $result"
} finally {
    try {
        if ($null -ne $client -and -not $client.HasExited) { $client.CloseMainWindow() | Out-Null }
        if ($null -ne $server -and -not $server.HasExited) {
            Rcon 'difficulty peaceful'
            Set-Regeneration 'true'
            Rcon 'stop'
        }
        if ($null -ne $client -and -not $client.WaitForExit(5000)) { $client.Kill(); $client.WaitForExit() }
        if ($null -ne $server -and -not $server.WaitForExit(15000)) { $server.Kill(); $server.WaitForExit() }
    } finally {
        foreach ($target in $installed) { Remove-Item -LiteralPath $target }
        Exit-Rig
    }
}
