param([string]$Exe = "target\debug\WinNexus.exe")
$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class PencereTarama {
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);
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
function Kaydir($p, $adim) {
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $r = $win.Current.BoundingRectangle
    [PencereTarama]::SetCursorPos([int]($r.Left + $r.Width * 0.7), [int]($r.Top + $r.Height * 0.6)) | Out-Null
    for ($i = 0; $i -lt $adim; $i++) {
        [PencereTarama]::mouse_event(0x0800, 0, 0, [uint32]4294967176, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 90
    }
    Start-Sleep -Milliseconds 500
}
function Kaydet($p, $dosya) {
    $rect = New-Object PencereTarama+RECT
    if (-not [PencereTarama]::GetWindowRect($p.MainWindowHandle, [ref]$rect)) { throw "Pencere dikdörtgeni alınamadı" }
    $genislik = $rect.Right - $rect.Left
    $yukseklik = $rect.Bottom - $rect.Top
    $bmp = New-Object System.Drawing.Bitmap $genislik, $yukseklik
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) $dosya), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
}
function Dene($arac, $tarama, $kaydirma, $bekle, $dosya) {
    $p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
    try {
        for ($i = 0; $i -lt 30; $i++) { Start-Sleep -Milliseconds 500; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
        if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
        [PencereTarama]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
        [PencereTarama]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
        [PencereTarama]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
        Start-Sleep -Milliseconds 900
        $hazir = $false
        for ($i = 0; $i -lt 20; $i++) {
            $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
            if ($null -ne (Bul $win "Araç Merkezi")) { $hazir = $true; break }
            Start-Sleep -Milliseconds 500
        }
        if (-not $hazir) { throw "Arayüz hazır olmadı" }
        Tikla $win "Araç Merkezi"
        if ($kaydirma -gt 0) { Kaydir $p $kaydirma }
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        Tikla $win "Aç · $arac"
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        Tikla $win $tarama
        Start-Sleep -Seconds $bekle
        $p.Refresh()
        $durum = if ($p.HasExited) { "ÇÖKTÜ" } else { "CANLI" }
        Kaydet $p $dosya
        "$arac TARAMA=$durum"
    }
    finally {
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
        Wait-Process -Id $p.Id -Timeout 5 -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 500
    }
}
Dene "Zorla kaldır" "Uygulamaları Tara" 0 5 "tests\ui-arac-merkezi-tarama-zorla.png"
Dene "Windows Güncelleme Yöneticisi" "Güncellemeleri Tara" 0 25 "tests\ui-arac-merkezi-tarama-guncelleme.png"
Dene "Tarayıcı Uzantısı Yöneticisi" "Uzantıları Tara" 0 8 "tests\ui-arac-merkezi-tarama-uzanti.png"
Dene "Bağlantı Yöneticisi" "Bağlantıları Yenile" 20 6 "tests\ui-arac-merkezi-tarama-baglanti.png"

function BekleButon($p, $ad, $saniye) {
    $son = (Get-Date).AddSeconds($saniye)
    while ((Get-Date) -lt $son) {
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        $buton = Bul $win $ad
        if ($null -ne $buton -and $buton.Current.IsEnabled) { return $win }
        Start-Sleep -Seconds 1
    }
    return $null
}

$p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
try {
    for ($i = 0; $i -lt 30; $i++) { Start-Sleep -Milliseconds 500; $p.Refresh(); if ($p.MainWindowHandle -ne 0) { break } }
    [PencereTarama]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereTarama]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
    [PencereTarama]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 1200
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Araç Merkezi"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Aç · Kurulum İzleme"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "İzlemeyi Başlat"
    $win = BekleButon $p "İzlemeyi Durdur ve Raporla" 180
    if ($null -eq $win) { throw "İzleme başlangıcı tamamlanmadı" }
    $p.Refresh(); "IZLEME_BASLADI=$(-not $p.HasExited)"
    Kaydet $p "tests\ui-arac-merkezi-izleme-basladi.png"
    Tikla $win "İzlemeyi Durdur ve Raporla"
    Start-Sleep -Seconds 90
    $p.Refresh(); "IZLEME_DURDU=$(-not $p.HasExited)"
    Kaydet $p "tests\ui-arac-merkezi-izleme-raporu.png"
}
finally { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }

"TARAMALAR=TAMAM"
