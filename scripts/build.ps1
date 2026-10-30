param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs
)

$ErrorActionPreference = "Stop"

$ortam = & (Join-Path $PSScriptRoot "derleme-ortami.ps1")
$komut = "`"$($ortam.Vcvars)`" >nul && cargo " + ($CargoArgs -join ' ')
cmd /c $komut
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
