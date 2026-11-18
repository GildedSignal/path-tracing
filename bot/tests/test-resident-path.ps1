$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$paths = @('bot/src/pathtracing/ResidentRunner.java','bot/src/pathtracing/NativeChannel.java','bot/src/pathtracing/ResidentSource.java','engine/native/src/resident.rs')
$forbidden = 'ProcessBuilder|Runtime\.getRuntime\(\)\.exec|plan-track|playPlan\(|write_script|writeScript|Thread\.sleep|Start-Sleep|ReadToEnd|from_secs|settle\('
foreach ($path in $paths) {
    $hits = Select-String -LiteralPath (Join-Path $root $path) -Pattern $forbidden
    if ($hits) { throw "Old click path in ${path}: $hits" }
}
$bot = Get-Content -Raw -LiteralPath "$root/bot/src/pathtracing/Bot.java"
if ($bot -match 'StraightLine|PLANNER\.plan|ProcessBuilder' -or !( $bot -match 'ResidentRunner\.request')) { throw 'Bot does not exclusively dispatch resident goals' }
if (Test-Path "$root/bot/src/pathtracing/planner/StraightLine.java") { throw 'StraightLine still exists' }
$command = [regex]::Match($bot,'private static void command\(String text\) \{\s*([^\r\n]+)').Groups[1].Value
if ($command -notmatch 'NativeChannel\.clock') { throw 'Command stamp is not the first statement' }
$pointer = Get-Content -Raw -LiteralPath "$root/bot/src/pathtracing/Pointer.java"
$click = [regex]::Match($pointer,'while \(mc\.options\.keyUse\.consumeClick\(\)\) \{\s*([^\r\n]+)').Groups[1].Value
if ($click -notmatch 'NativeChannel\.clock') { throw 'Issuance stamp is not the first click statement' }
$worker = Get-Content -Raw -LiteralPath "$root/engine/native/src/resident.rs"
if ($worker -notmatch 'name\("pathtracing-F"' -or $worker -notmatch 'Work::Goal\(request\)' -or $worker -notmatch 'start: state\.clone\(\)') { throw 'Observed F worker is missing' }
if ($worker -match 'WorldSnapshot::load|Command::new|std::process|std::fs::write') { throw 'Per-goal disk/process path found' }
Write-Output 'PASS: resident click path has no settle/process/log/script dispatch; stamps precede per-command work.'
