# Builds a MakerCAD package the way an 8 GB machine survives: one crate at a time, no incremental
# caches (Cargo.toml keeps solidcore at opt-level 1). Writes target\lowmem\<package>\build.log and
# build.done (exit code and minutes), like tools/p0_build.ps1. Watch it with
#   pwsh tools/p0_watch.ps1 -Dir target\lowmem\<package>
#
# Why this exists: Cargo hashes a path dependency by its path relative to the workspace root, so
# SolidCraft's crates compiled in SolidCraft's own workspace (the P0 build) are never reused by
# ours. The first build of mk-desktop with SolidCraft's crates compiles them all again, solidcore
# included: hours on the reference machine. Later builds reuse that.
param([string]$Package = 'mk-desktop', [int]$Jobs = 1)
$ErrorActionPreference = 'Continue'
$repo = Split-Path -Parent $PSScriptRoot
$out = Join-Path $repo "target\lowmem\$Package"
New-Item -ItemType Directory -Force $out | Out-Null
$log = Join-Path $out 'build.log'
$done = Join-Path $out 'build.done'
if (Test-Path $done) { Rename-Item $done "build.done.$(Get-Date -Format yyyyMMddHHmmss)" }
Set-Location $repo
$t0 = Get-Date
"[$(Get-Date -Format s)] cargo build -p $Package -j $Jobs ($(rustc -V))" | Out-File $log -Append -Encoding utf8
cargo build -p $Package -j $Jobs *>> $log
$code = $LASTEXITCODE
$min = [math]::Round(((Get-Date) - $t0).TotalMinutes, 1)
$free = [math]::Round((Get-PSDrive C).Free / 1GB, 2)
"exit=$code minutes=$min c_free_gb=$free" | Out-File $done -Encoding utf8
"[$(Get-Date -Format s)] done: exit=$code minutes=$min c_free_gb=$free" | Out-File $log -Append -Encoding utf8
