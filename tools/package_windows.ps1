# Packages the release makercad.exe as a test-build zip in dist/ (used by .github/workflows/build.yml).
#
#   pwsh tools/package_windows.ps1 -Build 12 -Commit <sha>
#
# Expects `cargo build --release -p mk-desktop --bin makercad --target x86_64-pc-windows-msvc` to
# have run. The zip holds makercad.exe (everything it needs is inside it: icons, Sancho's figure,
# fonts; the C runtime is linked in), READ-ME-FIRST.txt, the two licences and ATTRIBUTION.md.
param(
    [string]$Build = 'local',
    [string]$Commit = '',
    [string]$Target = 'x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $repo "third_party\solidcraft\target\$Target\release\makercad.exe"
if (-not (Test-Path $exe)) { throw "no release build at $exe" }
$short = if ($Commit.Length -ge 7) { $Commit.Substring(0, 7) } else { 'local' }
$name = "makercad-test$Build-windows-x64"
$dist = Join-Path $repo 'dist'
$stage = Join-Path $dist $name
if (Test-Path $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Force $stage | Out-Null
Copy-Item $exe $stage
foreach ($f in 'LICENSE-MIT', 'LICENSE-APACHE', 'ATTRIBUTION.md') { Copy-Item (Join-Path $repo $f) $stage }
$readme = @"
MakerCAD test build $Build (commit $short)
==========================================

An early test build, not a release. Thank you for trying it.

RUN IT
  Unzip this folder anywhere and double-click makercad.exe.
  Windows may say it "protected your PC", because the program is not signed yet:
  choose "More info", then "Run anyway".

WHAT TO TRY
  - The opening screen.
  - Examples: opens a sample part (a plate with two holes and a boss) in the Design view,
    with Sancho, the human-size figure, standing beside it to show how big it is.
  - In the Design view: drag to turn the view, right-drag to move it, scroll to zoom,
    double-click (or "Fit view") to frame the part. "Sancho 1.68 m" hides or shows him.
    "Home" goes back.

WHAT IS NOT THERE YET
  You cannot make or change parts, and nothing is saved: no projects or files yet.

IF THE WINDOW DOES NOT OPEN OR GOES BLACK
  MakerCAD starts on DirectX 12 and, if a start fails, tries OpenGL the next time.
  To force OpenGL, run in a terminal:   makercad.exe --backend gl
  To forget what it remembered:         makercad.exe --retest
  (It remembers in %LOCALAPPDATA%\MakerCAD\graphics.txt.)

Please tell Kris what worked, what did not, and what computer you used.

MakerCAD is MIT OR Apache-2.0 (LICENSE-MIT, LICENSE-APACHE). Third-party notices: ATTRIBUTION.md.
"@
[IO.File]::WriteAllText((Join-Path $stage 'READ-ME-FIRST.txt'), $readme.Replace("`r`n", "`n").Replace("`n", "`r`n"))
$zip = Join-Path $dist "$name.zip"
if (Test-Path $zip) { Remove-Item -LiteralPath $zip -Force }
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
$size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
"packaged $zip ($size MB)"
