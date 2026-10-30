param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereGod {
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
    Start-Sleep -Milliseconds 900
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
    if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
    [PencereGod]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereGod]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Seconds 1
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "God Mode"
    Kaydet $p "tests\ui-god-mode.png"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "God Mode'u Aç"
    Start-Sleep -Seconds 3
    [PencereGod]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 500
    Kaydet $p "tests\ui-god-mode-acildi.png"
    $pencereler = [System.Windows.Automation.AutomationElement]::RootElement.FindAll(
        [System.Windows.Automation.TreeScope]::Children,
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::Window)))
    $bulundu = $false
    for ($i = 0; $i -lt $pencereler.Count; $i++) {
        $ad = $pencereler.Item($i).Current.Name
        if ($ad -match 'God|Tüm Görevler|All Tasks|Denetim Masası|Control Panel') {
            $bulundu = $true
            Write-Output "GOD_PENCERESI=$ad"
            try {
                $kapat = $pencereler.Item($i).GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern)
                $kapat.Close()
            } catch { }
        }
    }
    if ($bulundu) { "GOD_MODE_TEST=TAMAM" } else { "GOD_MODE_TEST=PENCERE_BULUNAMADI" }
}
finally {
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
}
