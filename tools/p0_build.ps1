# P0: build stock SolidCraft (pinned checkout) on this machine and record how long it took.
# Runs detached; writes target\p0\build.log and target\p0\build.done (exit code + minutes).
#
# -LowMemory: for 8 GB machines. The first run (2026-11-19) stalled for hours on solidcore, the
# kernel (one 41 MB crate), which paged on a spinning disk. This builds one crate at a time and
# compiles solidcore with less optimisation (opt-level 1 instead of 2), which needs less memory
# and time. Measured frame rates from such a build say so in STATUS.md.
param([switch]$LowMemory)
$ErrorActionPreference = 'Continue'
$repo = Split-Path -Parent $PSScriptRoot
$out = Join-Path $repo 'target\p0'
New-Item -ItemType Directory -Force $out | Out-Null
$log = Join-Path $out 'build.log'
Remove-Item (Join-Path $out 'build.done') -ErrorAction SilentlyContinue
Set-Location (Join-Path $repo 'third_party\solidcraft')
$env:CARGO_INCREMENTAL = '0'
$t0 = Get-Date
$jobs = if ($LowMemory) { 1 } else { 3 }
$extra = if ($LowMemory) { @('--config', 'profile.dev.package.solidcore.opt-level=1') } else { @() }
"[$(Get-Date -Format s)] cargo build -p solidcraft -j $jobs $($extra -join ' ') (rev $(git rev-parse --short HEAD))" | Out-File $log -Append -Encoding utf8
cargo build -p solidcraft -j $jobs @extra *>> $log
$code = $LASTEXITCODE
$min = [math]::Round(((Get-Date) - $t0).TotalMinutes, 1)
$size = [math]::Round(((Get-ChildItem target -Recurse -Force -File -ErrorAction SilentlyContinue | Measure-Object Length -Sum).Sum / 1GB), 2)
$free = [math]::Round((Get-PSDrive C).Free / 1GB, 2)
"exit=$code minutes=$min target_gb=$size c_free_gb=$free low_memory=$([bool]$LowMemory)" | Out-File (Join-Path $out 'build.done') -Encoding utf8
"[$(Get-Date -Format s)] done: exit=$code minutes=$min target_gb=$size c_free_gb=$free" | Out-File $log -Append -Encoding utf8
