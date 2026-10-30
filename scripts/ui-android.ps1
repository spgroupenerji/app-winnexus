param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereAndroid {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
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
    if ($null -eq $b) { throw "Düğme bulunamadı: $ad" }
    $b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    Start-Sleep -Milliseconds 900
}
function Kaydet($p, $dosya) {
    $p.Refresh()
    if ($p.MainWindowHandle -eq 0) { return }
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
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
    if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
    [PencereAndroid]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereAndroid]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
    [PencereAndroid]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 1200
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Android Yönetimi"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    foreach ($ad in @("Cihazları Yenile", "ADB Seç", "Platform Tools İndir", "Durumu Tara", "Önbelleği Yükle", "JSON İçe Aktar")) {
        if ($null -eq (Bul $win $ad)) { throw "Android sayfasında düğme bulunamadı: $ad" }
    }
    Kaydet $p "tests\ui-android.png"
    Tikla $win "Durumu Tara"
    Start-Sleep -Seconds 10
    $p.Refresh()
    "SURUCU_TARAMA_EXITED=$($p.HasExited)"
    Kaydet $p "tests\ui-android-surucu.png"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Önbelleği Yükle"
    Start-Sleep -Seconds 3
    Kaydet $p "tests\ui-android-uad.png"
    "ANDROID_TEST=TAMAM"
}
finally { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
