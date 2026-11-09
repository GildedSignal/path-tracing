param(
    [Parameter(Mandatory)][string]$Edges20,
    [Parameter(Mandatory)][string]$Edges200,
    [Parameter(Mandatory)][string]$Teleport
)

$ErrorActionPreference = 'Stop'
$checker = (Resolve-Path "$PSScriptRoot/../../lab/gates/check-recording.ps1").Path
$powershell = (Get-Command powershell.exe).Source

function Assert-Refused([string]$File, [string[]]$Anomalies) {
    $result = @(& $powershell -NoProfile -ExecutionPolicy Bypass -File $checker $File)
    if ($LASTEXITCODE -ne 1 -or $result.Count -ne $Anomalies.Count) {
        throw "Unexpected result for ${File}: $result"
    }
    for ($i = 0; $i -lt $Anomalies.Count; $i++) {
        if ($result[$i] -cnotmatch ('^Anomaly at line \d+, ' + $Anomalies[$i])) {
            throw "Unexpected refusal for ${File}: $($result[$i])"
        }
    }
    Write-Output "$File refused with $($Anomalies.Count) expected anomaly rows."
}

$motion = '.*"type":"ClientboundSetEntityMotionPacket"'
$witness = "tick 174:$motion"
Assert-Refused $Edges20 @($witness)
Assert-Refused $Edges200 @($witness)
Assert-Refused $Teleport @('tick \d+:.*"type":"ClientboundPlayerPositionPacket"')

$lines = [IO.File]::ReadAllLines((Resolve-Path -LiteralPath $Edges20).Path)
$packet = [regex]::Match($lines[175], '"anomaly":(\[.*\])\}$')
if (-not $packet.Success -or $lines[176] -cnotmatch '^\{"t":175,' -or $lines[176] -cmatch '"anomaly":') {
    throw 'Missing clean target row or motion payload for the wrong-tick control.'
}
$lines[176] = $lines[176].Substring(0, $lines[176].Length - 1) + ',' + $packet.Value
$scratch = [IO.Path]::GetTempFileName()
try {
    [IO.File]::WriteAllLines($scratch, $lines)
    Assert-Refused $scratch @($witness, "tick 175:$motion")
} finally {
    Remove-Item -LiteralPath $scratch
}
