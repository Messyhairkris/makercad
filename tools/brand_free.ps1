# Fails when any of SolidCraft's brand-licensed crates is in MakerCAD's dependency graph
# (decision D23; AGENTS.md rule 3). Run from anywhere:  pwsh tools/brand_free.ps1
$repo = Split-Path -Parent $PSScriptRoot
$brand = @('solidcraft-ui-kit', 'solidcraft-look', 'solidcraft-theme', 'solidcraft-ui-egui')
Push-Location $repo
try {
    $tree = cargo tree --workspace --edges normal,build,dev --prefix none --format '{p}' 2>&1
    if ($LASTEXITCODE -ne 0) { $tree | Select-Object -Last 20; Write-Error 'cargo tree failed'; exit 2 }
    $found = $tree | ForEach-Object { ($_ -split ' ')[0] } | Where-Object { $brand -contains $_ } | Sort-Object -Unique
    $count = ($tree | ForEach-Object { ($_ -split ' ')[0] } | Sort-Object -Unique).Count
    if ($found) {
        "brand-licensed crates in the graph: $($found -join ', ')"
        exit 1
    }
    "brand-free: none of $($brand -join ', ') among $count packages"
} finally {
    Pop-Location
}
