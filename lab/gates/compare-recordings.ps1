param(
  [Parameter(Mandatory = $true, Position = 0)][string]$A,
  [Parameter(Mandatory = $true, Position = 1)][string]$B
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\..\scripts\common.ps1"

function Get-RawFields($Value, [string]$Path = '') {
  if ($null -eq $Value) { return }
  if ($Value -is [System.Array]) {
    for ($i = 0; $i -lt $Value.Count; $i++) { Get-RawFields $Value[$i] "$Path[$i]" }
  } elseif ($Value -is [pscustomobject]) {
    foreach ($property in ($Value.PSObject.Properties | Sort-Object Name)) {
      $child = "$Path.$($property.Name)"
      if ($property.Name -clike '*Bits') {
        if ($property.Value -isnot [string] -or $property.Value -cnotmatch '^(?:[0-9a-f]{8}|[0-9a-f]{16})$') {
          throw "Invalid raw-bit field $child."
        }
        "$child=$($property.Value)"
      } else { Get-RawFields $property.Value $child }
    }
  }
}

function Find-Difference($LeftValue, $RightValue, [string]$Path = '') {
  if ($null -eq $LeftValue -and $null -eq $RightValue) { return }
  if ($null -eq $LeftValue -or $null -eq $RightValue) { return "$Path (null or missing)" }
  if ($LeftValue.GetType() -ne $RightValue.GetType()) { return "$Path (value type)" }
  if ($LeftValue -is [System.Array]) {
    if ($LeftValue.Count -ne $RightValue.Count) { return "$Path.Length" }
    for ($i = 0; $i -lt $LeftValue.Count; $i++) {
      $difference = Find-Difference $LeftValue[$i] $RightValue[$i] "$Path[$i]"
      if ($null -ne $difference) { return $difference }
    }
  } elseif ($LeftValue -is [pscustomobject]) {
    $names = @(@($LeftValue.PSObject.Properties.Name) + @($RightValue.PSObject.Properties.Name) |
      Sort-Object -Unique -CaseSensitive)
    foreach ($name in $names) {
      $leftProperty = $LeftValue.PSObject.Properties[$name]
      $rightProperty = $RightValue.PSObject.Properties[$name]
      $child = "$Path.$name"
      if ($null -eq $leftProperty -or $null -eq $rightProperty -or
          $leftProperty.Name -cne $rightProperty.Name) { return "$child (missing field)" }
      $difference = Find-Difference $leftProperty.Value $rightProperty.Value $child
      if ($null -ne $difference) { return $difference }
    }
  } elseif ($LeftValue -cne $RightValue) { return "$Path ($LeftValue versus $RightValue)" }
}

$left = $null
$right = $null
try {
  $left = [IO.File]::OpenText((Resolve-Path -LiteralPath $A).Path)
  $right = [IO.File]::OpenText((Resolve-Path -LiteralPath $B).Path)
  $headers = @($left.ReadLine() | ConvertFrom-Json; $right.ReadLine() | ConvertFrom-Json)
  foreach ($header in $headers) {
    if ($header.type -ne 'recording') { throw 'Missing recording header.' }
  }
  foreach ($key in @('toggleSprint', 'toggleCrouch', 'autoJump', 'sprintWindow')) {
    if ($null -eq $headers[0].PSObject.Properties[$key] -or $null -eq $headers[1].PSObject.Properties[$key] -or
        $headers[0].$key -ne $headers[1].$key) { throw "Header differs: $key." }
  }
  $tick = 0
  while ($true) {
    $lineA = $left.ReadLine()
    $lineB = $right.ReadLine()
    if ($null -eq $lineA -and $null -eq $lineB) { throw 'Missing end trailers.' }
    if ($null -eq $lineA -or $null -eq $lineB) { throw "First difference at tick ${tick}: recording length." }
    $rowA = $lineA | ConvertFrom-Json
    $rowB = $lineB | ConvertFrom-Json
    if ($rowA.type -eq 'end' -or $rowB.type -eq 'end') {
      $difference = Find-Difference $rowA $rowB 'end'
      if ($null -ne $difference) { throw "First difference at tick ${tick}: $difference." }
      if ($null -eq $rowA.ticks -or $rowA.ticks -is [string] -or $rowA.ticks -ne $tick -or
          $rowA.complete -isnot [bool]) { throw "Invalid end trailer at tick $tick." }
      if ($null -ne $left.ReadLine() -or $null -ne $right.ReadLine()) { throw 'End trailer must be the final line.' }
      break
    }
    if ($null -eq $rowA.t -or $null -eq $rowB.t -or $rowA.t -ne $tick -or $rowB.t -ne $tick) {
      throw "First difference at tick ${tick}: invalid tick stamps."
    }
    $fieldsA = @(Get-RawFields $rowA)
    $fieldsB = @(Get-RawFields $rowB)
    if ($fieldsA.Count -eq 0 -or $fieldsB.Count -eq 0) { throw "Missing raw-bit fields at tick $tick." }
    $rowA.PSObject.Properties.Remove('tickCount')
    $rowB.PSObject.Properties.Remove('tickCount')
    $difference = Find-Difference $rowA $rowB
    if ($null -ne $difference) { throw "First difference at tick ${tick}: $difference." }
    $tick++
  }
  if ($tick -eq 0) { throw 'No recorded ticks.' }
  Write-Output 'identical'
} catch {
  Write-Output $_.Exception.Message
  exit 1
} finally {
  if ($null -ne $left) { $left.Dispose() }
  if ($null -ne $right) { $right.Dispose() }
}
