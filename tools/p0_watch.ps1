# Logs the P0 build's progress every few minutes to target\p0\watch.log, so a person (or Claude)
# can see from anywhere whether it is working or thrashing. Stops when build.done appears.
#   crates   distinct crates compiled so far (from build.log)
#   rustc    the crate each running rustc is compiling, its CPU minutes, committed and resident GB
#   free     free physical memory
# Thrashing looks like: CPU minutes barely moving between lines while committed >> resident.
# -Dir watches another build that writes the same build.log / build.done pair (tools/lowmem_build.ps1).
param([int]$EveryMinutes = 5, [double]$MaxHours = 12, [string]$Dir = '')
$repo = Split-Path -Parent $PSScriptRoot
$p0 = if ($Dir) { $Dir } else { Join-Path $repo 'target\p0' }
$log = Join-Path $p0 'watch.log'
$stop = (Get-Date).AddHours($MaxHours)
while ((Get-Date) -lt $stop) {
    $crates = 0
    if (Test-Path "$p0\build.log") {
        $crates = (Select-String -Path "$p0\build.log" -Pattern '^\s*Compiling ' |
            ForEach-Object { ($_.Line -split '\s+')[2] } | Sort-Object -Unique).Count
    }
    $rustc = Get-CimInstance Win32_Process -Filter "Name='rustc.exe'" | ForEach-Object {
        $name = if ($_.CommandLine -match '--crate-name\s+(\S+)') { $Matches[1] } else { '?' }
        $p = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue
        if ($p) {
            '{0} cpu={1:N1}m commit={2:N2}G resident={3:N2}G' -f $name, $p.TotalProcessorTime.TotalMinutes,
                ($p.PrivateMemorySize64 / 1GB), ($p.WorkingSet64 / 1GB)
        }
    }
    $os = Get-CimInstance Win32_OperatingSystem
    $line = '[{0}] crates={1} free={2:N2}G rustc: {3}' -f (Get-Date -Format s), $crates,
        ($os.FreePhysicalMemory / 1MB), $(if ($rustc) { $rustc -join ' | ' } else { 'none' })
    $line | Out-File $log -Append -Encoding utf8
    if (Test-Path "$p0\build.done") {
        "[$(Get-Date -Format s)] build.done: $(Get-Content "$p0\build.done" -Raw)" | Out-File $log -Append -Encoding utf8
        break
    }
    Start-Sleep -Seconds ($EveryMinutes * 60)
}
