# P0: build SolidCraft's headless CLI (for the oracle reference parts) on top of the P0 app build.
# Writes target\p0\cli-build.log and target\p0\cli-build.done (exit code + minutes).
#
# It must reuse the app build's compiled crates, or it rebuilds solidcore (1 h 44 min here). So it
# uses the same settings as `p0_build.ps1 -LowMemory` (CARGO_INCREMENTAL=0, solidcore at opt-level 1)
# and builds the app and the CLI in one invocation with the app's engine features: `cargo tree`
# shows that resolves every shared crate with the same features as the app build, and adds only
# solidcraft-cli, solidcraft-mcp and solidcraft-mcp-server. Building the CLI on its own would
# change the features of 27 shared crates (wgpu, naga, image, windows…) and rebuild them.
$ErrorActionPreference = 'Continue'
$repo = Split-Path -Parent $PSScriptRoot
$out = Join-Path $repo 'target\p0'
New-Item -ItemType Directory -Force $out | Out-Null
$log = Join-Path $out 'cli-build.log'
Remove-Item (Join-Path $out 'cli-build.done') -ErrorAction SilentlyContinue
Set-Location (Join-Path $repo 'third_party\solidcraft')
$env:CARGO_INCREMENTAL = '0'
$t0 = Get-Date
$cargoArgs = @('build', '-p', 'solidcraft', '-p', 'solidcraft-cli', '--features', 'solidcraft-cli/bundled-render-library',
               '-j', '1', '--config', 'profile.dev.package.solidcore.opt-level=1')
"[$(Get-Date -Format s)] cargo $($cargoArgs -join ' ') (rev $(git rev-parse --short HEAD))" | Out-File $log -Append -Encoding utf8
cargo @cargoArgs *>> $log
$code = $LASTEXITCODE
$min = [math]::Round(((Get-Date) - $t0).TotalMinutes, 1)
"exit=$code minutes=$min" | Out-File (Join-Path $out 'cli-build.done') -Encoding utf8
"[$(Get-Date -Format s)] done: exit=$code minutes=$min" | Out-File $log -Append -Encoding utf8
