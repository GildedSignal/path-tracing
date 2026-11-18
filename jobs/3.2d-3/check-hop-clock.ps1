param([Parameter(Mandatory)][string]$Capture,[switch]$Baseline)
$ErrorActionPreference = 'Stop'
$rows = @(Get-ChildItem -LiteralPath $Capture -Filter 'hops-*.tsv' | ForEach-Object { Get-Content -LiteralPath $_.FullName } | ForEach-Object {
    $p = $_ -split "`t",6
    if ($p.Count -ne 6) { throw 'Malformed hop row' }
    [pscustomobject]@{kind=$p[0];version=[long]$p[1];issue=[long]$p[2];qpc=[long]$p[3];tick=[long]$p[4];data=$p[5]}
})
$starts = @($rows | Where-Object kind -eq 'server-tick-start' | Sort-Object qpc)
$issues = @($rows | Where-Object kind -eq 'issuance')
if ($starts.Count -lt 2 -or !$issues.Count) { throw 'Missing actual issuance or server tick-start QPC samples' }
foreach ($issue in $issues) {
    $interval = @($starts | Where-Object qpc -le $issue.qpc | Select-Object -Last 1)
    $next = @($starts | Where-Object qpc -gt $issue.qpc | Select-Object -First 1)
    if (!$interval.Count -or !$next.Count) { throw 'Issuance is outside observed server tick intervals' }
    Write-Output "Raw issuance $($issue.qpc) belongs to actual server tick $($interval[0].tick), bounded by $($interval[0].qpc)..$($next[0].qpc)."
}
$sends = @($rows | Where-Object { $_.kind -like 'packet-send:*' -and $_.kind -match 'ServerboundMovePlayerPacket|ServerboundPlayerInputPacket|ServerboundPlayerCommandPacket|ServerboundPlayerActionPacket|ServerboundUseItem|ServerboundSetCarriedItem' })
if (!$sends.Count) { throw 'No real input packets captured; marker receipt is insufficient' }
foreach ($send in $sends) {
    $tag = $send.kind.Substring('packet-send:'.Length)
    $receipt = @($rows | Where-Object kind -eq "packet-receive:$tag")
    $apply = @($rows | Where-Object kind -eq "packet-apply:$tag")
    if ($receipt.Count -ne 1 -or $apply.Count -ne 1) { throw "Missing/duplicate real input receipt or application: $tag" }
    if ($send.qpc -gt $receipt[0].qpc -or $receipt[0].qpc -gt $apply[0].qpc) { throw "Non-causal packet stamps: $tag" }
}
$required = if ($Baseline) { @('chat-mod','legacy-rcon-send','legacy-rcon-receive','legacy-rcon-applied','legacy-planner-start','legacy-planner-end','arrival','restoration-closed') } else { @('chat-mod','mod-request','rust-result-mod','arrival','restoration-closed') }
foreach ($hop in $required) {
    if (!@($rows | Where-Object kind -eq $hop).Count) { throw "Missing required hop $hop" }
}
Write-Output 'HopClock complete: raw origin preserved; every real input has receipt and game-thread application.'
