# SP GROUP WinNexus — Temel Uygulama Planı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** GPUI Kit tabanlı, derlenebilir ve çalışan WinNexus temelini kurmak: kural motoru, kayıt defteri adaptörü, günlük/geri alma, uygulama kabuğu ve ilk 5 kanıtlı ince ayar.

**Architecture:** `sp-core` Windows’tan bağımsız domain + kural motoru; `sp-win` registry/SCM adaptörleri; `sp-app` GPUI Kit arayüzü. Tüm mutasyonlar tek kuyrukta, günlüklenerek ve geri alınabilir biçimde çalışır.

**Tech Stack:** Rust 1.98, GPUI Kit 0.8.1, serde/toml, winreg, rust-embed, time, tracing, thiserror.

**Spec:** `docs/superpowers/specs/2026-10-30-sp-group-winnexus-design.md`

## Global Constraints

- Uygulama adı: **SP GROUP WinNexus**; binary: `WinNexus.exe`.
- UI dili Türkçe; tam imlâ (ğ, ü, ş, ı, ö, ç).
- Tüm renkler `cx.theme()` token’larından; sabit hex/rgb yok.
- 4 px boşluk baremi; `h_flex`/`v_flex`; Lucide ikonları.
- Her mutasyon öncesi günlük; terslenebilir eylemlerde geri alma.
- Yönetici gerektiren kural, yönetici yoksa uygulanmaz.
- Winget Merkezi bu planın dışında; ayrı plan: `2026-10-30-winnexus-winget.md`.
- v1 dışı: kaldırıcı, bileşen yönetimi, çekirdek sürücüsü, tam servis yöneticisi.

---

## Dosya Yapısı

```text
WinNexus/
├─ Cargo.toml
├─ crates/sp-app/
│  ├─ Cargo.toml
│  └─ src/{main.rs, app.rs}
├─ crates/sp-core/
│  ├─ Cargo.toml
│  └─ src/{lib.rs, error.rs, model.rs, os.rs, validation.rs, adaptor.rs, motor.rs, gunluk.rs, ayarlar.rs, dizinler.rs, kural_yukleyici.rs}
├─ crates/sp-win/
│  ├─ Cargo.toml
│  └─ src/{lib.rs, registry.rs, sistem.rs}
├─ rules/tweaks/{explorer.toml, gizlilik.toml, gorev_cubugu.toml}
├─ scripts/build.ps1
└─ tests/fixtures/
```

Sorumluluk sınırları:
- `sp-core` Windows API çağırmaz; yalnız `SistemAdaptoru` trait’i ve saf mantık.
- `sp-win` UI bilmez; registry ve sistem bilgisi sağlar.
- `sp-app` yalnız `sp-core` + `sp-win` üzerinden çalışır.

---

### Task 1: Workspace ve derleme kanıtı

**Files:**
- Create: `Cargo.toml`
- Create: `crates/sp-app/Cargo.toml`
- Create: `crates/sp-core/Cargo.toml`
- Create: `crates/sp-win/Cargo.toml`
- Create: `crates/sp-app/src/main.rs`
- Create: `scripts/build.ps1`
- Create: `README.md`

**Interfaces:**
- Produces: `cargo build` başarıyla tamamlanır; `WinNexus.exe` üretilir.

- [ ] **Step 1: Workspace dosyalarını oluştur**

`Cargo.toml` (workspace) ve crate manifestleri oluşturulur. `gpui-kit = "=0.8.1"` sabittir. `scripts/build.ps1`, `vcvars64.bat` ile MSVC ortamını kurar; Git Bash `link.exe` çakışmasını önler.

- [ ] **Step 2: Minimal GPUI Kit uygulamasını yaz**

`crates/sp-app/src/main.rs`:

```rust
use gpui_kit::component::button::*;
use gpui_kit::component::*;
use gpui_kit::*;

struct WinNexus;

impl Render for WinNexus {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap_4()
            .child("SP GROUP WinNexus")
            .child(Button::new("basla").primary().label("Başla").on_click(|_, _, _| {}))
    }
}

fn main() {
    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| cx.new(|_| WinNexus))
            .expect("Pencere açılamadı");
    });
}
```

- [ ] **Step 3: Derle**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 build`
Expected: `Finished` ve `target/debug/WinNexus.exe`.

- [ ] **Step 4: Testleri çalıştır**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test`
Expected: `sp-core` ve `sp-win` testleri geçer.

---

### Task 2: Domain modeli ve kural doğrulama

