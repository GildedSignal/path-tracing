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

$left = $null
$right = $null
Enter-Rig 'compare-recordings'
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
    if ($null -eq $lineA -and $null -eq $lineB) { break }
    if ($null -eq $lineA -or $null -eq $lineB) { throw "First difference at tick ${tick}: recording length." }
    $rowA = $lineA | ConvertFrom-Json
    $rowB = $lineB | ConvertFrom-Json
    if ($null -eq $rowA.t -or $null -eq $rowB.t -or $rowA.t -ne $tick -or $rowB.t -ne $tick) {
      throw "First difference at tick ${tick}: invalid tick stamps."
    }
    $fieldsA = @(Get-RawFields $rowA)
    $fieldsB = @(Get-RawFields $rowB)
    if ($fieldsA.Count -eq 0 -or $fieldsB.Count -eq 0) { throw "Missing raw-bit fields at tick $tick." }
    $limit = [Math]::Max($fieldsA.Count, $fieldsB.Count)
    for ($i = 0; $i -lt $limit; $i++) {
      if ($fieldsA[$i] -cne $fieldsB[$i]) {
        throw "First difference at tick ${tick}: $($fieldsA[$i]) versus $($fieldsB[$i])."
      }
    }
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
  Exit-Rig
}
