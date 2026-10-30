param(
    [string]$Exe = "target\release\WinNexus.exe"
)

$ErrorActionPreference = "Stop"
$vcvars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
$mt = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\mt.exe"
$manifest = "crates\sp-app\app.manifest"

if (-not (Test-Path $vcvars)) { throw "vcvars64.bat bulunamadı: $vcvars" }
if (-not (Test-Path $mt)) { throw "mt.exe bulunamadı: $mt" }

$zaman = Get-Date -Format "yyyyMMddHHmm"
$env:WINNEXUS_BUILD_STAMP = $zaman

$temizle = "`"$vcvars`" >nul && cargo clean -p sp-app"
cmd /c $temizle
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$komut = "`"$vcvars`" >nul && cargo build --release -p sp-app"
cmd /c $komut
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not (Test-Path $Exe)) { throw "Release exe bulunamadı: $Exe" }
$tam = (Resolve-Path $Exe).Path
$mt_basarili = $false
for ($deneme = 1; $deneme -le 3; $deneme++) {
    & $mt -nologo -manifest $manifest "-outputresource:$tam;#1"
    if ($LASTEXITCODE -eq 0) {
        $mt_basarili = $true
        break
    }
    Start-Sleep -Seconds (2 * $deneme)
}
if (-not $mt_basarili) {
    throw "mt.exe manifesti uygulayamadı: $tam"
}

$baytlar = [System.IO.File]::ReadAllBytes($tam)
$metin = [System.Text.Encoding]::UTF8.GetString($baytlar)
if (-not $metin.Contains($zaman)) {
    throw "Derleme damgası exe içinde bulunamadı: $zaman"
}

"RELEASE=$tam"
"DERLEME_DAMGASI=$zaman"
& powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\release-check.ps1" -Exe $tam

$hedef_dizin = Join-Path (Get-Location) "release"
New-Item -ItemType Directory -Force -Path $hedef_dizin | Out-Null
Get-ChildItem $hedef_dizin -File -ErrorAction SilentlyContinue |
    Where-Object {
        $_.Name -like "WinNexus-v*.exe" -or $_.Name -like "WinNexus_beta-v*.exe"
    } |
    ForEach-Object {
        $eski = $_
        $silindi = $false
        for ($deneme = 1; $deneme -le 3; $deneme++) {
            try {
                Remove-Item $eski.FullName -Force -ErrorAction Stop
                $silindi = $true
                break
            } catch {
                Start-Sleep -Seconds 2
            }
        }
        if (-not $silindi) {
            Write-Warning "Eski sürüm kilitli: $($eski.Name); kapatıp silin."
        }
    }
$portatif = Join-Path $hedef_dizin "WinNexus_beta-v$zaman.exe"
Copy-Item $tam $portatif -Force
"PORTATIF=$portatif"
