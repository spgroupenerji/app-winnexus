param(
    [string]$Exe = "target\debug\WinNexus.exe",
    [string]$Cikti = "tests\ui-baslangic.png"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
public static class Pencere {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, System.Text.StringBuilder text, int count);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);
}
"@

function Kaydet([string]$dosya, [int]$sol, [int]$ust, [int]$gen, [int]$yuk) {
    $bmp = New-Object System.Drawing.Bitmap $gen, $yuk
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($sol, $ust, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) $dosya), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
}

[Pencere]::SetProcessDPIAware() | Out-Null
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

$baslik = New-Object System.Text.StringBuilder 256
[Pencere]::GetWindowText($p.MainWindowHandle, $baslik, 256) | Out-Null
[Pencere]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
[Pencere]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
[Pencere]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 800

$r = New-Object RECT
[Pencere]::GetWindowRect($p.MainWindowHandle, [ref]$r) | Out-Null
$w = $r.Right - $r.Left
$h = $r.Bottom - $r.Top
Kaydet $Cikti $r.Left $r.Top $w $h

$x = $r.Left + [int]($w * 0.10)
$y = $r.Top + [int]($h * 0.21)
[Pencere]::SetCursorPos($x, $y) | Out-Null
[Pencere]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)
[Pencere]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)
Start-Sleep -Milliseconds 700
Kaydet "tests\ui-ince-ayarlar.png" $r.Left $r.Top $w $h

"PID=$($p.Id) Baslik=$($baslik.ToString()) Boyut=${w}x${h} Cikti=$Cikti"
Stop-Process -Id $p.Id -Force
