param([string]$Class, [string[]]$Methods)
$lines = Get-Content (Join-Path $PSScriptRoot "../.bytecode/$Class.txt")
$printing = $false
foreach ($line in $lines) {
    if ($line -match '^  \S.*[;{]$') {
        $printing = $false
        foreach ($method in $Methods) {
            if ($line -match (' ' + [regex]::Escape($method) + '\(')) { $printing = $true }
        }
    }
    if ($printing) { $line }
}
