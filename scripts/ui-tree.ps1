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
$all = $win.FindAll([System.Windows.Automation.TreeScope]::Descendants, [System.Windows.Automation.Condition]::TrueCondition)
for ($i = 0; $i -lt $all.Count; $i++) {
    $e = $all.Item($i)
    "{0} | {1} | {2}" -f $e.Current.ControlType.ProgrammaticName, $e.Current.Name, $e.Current.AutomationId
}

$butonlar = $win.FindAll(
    [System.Windows.Automation.TreeScope]::Descendants,
    (New-Object System.Windows.Automation.PropertyCondition(
        [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
        [System.Windows.Automation.ControlType]::Button
    ))
)
"BUTON SAYISI=$($butonlar.Count)"
if ($butonlar.Count -gt 1) {
    $hedef = $butonlar.Item(1)
    "HEDEF=$($hedef.Current.Name)"
    $desen = $hedef.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $desen.Invoke()
    Start-Sleep -Milliseconds 1800
    $r = $win.Current.BoundingRectangle
    $bmp = New-Object System.Drawing.Bitmap ([int]$r.Width), ([int]$r.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.Left, [int]$r.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) "tests\ui-ince-ayarlar.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
    "TIKLANDI"
}
Stop-Process -Id $p.Id -Force
