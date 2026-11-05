# Stores Julien's Z.ai API key for GLM (run through omp), safely, in one click.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\set-glm-key.ps1          ask for the key and store it
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\set-glm-key.ps1 -Check   say whether a key is stored
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\set-glm-key.ps1 -Remove  delete the stored key
# The key is typed into a Windows dialog with a hidden field, never into a terminal or a chat. It is saved encrypted
# with DPAPI, so only this Windows account on this machine can decrypt it, outside the repository. Nothing prints it.
# scripts\common.ps1's Get-GlmKey decrypts it only to hand it to an omp process's environment.
param([switch]$Check, [switch]$Remove)
$ErrorActionPreference = 'Stop'
$dir = Join-Path $env:LOCALAPPDATA 'path-tracing'
$file = Join-Path $dir 'zai-key.xml'

if ($Check) {
  if (Test-Path $file) { "A GLM key is stored for $env:USERNAME (saved $((Get-Item $file).LastWriteTime))." } else { 'No GLM key is stored.' }
  return
}
if ($Remove) {
  if (Test-Path $file) { Remove-Item $file; 'The GLM key was deleted.' } else { 'No GLM key was stored.' }
  return
}

$cred = Get-Credential -UserName 'zai' -Message 'Paste your Z.ai API key into the password field, then press OK. It is stored encrypted for your Windows account only.'
if (-not $cred -or $cred.Password.Length -eq 0) { 'Cancelled: nothing was stored.'; return }
if ($cred.Password.Length -lt 20) { 'That looks too short to be an API key: nothing was stored.'; return }
New-Item -ItemType Directory -Force $dir | Out-Null
$cred | Export-Clixml -Path $file   # SecureString inside is DPAPI-encrypted for this user
# Only this user (and SYSTEM) may read the file.
$acl = New-Object Security.AccessControl.FileSecurity
$acl.SetAccessRuleProtection($true, $false)
foreach ($who in @("$env:USERDOMAIN\$env:USERNAME", 'NT AUTHORITY\SYSTEM')) {
  $acl.AddAccessRule((New-Object Security.AccessControl.FileSystemAccessRule($who, 'FullControl', 'Allow')))
}
Set-Acl -Path $file -AclObject $acl
"Stored: the GLM key is saved, encrypted, for $env:USERNAME only. You can close this window."
