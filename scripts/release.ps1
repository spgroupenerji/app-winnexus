param(
    [string]$Exe = "target\release\WinNexus.exe"
)

$ErrorActionPreference = "Stop"

$ortam = & (Join-Path $PSScriptRoot "derleme-ortami.ps1")
$vcvars = $ortam.Vcvars
$mt = $ortam.Mt
$manifest = Join-Path $PSScriptRoot "..\crates\sp-app\app.manifest"

if (-not (Test-Path $manifest)) { throw "Manifest bulunamadı: $manifest" }
if (-not $mt) { throw "mt.exe bulunamadı. Windows SDK (10) kurun." }

$zaman = Get-Date -Format "yyyyMMddHHmm"
$env:WINNEXUS_BUILD_STAMP = $zaman

$kok = Split-Path $PSScriptRoot -Parent
Push-Location $kok
try {
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
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "release-check.ps1") -Exe $tam

    $hedef_dizin = Join-Path $kok "release"
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

    $sabit = Join-Path $hedef_dizin "WinNexus_beta.exe"
    Copy-Item $tam $sabit -Force
    $ozet = (Get-FileHash -Algorithm SHA256 -LiteralPath $sabit).Hash.ToLowerInvariant()
    Set-Content -LiteralPath "$sabit.sha256" -Value ("{0}  WinNexus_beta.exe" -f $ozet) -Encoding ASCII

    "PORTATIF=$portatif"
    "SABIT=$sabit"
    "SHA256=$ozet"
}
finally {
    Pop-Location
}

exit 0
