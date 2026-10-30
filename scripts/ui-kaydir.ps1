param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereKaydir {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, int dwData, UIntPtr dwExtraInfo);
}
"@
function Ogeler($win, $tip) {
    return $win.FindAll([System.Windows.Automation.TreeScope]::Descendants,
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty, $tip)))
}
function Bul($win, $ad) {
    $b = Ogeler $win ([System.Windows.Automation.ControlType]::Button)
    for ($i = 0; $i -lt $b.Count; $i++) { if ($b.Item($i).Current.Name -eq $ad) { return $b.Item($i) } }
    return $null
}
function Tikla($win, $ad) {
    $b = Bul $win $ad
    if ($null -eq $b) { throw "Düğme yok: $ad" }
    ($b.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)).Invoke()
    Start-Sleep -Milliseconds 800
}
function Odakla($win, $ad) {
    $b = Bul $win $ad
    if ($null -eq $b) { throw "Odak düğmesi yok: $ad" }
    $b.SetFocus()
    Start-Sleep -Milliseconds 900
}
function Kaydir($proc, $adet) {
    $proc.Refresh()
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($proc.MainWindowHandle)
    $r = $win.Current.BoundingRectangle
    [PencereKaydir]::SetCursorPos([int]($r.Left + $r.Width * 0.7), [int]($r.Top + $r.Height * 0.6)) | Out-Null
    Start-Sleep -Milliseconds 200
    for ($i = 0; $i -lt $adet; $i++) {
        [PencereKaydir]::mouse_event(0x0800, 0, 0, -120, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 90
    }
    Start-Sleep -Milliseconds 600
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
function Metinler($win, $dosya) {
    $t = Ogeler $win ([System.Windows.Automation.ControlType]::Text)
    $satirlar = @()
    for ($i = 0; $i -lt $t.Count; $i++) { $satirlar += $t.Item($i).Current.Name }
    $satirlar | Out-File -FilePath $dosya -Encoding utf8
}
$p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
try {
    for ($i = 0; $i -lt 30; $i++) { Start-Sleep -Milliseconds 500; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
    [PencereKaydir]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereKaydir]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Seconds 1
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Temizlik"
    Tikla $win "Kategorileri Tara"
    Start-Sleep -Seconds 6
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Kaydet $p "tests\ui-temizlik-gelismis-scroll.png"

    Kaydir $p 16
    Kaydet $p "tests\ui-temizlik-orta-scroll.png"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kayıt Defterini Tara"
    Start-Sleep -Seconds 6
    Kaydir $p 8
    Kaydet $p "tests\ui-temizlik-registry-scroll.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kısayolları Tara"
    Start-Sleep -Seconds 8
    Kaydir $p 8
    Kaydet $p "tests\ui-temizlik-kisayol-scroll.png"

    Tikla $win "Bakım"
    Tikla $win "Geçmişi Yükle"
    Start-Sleep -Seconds 6
    Kaydir $p 12
    Kaydet $p "tests\ui-bakim-gecmis-scroll.png"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Noktaları Listele"
    Start-Sleep -Seconds 5
    Kaydir $p 4
    Kaydet $p "tests\ui-bakim-geri-scroll.png"
    "KAYDIRMA_TAMAM"
}
finally {
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
}
