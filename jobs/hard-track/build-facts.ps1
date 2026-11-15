$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$taskJar = 'D:\path-tracing-deps\data\libraries\com\mojang\minecraft\26.2\minecraft-26.2-client.jar'
$taskJavap = 'D:\path-tracing-deps\data\java\java-runtime-epsilon\bin\javap.exe'
$taskBytecode = (& $taskJavap -classpath $taskJar -c -p net.minecraft.world.level.gamerules.GameRules) -join "`n"
if ($LASTEXITCODE -ne 0) { throw '26.2 GameRules javap failed' }
$taskLimitText = [regex]::Match($taskBytecode, 'String max_block_modifications[\s\S]*?// int (\d+)').Groups[1].Value
if (-not $taskLimitText) { throw 'max_block_modifications default not found in the staged jar' }
$taskVolumes = @()
foreach ($line in Get-Content -LiteralPath (Join-Path $taskRoot 'lab/tracks/hardtrack.build.txt')) {
    $taskCommand = $line.Split('#')[0].Trim()
    if (-not $taskCommand) { continue }
    $f = $taskCommand.Split(' ')
    if ($f[0] -eq '/fill') {
        $taskVolume = 1L
        for ($axis = 0; $axis -lt 3; $axis++) { $taskVolume *= ([int]$f[4+$axis] - [int]$f[1+$axis] + 1) }
        $taskVolumes += $taskVolume
    }
}
$taskLaterMaximum = ($taskVolumes | Select-Object -Skip 1 | Measure-Object -Maximum).Maximum
if ($taskLaterMaximum -gt [long]$taskLimitText) { throw 'a later fill exceeds the default limit' }
if ($taskVolumes[0] -ne 106641) { throw 'the first clear volume changed; review the build prerequisite' }
$taskJarHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $taskJar).Hash
$taskFacts = @"
# Vanilla build facts

Generated offline by ``build-facts.ps1`` from the build list and staged Minecraft 26.2 ``GameRules`` bytecode.

- Default ``minecraft:max_block_modifications``: $taskLimitText.
- First fill volume: $($taskVolumes[0]); temporarily set the rule to this value before executing it, then restore the previously observed value after the build.
- Largest later fill: $taskLaterMaximum; it fits the default.
- Staged client jar SHA-256: ``$taskJarHash``. No jar is included in the deliverables.
- No game, server or RCON connection was opened.
"@
[IO.File]::WriteAllText((Join-Path $PSScriptRoot 'build.md'), $taskFacts, (New-Object Text.UTF8Encoding($false)))
