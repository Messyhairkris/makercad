# Keeps Windows from idle-sleeping while long MakerCAD work runs, then lets go.
# It asks Windows the way a video player does (SetThreadExecutionState); no power setting changes.
# The screen may still turn off; a closed lid or choosing Sleep still sleeps.
# Exits when every watched file exists (the work finished) or after -MaxHours.
param(
    [string[]]$UntilFiles = @(
        "$PSScriptRoot\..\target\p0\build.done",
        "$env:USERPROFILE\ATLAS-api-harness\runs\crew\campaigns\mk-core-1\summary.json"
    ),
    [double]$MaxHours = 14
)
Add-Type -Namespace MakerCad -Name Power -MemberDefinition @'
[DllImport("kernel32.dll")] public static extern uint SetThreadExecutionState(uint esFlags);
'@
$ES_CONTINUOUS = [uint32]'0x80000000'
$ES_SYSTEM_REQUIRED = [uint32]'0x00000001'
$log = Join-Path $PSScriptRoot '..\target\p0\keep_awake.log'
$stop = (Get-Date).AddHours($MaxHours)
[void][MakerCad.Power]::SetThreadExecutionState($ES_CONTINUOUS -bor $ES_SYSTEM_REQUIRED)
"[$(Get-Date -Format s)] holding the PC awake until: $($UntilFiles -join ', ') (max $MaxHours h)" | Out-File $log -Append -Encoding utf8
while ((Get-Date) -lt $stop) {
    if (-not ($UntilFiles | Where-Object { -not (Test-Path $_) })) { break }
    Start-Sleep -Seconds 60
}
[void][MakerCad.Power]::SetThreadExecutionState($ES_CONTINUOUS)
"[$(Get-Date -Format s)] released" | Out-File $log -Append -Encoding utf8
