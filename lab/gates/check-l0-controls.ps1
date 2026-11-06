# Qualifies the movement controls on saved vanilla recording/world pairs, independently of L0 resources.
param(
  [Parameter(Mandatory)][string]$Corner,
  [Parameter(Mandatory)][string]$StepBelow,
  [Parameter(Mandatory)][string]$StepEqual,
  [Parameter(Mandatory)][string]$Ceiling,
  [Parameter(Mandatory)][string]$SneakEdge,
  [string]$Revision = 'HEAD',
  [string]$RustHome = 'D:\rust'
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath "$PSScriptRoot\..\..").Path
$commit = git -C $root rev-parse --verify ($Revision + '^{commit}')
if ($LASTEXITCODE) { throw "Invalid engine revision $Revision." }
$stamp = [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ')
$result = Join-Path $PSScriptRoot "results\$stamp-l0-controls"
$cases = @(
  @{ Name = 'ceiling'; Source = $Ceiling; Mutant = 'ceiling'; FirstDifference = 'tick 0 velocity\.y:' },
  @{ Name = 'corner'; Source = $Corner; Mutant = 'axis-tie'; FirstDifference = 'tick 21 position\.x:' },
  @{ Name = 'edge'; Source = $SneakEdge; Mutant = 'edge-backoff'; FirstDifference = 'tick 26 position\.z:' },
  @{ Name = 'step-below'; Source = $StepBelow; Mutant = 'step-up'; FirstDifference = 'tick 6 position\.y:' },
  @{ Name = 'step-equal'; Source = $StepEqual }
)
$fixtures = Join-Path $result 'lab\fixtures'
$null = New-Item -ItemType Directory -Path $fixtures -Force
git -C $root archive --format=zip "--output=$result\engine.zip" $commit engine
if ($LASTEXITCODE) { throw "Cannot archive engine $commit." }
Expand-Archive -LiteralPath "$result\engine.zip" -DestinationPath $result
$hashes = foreach ($case in $cases) {
  $source = (Resolve-Path -LiteralPath $case.Source).Path
  $pair = [IO.Path]::ChangeExtension($source, 'world.jsonl')
  Copy-Item -LiteralPath $source -Destination (Join-Path $fixtures ($case.Name + '.jsonl'))
  Copy-Item -LiteralPath $pair -Destination (Join-Path $fixtures ($case.Name + '.world.jsonl'))
  Get-FileHash -LiteralPath $source, $pair
}
$hashes | ConvertTo-Json | Set-Content -LiteralPath "$result\fixture-sha256.json" -Encoding ASCII
$commit | Set-Content -LiteralPath "$result\engine-commit.txt" -Encoding ASCII

function Invoke-Replay([string]$Log, [string]$Mutant) {
  $cargoArgs = @('test', '--offline', '--release', '--test', 'replay')
  if ($Mutant) { $cargoArgs += @('--features', "mutant-$Mutant") }
  $cargoArgs += @('replay', '--', '--exact', '--nocapture')
  # Windows PowerShell 5.1 turns Cargo's stderr into ErrorRecords even on success.
  $ErrorActionPreference = 'Continue'
  & "$RustHome\cargo\bin\cargo.exe" @cargoArgs 2>&1 | Out-File -LiteralPath $Log -Encoding UTF8
  $LASTEXITCODE
}

$priorTarget = $env:CARGO_TARGET_DIR
$priorCargo = $env:CARGO_HOME
$priorRustup = $env:RUSTUP_HOME
Push-Location "$result\engine"
try {
  $env:CARGO_TARGET_DIR = Join-Path $result 'target'
  $env:CARGO_HOME = Join-Path $RustHome 'cargo'
  $env:RUSTUP_HOME = Join-Path $RustHome 'rustup'
  if ((Invoke-Replay "$result\baseline.log" '') -ne 0) { throw "Baseline failed. See $result\baseline.log." }
  foreach ($case in $cases | Where-Object Mutant) {
    $log = "$result\mutant-$($case.Mutant).log"
    $exitCode = Invoke-Replay $log $case.Mutant
    $details = Get-Content -LiteralPath $log -Raw
    $lastReplay = [regex]::Matches($details, '(?m)^Replaying [^\r\n]+') | Select-Object -Last 1
    $witness = $case.Name + '.jsonl'
    if ($exitCode -ne 101 -or $null -eq $lastReplay -or -not $lastReplay.Value.EndsWith("\$witness") -or
        $details -notmatch $case.FirstDifference) {
      throw "Mutant $($case.Mutant) survived or failed outside its witness. See $log."
    }
    "PASS mutant-$($case.Mutant) on $witness"
  }
  "PASS: baseline and all four movement controls. Evidence: $result"
} finally {
  Pop-Location
  $env:CARGO_TARGET_DIR = $priorTarget
  $env:CARGO_HOME = $priorCargo
  $env:RUSTUP_HOME = $priorRustup
}
