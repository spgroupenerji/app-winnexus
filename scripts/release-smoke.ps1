param(
    [string]$Exe = "",
    [string]$Cikti = "tests\ui-release.png"
)

$ErrorActionPreference = "Stop"
if ([string]::IsNullOrWhiteSpace($Exe)) {
    $Exe = (Get-ChildItem "release\WinNexus_beta-v*.exe" |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1).FullName
}
if ([string]::IsNullOrWhiteSpace($Exe)) {
    throw "release klasöründe beta exe bulunamadı."
}
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
}
"@

try {
    $p = Start-Process -FilePath $Exe -ArgumentList "--test" -Verb RunAs -PassThru
} catch {
    throw "UAC onayı alınamadı veya uygulama başlatılamadı: $($_.Exception.Message)"
}

$p.Refresh()
$hedef = $p
for ($i = 0; $i -lt 120; $i++) {
    Start-Sleep -Milliseconds 500
    $p.Refresh()
    if ($p.MainWindowHandle -ne 0) { $hedef = $p; break }
    $aday = Get-Process WinNexus -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowHandle -ne 0 } |
        Select-Object -First 1
    if ($null -ne $aday) { $hedef = $aday; break }
}
if ($hedef.MainWindowHandle -eq 0) {
    throw "Yükseltilmiş uygulama penceresi bulunamadı (UAC onayı verilmemiş olabilir)."
}

[Pencere]::ShowWindow($hedef.MainWindowHandle, 9) | Out-Null
[Pencere]::SetWindowPos($hedef.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
[Pencere]::SetForegroundWindow($hedef.MainWindowHandle) | Out-Null
Start-Sleep -Milliseconds 800

$r = New-Object RECT
[Pencere]::GetWindowRect($hedef.MainWindowHandle, [ref]$r) | Out-Null
$w = $r.Right - $r.Left
$h = $r.Bottom - $r.Top
$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($r.Left, $r.Top, 0, 0, $bmp.Size)
$bmp.Save((Join-Path (Get-Location) $Cikti), [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()

"PID=$($hedef.Id) BASLIK=$($hedef.MainWindowTitle) BOYUT=${w}x${h} CIKTI=$Cikti"

$hedef.CloseMainWindow() | Out-Null
Start-Sleep -Seconds 1
if (-not $hedef.HasExited) {
    Stop-Process -Id $hedef.Id -Force -ErrorAction SilentlyContinue
}