**Files:**
- Create: `crates/sp-core/src/error.rs`
- Create: `crates/sp-core/src/model.rs`
- Create: `crates/sp-core/src/os.rs`
- Create: `crates/sp-core/src/validation.rs`
- Modify: `crates/sp-core/src/lib.rs`
- Create: `crates/sp-core/tests/kural_dogrulama.rs`

**Interfaces:**
- Produces:
  - `Risk`, `KuralDurumu`, `Kove`, `RegistryVeri`, `RegistryKonum`
  - `Algilama::konum() -> RegistryKonum`, `Eylem::konum() -> RegistryKonum`, `Eylem::aciklama() -> String`
  - `Dogrulama::konum() -> RegistryKonum`, `Dogrulama::beklenen() -> Option<&RegistryVeri>`
  - `Kural`, `KuralDosyasi`
  - `OsKosulu::ayristir(&str) -> Sonuc<OsKosulu>`, `uygun(&Surum) -> bool`
  - `kural_dogrula(&Kural) -> Sonuc<()>`, `kurallari_dogrula(&[Kural]) -> Sonuc<()>`

- [ ] **Step 1: Başarısız doğrulama testlerini yaz**

`tests/kural_dogrulama.rs` içinde en az:
- Geçerli kural `Ok`.
- Aynı kimlik iki kez `Err`.
- Boş `baslik` `Err`.
- `yol` içinde `..` `Err`.
- `gorunum = 16` `Err`.
- `dogrula` boş `Err`.
- `kanit` `docs/kanit/` ile başlamıyorsa `Err`.
- `os = ">=11.0"` parse edilir.

- [ ] **Step 2: Testleri çalıştır, başarısız olduğunu doğrula**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test -p sp-core`
Expected: FAIL (modüller yok).

- [ ] **Step 3: Model ve doğrulamayı yaz**

`model.rs` temel tipleri ve serde eşlemelerini içerir. `Kove` için `HKLM`, `HKCU`, `HKCR`, `HKU` değerleri kullanılır. `RegistryVeri` dış etiketli enum’dur:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryVeri {
    Dword(u32),
    Sz(String),
    GenisSz(String),
    Binary(Vec<u8>),
}
```

`Dogrulama` alanı `beklenen: Option<RegistryVeri>`; `None` “değer yok” anlamına gelir. `OsKosulu`, `>=`, `>`, `<=`, `<`, `=` öneklerini ve 2–3 parçalı sürümleri destekler.

`validation.rs` kontrol listesi:
- Kimlik: `^[a-z0-9]+(\.[a-z0-9_]+)+$`, tüm kurallarda benzersiz.
- `baslik`, `ozet`, `kategori`, `kanit` boş olamaz; `kanit` `docs/kanit/` ile başlar.
- `kategori` allowlist: `gizlilik`, `gezgin`, `gorev_cubugu`, `performans`, `ag`, `guncelleme`, `sistem`, `oyun`, `depolama`, `baglam_menusu`.
- `algila`, `uygula`, `dogrula` boş olamaz.
- Registry yolu: boş değil, `..` yok, baş/son `\` yok, 512 karakterden kısa.
- `gorunum` yalnız `None`, `Some(32)`, `Some(64)`.
- `os` varsa `OsKosulu::ayristir` başarılı olmalı.

- [ ] **Step 4: Testleri geçir**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test -p sp-core`
Expected: PASS.

---

### Task 3: Adaptör trait’i ve iş motoru

**Files:**
- Create: `crates/sp-core/src/adaptor.rs`
- Create: `crates/sp-core/src/gunluk.rs`
- Create: `crates/sp-core/src/motor.rs`
- Modify: `crates/sp-core/src/lib.rs`

**Interfaces:**
- Produces:
  - `trait SistemAdaptoru: Send + Sync` — `registry_oku`, `registry_yaz`, `registry_sil`, `yonetici_mi`
  - `type AdaptorRef = Arc<dyn SistemAdaptoru>`
  - `SistemBilgisi { surum: Surum, yonetici: bool }`
  - `IsMotoru::yeni(AdaptorRef, Arc<Gunluk>, SistemBilgisi)`
  - `IsMotoru::durum(&Kural) -> KuralDurumu`
  - `IsMotoru::onizle(&Kural) -> Sonuc<IsPlani>`
  - `IsMotoru::uygula(&IsPlani) -> Sonuc<IsSonucu>`
  - `IsMotoru::geri_al(&IsPlani) -> Sonuc<()>`
  - `Gunluk::yeni(PathBuf)`, `yaz(&GunlukKaydi)`, `son_kayitlar(usize)`

