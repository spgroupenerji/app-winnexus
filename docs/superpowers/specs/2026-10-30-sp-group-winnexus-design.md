# SP GROUP WinNexus — Tasarım Dokümanı

- **Tarih:** 2026-10-30
- **Durum:** Onaylandı (kullanıcı: “tasarımın tamamını onaylıyorum”)
- **Proje kökü:** `C:\01_Dosyalarım\app-windows\WinNexus`
- **Ürün:** SP GROUP WinNexus
- **Hedef sürüm:** v1.0
- **Arayüz dili:** Türkçe
- **Platform:** Windows 11 25H2 birincil; Windows 10/11 22H2+ kural bazlı destek
- **Mimari:** Rust + GPUI Kit 0.8.1 + bağımsız Windows adaptörleri

---

## 1. Amaç

Kullanıcının Windows üzerinde en sık kullandığı sistem araçlarını tek, modern, sezgisel ve güvenli bir masaüstü uygulamasında birleştirmek. v1; uygulama kabuğu, ince ayar/optimizasyon, Winget Merkezi ve araç kutusunu içerir. Uygulama; herhangi bir üçüncü taraf kaldırıcıya, bileşen aracına veya ince ayar aracına çalışma anında bağımlı değildir; işlevler özgün olarak geliştirilir.

## 2. İlkeler

1. **Doğruluk ve güvenlik önce gelir.** Hız veya kolaylık için güvenlik, erişilebilirlik, veri bütünlüğü veya geri dönüş yeteneği feda edilmez.
2. **Tek mutasyon kuyruğu.** Sistemi değiştiren tüm işler seri kuyruktan geçer; okuma taramaları paralel çalışabilir.
3. **Önce kanıt.** Her ince ayar; algılama, uygulama, doğrulama ve geri alma adımlarına ve bir kanıt kaydına sahiptir. Kanıtsız kural pakete girmez.
4. **Geri alınabilirlik.** Her mutasyon öncesi önceki durum günlüğe yazılır. Yüksek riskli toplu işlem öncesi geri yükleme noktası istenir.
5. **Bağımsızlık.** Eski kapalı kaynak araçların kodu, metni veya ikili dosyası ürüne dahil edilmez; yalnız davranış notu ve işlevsel bilgi kullanılır.
6. **Türkçe ve erişilebilirlik.** Tüm kullanıcı metinleri tam Türkçe imlâ ile yazılır; WCAG 2.2 AA hedeflenir.
7. **Varsayım yok.** Doğrulanamayan API, sürüm, parametre veya davranış “doğrulanmalı” etiketiyle M0’a bırakılır; kullanıcıya gerçek dışı bilgi gösterilmez.

## 3. Kapsam

### 3.1 v1 kapsamı

- Uygulama kabuğu, tema, gezinme, ayarlar, iş günlüğü ve geri alma.
- İnce ayar/optimizasyon modülü (hedef 40–60 kanıtlı kural).
- Winget Merkezi: profil içe/dışa aktarma, tarama, eksik kurulum, güncelleme yönetimi, pin politikaları, yeniden adlandırılmış paket önerileri, komut günlüğü.
- Araç kutusu: BIOS’a yeniden başlatma, ADB ile MIUI bloatware kaldırma, Bu Bilgisayar klasörleri, Hızlı Erişim, Fotoğraf Görüntüleyici ilişkilendirmeleri, “Sahipliği Al” sağ tık menüsü, Explorer’ı yeniden başlatma, 4 winget profili, sistem bilgisi.

### 3.2 v1 dışı

- Uygulama kaldırıcı, artık dosya tarama/temizleme, toplu kaldırma. **v2 notu (2026-10-30):** Kaldırıcı, artık tarama/karantina ve Store uygulamaları eklendi. **v3 notu:** Tekil ve toplu seçim modu, toplu kaldırma sonrası otomatik artık taraması/karantinası ve mod ayrımı eklendi.
- DISM/WinSxS temizliği, imaj bağlama/uygulama/yakalama, sürücü/özellik/Appx/güncelleme yönetimi. **v2 notu:** DISM bileşen analizi/temizliği/sağlık/onarım, Appx, Windows özellikleri ve sürücü dışa aktarma eklendi. **v3 notu:** SFC, DISM ResetBase, Appx onarım, Windows Update geçmişi görüntüleme/temizleme, eski geri yükleme noktalarının silinmesi, simge önbelleği onarımı ve Windows sorun gidericileri eklendi; imaj bağlama/uygulama/yakalama hâlâ açık.
- Tam servis yöneticisi ve tam başlangıç yöneticisi. **v2 notu:** Başlangıç, servis, görev ve süreç yönetimi eklendi; servis bağımlılık grafiği ve başlangıç etki analizi hâlâ açık.
- Çekirdek sürücülü sensör izleme (WinRing0 ve benzeri). **v2 notu:** CIM/registry tabanlı salt okunur donanım bilgisi eklendi; çekirdek sürücüsü bilinçli olarak kullanılmıyor.

