param(

    [string]$Exe = "target\release\WinNexus-test.exe"

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

    Start-Sleep -Milliseconds 1600

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

    for ($i = 0; $i -lt 30; $i++) {

        Start-Sleep -Milliseconds 500

        $p.Refresh()

        if ($p.MainWindowHandle -ne 0) { break }

    }

    if ($p.MainWindowHandle -eq 0) { throw "Test penceresi açılmadı" }

    [Pencere]::ShowWindow($p.MainWindowHandle, 9) | Out-Null

    [Pencere]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null

    [Pencere]::SetForegroundWindow($p.MainWindowHandle) | Out-Null

    Start-Sleep -Milliseconds 800



    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)

    $onay = $win.FindFirst(
        [System.Windows.Automation.TreeScope]::Descendants,
        (New-Object System.Windows.Automation.PropertyCondition(
            [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
            [System.Windows.Automation.ControlType]::CheckBox)))
    if ($null -ne $onay) {
        $onay.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
        Start-Sleep -Milliseconds 500
        $kabul = $win.FindFirst(
            [System.Windows.Automation.TreeScope]::Descendants,
            (New-Object System.Windows.Automation.PropertyCondition(
                [System.Windows.Automation.AutomationElement]::NameProperty,
                "Kabul Ediyorum")))
        if ($null -ne $kabul) {
            $kabul.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
            Start-Sleep -Seconds 2
            $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        }
    }
    Kaydet $win "tests\ui-release.png"

    Tikla (Butonlar $win).Item(14)

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)

    Kaydet $win "tests\ui-release-ayarlar.png"

    "RELEASE_TEST=TAMAM PID=$($p.Id)"

}

finally {

    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue

}