- [ ] **Step 1: Bellek adaptörü ve motor testlerini yaz**

`motor.rs` test modülünde:

```rust
struct BellekAdaptoru {
    veri: Mutex<HashMap<(Kove, String, String), RegistryVeri>>,
    yonetici: bool,
    hata_verisi: Option<RegistryVeri>,
}

impl SistemAdaptoru for BellekAdaptoru {
    fn registry_oku(&self, k: &RegistryKonum) -> Sonuc<Option<RegistryVeri>> {
        Ok(self.veri.lock().unwrap().get(&(k.kove, k.yol.clone(), k.deger.clone())).cloned())
    }
    fn registry_yaz(&self, k: &RegistryKonum, v: &RegistryVeri) -> Sonuc<()> {
        if self.hata_verisi.as_ref() == Some(v) {
            return Err(Hata::Registry("test hatası".into()));
        }
        self.veri.lock().unwrap().insert((k.kove, k.yol.clone(), k.deger.clone()), v.clone());
        Ok(())
    }
    fn registry_sil(&self, k: &RegistryKonum) -> Sonuc<()> {
        self.veri.lock().unwrap().remove(&(k.kove, k.yol.clone(), k.deger.clone()));
        Ok(())
    }
    fn yonetici_mi(&self) -> bool { self.yonetici }
}
```

Testler:
- `uygula_ve_geri_al`: başlangıç `Uygulanmadi`, uygulama sonrası `Uygulandi`, geri alma sonrası `Uygulanmadi`.
- `ikinci_adim_hatasinda_birinci_geri_alinir`.
- `yonetici_yoksa_onizle_yetki_hatasi`.
- `os_desteklenmiyorsa_durum_desteklenmiyor`.
- `dogrulama_basarisizsa_uygula_geri_alir`.

- [ ] **Step 2: Testleri çalıştır, FAIL doğrula**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test -p sp-core motor`
Expected: FAIL.

- [ ] **Step 3: Motoru ve günlüğü yaz**

`IsMotoru::durum`:
1. OS koşulu uygun değilse `Desteklenmiyor`.
2. Her `dogrula` için `registry_oku`; `Some(v)` ile `beklenen` eşleşiyorsa uyan say.
3. Hata varsa `Bilinmiyor`; tümü uyuyorsa `Uygulandi`; hiçbiri uymuyorsa `Uygulanmadi`; aksi halde `Kismi`.

`IsMotoru::onizle`:
1. OS ve yönetici ön denetimi.
2. Her `uygula` için önceki değeri oku.
3. `IsPlani { kural_id, baslik, risk, adimlar, yeniden_baslatma }` döndür.

`IsMotoru::uygula`:
1. Her adım için `GunlukKaydi::niyet` yaz, `fsync` et.
2. Registry yaz/sil.
3. Başarılıysa `GunlukKaydi::basarili` yaz.
4. Hata varsa `GunlukKaydi::hatali` yaz, tamamlanan adımları ters sırayla geri al, `Hata::Registry` döndür.
5. Tüm adımlar bitince her `dogrula` ile doğrula; başarısızsa geri al ve `Hata::Dogrulama` döndür.

`Gunluk`, JSONL satırları yazar; `son_kayitlar` bozuk satırları atlar ve son N kaydı döndürür.

- [ ] **Step 4: Testleri geçir**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test -p sp-core`
Expected: PASS.

---

### Task 4: Ayarlar ve dizinler

**Files:**
- Create: `crates/sp-core/src/ayarlar.rs`
- Create: `crates/sp-core/src/dizinler.rs`
- Modify: `crates/sp-core/src/lib.rs`

**Interfaces:**
- Produces:
  - `Ayarlar { tema: Tema, dil: String, winget_lisans_onayi: bool, winget_kaynak_onayi: bool, adb_yolu: Option<String> }`
  - `Tema { Sistem, Acik, Koyu }`
  - `Ayarlar::yukle(&Path) -> Sonuc<Ayarlar>`, `kaydet(&Path) -> Sonuc<()>`
  - `Dizinler::varsayilan() -> Sonuc<Dizinler>`, `olustur()`, `ayarlar_dosyasi()`, `gunluk_dosyasi()`, `log_dizini()`, `karantina_dizini()`

- [ ] **Step 1: Testleri yaz**

Temp dizinde ayar kaydet/yükle; eksik dosyada varsayılan; bozuk TOML’da `Err`.

- [ ] **Step 2: Uygula**