**v3 kapsam notu (2026-10-30):** Gelişmiş temizlik kategorileri (simge/thumbnail, son kullanılanlar, WER, Windows günlükleri, Windows Update indirmeleri, Teslimat İyileştirme, Defender geçmişi, geliştirici önbellekleri, Visual Studio kaynağı, Chrome kurulum önbelleği, INetCache), eski Chrome sürüm temizliği, kayıt defteri ve bozuk kısayol temizleyicileri, `Shell Extensions\Blocked` tabanlı bağlam menüsü yöneticisi ve klasik menü anahtarı eklendi. Winget Merkezi kaynak ekleme/kaldırma, mağaza arama, kurulu paket taraması, seçili kurulum/güncelleme/kaldırma ve liste içe/dışa aktarma ile genişletildi. Ayarlar ve veri exe yanına taşındı; uygulama açılışta maximize edilir. God Mode sekmesi, `shell:::{ED7BA470-8E54-465E-825C-99712043E01C}` ile denetim masası görevlerini masaüstünde klasör oluşturmadan ayrı Explorer penceresinde açar. İlk açılışta “Yasal Uyarı ve Sorumluluk Reddi” clickwrap kapısı gösterilir; kabul `settings.toml` içinde saklanır ve Ayarlar sayfasında kalıcı metin bulunur. Kalan kapsam: imaj bağlama/uygulama/yakalama, toplu sürücü/özellik yönetimi, servis bağımlılık grafiği, başlangıç etki analizi ve gerçek ADB cihaz testi.

**v4 kapsam notu (2026-10-30):** 21 sistem aracının tamamı uygulamaya alındı. Yeni **Araç Merkezi** sayfası araçları mantıksal gruplarla listeler; mevcut araçlar ilgili sayfaya yönlendirilir. Yeni arka uçlar: Windows Güncelleme Yöneticisi (`Get-HotFix` + doğrulanmış KB için `wusa`), Tarayıcı Uzantısı Yöneticisi (Chromium profilleri + Firefox `extensions.json`, karantinalı kaldırma), Bağlantı Yöneticisi (`Get-NetTCPConnection`/`Get-NetUDPEndpoint` + süreç eşlemesi), Dosya ve Kayıt Bulucu (jokerli dosya araması, bütçeli registry araması), Disk Analisti (boyut ağacı + en büyük dosyalar), Kurulum İzleme (sürücüsüz dosya/kayıt anlık görüntüsü farkı) ve Zorla Kaldırma (karantinalı klasör taşıma + `.reg` yedekli kayıt silme). Kayıt defteri temizleyici yazı tipi, ses olayı, MUI ve yardım dosyası kategorileriyle genişletildi; uygulama yolu/paylaşılan DLL taramasındaki çift-yol kök hatası düzeltildi. Sistem Geri Yükleme Yöneticisi nokta oluşturma, geri dönme ve korumayı açma/kapatma ile tamamlandı. Salt okunur listelerde en fazla 40 etkileşimli satır ve arama filtresi kullanılır; her düğme kimliği benzersizdir. Kalan kapsam: imaj bağlama/uygulama/yakalama, toplu sürücü yönetimi, servis bağımlılık grafiği, başlangıç etki analizi ve gerçek ADB cihaz testi.
- ARM64, Windows 10 22H2 öncesi sürümler, otomatik güncelleme, bulut eşitleme, telemetri.
- Winget kaynak ekleme/kaldırma; yalnız kaynak listeleme ve güncelleme.

## 4. Terimler

| Terim | Tanım |
|---|---|
| Kural | Bir sistem ayarının algılama, uygulama, doğrulama ve geri alma tanımı |
| İş (Job) | Bir veya daha fazla mutasyon adımından oluşan, günlüklenen çalıştırma |
| Politika | Winget paketi için güncelleme davranışı: normal, bu sefer güncelleme, asla güncelle, sürüm sabitle |
| Pin | Winget’in kendi kalıcı sürüm/güncelleme kilidi |
| Blokaj | Doğrudan güncellemeyi de engelleyen pin türü |
| Profil | Kurulacak/takip edilecek paket listesi |
| Tarama | Profil paketleri ile kurulu/güncel/pinli durumların karşılaştırılması |

## 5. Mimari

### 5.1 Workspace

```text
WinNexus/
├─ Cargo.toml                     # workspace
├─ crates/
│  ├─ sp-app/                     # GPUI Kit UI, sayfalar, bileşenler
│  ├─ sp-core/                    # domain, kural motoru, iş kuyruğu, günlük, winget modeli
│  └─ sp-win/                     # Windows adaptörleri ve süreç çalıştırıcı
├─ rules/tweaks/*.toml            # gömülü ince ayar kuralları
├─ assets/                        # tema, ikon eklentileri, gömülü profil dosyaları
├─ tests/fixtures/                # winget ve kural test verileri
├─ docs/
│  ├─ re/                         # tersine mühendislik kanıt notları
│  └─ superpowers/
│     ├─ specs/                   # bu doküman
│     └─ plans/                   # uygulama planı
└─ README.md                      # derleme ve çalıştırma
```

### 5.2 Sorumluluklar

| Crate | Sorumluluk | Sınır |
|---|---|---|
| `sp-app` | GPUI Kit pencereleri, sayfalar, tema, klavye, erişilebilirlik, olay bağları | Registry/komut çalıştırmaz |
| `sp-core` | Kural modeli, doğrulama, durum makinesi, iş kuyruğu, günlük, winget politikaları ve ayrıştırıcılar | Windows API çağırmaz; adaptör trait’leriyle çalışır |
| `sp-win` | Registry, SCM, COM görevler, kabuk, güç, geri yükleme, süreç çalıştırıcı, dosya karantinası | UI bilmez; domain kararı vermez |

`sp-core`, Windows bağımlılıklarını doğrudan bilmez. `sp-win`, `sp-core` içindeki adaptör trait’lerini uygular. Bu ayrım, kural motorunun ana makinede yıkıcı işlem yapmadan test edilmesini sağlar.

### 5.3 Veri akışı

```text
UI (sp-app)
  → AppState
  → RuleEngine / JobQueue / WingetService (sp-core)
  → adaptör trait’leri
  → sp-win
  → Windows
  ← olaylar / ilerleme / sonuçlar
UI
```

