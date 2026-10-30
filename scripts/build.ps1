param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"
$vcvars = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"

if (-not (Test-Path $vcvars)) {
    throw "vcvars64.bat bulunamadı: $vcvars"
}

$komut = "`"$vcvars`" >nul && cargo " + ($CargoArgs -join ' ')
cmd /c $komut
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
