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

function Bul($win, $ad) {
    $butonlar = Butonlar $win
    for ($i = 0; $i -lt $butonlar.Count; $i++) {
        if ($butonlar.Item($i).Current.Name -eq $ad) { return $butonlar.Item($i) }
    }
    return $null
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
try {
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Milliseconds 500
        $p.Refresh()
        if ($p.MainWindowHandle -ne 0) { break }
    }
    if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
    [Pencere]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [Pencere]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
    [Pencere]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 800

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla (Butonlar $win).Item(7)
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $tara = Bul $win "Profilleri Tara"
    if ($null -eq $tara) { throw "Profilleri Tara düğmesi bulunamadı" }
    Tikla $tara

    $bitti = $false
    for ($i = 0; $i -lt 120; $i++) {
        Start-Sleep -Milliseconds 1000
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        $metinler = $win.FindAll(
            [System.Windows.Automation.TreeScope]::Descendants,
            (New-Object System.Windows.Automation.PropertyCondition(
                [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
                [System.Windows.Automation.ControlType]::Text
            ))
        )
        for ($j = 0; $j -lt $metinler.Count; $j++) {
            $ad = $metinler.Item($j).Current.Name
            if ($ad -like "*bulundu*" -or $ad -like "*hata*") {
                $bitti = $true
                break
            }
        }
        if ($bitti) { break }
    }
    Kaydet $win "tests\ui-profil-tarama.png"
    if ($bitti) { "PROFIL_TARAMA=BITTI" } else { "PROFIL_TARAMA=ZAMAN_ASIMI" }
}
finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