UI iş parçacığı hiçbir zaman bloklanmaz. Uzun işler GPUI arka plan yürütücüsünde çalışır; ilerleme kanallarla UI’ya akar. Mutasyonlar tek kuyrukta serileştirilir; okuma taramaları paralel olabilir.

## 6. Kural Motoru

### 6.1 Kural şeması

Kurallar `rules/tweaks/*.toml` altında tutulur ve `rust-embed` ile derlemeye gömülür. Alan adları Türkçe ASCII’dir; dış format anahtarları (registry, dword) korunur.

```toml
[[rule]]
id = "gizlilik.tanilama.telemetri"
baslik = "Tanılama verilerini en aza indir"
ozet = "Windows tanılama verisini temel düzeye çeker."
kategori = "gizlilik"
risk = "orta"
yonetici = true
os = ">=10.0.22000"
yeniden_baslatma = false
kanit = "docs/kanit/betikler.md#gizlilik"

[[rule.algila]]
tur = "registry"
kove = "HKLM"
yol = 'SOFTWARE\Policies\Microsoft\Windows\DataCollection'
deger = "AllowTelemetry"
gorunum = 64

[[rule.uygula]]
tur = "registry_yaz"
kove = "HKLM"
yol = 'SOFTWARE\Policies\Microsoft\Windows\DataCollection'
deger = "AllowTelemetry"
veri = { dword = 0 }

[[rule.dogrula]]
tur = "registry"
kove = "HKLM"
yol = 'SOFTWARE\Policies\Microsoft\Windows\DataCollection'
deger = "AllowTelemetry"
beklenen = { dword = 0 }
```

Desteklenen algılama türleri: `registry`, `servis_baslangic`, `servis_durum`, `gorev_durum`, `dosya_var`, `komut_cikti`. Desteklenen eylem türleri: `registry_yaz`, `registry_sil`, `servis_baslangic`, `servis_durum`, `gorev_durum`, `guc_plani`, `dosya_karantina`, `komut`. `komut` yalnız allowlist’li program ve doğrulanmış argüman dizisiyle çalışır; serbest kabuk yoktur.

### 6.2 Durumlar

| Durum | Anlam |
|---|---|
| `Uygulandi` | Tüm algılama adımları beklenen değeri veriyor |
| `Uygulanmadi` | En az bir algılama adımı beklenen değeri vermiyor |
| `Kismi` | Bazı alt değerler uygulanmış, bazıları değil |
| `Desteklenmiyor` | OS sürümü veya özellik kuralın kapsamı dışında |
| `Bilinmiyor` | Erişim reddi, zaman aşımı veya okunamayan değer |
| `Hata` | Algılama sırasında beklenmeyen hata |

Durum yalnız renkle anlatılmaz; metin ve ikonla birlikte gösterilir.

### 6.3 İş kuyruğu

Her mutasyon bir `Job` olur: kimlik, başlık, risk, adımlar, geri yükleme noktası gereksinimi, yeniden başlatma gereksinimi, oluşturulma zamanı.

1. **Ön denetim:** OS sürümü, yönetici durumu, disk alanı, geri yükleme noktası kullanılabilirliği.
2. **Önizleme:** Her adım için “önce → sonra” listesi; kullanıcı onayı.
3. **Günlük:** Niyet ve önceki durum diske yazılır ve `fsync` edilir.
4. **Uygulama:** Adımlar sırayla; her adımdan sonra doğrulama.
5. **Hata:** Tamamlanan adımlar ters sırayla geri alınır. Geri alma başarısızsa iş `Kritik` işaretlenir ve manuel kurtarma bilgisi gösterilir.
6. **Kapanış:** Özet, süre, günlük kimliği ve “Geri Al” eylemi.

İptal yalnız adımlar arasında kabul edilir; kayıt defteri yazımının veya servis değişikliğinin ortasında iptal edilmez.

### 6.4 Risk seviyeleri

| Seviye | Örnek | Kapı |
|---|---|---|
| `dusuk` | Kullanıcıya özel kozmetik ayar | Tek onay, otomatik geri alma |
| `orta` | Makine geneli ayar, yeniden başlatma | Önizleme + onay |
| `yuksek` | Ağ/güvenlik/güncelleme/servis | Geri yükleme noktası + açık onay |
| `kritik` | BIOS yeniden başlatma, geri alınamaz eylem | Yazılı onay + geri alma yok uyarısı |

Toplu uygulamada yalnız `dusuk` ve `orta` riskli kurallar seçilebilir. `yuksek` ve `kritik` kurallar tek tek onaylanır.

### 6.5 Günlük ve geri alma

