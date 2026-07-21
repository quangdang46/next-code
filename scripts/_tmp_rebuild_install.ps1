$ErrorActionPreference = "Stop"
Set-Location "C:\Users\ADMIN\Documents\Projects\next-code"

Write-Host "=== cargo build --release ==="
& "C:\Users\ADMIN\.cargo\bin\cargo.exe" build --profile release -p next-code --bin next-code
if ($LASTEXITCODE -ne 0) { throw "cargo build failed: $LASTEXITCODE" }

$src = Join-Path (Get-Location) "target\release\next-code.exe"
if (-not (Test-Path $src)) { throw "missing $src" }

$hash = (git rev-parse --short HEAD)
if (git status --porcelain) { $hash = "$hash-dirty" }
$la = Join-Path $env:LOCALAPPDATA "next-code"
$homeNc = Join-Path $env:USERPROFILE ".next-code"
$verLa = Join-Path $la "builds\versions\$hash"
$curLa = Join-Path $la "builds\current"
$stableLa = Join-Path $la "builds\stable"
$binLa = Join-Path $la "bin"
$verHome = Join-Path $homeNc "builds\versions\$hash"
$curHome = Join-Path $homeNc "builds\current"
$binLocal = "C:\Users\ADMIN\.local\bin"

foreach ($d in @($verLa, $curLa, $stableLa, $binLa, $verHome, $curHome, $binLocal)) {
    New-Item -ItemType Directory -Force -Path $d | Out-Null
}

$targets = @(
    (Join-Path $verLa "next-code.exe"),
    (Join-Path $curLa "next-code.exe"),
    (Join-Path $stableLa "next-code.exe"),
    (Join-Path $binLa "next-code.exe"),
    (Join-Path $binLa "nextcode.exe"),
    (Join-Path $verHome "next-code.exe"),
    (Join-Path $curHome "next-code.exe"),
    (Join-Path $binLocal "next-code.exe"),
    (Join-Path $binLocal "next-code"),
    (Join-Path $binLocal "nextcode.exe"),
    (Join-Path $binLocal "nextcode")
)
foreach ($dest in $targets) {
    Copy-Item -Force $src $dest
}

Set-Content -NoNewline -Path (Join-Path $la "builds\current-version") -Value $hash
Set-Content -NoNewline -Path (Join-Path $la "builds\stable-version") -Value $hash
Set-Content -NoNewline -Path (Join-Path $homeNc "builds\current-version") -Value $hash

Write-Host "=== installed ==="
& (Join-Path $binLa "next-code.exe") --version
Write-Host ("SIZE=" + (Get-Item (Join-Path $binLa "next-code.exe")).Length)
Write-Host ("MTIME=" + (Get-Item (Join-Path $binLa "next-code.exe")).LastWriteTime)
Write-Host "PATH primary: $binLa"
