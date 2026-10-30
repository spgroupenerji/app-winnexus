param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereToplu {
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
function Tikla($win, $ad) {
    $b = Bul $win $ad
    if ($null -eq $b) { throw "Düğme yok: $ad" }
    ($b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)).Invoke()
    Start-Sleep -Milliseconds 800
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
$p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
try {
    for ($i = 0; $i -lt 30; $i++) { Start-Sleep -Milliseconds 500; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
    [PencereToplu]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereToplu]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Seconds 1
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kaldırıcı"
    Tikla $win "Uygulamaları Tara"
    Start-Sleep -Seconds 10
    $p.Refresh()
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Toplu Seçim"
    Start-Sleep -Milliseconds 800
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Tümünü Seç"
    Start-Sleep -Seconds 1
    Kaydet $p "tests\ui-kaldirici-toplu-secili.png"
    "TOPLU_TEST_TAMAM"
}
finally {
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
}
