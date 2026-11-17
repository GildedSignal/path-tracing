# Keep the frozen verifier unchanged and expose its own evidence directory.
. (Join-Path $PSScriptRoot '../1.2-footprints/verify.ps1')
Write-Output "U1_FOOTPRINT_OUTPUT=$output"
