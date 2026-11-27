[CmdletBinding()]
param(
    [string]$Commands,
    [int[]]$Box,
    [string]$Views,
    [Parameter(Mandatory=$true)][string]$Output,
    [string]$Jar,
    [string]$McVersion = $env:PATHTRACING_MC_VERSION,
    [switch]$Fixture
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1" -McVersion $McVersion
if (-not $Jar) { $Jar = Get-VerifiedClientJar }
$previewJava = Join-Path $Jdk 'java.exe'
if (-not (Test-Path -LiteralPath $previewJava)) {
    $previewJava = 'D:\path-tracing-deps\data\java\java-runtime-epsilon\bin\java.exe'
}
if (-not (Test-Path -LiteralPath $previewJava)) { throw "Rig JDK not found: $previewJava" }
$previewArgs = @('-Djava.awt.headless=true', (Join-Path $Root 'tools\preview\Preview.java'))
if ($Fixture) { $previewArgs += @('--fixture', $Jar, $Output) }
else {
    if (-not $Commands -or -not $Views -or $Box.Count -ne 6) { throw 'Use -Commands <file> -Box @(x0,y0,z0,x1,y1,z1) -Views <file> -Output <folder>.' }
    $previewArgs += @($Commands) + @($Box | ForEach-Object { $_.ToString([Globalization.CultureInfo]::InvariantCulture) }) + @($Jar, $Views, $Output)
}
# Windows command-line quoting (including trailing backslashes); no shell evaluates arguments.
function Quote-PreviewArgument([string]$Value) {
    '"' + [regex]::Replace([regex]::Replace($Value, '(\\*)"', '$1$1\"'), '(\\+)$', '$1$1') + '"'
}
$previewInfo = [Diagnostics.ProcessStartInfo]::new()
$previewInfo.FileName = $previewJava
$previewInfo.Arguments = ($previewArgs | ForEach-Object { Quote-PreviewArgument $_ }) -join ' '
$previewInfo.WorkingDirectory = $Root
$previewInfo.UseShellExecute = $false
$previewInfo.CreateNoWindow = $true
$previewInfo.RedirectStandardOutput = $true
$previewInfo.RedirectStandardError = $true
$previewProcess = [Diagnostics.Process]::Start($previewInfo)
try {
    if (-not $previewProcess.HasExited) { $previewProcess.PriorityClass = [Diagnostics.ProcessPriorityClass]::Normal }
    $previewPriority = $previewProcess.PriorityClass.ToString()
    $previewStdout = $previewProcess.StandardOutput.ReadToEndAsync()
    $previewStderr = $previewProcess.StandardError.ReadToEndAsync()
    $previewProcess.WaitForExit()
    if ($previewStdout.Result) { Write-Output $previewStdout.Result.TrimEnd() }
    if ($previewStderr.Result) { Write-Output $previewStderr.Result.TrimEnd() }
    $previewOutput = if ([IO.Path]::IsPathRooted($Output)) { $Output } else { Join-Path $Root $Output }; New-Item -ItemType Directory -Force $previewOutput | Out-Null
    [IO.File]::WriteAllLines((Join-Path $previewOutput 'wrapper-run.txt'), [string[]]@("priority=$previewPriority", "exit=$($previewProcess.ExitCode)"), [Text.UTF8Encoding]::new($false))
    if ($previewProcess.ExitCode -ne 0) { throw "Preview exited with code $($previewProcess.ExitCode)." }
} finally { $previewProcess.Dispose() }