`directories::BaseDirs` ile `%APPDATA%\SP GROUP\WinNexus` ve `%LOCALAPPDATA%\SP GROUP\WinNexus` yolları üretilir. `varsayilan()` ortam yoksa `Hata::Beklenmeyen` döndürür. `Ayarlar::yukle` dosya yoksa varsayılan döndürür; bozuk dosyayı sessizce yutmaz.

- [ ] **Step 3: Testleri geçir**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test -p sp-core`
Expected: PASS.

---

### Task 5: Windows registry adaptörü ve sistem bilgisi

**Files:**
- Modify: `crates/sp-win/Cargo.toml`
- Create: `crates/sp-win/src/registry.rs`
- Create: `crates/sp-win/src/sistem.rs`
- Modify: `crates/sp-win/src/lib.rs`

**Interfaces:**
- Produces:
  - `WindowsAdaptoru` implementing `SistemAdaptoru`
  - `sistem_bilgisi() -> SistemBilgisi`
  - `yonetici_mi() -> bool`

- [ ] **Step 1: Testleri yaz**

`HKCU\Software\SP GROUP\WinNexus\Test` altında:
- Dword yaz/oku/sil.
- Sz yaz/oku.
- Olmayan değer `None`.
- 32/64 görünüm kabulü.

Test sonunda anahtar temizlenir.

- [ ] **Step 2: Uygula**

`winreg` ile:
- `Kove` → `HKEY_*`.
- `gorunum` → `KEY_WOW64_64KEY` / `KEY_WOW64_32KEY`.
- `open_subkey_with_flags` / `create_subkey_with_flags`.
- `get_raw_value`; `io::ErrorKind::NotFound` → `Ok(None)`.
- `set_raw_value`, `delete_value`; silmede `NotFound` → `Ok(())`.
- `REG_DWORD`, `REG_SZ`, `REG_EXPAND_SZ`, `REG_BINARY` dönüşümleri.
- `CurrentMajorVersionNumber`, `CurrentMinorVersionNumber`, `CurrentBuildNumber` ile `Surum`.
- `IsUserAnAdmin` ile yönetici kontrolü.

- [ ] **Step 3: Testleri geçir**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test -p sp-win`
Expected: PASS.

---

### Task 6: Kural yükleyici ve ilk 5 kural

**Files:**
- Create: `crates/sp-core/src/kural_yukleyici.rs`
- Create: `rules/tweaks/explorer.toml`
- Create: `rules/tweaks/gizlilik.toml`
- Create: `rules/tweaks/gorev_cubugu.toml`
- Modify: `crates/sp-core/src/lib.rs`

**Interfaces:**
- Produces: `gomulu_kurallari_yukle() -> Sonuc<Vec<Kural>>`

- [ ] **Step 1: Testleri yaz**

Gömülü kurallar yüklenir; en az 5 kural; kimlikler benzersiz; tümü doğrulamadan geçer.

- [ ] **Step 2: Kural dosyalarını yaz**

İlk 5 kural (hepsi HKCU, `dusuk` veya `orta` risk):

1. `explorer.gizli_dosyalar` — `...\Explorer\Advanced\Hidden` = DWORD 1.
2. `explorer.dosya_uzantilari` — `...\Explorer\Advanced\HideFileExt` = DWORD 0.
3. `gizlilik.reklam_kimligi` — `...\AdvertisingInfo\Enabled` = DWORD 0.
4. `gorev_cubugu.saniye_goster` — `...\Explorer\Advanced\ShowSecondsInSystemClock` = DWORD 1.
5. `gorev_cubugu.gorev_cubugu_sola` — `...\Explorer\Advanced\TaskbarAl` = DWORD 0.

Her kuralda:
- `algila`, `uygula`, `dogrula` registry blokları.
- `kanit = "docs/kanit/betikler.md#..."`.
- `yonetici = false`.
- `os = ">=10.0"`.
- `risk = "dusuk"`.

- [ ] **Step 3: Yükleyiciyi yaz**

`rust-embed` ile `../../rules/tweaks` klasörü gömülür; `.toml` dosyaları `KuralDosyasi` olarak ayrıştırılır; `kurallari_dogrula` çağrılır.

