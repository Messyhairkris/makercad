# Sizes large folders so stale programs and caches can be proposed for removal. Read-only.
param([string]$Out)
$ErrorActionPreference = 'SilentlyContinue'
function Size([string]$p) {
    $sum = 0L; $newest = [datetime]::MinValue
    try {
        $opts = New-Object System.IO.EnumerationOptions
        $opts.RecurseSubdirectories = $true; $opts.IgnoreInaccessible = $true; $opts.AttributesToSkip = [System.IO.FileAttributes]::ReparsePoint
        foreach ($f in [System.IO.Directory]::EnumerateFiles($p, '*', $opts)) {
            try { $fi = [System.IO.FileInfo]::new($f); $sum += $fi.Length; if ($fi.LastWriteTime -gt $newest) { $newest = $fi.LastWriteTime } } catch {}
        }
    } catch {}
    [pscustomobject]@{ GB = [math]::Round($sum/1GB, 2); Newest = $newest.ToString('yyyy-MM-dd'); Path = $p }
}
$targets = @()
$roots = @('C:\', 'C:\Program Files', 'C:\Program Files (x86)', "$env:USERPROFILE", "$env:USERPROFILE\AppData\Local", "$env:USERPROFILE\AppData\Roaming", "$env:USERPROFILE\AppData\Local\Programs")
foreach ($r in $roots) { Get-ChildItem $r -Directory -Force | Where-Object { -not ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) } | ForEach-Object { $targets += $_.FullName } }
$skip = @('C:\Users', 'C:\Windows', 'C:\Program Files', 'C:\Program Files (x86)', "$env:USERPROFILE\AppData", "$env:USERPROFILE\AppData\Local", "$env:USERPROFILE\AppData\Roaming", "$env:USERPROFILE\AppData\Local\Programs")
$rows = foreach ($t in ($targets | Sort-Object -Unique)) { if ($skip -notcontains $t) { Size $t } }
$rows | Where-Object { $_.GB -ge 0.25 } | Sort-Object GB -Descending | Format-Table -AutoSize | Out-String -Width 220 | Set-Content $Out
# build caches anywhere under the profile (Rust target, node_modules), depth-limited
"`n=== build caches (target / node_modules) >= 0.2 GB ===" | Add-Content $Out
Get-ChildItem "$env:USERPROFILE" -Directory -Recurse -Depth 4 -Force | Where-Object { $_.Name -in 'target','node_modules' -and $_.FullName -notmatch '\\node_modules\\.+\\node_modules' } | ForEach-Object { Size $_.FullName } | Where-Object { $_.GB -ge 0.2 } | Sort-Object GB -Descending | Format-Table -AutoSize | Out-String -Width 220 | Add-Content $Out
"done" | Add-Content $Out
