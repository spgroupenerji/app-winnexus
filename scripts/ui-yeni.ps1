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
public static class PencereYeni {
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

function Kaydet($proc, $dosya) {
    $proc.Refresh()
    if ($proc.MainWindowHandle -eq 0) { throw "Pencere kapanmış" }
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($proc.MainWindowHandle)
    $r = $win.Current.BoundingRectangle
    if ($r.Width -lt 10 -or $r.Height -lt 10) { throw "Pencere alanı geçersiz" }
    $bmp = New-Object System.Drawing.Bitmap ([int]$r.Width), ([int]$r.Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen([int]$r.Left, [int]$r.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path (Get-Location) $dosya), [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
}

function Bekle($p, $ad, $saniye) {
    $son = (Get-Date).AddSeconds($saniye)
    while ((Get-Date) -lt $son) {
        $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
        $buton = Bul $win $ad
        if ($null -ne $buton -and $buton.Current.IsEnabled) { return $win }
        Start-Sleep -Milliseconds 500
    }
    return [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
}

$p = Start-Process -FilePath $Exe -ArgumentList "--test" -PassThru
try {
    for ($i = 0; $i -lt 30; $i++) {
        Start-Sleep -Milliseconds 500
        $p.Refresh()
        if ($p.MainWindowHandle -ne 0) { break }
    }
    if ($p.MainWindowHandle -eq 0) { throw "Pencere açılmadı" }
    [PencereYeni]::ShowWindow($p.MainWindowHandle, 9) | Out-Null
    [PencereYeni]::SetWindowPos($p.MainWindowHandle, [IntPtr](-1), 0, 0, 0, 0, 0x0001 -bor 0x0002) | Out-Null
    [PencereYeni]::SetForegroundWindow($p.MainWindowHandle) | Out-Null
    Start-Sleep -Milliseconds 1200

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Winget Merkezi"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Mağaza"
    Kaydet $p "tests\ui-winget-magaza.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kaynaklar"
    Tikla $win "Kaynakları Listele"
    Start-Sleep -Seconds 4
    Kaydet $p "tests\ui-winget-kaynaklar.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Liste"
    Kaydet $p "tests\ui-winget-liste.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Profiller"
    Kaydet $p "tests\ui-winget-profiller.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kurulu"
    Tikla $win "Kurulu Paketleri Tara"
    Start-Sleep -Seconds 12
    Kaydet $p "tests\ui-winget-kurulu.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Mağaza"
    Tikla $win "Ara"
    Start-Sleep -Seconds 15
    Kaydet $p "tests\ui-winget-arama.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Temizlik"
    Tikla $win "Kategorileri Tara"
    $win = Bekle $p "Kategorileri Tara" 90
    Kaydet $p "tests\ui-temizlik-gelismis.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kayıt Defterini Tara"
    Start-Sleep -Seconds 8
    Kaydet $p "tests\ui-temizlik-registry.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kısayolları Tara"
    Start-Sleep -Seconds 10
    Kaydet $p "tests\ui-temizlik-kisayol.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Bakım"
    Tikla $win "Geçmişi Yükle"
    Start-Sleep -Seconds 8
    Kaydet $p "tests\ui-bakim-gecmis.png"
    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Noktaları Listele"
    Start-Sleep -Seconds 6
    Kaydet $p "tests\ui-bakim-geri.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Araç Kutusu"
    Tikla $win "Bağlam Menüsünü Tara"
    Start-Sleep -Seconds 6
    Kaydet $p "tests\ui-arac-baglam.png"

    $win = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
    Tikla $win "Kaldırıcı"
    Kaydet $p "tests\ui-kaldirici-toplu.png"
    "UI_TEST_TAMAM"
}
finally {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
}
