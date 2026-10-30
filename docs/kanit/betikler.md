# Betik Kaynaklı Kural Kanıtları

Tarih: 2026-10-30
Durum: İlk 5 kural, uygulama içi doğrulama ve geri alma ile test edilecek.

## Gezgin

| Kural | Registry yolu | Değer | Kaynak |
|---|---|---|---|
| `gezgin.gizli_dosyalar` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `Hidden` = 1 | Windows Gezgin görünüm ayarı |
| `gezgin.dosya_uzantilari` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `HideFileExt` = 0 | Windows Gezgin görünüm ayarı |

## Gizlilik

| Kural | Registry yolu | Değer | Kaynak |
|---|---|---|---|
| `gizlilik.reklam_kimligi` | `HKCU\Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo` | `Enabled` = 0 | Windows gizlilik ayarı |

## Görev Çubuğu

| Kural | Registry yolu | Değer | Kaynak |
|---|---|---|---|
| `gorev_cubugu.saniye_goster` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `ShowSecondsInSystemClock` = 1 | Windows görev çubuğu ayarı |
| `gorev_cubugu.sola_hizala` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarAl` = 0 | Windows 11 görev çubuğu ayarı |

## Araç Kutusu

| Kural | Registry yolu | Değer | Kaynak |
|---|---|---|---|
| `sistem.bu_bilgisayar_gizle` | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FolderDescriptions\{GUID}\PropertyBag` | `ThisPCPolicy` = `Hide` | `Windows10_BuBilgisayar_Klasörlerini_Kaldırma.bat` |
| `sistem.hizli_erisim_kapat` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer` | `HubMode` = 1 | `Windows_Hızlı Erişim Kaldır Geri Yükle.bat` |
| `sistem.fotograf_goruntuleyici` | `HKLM\SOFTWARE\Microsoft\Windows Photo Viewer\Capabilities\FileAssociations` | 12 uzantı = `PhotoViewer.FileAssoc.Tiff` | `Windows_Photo Viewer Etkinleştirme.bat` |

Komutlar:
- Explorer yeniden başlatma: `taskkill /f /im explorer.exe` + `explorer.exe`.
- BIOS'a yeniden başlatma: `shutdown /r /fw /t 0`; UEFI ve üretici desteği gerekir.

## Ek Kurallar

| Kural | Registry yolu | Değer | Kaynak |
|---|---|---|---|
| `gorev_cubugu.widgets_gizle` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarDa` = 0 | Windows 11 görev çubuğu |
| `gorev_cubugu.sohbet_gizle` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced` | `TaskbarMn` = 0 | Windows 11 görev çubuğu |
| `gorev_cubugu.arama_kutusu_gizle` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search` | `SearchboxTaskbarMode` = 0 | Windows arama ayarı |
| `gezgin.bing_aramasi_kapat` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Search` | `BingSearchEnabled` = 0 | Windows arama ayarı |
| `gezgin.onedrive_baslangictan_kaldir` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | `OneDrive` değerini sil | OneDrive başlangıç kaydı |
| `gizlilik.telemetri_temel` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection` | `AllowTelemetry` = 0 | Windows tanılama politikası |
| `gizlilik.etkinlik_gecmisi` | `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `EnableActivityFeed`, `PublishUserActivities`, `UploadUserActivities` = 0 | Windows etkinlik geçmişi politikası |
| `sistem.uzun_yol_destegi` | `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` | `LongPathsEnabled` = 1 | Windows uzun yol desteği |

## Doğrulama Notları

- Tüm kurallar kullanıcı kapsamındadır; yönetici izni gerektirmez.
- Her kural algılama, uygulama ve doğrulama adımlarına sahiptir.
- Geri alma, işlem öncesi değeri günlükten geri yazar; değer yoksa siler.
- Windows 11 25H2 (build 26200) üzerinde manuel doğrulama yapılacaktır.
- Yüksek riskli veya geri alınamaz işlem yoktur.
