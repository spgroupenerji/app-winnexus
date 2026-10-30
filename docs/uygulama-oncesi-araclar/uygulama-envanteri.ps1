[CmdletBinding()]
param(
    [switch]$Yenile,
    [switch]$Denetle
)

$ErrorActionPreference = 'Stop'

$kokDizin = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$profilDosyasi = Join-Path $kokDizin 'profiles\kullandigim.toml'
$envanterDosyasi = Join-Path $PSScriptRoot 'kullandigim-uygulamalar.md'
$kimlikDeseni = '^[A-Za-z0-9][A-Za-z0-9._+-]{0,127}$'

$adDuzeltmeleri = @{
    '7zip.7zip'                                = '7-Zip'
    '9P8LTPGCBZXD'                             = 'Wintoys'
    '9PM860492SZD'                             = 'PC Manager'
    'ACS.CCIDPCSCDriver'                       = 'ACS CCID PC/SC Driver'
    'ACS.UnifiedPCSCDriver'                    = 'ACS Unified PC/SC Driver'
    'Alibaba.Qoder'                            = 'Qoder'
    'BlenderFoundation.Blender'                = 'Blender'
    'Chocolatey.Chocolatey'                    = 'Chocolatey'
    'Cline.ClineDesktop'                       = 'Cline'
    'ClockworkMod.UniversalADBDriver'          = 'Universal ADB Driver'
    'ElectronicArts.EADesktop'                 = 'EA app'
    'EpicGames.EpicGamesLauncher'              = 'Epic Games Launcher'
    'EpicGames.EpicOnlineServices'             = 'Epic Online Services'
    'geeksoftwareGmbH.PDF24Creator'            = 'PDF24 Creator'
    'Git.Git'                                  = 'Git'
    'GitHub.cli'                               = 'GitHub CLI'
    'Google.Antigravity'                       = 'Antigravity'
    'Google.AntigravityIDE'                    = 'Antigravity IDE'
    'Google.Chrome'                            = 'Google Chrome'
    'Google.EarthPro'                          = 'Google Earth Pro'
    'Google.GeminiDesktop'                     = 'Gemini'
    'HiBitSoftware.HiBitUninstaller'           = 'HiBit Uninstaller'
    'IObit.IObitUnlocker'                      = 'IObit Unlocker'
    'KDE.Krita'                                = 'Krita'
    'Klocman.BulkCrapUninstaller'              = 'BCUninstaller'
    'Microsoft.AppInstaller'                   = 'Uygulama Yükleyicisi'
    'Microsoft.DotNet.AspNetCore.3_1'          = 'ASP.NET Core 3.1'
    'Microsoft.DotNet.AspNetCore.5'            = 'ASP.NET Core 5'
    'Microsoft.DotNet.AspNetCore.6'            = 'ASP.NET Core 6'
    'Microsoft.DotNet.AspNetCore.7'            = 'ASP.NET Core 7'
    'Microsoft.DotNet.AspNetCore.8'            = 'ASP.NET Core 8'
    'Microsoft.DotNet.AspNetCore.9'            = 'ASP.NET Core 9'
    'Microsoft.DotNet.DesktopRuntime.3_1'      = 'Windows Desktop Runtime 3.1'
    'Microsoft.DotNet.DesktopRuntime.5'        = 'Windows Desktop Runtime 5'
    'Microsoft.DotNet.DesktopRuntime.6'        = 'Windows Desktop Runtime 6'
    'Microsoft.DotNet.DesktopRuntime.7'        = 'Windows Desktop Runtime 7'
    'Microsoft.DotNet.DesktopRuntime.8.x64'    = 'Windows Desktop Runtime 8'
    'Microsoft.DotNet.DesktopRuntime.9'        = 'Windows Desktop Runtime 9'
    'Microsoft.DotNet.DesktopRuntime.10'       = 'Windows Desktop Runtime 10'
    'Microsoft.DotNet.HostingBundle.3_1'       = '.NET Hosting Bundle 3.1'
    'Microsoft.DotNet.HostingBundle.5'         = '.NET Hosting Bundle 5'
    'Microsoft.DotNet.HostingBundle.6'         = '.NET Hosting Bundle 6'
    'Microsoft.DotNet.HostingBundle.7'         = '.NET Hosting Bundle 7'
    'Microsoft.DotNet.HostingBundle.8'         = '.NET Hosting Bundle 8'
    'Microsoft.DotNet.HostingBundle.9'         = '.NET Hosting Bundle 9'
    'Microsoft.DotNet.Native.Runtime'          = '.NET Native Runtime 2.2'
    'Microsoft.DotNet.Runtime.3_1'             = '.NET Runtime 3.1'
    'Microsoft.DotNet.Runtime.5'               = '.NET Runtime 5'
    'Microsoft.DotNet.Runtime.6'               = '.NET Runtime 6'
    'Microsoft.DotNet.Runtime.7'               = '.NET Runtime 7'
    'Microsoft.DotNet.Runtime.8'               = '.NET Runtime 8'
    'Microsoft.DotNet.Runtime.9'               = '.NET Runtime 9'
    'Microsoft.DotNet.SDK.3_1'                 = '.NET SDK 3.1'
    'Microsoft.DotNet.SDK.5'                   = '.NET SDK 5'
    'Microsoft.DotNet.SDK.6'                   = '.NET SDK 6'
    'Microsoft.DotNet.SDK.7'                   = '.NET SDK 7'
    'Microsoft.DotNet.SDK.8'                   = '.NET SDK 8'
    'Microsoft.DotNet.SDK.9'                   = '.NET SDK 9'
    'Microsoft.UI.Xaml.2.8'                    = 'Microsoft UI Xaml 2.8'
    'Microsoft.VCLibs.14'                      = 'VCLibs 14 UWP Runtime'
    'Microsoft.VCLibs.Desktop.14'              = 'VCLibs 14 UWP Desktop Runtime'
    'Microsoft.VCRedist.2005.x86'              = 'Visual C++ 2005 x86'
    'Microsoft.VCRedist.2008.x64'              = 'Visual C++ 2008 x64'
    'Microsoft.VCRedist.2008.x86'              = 'Visual C++ 2008 x86'
    'Microsoft.VCRedist.2010.x64'              = 'Visual C++ 2010 x64'
    'Microsoft.VCRedist.2010.x86'              = 'Visual C++ 2010 x86'
    'Microsoft.VCRedist.2012.x64'              = 'Visual C++ 2012 x64'
    'Microsoft.VCRedist.2012.x86'              = 'Visual C++ 2012 x86'
    'Microsoft.VCRedist.2013.x64'              = 'Visual C++ 2013 x64'
    'Microsoft.VCRedist.2013.x86'              = 'Visual C++ 2013 x86'
    'Microsoft.VCRedist.2015+.x64'             = 'Visual C++ 2015+ x64'
    'Microsoft.VCRedist.2015+.x86'             = 'Visual C++ 2015+ x86'
    'Microsoft.VisualStudioCode'               = 'Visual Studio Code'
    'Microsoft.WindowsAppRuntime.1.0'          = 'Windows App Runtime 1.0'
    'Microsoft.WindowsAppRuntime.1.1'          = 'Windows App Runtime 1.1'
    'Microsoft.WindowsAppRuntime.1.2'          = 'Windows App Runtime 1.2'
    'Microsoft.WindowsAppRuntime.1.3'          = 'Windows App Runtime 1.3'
    'Microsoft.WindowsAppRuntime.1.4'          = 'Windows App Runtime 1.4'
    'Microsoft.WindowsAppRuntime.1.5'          = 'Windows App Runtime 1.5'
    'Microsoft.WindowsAppRuntime.1.6'          = 'Windows App Runtime 1.6'
    'Microsoft.WindowsAppRuntime.1.7'          = 'Windows App Runtime 1.7'
    'Microsoft.WindowsAppRuntime.1.8'          = 'Windows App Runtime 1.8'
    'Microsoft.WindowsAppRuntime.2'            = 'Windows App Runtime 2'
    'Microsoft.WindowsSDK.10.0.26100'          = 'Windows SDK 10.0.26100'
    'Mozilla.Thunderbird.tr'                   = 'Mozilla Thunderbird'
    'Notepad++.Notepad++'                      = 'Notepad++'
    'ONLYOFFICE.DesktopEditors'                = 'ONLYOFFICE'
    'OpenAI.ChatGPT'                           = 'ChatGPT'
    'OpenJS.NodeJS.LTS'                        = 'Node.js'
    'Oracle.JavaRuntimeEnvironment'            = 'Java 8 Runtime'
    'Oracle.JDK.24'                            = 'Java SE Development Kit 24'
    'OSGeo.QGIS_LTR'                           = 'QGIS LTR'
    'Python.Launcher'                          = 'Python Launcher'
    'Rustlang.Rustup'                          = 'Rustup'
    'SST.OpenCodeDesktop'                      = 'OpenCode'
    'Thales.SafeNetAuthenticationClient'       = 'SafeNet Authentication Client'
    'VideoLAN.VLC'                             = 'VLC'
    'WiseCleaner.WiseMemoryOptimizer'          = 'Wise Memory Optimizer'
    'Zen-Team.Zen-Browser'                     = 'Zen Browser'
    'ZhipuAI.ZCode'                            = 'ZCode'
}
function WingetSatirlari {
    $cikti = & winget list --accept-source-agreements --disable-interactivity 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "winget list başarısız (kod $LASTEXITCODE); winget kurulu ve erişilebilir olmalı."
    }
    $satirlar = @($cikti | ForEach-Object { $_.ToString() })
    $baslik = $satirlar | Where-Object { $_ -match '^Name\s+Id\s+Version' } | Select-Object -First 1
    if (-not $baslik) { throw 'winget list başlığı okunamadı; çıktı biçimi beklenenden farklı.' }
    $idKonum = $baslik.IndexOf('Id')
    $surumKonum = $baslik.IndexOf('Version')
    $kaynakKonum = $baslik.IndexOf('Source')
    $mevcutKonum = $baslik.IndexOf('Available')
    if ($kaynakKonum -lt 0 -or $idKonum -lt 0 -or $surumKonum -lt 0) {
        throw 'winget list sütun başlıkları çözümlenemedi.'
    }
    if ($mevcutKonum -lt $surumKonum) { $mevcutKonum = $kaynakKonum }
    $sonuc = [System.Collections.Generic.List[object]]::new()
    foreach ($satir in $satirlar) {
        if ([string]::IsNullOrWhiteSpace($satir)) { continue }
        if ($satir -match '^Name\s') { continue }
        if ($satir -match '^-+\s*$') { continue }
        if ($satir -match 'upgrades available|güncelleme mevcut') { continue }
        if ($satir.Length -lt ($idKonum + 5)) { continue }
        $dolu = $satir.PadRight($baslik.Length + 40)
        $ad = $dolu.Substring(0, $idKonum).Trim()
        $kimlik = $dolu.Substring($idKonum, $surumKonum - $idKonum).Trim()
        $surum = $dolu.Substring($surumKonum, $mevcutKonum - $surumKonum).Trim()
        $kaynak = $dolu.Substring($kaynakKonum).Trim()
        if (-not $ad -or -not $kimlik) { continue }
        $sonuc.Add([pscustomobject]@{ Ad = $ad; Kimlik = $kimlik; Surum = $surum; Kaynak = $kaynak })
    }
    return ,$sonuc
}

