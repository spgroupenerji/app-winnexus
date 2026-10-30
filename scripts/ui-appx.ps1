param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereAppx {
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
    [PencereAppx]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereAppx]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Seconds 1
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Bakım"
    Tikla $win "Paketleri Listele"
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Seconds 1
        $p.Refresh()
        Write-Output "T=$i EXITED=$($p.HasExited) HANDLE=$($p.MainWindowHandle)"
        if ($p.HasExited) { Write-Output "EXITCODE=$($p.ExitCode)"; break }
    }
    if (-not $p.HasExited) { Kaydet $p "tests\ui-bakim-appx-onar.png"; "APPX_TEST_TAMAM" }
}
finally {
    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
}
