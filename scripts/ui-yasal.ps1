param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereYasal {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}
"@
function Butonlar($win) {
    return $win.FindAll([System.Windows.Automation.TreeScope]::Descendants,
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Button)))
}
function Bul($win, $ad) {
    $b = Butonlar $win
    for ($i = 0; $i -lt $b.Count; $i++) { if ($b.Item($i).Current.Name -eq $ad) { return $b.Item($i) } }
    return $null
}
function Kaydet($proc, $dosya) {
    $proc.Refresh()
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($proc.MainWindowHandle)
    $r = $win.Current.BoundingRectangle
    $bmp = New-Object System.Drawing.Bitmap ([int]$r.Width), ([int]$r.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.Left, [int]$r.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) $dosya), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
}
$p = Start-Process -FilePath $Exe -PassThru
try {
    for ($i = 0; $i -lt 30; $i++) { Start-Sleep -Milliseconds 500; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
    if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
    [PencereYasal]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereYasal]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Seconds 1
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Kaydet $p "tests\ui-yasal-uyari.png"
    $onay = $win.FindFirst(
        [System.Windows.Automation.TreeScope]::Descendants,
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::CheckBox)))
    if ($null -eq $onay) { throw "Yasal onay kutusu bulunamadı" }
    $onay.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
    Start-Sleep -Milliseconds 500
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $kabul = Bul $win "Kabul Ediyorum"
    if ($null -eq $kabul) { throw "Kabul düğmesi bulunamadı" }
    $kabul.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    Start-Sleep -Seconds 2
    $p.Refresh()
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Kaydet $p "tests\ui-yasal-kabul-sonrasi.png"
    if ($null -ne (Bul $win "Genel Bakış")) {
        "YASAL_TEST=TAMAM"
    } else {
        "YASAL_TEST=ANA_EKRAN_BULUNAMADI"
    }
}
finally {
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
}
