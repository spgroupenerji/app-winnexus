param()

$ErrorActionPreference = "Stop"

$adaylar = New-Object System.Collections.Generic.List[string]
$vswhere = Join-Path "${env:ProgramFiles(x86)}" "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $kurulum = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($kurulum) {
        $adaylar.Add((Join-Path $kurulum "VC\Auxiliary\Build\vcvars64.bat"))
    }
}
foreach ($kok in @("${env:ProgramFiles}\Microsoft Visual Studio\2022", "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022")) {
    if (Test-Path $kok) {
        Get-ChildItem $kok -Directory -ErrorAction SilentlyContinue |
            ForEach-Object { $adaylar.Add((Join-Path $_.FullName "VC\Auxiliary\Build\vcvars64.bat")) }
    }
}
$adaylar.Add("C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat")

$vcvars = $adaylar | Select-Object -Unique | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $vcvars) {
    throw "MSVC ortamı bulunamadı. Visual Studio 2022 Build Tools (Desktop development with C++) kurun."
}

$mt = $null
$kits = "C:\Program Files (x86)\Windows Kits\10\bin"
if (Test-Path $kits) {
    $mt = Get-ChildItem $kits -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending |
        ForEach-Object { Join-Path $_.FullName "x64\mt.exe" } |
        Where-Object { Test-Path $_ } |
        Select-Object -First 1
}

[pscustomobject]@{
    Vcvars = $vcvars
    Mt     = $mt
}
