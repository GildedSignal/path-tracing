# Sends server commands over RCON (localhost only) and prints the replies.
#   .\rcon.ps1 "time set noon" "fill 0 -60 0 10 -58 10 stone"
[CmdletBinding(PositionalBinding = $false)]
param(
  [Parameter(Mandatory, ValueFromRemainingArguments)][string[]]$Commands,
  [ValidateRange(1, 60000)][int]$SocketTimeoutMilliseconds
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\common.ps1"

$password = ((Get-Content "$Rig\server\server.properties") -match '^rcon\.password=' -replace '^rcon\.password=', '')
$client = [Net.Sockets.TcpClient]::new($ServerHost, $RconPort)
if ($SocketTimeoutMilliseconds) {
  $client.ReceiveTimeout = $SocketTimeoutMilliseconds
  $client.SendTimeout = $SocketTimeoutMilliseconds
}
$stream = $client.GetStream()
$reader = [IO.BinaryReader]::new($stream)

function Send-Packet([int]$id, [int]$type, [string]$body) {
  $bytes = [Text.Encoding]::UTF8.GetBytes($body)
  $packet = [IO.MemoryStream]::new()
  $w = [IO.BinaryWriter]::new($packet)
  $w.Write([int](10 + $bytes.Length)); $w.Write($id); $w.Write($type); $w.Write($bytes); $w.Write([byte]0); $w.Write([byte]0)
  $stream.Write($packet.ToArray(), 0, $packet.Length)
}
function Read-Packet {
  $length = $reader.ReadInt32()
  $id = $reader.ReadInt32(); $null = $reader.ReadInt32()
  $body = [Text.Encoding]::UTF8.GetString($reader.ReadBytes($length - 10)); $null = $reader.ReadBytes(2)
  [pscustomobject]@{ Id = $id; Body = $body }
}

try {
  Send-Packet 1 3 $password
  if ((Read-Packet).Id -eq -1) { throw 'RCON password rejected' }
  $i = 2
  foreach ($c in $Commands) {
    Send-Packet $i 2 $c
    $reply = Read-Packet
    if ($reply.Body) { $reply.Body }
    $i++
  }
} finally { $client.Dispose() }
