# Stores Julien's Z.ai API key for GLM (run through omp), safely, in one click.
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\set-glm-key.ps1          ask for the key and store it
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\set-glm-key.ps1 -Check   say whether a key is stored
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\set-glm-key.ps1 -Remove  delete the stored key
# The key is typed into a Windows dialog with a hidden field, never into a terminal or a chat. It is saved encrypted
# with DPAPI, so only this Windows account on this machine can decrypt it, outside the repository. Nothing prints it.
# scripts\common.ps1's Get-GlmKey decrypts it only to hand it to an omp process's environment.
# GLM runs on GitHub's machines (S21), so the script also sets the repository's Actions secret ZAI_API_KEY.
param([switch]$Check, [switch]$Remove)
$ErrorActionPreference = 'Stop'
$dir = Join-Path $env:LOCALAPPDATA 'path-tracing'
$file = Join-Path $dir 'zai-key.xml'

if ($Check) {
  if (Test-Path $file) { "A GLM key is stored for $env:USERNAME (saved $((Get-Item $file).LastWriteTime))." } else { 'No GLM key is stored.' }
  $listed = & gh secret list --repo GildedSignal/path-tracing 2>$null | Select-String -SimpleMatch 'ZAI_API_KEY'
  if ($listed) { 'The Actions secret ZAI_API_KEY is set.' } else { 'The Actions secret ZAI_API_KEY is not set.' }
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
"Stored: the GLM key is saved, encrypted, for $env:USERNAME only."

# GLM runs on GitHub's machines (S21): the same key becomes the repository's Actions secret. gh encrypts it with the
# repository's public key before it leaves this machine; it is passed through stdin, never on a command line.
$repo = 'GildedSignal/path-tracing'
$gh = Get-Command gh -ErrorAction SilentlyContinue
if (-not $gh) { "GitHub CLI not found: the Actions secret was not set."; return }
$cred.GetNetworkCredential().Password | & $gh.Source secret set ZAI_API_KEY --repo $repo
if ($LASTEXITCODE -eq 0) { "Set: ZAI_API_KEY is now an Actions secret of $repo. You can close this window." }
else { "The Actions secret was not set (gh exit $LASTEXITCODE). The local copy is stored." }
