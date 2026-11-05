# Exercise the actual driver's pure predicates without loading the rig.
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile("$PSScriptRoot\..\record-fixtures.ps1", [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors -join "`n") }
foreach ($name in @('Axis-TieCollision', 'Signed-TickOffset', 'At-Rest', 'Require-StableResources')) {
  $function = $ast.Find({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq $name }, $true)
  . ([scriptblock]::Create($function.Extent.Text))
}
function Assert([bool]$Value, [string]$Message) { if (-not $Value) { throw $Message } }
$one = '3ff0000000000000'; $minusOne = 'bff0000000000000'; $two = '4000000000000000'; $zero = '0000000000000000'
$cases = @(
  @($one, $two, $zero, $two, $false),
  @($one, $minusOne, $one, $zero, $true),
  @($minusOne, $minusOne, $zero, $minusOne, $true),
  @($one, $one, $one, $one, $false),
  @($one, $one, $zero, $zero, $false),
  @($one, $one, $one, $zero, $true),
  @($one, $one, $zero, $one, $true),
  @($zero, '8000000000000000', $one, $zero, $false)
)
function Check-Witness([scriptblock]$Predicate) {
  foreach ($case in $cases) {
    $step = [pscustomobject]@{ requested = @{ xBits=$case[0]; zBits=$case[1] }; result = @{ xBits=$case[2]; zBits=$case[3] } }
    Assert ((& $Predicate $step) -eq $case[4]) "Wrong corner witness for $($case -join ',')."
  }
}
Check-Witness { param($step) Axis-TieCollision $step }
foreach ($mutant in @({ param($step) $true }, { param($step) $false })) {
  $caught = $false
  try { Check-Witness $mutant } catch { $caught = $true }
  Assert $caught 'Constant witness mutant survived.'
}
Assert ((Signed-TickOffset 97 100) -eq -3) 'Negative offset wrapped.'
Assert ((Signed-TickOffset 104 100) -eq 4) 'Positive offset changed.'
Assert ((Signed-TickOffset 2 2147483646) -eq 3) 'Forward wrap changed.'
Assert ((Signed-TickOffset 2147483646 2147483649) -eq -3) 'Backward wrap changed.'
Assert (At-Rest @{ onGround=$true; velocity=@{ x=0; z=0 } }) 'Rest rejected.'
Assert (-not (At-Rest @{ onGround=$false; velocity=@{ x=0; z=0 } })) 'Airborne accepted as rest.'
Assert (-not (At-Rest @{ onGround=$true; velocity=@{ x=0.1; z=0 } })) 'Motion accepted as rest.'
Assert (-not (At-Rest @{ onGround=$true })) 'Missing velocity accepted as rest.'
$first = [pscustomobject]@{ Health=20; foodLevel=20; foodSaturationLevel=5; foodExhaustionLevel=0; foodTickTimer=0 }
Require-StableResources $first $first
foreach ($field in @('Health', 'foodLevel', 'foodSaturationLevel', 'foodExhaustionLevel', 'foodTickTimer')) {
  $second = $first | ConvertTo-Json | ConvertFrom-Json
  $second.$field += 1
  $caught = $false
  try { Require-StableResources $first $second } catch { $caught = $true }
  Assert $caught "Changing $field accepted."
}
$second = $first | ConvertTo-Json | ConvertFrom-Json
$second.foodExhaustionLevel = [BitConverter]::ToSingle([BitConverter]::GetBytes([uint32]2147483648), 0)
$caught = $false
try { Require-StableResources $first $second } catch { $caught = $true }
Assert $caught 'Signed-zero exhaustion bits were treated as identical.'
Write-Output 'PASS fixture predicates, constant witness mutants, signed offsets, rest and every start resource field.'