- [ ] **Step 4: Testleri geçir**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test`
Expected: PASS.

---

### Task 7: GPUI Kit kabuğu ve sayfalar

**Files:**
- Modify: `crates/sp-app/Cargo.toml`
- Modify: `crates/sp-app/src/main.rs`
- Create: `crates/sp-app/src/app.rs`

**Interfaces:**
- Produces: çalışan uygulama; Genel Bakış, İnce Ayarlar, Geçmiş, Ayarlar sayfaları; kural uygulama/geri alma.

- [ ] **Step 1: Bağımlılıkları ekle**

`sp-app`, `sp-core`, `sp-win`, `tracing`, `tracing-subscriber`, `time` kullanır. `gpui-kit` sabittir.

- [ ] **Step 2: Uygulama durumunu yaz**

```rust
pub struct WinNexusApp {
    sayfa: Sayfa,
    kurallar: Vec<Kural>,
    durumlar: HashMap<String, KuralDurumu>,
    secili: Option<String>,
    onay: Option<IsPlani>,
    son_plan: Option<IsPlani>,
    mesaj: Option<String>,
    hata: Option<String>,
    motor: IsMotoru,
    gunluk: Arc<Gunluk>,
    ayarlar: Ayarlar,
    dizinler: Dizinler,
}
```

`new(cx)`:
1. `tracing` başlat.
2. `Dizinler::varsayilan()` + `olustur()`.
3. `Ayarlar::yukle`.
4. `gomulu_kurallari_yukle`.
5. `WindowsAdaptoru`, `Gunluk`, `IsMotoru`.
6. Tüm kural durumlarını hesapla.
7. Hata varsa `hata` alanına yaz; uygulama açılmaya devam eder.

- [ ] **Step 3: Render**

- Sol sabit 240 px sidebar: marka, 4 nav butonu, alt bilgi.
- Sağ içerik: sayfa başlığı + içerik.
- Genel Bakış: 4 sayaç kartı, son işlem, hızlı eylemler.
- İnce Ayarlar: kural listesi (başlık, özet, durum rozeti, risk) + detay paneli.
- Detay: `Uygula` → `onizle` → onay paneli → `uygula`; `Geri Al` → `son_plan`.
- Geçmiş: `gunluk.son_kayitlar(50)` tablosu.
- Ayarlar: tema (Sistem/Açık/Koyu) `Theme::change`, veri yolları, sürüm.
- Tüm durum rozetleri metin + ikon; yalnız renk yok.
- `Button` bileşenleri; `accessibility_label` ikon düğmelerde.

- [ ] **Step 4: Derle ve çalıştır**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 run -p sp-app`
Expected: Pencere açılır; 5 kural listelenir; bir kural uygulanır ve geri alınır.

---

### Task 8: Entegrasyon ve sürüm kapısı

**Files:**
- Modify: gerekli gördüğün dosyalar.
- Create: `docs/kanit/betikler.md` (ilk 5 kuralın kanıt notu).

- [ ] **Step 1: Biçim ve lint**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 fmt --check`
Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 clippy -- -D warnings`
Expected: temiz.

- [ ] **Step 2: Testler**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 test`
Expected: tümü PASS.

- [ ] **Step 3: Release derlemesi**

Run: `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1 build --release`
Expected: `target/release/WinNexus.exe`.

- [ ] **Step 4: Manuel doğrulama**

- Pencere açılışı, 5 kural durumu.
- Gizli dosyalar kuralını uygula; Explorer’da değişikliği doğrula.
- Geri al; eski değerin döndüğünü doğrula.
- Geçmişte iki günlük kaydı gör.
- Tema değiştir; açık/koyu kontrastı gözle.
- Klavye ile tüm sayfalarda gezin.

- [ ] **Step 5: Kabul notu**

`docs/kanit/betikler.md` içine kural kimlikleri, registry yolları ve doğrulama tarihini yaz. Yüksek riskli kural yoktur; VM gerekmez.

---

## Self-Review

- **Spec kapsamı:** Bu plan M0–M2 temelini kapsar. Winget Merkezi, 40–60 kural, araç kutusu ve sertleştirme ayrı planlardadır; spec’teki tamamı bu plana sığdırılmaya çalışılmamıştır.
- **Kimlik tutarlılığı:** `IsPlani`, `PlanAdimi`, `SistemAdaptoru`, `GunlukKaydi`, `RegistryKonum` adları tüm görevlerde aynıdır.
- **Güvenlik:** Yönetici kontrolü, günlük-before-mutate, geri alma, path doğrulama ve sır yokluğu tüm görevlerde korunur.
- **Doğrulama:** Her görevde çalıştırılabilir test/derleme komutu vardır; manuel UI doğrulaması Task 8’dedir.
- **Bilinen sınır:** Bu plan yalnız registry tabanlı kuralları uygular; servis/görev/güç eylemleri sonraki plana bırakılmıştır. Bu, spec’in v1 hedefinin bir alt kümesidir ve çalışan bir dikey dilim üretir.