function PaketAdi {
    param($Satir)
    if ($adDuzeltmeleri.ContainsKey($Satir.Kimlik)) { return $adDuzeltmeleri[$Satir.Kimlik] }
    $ad = ($Satir.Ad -replace '\s+', ' ').Trim()
    $surum = ($Satir.Surum -replace '^[<>]\s*', '').Trim()
    if ($surum -and $surum -ne 'Unknown') {
        $parca = ($surum -split '\s+')[0]
        if ($parca.Length -ge 3) {
            $ad = (($ad -replace ('\s*' + [regex]::Escape($parca) + '(?=\s|\(|$)'), ' ') -replace '\s+', ' ').Trim()
        }
    }
    if ($ad.Length -lt 2) { return $Satir.Kimlik }
    return $ad
}

function YonetilenPaketler {
    param($Satirlar)
    $gorulen = [ordered]@{}
    foreach ($satir in $Satirlar) {
        if ($satir.Kaynak -notin @('winget', 'msstore')) { continue }
        if ($satir.Kimlik -cnotmatch $kimlikDeseni) { continue }
        if ($gorulen.Contains($satir.Kimlik)) { continue }
        $gorulen[$satir.Kimlik] = [pscustomobject]@{
            Kimlik = $satir.Kimlik
            Ad     = (PaketAdi $satir)
            Kaynak = $satir.Kaynak
        }
    }
    return @($gorulen.Values | Sort-Object -Property Kimlik)
}