- `%LOCALAPPDATA%\SP GROUP\WinNexus\journal-YYYYMM.jsonl`: satır başına bir olay; niyet, önceki değer, sonuç, hata, geri alma jetonu.
- Registry yedeği: kovan, anahtar, değer adı, tür, veri (ikili veri base64).
- Dosya eylemleri kalıcı silme yerine `%LOCALAPPDATA%\SP GROUP\WinNexus\quarantine\<job-id>\` altına taşınır; 30 gün saklanır.
- Yüksek riskli toplu işlem öncesi `SRSetRestorePointW` ile geri yükleme noktası istenir. Sistem Koruması kapalıysa açık uyarı ve ek onay gerekir.
- Uygulama açılışında yarım kalmış işler taranır; “Kurtar / Geri Al” önerisi sunulur.

## 7. Windows Adaptörleri

| Adaptör | Kullanım | v1 |
|---|---|---|
| Registry | Kural algılama/uygulama, Explorer ve bağlam menüsü | Evet |
| SCM | Servis başlangıç türü/durumu | Evet, sınırlı |
| Görev Zamanlayıcı COM | Kural gerektiren zamanlanmış görev durumu | Evet, sınırlı |
| Kabuk | `SHChangeNotify`, bilinen klasörler, Explorer yeniden başlatma | Evet |
| Güç | Kapatma/yeniden başlatma, BIOS’a yeniden başlatma | Evet |
| Sistem Geri Yükleme | Yüksek risk öncesi nokta | Evet |
| Süreç çalıştırıcı | `winget`, `adb`, `takeown`, `icacls` | Evet |
| WMI | Yalnız zorunlu hallerde | Sınırlı |
| DISM / sürücü / imaj | Bakım fazı | Hayır |

Kurallar `sp-core` içinde Windows’tan bağımsız tanımlanır; `sp-win` adaptörleri gerçek çağrıları yapar. Registry erişiminde 32/64 bit görünüm açıkça belirtilir; WOW64 yönlendirmesi test edilir.

## 8. Winget Merkezi

### 8.1 Doğrulanan winget gerçekleri

Bu bölüm context7 (`/microsoft/winget-cli`) ve yerel `winget v1.29.380` çıktılarıyla doğrulanmıştır:

- `winget list` JSON üretmez; tablo çıktısı verir.
- `winget list --upgrade-available --include-unknown --include-pinned` mevcuttur.
- `winget export -o <dosya> --include-versions` JSON üretir; şema 2.0 `Sources[].Packages[]` yapısındadır; 1.0 biçimi de desteklenir.
- `winget import` `--ignore-unavailable`, `--ignore-versions`, `--no-upgrade` destekler.
- Pin türleri: **Pinning** (`upgrade --all` atlar; doğrudan upgrade geçer), **Blocking** (doğrudan upgrade’i de engeller), **Gating** (`--version`, son parçada joker).
- `winget pin add --blocking`, `winget pin remove`, `winget pin list`, `winget pin reset` mevcuttur.
- Export, kaynakta eşleşmeyen paketleri uyarıyla atlar; bu paketler profil taramasında `Kaynakta yok` olarak ele alınır.
- WinGet PowerShell modülü bu makinede kurulu değildir; COM aktivasyonu doğrulanmamıştır. v1 CLI + JSON/tablo ayrıştırma yolunu kullanır.

### 8.2 Ekranlar

**Sekmeler:** Profiller · Paketler · Güncellemeler · Komut Günlüğü · Winget Ayarları

1. **Profiller:** 4 yerleşik profil (Geliştirici, Uygulama, İş, Mağaza) ve içe aktarılan profiller. Kartta paket sayısı, kurulu/eksik/güncelleme sayıları, son tarama zamanı. Eylemler: İçe Aktar, Tara, Eksikleri Kur, Dışa Aktar.
2. **Paketler:** Profil ve kurulu paketlerin birleşik tablosu. Kolonlar: Ad, Kimlik, Kaynak, Kurulu sürüm, Mevcut sürüm, Durum, Politika, İşlem. Durumlar: `Kurulu`, `Eksik`, `Güncelleme var`, `Adı değişmiş olabilir`, `Kaynakta yok`, `Sabitlenmiş`, `Hariç tutuldu`.
3. **Güncellemeler:** `winget list --upgrade-available` sonucu. Satır eylemleri: Güncelle, Bu sefer güncelleme, Asla güncelle, Sürüm sabitle, Sabitlemeyi kaldır. Toplu eylemler: Seçilenleri güncelle, Tümünü güncelle (filtreli).
4. **Komut Günlüğü:** Her komutun tam argümanları, süresi, çıkış kodu ve çıktı özeti. Komutu kopyala; winget günlük klasörünü aç. Serbest komut girişi yoktur.
5. **Winget Ayarları:** Kaynak güncelleme, lisans/kaynak sözleşmesi onayı, varsayılan kapsam (machine/user), sessiz kurulum, zaman aşımı, yeniden deneme. Paket lisanslarını otomatik kabul varsayılan kapalıdır; açılmadan toplu işlem başlatılmaz.

### 8.3 Profil içe/dışa aktarma

Desteklenen biçimler:

- **Winget export JSON:** şema 1.0 ve 2.0; `Sources[].Packages[]` düzleştirilir; `PackageIdentifier` ve isteğe bağlı `Version` okunur; kaynak adı korunur.
- **WinNexus profil TOML:** paket kimliği, görünen ad, yayıncı, isteğe bağlı sürüm, `replaces`, `zorunlu`, `etiketler`.

Doğrulama: dosya boyutu ≤ 2 MB, paket sayısı ≤ 500, kimlik `^[A-Za-z0-9][A-Za-z0-9._+-]{0,127}$` (winget kimlikleri `+` içerebilir), sürüm biçimi, yinelenen kimlik, geçersiz kaynak. Önizleme; onay; profil olarak kaydetme. Hiçbir paket otomatik kurulmaz.

> “Winget dosyaları” olarak export JSON ve WinNexus TOML yorumlanmıştır. Yerel manifest YAML (`winget install -m`) v1 kapsamında değildir.

### 8.4 Tarama akışı

1. İsteğe bağlı **Kaynakları Güncelle** (`winget source update`); ağ işlemi olduğu için açık kullanıcı eylemidir.
2. Kurulu paketler: `winget export -o <geçici>.json --include-versions` ve profil paketleri için `winget list --id <id> --exact`.
3. Güncellemeler: `winget list --upgrade-available --include-unknown --include-pinned`.
4. Pinler: `winget pin list`.
5. Her profil paketi için:
   - Kurulu → `Kurulu`
   - Güncelleme var → `Güncelleme var`
   - Pin var → `Sabitlenmiş` / `Hariç`
   - Kurulu değil → `winget show --id <id> --exact` ile kaynakta var mı?
     - Varsa → `Eksik`
     - Yoksa → benzer/yeniden adlandırılmış paket araştırması.

Tarama sonucu `scan-cache.json` dosyasına zaman damgasıyla yazılır; UI son tarama zamanını gösterir.

### 8.5 Yeniden adlandırılmış paket önerisi

- `winget show --id <eski-id> --exact` başarısız olursa:
  - `winget search --name "<kayıtlı ad>" --source winget --count 10`
  - `winget search --id <eski-id>` (kısmi kimlik)
- Sonuçlar yerel olarak sıralanır: normalize ad benzerliği (Jaro-Winkler), kimlik benzerliği, yayıncı eşleşmesi (ilk 3 aday için `winget show`), kaynak tercihi.
- Kullanıcıya: “Bu paket artık `<yeni-id>` olabilir. Eski kimlik: `<eski-id>`. Yeni kimliği kullanalım mı?”
- Eylemler: Yeni kimliği kullan, Profilden kaldır, Şimdilik koru.
- Otomatik değiştirme ve otomatik kurulum yoktur. Karar profil dosyasına `replaces = ["eski.id"]` olarak yazılır.
- Araştırma yalnız winget arama sonuçları ve yerel sıralama ile yapılır; harici yapay zekâ, telemetri veya üçüncü taraf servis kullanılmaz.

### 8.6 Güncelleme politikaları

| Politika | Uygulama | Kalıcılık |
|---|---|---|
| `normal` | `winget upgrade --id <id> --exact` | — |
| `bu_sefer` | Bu çalıştırmada atlanır | Uygulama içi; çalıştırma sonunda temizlenir |
| `asla` | `winget pin add --id <id> --exact --blocking` | Winget’te kalıcı; uygulama dışında da geçerli |
| `surum_sabitle` | `winget pin add --id <id> --exact --version <sürüm>` | Winget’te kalıcı; son parçada joker |
| `sabitleme_kaldir` | `winget pin remove --id <id> --exact` | Winget durumu temizlenir |

- Normal akışta `winget upgrade --all` kullanılmaz. Güncellemeler tek tek yürütülür; politika, paket bazlı sonuç ve hata denetimi korunur.
- “Asla güncelle” için blocking pin kullanılır; normal Pinning doğrudan upgrade’i engellemez.
- Uygulama dışında eklenen pinler `winget pin list` ile okunur, arayüzde gösterilir ve kaldırılabilir.
- Var olan her pin türü (Pinning, Blocking, Gating) toplu güncellemeden varsayılan olarak çıkarılır; kullanıcı yalnız açık seçimle ve uyarıyla dahil edebilir.
- Toplu güncelleme öncesi seçim ekranı gösterilir; hiçbir paket sessizce güncellenmez. Yeniden başlatma otomatik yapılmaz; yalnız bildirilir.

Güncelleme akışı:

1. Kaynakları güncelle (isteğe bağlı).
2. `winget list --upgrade-available --include-unknown --include-pinned` ile adayları topla.
3. Uygulama politikalarını birleştir: `asla` ve `surum_sabitle` toplu seçimden çıkar; `bu_sefer` bu çalıştırmada atlanır.
4. Seçim ekranı; varsayılan olarak yalnız `normal` paketler seçili.
5. Her paket için `winget upgrade --id <id> --exact --silent --disable-interactivity` çalıştırılır; onay ayarı açıksa `--accept-package-agreements --accept-source-agreements` eklenir.
6. Çıkış kodu ve çıktı kaydedilir; yalnız geçici ağ hatalarında 3 deneme, 1-2-4-8 sn geri çekilme uygulanır.
7. Yeniden başlatma kodu algılanırsa kullanıcıya bildirilir; otomatik yeniden başlatma yok.
8. Özet: başarılı, başarısız, atlanan, sabitlenmiş. Başarısızlar için “Yeniden dene”.
9. `bu_sefer` işaretleri temizlenir.

### 8.7 Komut güvenliği ve ayrıştırma

- Tüm komutlar sabit şablonlardan üretilir; paket kimliği doğrulanır; kabuk (`cmd /c`) kullanılmaz.
- Standart bayraklar: `--disable-interactivity`, `--exact`, `--id`, `--silent` (kurulum/güncelleme).
- `--accept-package-agreements` ve `--accept-source-agreements` yalnız kullanıcı ayarı açıksa eklenir; değilse işlem engellenir ve gerekçe gösterilir.
- Çıkış kodları ham olarak günlüğe yazılır; yalnız doğrulanmış kodlar Türkçe mesaja eşlenir; bilinmeyen kodlar hex olarak gösterilir.
- Zaman aşımı: kaynak güncelleme 2 dk, tarama 2 dk, paket kurulum/güncelleme 30 dk. İptal yalnız paketler arasında.
- Export JSON şema 1.0/2.0 ayrımı; `Sources` dizisi düzleştirilir.
- Upgrade tablosu sağdan ayrıştırılır: kaynak → mevcut sürüm → kurulu sürüm → kimlik; `< 3.14.7` gibi bilinmeyen sürümler iki token olarak ele alınır; kimlik regex ile doğrulanır; özet satırı yok sayılır.
- Pin tablosu kimlik + pin türü olarak ayrıştırılır; tür bilinmiyorsa “bilinmiyor” gösterilir.
- Ayrıştırma başarısızsa tahmin yürütülmez; ham çıktı günlüğe yazılır ve “winget çıktısı ayrıştırılamadı; sürüm değişmiş olabilir” hatası gösterilir.
- Çıktı kodlaması M0’da doğrulanır; gerekirse Windows kod sayfası için `encoding_rs` kullanılır.

### 8.8 Veri yerleşimi

```text
%APPDATA%\SP GROUP\WinNexus\
  settings.toml
  profiles\*.toml
  winget-policies.toml
  scan-cache.json
