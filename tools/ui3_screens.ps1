# Takes every screen of the program as a PNG (the window is as big as the screen allows, 1920 x 1017 here),
# for the pause-point test pass and UI pass 3. Then: python tools/ui3_prepare.py (the repo and embedded copies).
#   pwsh tools/ui3_screens.ps1 [-Exe PATH] [-Library PATH]
param([string]$Exe = (Join-Path $PSScriptRoot '..\third_party\solidcraft\target\debug\makercad.exe'),
      [string]$Library = (Join-Path $env:TEMP 'sample-library.db'))
$exe = $Exe
$out = Join-Path $PSScriptRoot '..\target\p1\ui3'
New-Item -ItemType Directory -Force $out | Out-Null
# The workshop screens need a library: python tools/make_sample_library.py $Library
if (-not (Test-Path $Library)) { python (Join-Path $PSScriptRoot 'make_sample_library.py') $Library }
$env:MAKERCAD_LIBRARY = $Library
$shots = @(
  @('01-home',            @('--look','daydream')),
  @('02-home-night',      @('--look','night-workshop')),
  @('03-home-pastel',     @('--look','pastel')),
  @('04-design',          @('--sample','--look','daydream')),
  @('05-command-search',  @('--sample','--search','round','--look','daydream')),
  @('06-stock-tab',       @('--sample','--tab','stock','--look','daydream')),
  @('07-quest-name',      @('--quest','1','--look','daydream')),
  @('08-quest-size',      @('--quest','3','--typed','--look','daydream')),
  @('09-quest-tools',     @('--quest','4','--look','daydream')),
  @('10-quest-materials', @('--quest','5','--look','daydream')),
  @('11-quest-review',    @('--quest','9','--look','daydream')),
  @('12-workshop-tools',  @('--workshop','1','--look','daydream')),
  @('13-workshop-days',   @('--workshop','1','--when','--look','daydream')),
  @('14-finish-offer',    @('--finish','--look','daydream')),
  @('15-findings',        @('--findings','dresser','--look','daydream'))
)
foreach ($s in $shots) {
  $png = Join-Path $out ($s[0] + '.png')
  Remove-Item $png -ErrorAction SilentlyContinue
  & $exe @($s[1]) --size 1440x900 --screenshot $png 2>&1 | Out-Null
  if (Test-Path $png) { '{0}: {1} KB' -f $s[0], [int]((Get-Item $png).Length / 1024) } else { '{0}: MISSING' -f $s[0] }
}
