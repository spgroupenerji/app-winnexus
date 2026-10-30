# Sistem Bakımı Kanıtları

Bu belge, WinNexus'un sistem bakımı ve onarım araçlarının dayandığı Windows davranışlarını ve komutlarını listeler. Uygulama, üçüncü taraf araçların kodu veya metni kopyalanmadan kendi servisleriyle çalışır.

## Num Lock

Açılışta Num Lock durumu `HKCU\Control Panel\Keyboard\InitialKeyboardIndicators` değeriyle denetlenir. Değer `2` iken Num Lock açık, `0` iken kapalı başlar. Varsayılan kullanıcı profili için aynı değer `HKU\.DEFAULT\Control Panel\Keyboard` altında tutulur. Windows 11'de hızlı başlatma açıkken kapatma sırasındaki klavye durumu da etkili olabilir; bu nedenle kural hem oturum hem varsayılan profil değerini yazar.

Kaynaklar:

- Microsoft Q&A: "How to turn Num lock on" — `InitialKeyboardIndicators` değerinin `2` ile Num Lock açık, `0` ile kapalı olduğu açıklaması.
- Microsoft Q&A: "Turn NUM LOCK on at start-up" — varsayılan kullanıcı profilindeki `InitialKeyboardIndicators` değeri.
- Microsoft Learn: "How do you default NumLock on at startup?" — Windows 10/11'de kullanıcı ve varsayılan profil davranışı.

## Simge ve küçük resim önbelleği

Windows Gezgini simge ve küçük resim önbelleğini `%LOCALAPPDATA%\Microsoft\Windows\Explorer` altında `iconcache_*.db` ve `thumbcache_*.db` dosyalarında tutar. Bu dosyalar Gezgini kapalıyken silinebilir; Gezgini yeniden başlatıldığında Windows önbelleği yeniden oluşturur. WinNexus onarım işlemi Gezgin'i geçici olarak durdurur, önbellek dosyalarını siler ve Gezgin'i yeniden başlatır.

## Windows Update, hata raporları ve günlükler

- Windows Update indirme önbelleği: `%SystemRoot%\SoftwareDistribution\Download`. Temizlik öncesinde `wuauserv` ve `bits` hizmetleri durdurulur, sonra yeniden başlatılır.
- Windows Update veri deposu ve güncelleme geçmişi: `%SystemRoot%\SoftwareDistribution\DataStore` ve `Logs`. Yalnızca kullanıcı onayıyla, hizmetler durdurularak temizlenir.
- Windows hata raporları: `%ProgramData%\Microsoft\Windows\WER\ReportQueue`, `ReportArchive` ve `Temp`.
- Windows günlükleri: `%SystemRoot%\Logs\CBS`, `Logs\DISM` ve `Logs\WindowsUpdate`.
- Teslimat İyileştirme önbelleği: `%SystemRoot%\ServiceProfiles\NetworkService\AppData\Local\Microsoft\Windows\DeliveryOptimization\Cache`.
- Defender tarama geçmişi: `%ProgramData%\Microsoft\Windows Defender\Scans\History\Results`. Bu klasör sistem tarafından korunabilir; WinNexus sonucu özetler ve ham hata göstermez.

Kaynak: Microsoft Learn, Windows Update ve BITS hizmetleri ile SoftwareDistribution klasörünün sıfırlanmasına ilişkin resmi yönergeler; Delivery Optimization önbellek konumu.

## Bileşen deposu ve sistem dosyaları

- `Dism.exe /Online /Cleanup-Image /AnalyzeComponentStore`: WinSxS bileşen deposu analizi.
- `Dism.exe /Online /Cleanup-Image /StartComponentCleanup`: eski bileşen sürümlerinin temizliği.
- `Dism.exe /Online /Cleanup-Image /StartComponentCleanup /ResetBase`: geri alınamayan temizlik; tüm eski bileşen sürümlerini kaldırır. Yalnızca onayla çalıştırılır.
- `Dism.exe /Online /Cleanup-Image /ScanHealth` ve `/RestoreHealth`: bileşen deposu sağlık taraması ve onarımı.
- `sfc.exe /scannow`: sistem dosyası bütünlüğü taraması ve onarımı.

Kaynak: Microsoft Learn, DISM bileşen deposu temizliği ve System File Checker belgeleri.

## Geri yükleme noktaları

`Get-ComputerRestorePoint` ile listelenir. Eski noktaların silinmesi `root\default:SystemRestore` WMI sınıfının `DeleteRestorePoint` yöntemiyle yapılır; en yeni nokta korunur. Sistem Koruması kapalıysa veya yönetici izni yoksa işlem yapılmaz.

## Sorun gidericiler

Windows 10 ve erken Windows 11 sürümlerinde `msdt.exe /id <tanımlayıcı>` ile çalışır. Microsoft Support Diagnostic Tool (MSDT) ve MSDT tabanlı sorun gidericiler Windows'un gelecek sürümlerinde kaldırılacağı için WinNexus, `msdt.exe` bulunamazsa `ms-settings:troubleshoot` sayfasını açar. Tanımlayıcılar sabit bir allowlist ile sınırlandırılır; serbest komut çalıştırılmaz.

Kaynak: Microsoft Support, "Deprecation of Microsoft Support Diagnostic Tool (MSDT) and MSDT troubleshooters".

## Chrome eski sürümleri

Chrome'un geçerli sürümü kayıt defterindeki `Google Chrome` kaldırma kaydının `DisplayVersion` değerinden okunur. `Application` klasöründeki sürüm biçimindeki alt klasörlerden geçerli sürüm dışındakiler eski sürüm olarak listelenir. Geçerli sürüm okunamazsa hiçbir klasör silinmez. Silme sırasında her yol yeniden doğrulanır; yalnızca Chrome `Application` kökünün doğrudan altındaki, sürüm biçimindeki ve geçerli sürüm olmayan klasörler kaldırılır.

## Geliştirici önbellekleri

NuGet (`%LOCALAPPDATA%\NuGet\v3-cache`), pip (`%LOCALAPPDATA%\pip\Cache`) ve npm (`%APPDATA%\npm-cache`) önbellekleri kullanıcı kapsamındadır. Silme yalnızca bu sabit köklerde yapılır; sembolik bağlantılar ve yeniden ayrıştırma noktaları izlenmez.

## Visual Studio kurulum kaynağı

`%ProgramData%\Microsoft\VisualStudio\Packages` ve `%ProgramData%\Package Cache` onarım/kaldırma işlemleri için kullanılabilir. WinNexus bu kategoriyi "riskli" olarak işaretler, varsayılan olarak seçmez ve yalnızca açık onayla temizler.