%LOCALAPPDATA%\SP GROUP\WinNexus\
  journal-YYYYMM.jsonl
  logs\
  quarantine\
  winget-export-<zaman>.json
```

Yerleşik profiller derlemeye gömülüdür; kullanıcı profilleri yukarıdaki dizinde tutulur. Profil ve ayar dosyaları kullanıcı tarafından düzenlenebilir; uygulama şema doğrulaması yapar.

## 9. Araç Kutusu

1. **BIOS’a yeniden başlat:** `shutdown /r /fw /t 0`; UEFI ve üretici desteği ön koşul; yazılı onay.
2. **ADB ile MIUI bloatware kaldırma:** `adb` yolu ayarlardan seçilir; cihaz listesi ve paket allowlist’i gösterilir; `adb shell pm uninstall -k --user 0 <paket>` yalnız seçilenler için çalışır; `cmd package install-existing <paket>` ile geri yükleme rehberi sunulur. Toplu kaldırma yoktur.
3. **Bu Bilgisayar klasörleri:** 6 GUID için `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FolderDescriptions\{GUID}\PropertyBag\ThisPCPolicy` değeri Show/Hide; kabuk bildirimi.
4. **Hızlı Erişim:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\HubMode` DWORD 1/delete; Explorer yeniden başlatma önerisi.
5. **Fotoğraf Görüntüleyici:** `HKLM\SOFTWARE\Microsoft\Windows Photo Viewer\Capabilities\FileAssociations`; Windows 11’de varsayılan uygulama seçiminin Ayarlar üzerinden yapılması gerektiği açıkça belirtilir.
6. **“Sahipliği Al” sağ tık menüsü:** `HKCR\*\shell\runas`, `HKCR\Directory\shell\runas`, `HKCR\Directory\Background\shell\runas`; menü, `WinNexus.exe --sahipligi-al "<yol>"` gizli modunu çağırır; yol doğrulanır; `takeown` ve `icacls` argüman dizisiyle çalışır.
7. **Explorer’ı yeniden başlat:** Sabit argümanlı `taskkill` + `explorer.exe`; onay.
8. **Winget profilleri:** Geliştirici, Uygulama, İş, Mağaza; Winget Merkezi üzerinden yönetilir.
9. **Sistem bilgisi:** OS/build, CPU, RAM, disk, yönetici durumu, geri yükleme noktası durumu.

