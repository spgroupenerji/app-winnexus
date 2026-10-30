param(
    [string]$Exe = "target\release\WinNexus.exe"
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path $Exe)) {
    throw "Release exe bulunamadı: $Exe"
}

$tam = (Resolve-Path $Exe).Path
$bayt = [System.IO.File]::ReadAllBytes($tam)
$metin = [System.Text.Encoding]::UTF8.GetString($bayt)
$yonetici = $metin.Contains("requireAdministrator")
$bilgi = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($tam)
Add-Type -AssemblyName System.Drawing
$ikon = [System.Drawing.Icon]::ExtractAssociatedIcon($tam)

"EXE=$tam"
"BOYUT=$((Get-Item $tam).Length)"
"YONETICI_MANIFESTI=$yonetici"
"URUN=$($bilgi.ProductName)"
"SURUM=$($bilgi.FileVersion)"
if ($ikon) {
    "IKON=$($ikon.Width)x$($ikon.Height)"
} else {
    "IKON=YOK"
}

if (-not $yonetici) {
    throw "requireAdministrator manifesti bulunamadı"
}
if (-not $ikon) {
    throw "Uygulama ikonu exe içinde bulunamadı"
}