function TomlMetni {
    param([string]$Metin)
    return ($Metin -replace '\\', '\\\\') -replace '"', '\"'
}

function ProfilIcerigi {
    param($Paketler)
    $satirlar = [System.Collections.Generic.List[string]]::new()
    $satirlar.Add('id = "kullandigim"')
    $satirlar.Add('ad = "Kullandığım Uygulamalar"')
    $satirlar.Add('aciklama = "Bu makinede winget ile yönetilen tüm uygulamalar; yeni makinede tek adımda geri yükleme profili."')
    foreach ($paket in $Paketler) {
        $satirlar.Add('')
        $satirlar.Add('[[paket]]')
        $satirlar.Add(('id = "' + $paket.Kimlik + '"'))
        $satirlar.Add(('ad = "' + (TomlMetni $paket.Ad) + '"'))
    }
    return ($satirlar -join ([char]10)) + ([char]10)
}

function TabloMetni {
    param([string]$Metin)
    return ($Metin -replace '\|', '\|')
}

function DosyaYaz {
    param([string]$Yol, [string]$Icerik)
    $gecici = $Yol + '.gecici'
    [System.IO.File]::WriteAllText($gecici, $Icerik, [System.Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $gecici -Destination $Yol -Force
}

function ProfilKimlikleri {
    param([string]$Metin)
    $kimlikler = [System.Collections.Generic.HashSet[string]]::new()
    foreach ($blok in ($Metin -split '\[\[paket\]\]' | Select-Object -Skip 1)) {
        $eslesme = [regex]::Match($blok, '^\s*id\s*=\s*"([^"]+)"', 'Multiline')
        if ($eslesme.Success) { [void]$kimlikler.Add($eslesme.Groups[1].Value) }
    }
    return ,$kimlikler
}

function EnvanterIcerigi {
    param($Paketler, $Masaustu, $MsixSayisi)
    $satirlar = [System.Collections.Generic.List[string]]::new()
    $satirlar.Add('# Kullandığım Uygulamalar')
    $satirlar.Add('')
    $satirlar.Add('Bu envanter winget list --accept-source-agreements çıktısından üretilir; elle düzenlenmez. Güncellemek için uygulama-envanteri.ps1 -Yenile çalıştırın.')
    $satirlar.Add('')
    $satirlar.Add("## Winget ile yönetilen uygulamalar ($($Paketler.Count))")
    $satirlar.Add('')
    $satirlar.Add('| Uygulama | Winget ID | Kaynak |')
    $satirlar.Add('| --- | --- | --- |')
    foreach ($paket in $Paketler) {
        $satirlar.Add(('| ' + (TabloMetni $paket.Ad) + ' | ' + $paket.Kimlik + ' | ' + $paket.Kaynak + ' |'))
    }
    $satirlar.Add('')
    $satirlar.Add("## Winget kaynağında bulunmayan masaüstü uygulamaları ($($Masaustu.Count))")
    $satirlar.Add('')
    $satirlar.Add('Bu uygulamalar hiçbir winget kaynağında bulunmadığı için profille geri yüklenemez; kurulumları elle yapılır.')
    $satirlar.Add('')
    $satirlar.Add('| Uygulama |')
    $satirlar.Add('| --- |')
    foreach ($uygulama in $Masaustu) {
        $satirlar.Add(('| ' + (TabloMetni $uygulama.Ad) + ' |'))
    }
    $satirlar.Add('')
    $satirlar.Add("MSIX/sistem paketleri ($MsixSayisi adet) listelenmez; bunlar Mağaza veya Windows tarafından yönetilir.")
    return ($satirlar -join ([char]10)) + ([char]10)
}

function Denetle {
    param($Paketler)
    if (-not (Test-Path -LiteralPath $profilDosyasi)) { throw "Profil bulunamadı: $profilDosyasi" }
    if (-not (Test-Path -LiteralPath $envanterDosyasi)) { throw "Envanter bulunamadı: $envanterDosyasi" }
    $profilKimlikleri = ProfilKimlikleri (Get-Content -LiteralPath $profilDosyasi -Raw -Encoding utf8)
    $envanterMetni = Get-Content -LiteralPath $envanterDosyasi -Raw -Encoding utf8
    $eksikProfil = @($Paketler | Where-Object { -not $profilKimlikleri.Contains($_.Kimlik) })
    $eksikEnvanter = @($Paketler | Where-Object { $envanterMetni -notmatch [regex]::Escape($_.Kimlik) })
    $fazlaProfil = @($profilKimlikleri | Where-Object { $Paketler.Kimlik -notcontains $_ })
    Write-Output ("Yönetilen paket: {0} · Profil kaydı: {1}" -f $Paketler.Count, $profilKimlikleri.Count)
    if ($eksikProfil.Count -gt 0) { Write-Output ('Profilde eksik: ' + ($eksikProfil.Kimlik -join ', ')) }
    if ($eksikEnvanter.Count -gt 0) { Write-Output ('Envanterde eksik: ' + ($eksikEnvanter.Kimlik -join ', ')) }
    if ($fazlaProfil.Count -gt 0) { Write-Output ('Makinede bulunmayan profil kaydı: ' + ($fazlaProfil -join ', ')) }
    if ($eksikProfil.Count -gt 0 -or $eksikEnvanter.Count -gt 0) {
        throw 'Envanter veya profil eksik; -Yenile ile yenileyin.'
    }
    Write-Output 'Envanter ve profil güncel.'
}

$satirlar = WingetSatirlari
$yonetilen = YonetilenPaketler $satirlar

if ($Yenile) {
    $masaustu = @($satirlar |
        Where-Object { $_.Kaynak -notin @('winget', 'msstore') -and $_.Kimlik.StartsWith('ARP\') } |
        Sort-Object -Property Ad)
    $msixSayisi = @($satirlar |
        Where-Object { $_.Kaynak -notin @('winget', 'msstore') -and -not $_.Kimlik.StartsWith('ARP\') }).Count
    if ($yonetilen.Count -eq 0) { throw 'winget yönetimli paket bulunamadı; profil yazılmadı.' }
    DosyaYaz $profilDosyasi (ProfilIcerigi $yonetilen)
    DosyaYaz $envanterDosyasi (EnvanterIcerigi $yonetilen $masaustu $msixSayisi)
    Write-Output ("Profil yazıldı: {0} paket · Envanter yazıldı: {1} yönetilen, {2} masaüstü, {3} MSIX/sistem" -f $yonetilen.Count, $yonetilen.Count, $masaustu.Count, $msixSayisi)
}

if ($Denetle -or -not $Yenile) {
    Denetle $yonetilen
}
