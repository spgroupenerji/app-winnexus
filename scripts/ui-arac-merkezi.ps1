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
public static class PencereMerkez {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint dwData, UIntPtr dwExtraInfo);
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

function Bul($win, $ad) {
    $butonlar = Butonlar $win
    for ($i = 0; $i -lt $butonlar.Count; $i++) {
        if ($butonlar.Item($i).Current.Name -eq $ad) {
            return $butonlar.Item($i)
        }
    }
    return $null
}

function Tikla($win, $ad) {
    $buton = Bul $win $ad
    if ($null -eq $buton) { throw "Düğme bulunamadı: $ad" }
    $desen = $buton.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
    $desen.Invoke()
    Start-Sleep -Milliseconds 700
}

function Kaydet($p, $dosya) {
    $p.Refresh()
    if ($p.MainWindowHandle -eq 0) { throw "Pencere kapanmış" }
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $r = $win.Current.BoundingRectangle
    if ($r.Width -lt 10 -or $r.Height -lt 10) { throw "Pencere alanı geçersiz" }
    $bmp = New-Object System.Drawing.Bitmap ([int]$r.Width), ([int]$r.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.Left, [int]$r.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) $dosya), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
}

function Kaydir($p, $adim) {
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $r = $win.Current.BoundingRectangle
    [PencereMerkez]::SetCursorPos([int]($r.Left + $r.Width * 0.7), [int]($r.Top + $r.Height * 0.6)) | Out-Null
    $veri = if ($adim -lt 0) { [uint32]120 } else { [uint32]4294967176 }
    $adet = [math]::Abs($adim)
    for ($i = 0; $i -lt $adet; $i++) {
        [PencereMerkez]::mouse_event(0x0800, 0, 0, $veri, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 80
    }
    Start-Sleep -Milliseconds 500
}

function OdaklaTikla($win, $ad) {
    $buton = Bul $win $ad
    if ($null -eq $buton) { throw "Düğme bulunamadı: $ad" }
    $buton.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    Start-Sleep -Milliseconds 900
}

$araclar = @(
    "Zorla kaldır",
    "Kurulum İzleme",
    "Windows Mağazası Uygulama Yöneticisi",
    "Program Bileşen Yöneticisi",
    "Windows Güncelleme Yöneticisi",
    "Tarayıcı Uzantısı Yöneticisi",
    "Kayıt Defteri Temizleyici",
    "Gereksiz Dosya Temizleyici",
    "Boş Klasör Temizleyici",
    "Kısayolları Düzelt",
    "Dosya ve Kayıt Bulucu",
    "Kopya Dosya Bulucu",
    "Disk Analisti",
    "Dosyaları kalıcı sil",
    "İşlem Yöneticisi",
    "Bağlantı Yöneticisi",
    "Başlangıç Yöneticisi",
    "Hizmet Yöneticisi",
    "Zamanlanmış Görev Yöneticisi",
    "Sağtık Menü Yöneticisi",
    "Sistem Geri Yükleme Yöneticisi"
)

$p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
try {
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 500
        $p.Refresh()
        if ($p.MainWindowHandle -ne 0) { break }
    }
    if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
    [PencereMerkez]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereMerkez]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
    [PencereMerkez]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 1200

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Araç Merkezi"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    $eksik = @()
    foreach ($arac in $araclar) {
        if ($null -eq (Bul $win "Aç · $arac")) { $eksik += $arac }
    }
    if ($eksik.Count -gt 0) { throw "Eksik araç düğmeleri: $($eksik -join ', ')" }
    "ARAC_MERKEZI_SAYISI=$($araclar.Count)"
    Kaydet $p "tests\ui-arac-merkezi.png"

    $paneller = @(
        @{ Arac = "Zorla kaldır"; Kontrol = "Uygulamaları Tara" },
        @{ Arac = "Kurulum İzleme"; Kontrol = "İzlemeyi Başlat" },
        @{ Arac = "Windows Güncelleme Yöneticisi"; Kontrol = "Güncellemeleri Tara" },
        @{ Arac = "Tarayıcı Uzantısı Yöneticisi"; Kontrol = "Uzantıları Tara" },
        @{ Arac = "Dosya ve Kayıt Bulucu"; Kontrol = "Ara" },
        @{ Arac = "Disk Analisti"; Kontrol = "Analiz Et" },
        @{ Arac = "Bağlantı Yöneticisi"; Kontrol = "Bağlantıları Yenile" }
    )
    $sira = 0
    foreach ($panel in $paneller) {
        $sira++
        if ($sira -eq 5) { Kaydir $p 8 }
        if ($sira -eq 6) { Kaydir $p 12 }
        if ($sira -eq 7) { Kaydir $p 20 }
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        OdaklaTikla $win "Aç · $($panel.Arac)"
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        if ($null -eq (Bul $win "Geri")) { throw "$($panel.Arac) paneli açılmadı" }
        if ($null -eq (Bul $win $panel.Kontrol)) {
            throw "$($panel.Arac) panelinde '$($panel.Kontrol)' bulunamadı"
        }
        Kaydet $p ("tests\ui-arac-merkezi-panel-{0}.png" -f $sira)
        Tikla $win "Geri"
    }
    "ARAC_MERKEZI=TAMAM"
}
finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
