# Starts the server, the bot's client (moved to your second monitor) and your client, both joining the server.
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\scripts\common.ps1"

if (-not (Test-Path "$Rig\clients\bot\mods\pathtracing.jar")) { & "$Root\scripts\setup.ps1" }

& "$Root\scripts\start-server.ps1"
Write-Host 'Waiting for the server...'
$deadline = (Get-Date).AddMinutes(3)
while (-not (Test-Port $ServerHost $ServerPort)) {
  if ((Get-Date) -gt $deadline) { throw 'Server did not open its port within 3 minutes; check the server window.' }
  Start-Sleep -Seconds 1
}

$bot = & "$Root\scripts\launch-client.ps1" -Role bot
$null = & "$Root\scripts\launch-client.ps1" -Role player

# Put the bot's window on the first non-primary monitor, maximised.
Add-Type -AssemblyName System.Windows.Forms
$second = [Windows.Forms.Screen]::AllScreens | Where-Object { -not $_.Primary } | Select-Object -First 1
if (-not $second) { Write-Host 'Only one monitor found; leaving the bot window where it opens.'; return }
Add-Type -Namespace Win32 -Name Window -MemberDefinition @'
[DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int hgt, uint flags);
[DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
'@
$deadline = (Get-Date).AddMinutes(2)
do { Start-Sleep -Milliseconds 500; $bot.Refresh() } until ($bot.MainWindowHandle -ne 0 -or $bot.HasExited -or (Get-Date) -gt $deadline)
if ($bot.MainWindowHandle -ne 0) {
  $b = $second.WorkingArea
  [Win32.Window]::ShowWindow($bot.MainWindowHandle, 9) | Out-Null  # restore, so it can be moved
  [Win32.Window]::SetWindowPos($bot.MainWindowHandle, [IntPtr]::Zero, $b.X + 50, $b.Y + 50, 1280, 720, 0x0014) | Out-Null
  [Win32.Window]::ShowWindow($bot.MainWindowHandle, 3) | Out-Null  # maximise
}