Tüm eylemler tek mutasyon kuyruğundan geçer; her biri günlüklenir ve mümkünse geri alınabilir.

## 10. Arayüz ve Tasarım Sistemi

**Sayfalar:** Genel Bakış · İnce Ayarlar · Kaldırıcı · Sistem Yönetimi · Temizlik · Bakım · Donanım · Winget Merkezi · Araç Merkezi · Android Yönetimi · Araç Kutusu · God Mode · Sorun Giderme · Geçmiş · Ayarlar. Uygulanmamış modüller menüde görünmez.

**Genel Bakış:** OS/build, donanım özeti, yönetici ve geri yükleme durumu, kural sayaçları, winget durumu (kurulu/güncellenebilir/sabit), son işlemler, hızlı eylemler. Uydurma sistem puanı yoktur.

**İnce Ayarlar:** Kategori kenar çubuğu, arama, durum/risk filtreleri, sanallaştırılmış liste, detay paneli. Detayda açıklama, risk, gereksinimler, “önce → sonra” önizlemesi, Uygula/Geri Al.

**Winget Merkezi:** Profil kartları, sanallaştırılmış paket/güncelleme tabloları, komut günlüğü, ilerleme çekmecesi. Yükleme, boş, hata, çevrimdışı ve sözleşme onayı durumları tasarlanmıştır.

**Araç Kutusu:** Risk rozetli kartlar; kritik eylemlerde yazılı onay veya işaret kutusu.

Tasarım anayasası:

- Tüm renk, yarıçap, tipografi ve hareket `cx.theme()` token’larından gelir; sabit hex/rgb yoktur.
- 4 px boşluk baremi; `h_flex`/`v_flex`; `min_w_0`/`min_h_0`; mutlak konum yoktur.
- İkonlar yalnız `Icon::new(IconName::...)` ve `gpui-kit-assets` Lucide kataloğundan gelir; yerel SVG yoktur.
- Butonlar `Button::new(...).label(...).icon(...).on_click(...)`; varyantlar `ButtonVariants` ile seçilir.
- Durum renkleri tek eşleme: başarı emerald, uyarı amber, hata rose, bilgi sky; ayrıca metin + ikon.
- Kontrast: metin ≥ 4,5:1; büyük metin/bileşen ≥ 3:1; görünür odak halkası.
- Hareket: yalnız durum değişimi; `cx.theme().motion_tokens()`; `cx.set_reduce_motion(true)` ile hedefe sıçrama.
- Pencere: varsayılan 1200×800, asgari 960×640; DPI 100/150/200% testi; bölünmüş paneller yeniden boyutlandırılabilir.
- Türkçe: tam imlâ (ğ, ü, ş, ı, ö, ç); sayı/tarih `tr-TR`; hata mesajları eylem odaklı.

**Erişilebilirlik:** AccessKit semantiği; ikon düğmelerde `aria_label`; görsel sırayla aynı sekme sırası; 24×24 asgari hedef; ekran okuyucu duyuruları; hareket azaltma.

## 11. Güvenlik ve Hata Yönetimi

