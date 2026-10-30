param(
    [string]$Exe = "target\debug\WinNexus.exe"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Pencere {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
}
"@

function Butonlar($win) {
    return $win.FindAll(
        [System.Windows.Automation.TreeScope]::Descendants,
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button
        ))
    )
}

function Tikla($buton) {
    $desen = $buton.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $desen.Invoke()
    Start-Sleep -Milliseconds 700
}

function Kaydet($win, $dosya) {
    $r = $win.Current.BoundingRectangle
    $bmp = New-Object System.Drawing.Bitmap ([int]$r.Width), ([int]$r.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.Left, [int]$r.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) $dosya), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
}

$p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
for ($i = 0; $i -lt 20; $i++) {
    Start-Sleep -Milliseconds 500
    $p.Refresh()
    if ($p.MainWindowHandle -ne 0) { break }
}
if ($p.MainWindowHandle -eq 0) {
    Stop-Process -Id $p.Id -Force
    throw "Pencere açılmadı"
}
[Pencere]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
[Pencere]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
[Pencere]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 800

$win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
$butonlar = Butonlar $win
Tikla $butonlar.Item(1)
$win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
$butonlar = Butonlar $win

$inceleler = @()
for ($i = 0; $i -lt $butonlar.Count; $i++) {
    if ($butonlar.Item($i).Current.Name -like "*ncele*") {
        $inceleler += $butonlar.Item($i)
    }
}
"INCELE=$($inceleler.Count)"
if ($inceleler.Count -lt 4) {
    Stop-Process -Id $p.Id -Force
    throw "İncele düğmeleri bulunamadı"
}

Tikla $inceleler[3]
$win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
$butonlar = Butonlar $win
$uygula = $null
for ($i = 0; $i -lt $butonlar.Count; $i++) {
    if ($butonlar.Item($i).Current.Name -eq "Uygula") { $uygula = $butonlar.Item($i); break }
}
if ($null -eq $uygula) {
    Stop-Process -Id $p.Id -Force
    throw "Uygula düğmesi bulunamadı"
}

Tikla $uygula
$win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
Kaydet $win "tests\ui-onay.png"
$butonlar = Butonlar $win
$onayla = $null
for ($i = 0; $i -lt $butonlar.Count; $i++) {
    if ($butonlar.Item($i).Current.Name -eq "Onayla") { $onayla = $butonlar.Item($i); break }
}
if ($null -eq $onayla) {
    Stop-Process -Id $p.Id -Force
    throw "Onayla düğmesi bulunamadı"
}
Tikla $onayla
$win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
Kaydet $win "tests\ui-uygulandi.png"

$butonlar = Butonlar $win
$geri_al = $null
for ($i = 0; $i -lt $butonlar.Count; $i++) {
    if ($butonlar.Item($i).Current.Name -eq "Geri Al") { $geri_al = $butonlar.Item($i); break }
}
if ($null -eq $geri_al) {
    Stop-Process -Id $p.Id -Force
    throw "Geri Al düğmesi bulunamadı"
}
Tikla $geri_al
$win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
Kaydet $win "tests\ui-geri-alindi.png"

"AKIS=TAMAM"
Stop-Process -Id $p.Id -Force
