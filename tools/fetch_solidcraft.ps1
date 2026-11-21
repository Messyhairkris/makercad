# Fetch the pinned SolidCraft checkout into third_party/solidcraft (shallow, one commit).
# The rev lives in third_party/SOLIDCRAFT_REV; an upstream update is a change to that one line.
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$rev = (Get-Content (Join-Path $repo 'third_party\SOLIDCRAFT_REV') -TotalCount 1).Trim()
$dir = Join-Path $repo 'third_party\solidcraft'
if (-not (Test-Path (Join-Path $dir '.git'))) {
    git init -q $dir
    git -C $dir remote add origin https://github.com/tangency-labs/solidcraft.git
}
git -C $dir fetch -q --depth 1 origin $rev
git -C $dir checkout -q --detach FETCH_HEAD
$have = (git -C $dir rev-parse HEAD).Trim()
if ($have -ne $rev) { throw "third_party/solidcraft is at $have, expected $rev" }
"third_party/solidcraft at $rev"