- Uygulama `requireAdministrator` manifesti ile çalışır; UAC reddedilirse açıklayıcı ekran gösterilir.
- Registry yazımları yalnız kuralda tanımlı kovan/anahtar/değer ve doğrulanmış türlerle yapılır.
- Dosya yolları kanonikleştirilir; `..`, sembolik bağ ve izin verilen kök dışı yollar reddedilir.
- Komutlar kabuk kullanmadan argüman dizisiyle çalıştırılır; program allowlist’i ve paket kimliği doğrulaması vardır.
- `thiserror` tabanlı tipli hatalar; kullanıcıya Türkçe eylem mesajı, ayrıntı günlüğe. Boş `catch` yoktur; yığın izi gösterilmez.
- Dış çağrılarda zaman aşımı, 3 deneme ve 1-2-4-8 sn geri çekilme; kalıcı hatalarda yeniden deneme yok.
- Günlüklerde parola, token, PII veya ham registry ikili verisi gösterilmez; yalnız gerekli özet tutulur.
- Winget lisans/kaynak sözleşmeleri sessizce kabul edilmez; ayar kapalıysa toplu işlem engellenir.
- “Asla güncelle” blocking pininin uygulama dışında da geçerli olduğu kullanıcıya açıkça söylenir.

## 12. Davranış Kanıtı Yöntemi

| Hedef | Yöntem | Çıktı |
|---|---|---|
| Kayıt defteri ve komut davranışları | Microsoft belgeleri + yerel doğrulama | `docs/kanit/betikler.md` |
| Optimizasyon eşlemeleri | İşlevsel registry doğrulaması | `docs/kanit/optimizasyon.md` |
| Bakım komutları | Yerel komut çıktıları ve Windows API belgeleri | `docs/kanit/sistem_bakimi.md` |

Kurallar:

- Üçüncü taraf kodu veya metni kopyalanmaz; yalnız işlevsel davranış ve Windows API/registry bilgisi kullanılır.
- Kanıtlar dosya yolu ve davranış açıklamasıyla saklanır; decompiled kaynak depoya konmaz.
- Her iddia **görüldü / çıkarıldı / bilinmiyor** olarak işaretlenir.
- `WinRing0` ve benzeri çekirdek sürücüleri v1’de gönderilmez.
- `codegraph`, `gitnexus`, `graphify`, `trail` bu ortamda yoktur; dosya tarama, `ilspycmd` ve kaynak okuma kullanılmıştır.

## 13. Test Stratejisi

- **Birim:** Kural şeması, durum makinesi, önce/sonra planlayıcı, registry yol doğrulama, komut argüman doğrulama, winget ayrıştırıcıları, günlük serileştirme, geri alma sırası.
- **Sözleşme:** Tüm kural paketleri yüklenir; kimlik benzersizliği, Türkçe metin, risk, algılama/uygulama/doğrulama/geri alma ve kanıt bağlantısı zorunludur.
- **Ana makine:** Yalnız okuma taramaları ve `HKCU\Software\SP GROUP\WinNexus\Test` altında izole testler; gerçek sistem anahtarlarına birim testinde dokunulmaz.
- **VM:** Uygula/geri al matrisi; Windows 11 25H2 ve Windows 10 22H2 anlık görüntüleri. Home sürümünde Hyper-V/Windows Sandbox yoktur; VirtualBox/VMware veya ikinci makine kullanılır. VM kanıtı olmayan yüksek riskli kural yayınlanmaz.
- **Winget fixture’ları:** Export 1.0/2.0, upgrade tablosu (TR/EN, `<` sürüm), pin listesi, arama sonuçları, hata çıktıları.
- **Sahte winget:** Test double ile doğru argümanlar, politika filtresi, retry, iptal ve çıkış kodu eşleme test edilir.
- **UI:** GPUI Kit test koşumu; yoksa klavye senaryoları ve ekran görüntüsü karşılaştırması.
- **Erişilebilirlik:** Klavye-only tur, kontrast ölçümü, Narrator/NVDA duman testi, 24×24 hedef ölçümü.
- **Performans:** Soğuk açılış < 1,5 sn; 50 kural taraması < 500 ms; kaydırmada kare düşmesi yok (release).
- **CI:** `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `cargo deny`; `windows-latest` derleme; portatif exe artefaktı.

## 14. Fazlar

| Faz | Teslimat | Kapı |
|---|---|---|
| M0 | Workspace, GPUI Kit derleme spike’ı, tema iskeleti, CI, MSVC linker çözümü | `cargo build --release` kanıtı |
| M1 | RE kataloğu, kural şeması, motor çekirdeği, günlük | Kural sözleşmesi testleri |
| M2 | Kabuk, Genel Bakış, Ayarlar, Geçmiş, ilk 15 kural | UI + geri alma demosu |
| M3 | 40–60 kural, toplu uygulama, arama/filtre, geri alma UI | VM uygula/geri al matrisi |
| M4a | Winget veri katmanı, JSON/TOML içe aktarma, tarama, eksik kurulum, komut günlüğü | Fixture + sahte winget testleri |
| M4b | Güncelleme listesi, politikalar/pinler, yeniden adlandırma önerileri, dışa aktarma | VM winget senaryoları |
| M4c | Araç kutusu 9 eylemi | Tek tek doğrulama |
| M5 | Güvenlik, erişilebilirlik, performans, portatif sürüm | Tüm kabul kriterleri |

## 15. Kabul Kriterleri

1. `cargo build --release` Windows’ta başarılı; portatif `WinNexus.exe` kurulumsuz çalışır.
2. 40–60 kural; tümü sözleşme testinden geçer; her kuralın kanıt bağlantısı vardır.
3. Winget Merkezi: JSON/TOML içe aktarma, tarama, eksik kurulum, güncelleme listesi, dört politika, yeniden adlandırma önerisi ve komut günlüğü çalışır.
4. Her mutasyon günlüklenir; terslenebilir kurallarda geri alma çalışır; yüksek riskli işlemlerde geri yükleme noktası/onay kapısı vardır.
5. Açık/koyu tema, klavye-only kullanım, ekran okuyucu etiketleri, 24×24 hedefler ve hareket azaltma doğrulanır.
6. Anayasa grep kapıları temiz: sabit renk yok, tüm ikonlar katalogdan, yerel SVG yok.
7. Türkçe imlâ tam; hata mesajları eylem odaklı; sayı/tarih yerelleşmiş.
8. Ana makinede yıkıcı test yok; VM kanıtları kayıtlı.
9. Winget lisans/kaynak onayı kapalıyken toplu işlem engellenir.
10. Ayrıştırma hatasında tahmin yok; ham çıktı günlükte ve kullanıcıya açık hata.

## 16. Riskler ve Önlemler

| Risk | Önlem |
|---|---|
| GPUI Kit 0.x API değişimi | `=0.8.1` sabitle; UI’yı ince tut; core’u UI’dan bağımsız yaz; yükseltmeyi test arkasında yap |
| Git Bash’teki `link.exe` MSVC linker’ı gölgeler | `vcvars64.bat` ile derleme veya MSVC linker yolu; M0 kanıtı |
| Home sürümünde VM yok | VirtualBox/VMware veya ikinci makine; VM kanıtı olmadan yüksek riskli kural yayınlanmaz |
| Tweak’lerin build’e göre değişmesi | Kural bazlı OS kapısı, algılama/doğrulama, `Desteklenmiyor` durumu |
| ADB kaldırmanın geri dönüşü | Allowlist, cihaz onayı, `install-existing` rehberi, toplu kaldırma yok |
| Winget ağ/çıkış kodu hataları | Paket bazlı sonuç, 3 deneme, üstel geri çekilme, sessiz başarısızlık yok |
| Winget çıktı biçimi değişimi | Fixture testleri; ayrıştırma hatasında tahmin yok; ham çıktı günlükte |
| Lisans/kaynak sözleşmesi | Varsayılan kapalı; açık kullanıcı onayı olmadan toplu işlem yok |
| Telif/lisans | Bağımsız uygulama; decompiled kod/metin depoya girmez; yalnız davranış notu |
| Kapsam kayması | v1 kapısı; kaldırıcı/bileşen yönetimi ayrı spec ve faz |

## 17. M0’da Doğrulanacaklar

- GPUI Kit `Button`/`ButtonVariants` ve test koşumu API’lerinin 0.8.1’deki kesin imzası.
- GPUI Kit’in bu makinede `cargo build --release` ile derlenmesi; gerekli Windows SDK/VS bileşenleri.
- MSVC `link.exe` yolunun Git Bash dışında doğru çözülmesi.
- Winget çıktı kodlaması (UTF-8/Windows-1254) ve Türkçe çıktıda ayrıştırma.
- `winget list --upgrade-available` tablo biçiminin sürümler arasında kararlılığı.
- Winget çıkış kodlarının doğrulanmış alt kümesi.
- Windows 11 25H2’de Fotoğraf Görüntüleyici ilişkilendirmesinin etkisi.
- Bu Bilgisayar GUID politikalarının 25H2 Home’da davranışı.
- `shutdown /r /fw` için UEFI/üretici desteği.
- `SRSetRestorePointW` ve Sistem Koruması’nın Home’da etkinliği.
- ADB `cmd package install-existing` davranışının cihaz/ROM bazında doğrulanması.
- `winget show` yayıncı alanının yerelleşmiş çıktıda ayrıştırılabilirliği.

## 18. Kaynaklar

- Context7: `/microsoft/winget-cli` — winget CLI belgeleri ve pin/export/import spec’leri.
- `https://github.com/microsoft/winget-cli/blob/master/doc/windows/package-manager/winget/export.md`
- `https://github.com/microsoft/winget-cli/blob/master/doc/windows/package-manager/winget/import.md`
- `https://github.com/microsoft/winget-cli/blob/master/doc/specs/`
- `https://gpui-kit.com/` ve `https://github.com/longbridge/gpui-kit`
- Yerel doğrulama: `winget v1.29.380`, `winget list --upgrade-available`, `winget export --include-versions`, `winget pin --help`, `winget list --help`, `winget upgrade --help`.

**v5 kapsam notu (2026-10-30):** Android Yönetimi sayfası eklendi. ADB için gömülü/indirilen Platform Tools, sistem ADB'si ve saf Rust `adb_client` (TCP sunucu + USB) arka uçları; cihaz/marka/Android sürümü algılama; Xiaomi/HyperOS, Oppo/Realme/OnePlus ve Samsung Auto Blocker uyarıları; USB VID/PID ve PnP durumu; imzalı Google WinUSB, Samsung resmî paketi, yerel INF ve isteğe bağlı libwdi ile sürücü kurulumu; UAD-ng veritabanının (GPL-3.0, gömülmez) resmî kaynaktan indirilmesi/önbelleğe alınması ve JSON içe aktarma; paket risk/bağımlılık/açıklama verisi; kullanıcı 0 için kaldırma, devre dışı bırakma, etkinleştirme, `install-existing` ile geri yükleme ve yalnızca Önerilen/Gelişmiş paketlerde toplu işlem. Güvensiz paketler kaldırılamaz; Uzman/Bilinmeyen riskler açık onay ister. Sürücü kurulumu geçerli Authenticode imzası ve yönetici izni gerektirir. Kapsam dışı: Windows bloatware modülü ve root gerektiren sistem bölümü işlemleri.
