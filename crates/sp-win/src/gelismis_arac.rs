// WinNexus gelişmiş araç servisi güncelleme, tarayıcı uzantısı ve bağlantı servisi.
use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::error::{Hata, Sonuc};
use sp_core::gelismis_arac::{
    baglanti_json_ayristir, guncelleme_json_ayristir, kb_gecerli_mi, manifest_alani, mesaj_adi_coz,
    uzanti_kimlik_olustur, yol_kok_altinda, Baglanti, Guncelleme, TarayiciUzantisi,
};
use sp_core::sistem_yonetimi::serde_json::{self, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TARAMA_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const KALDIRMA_ZAMAN_ASIMI: Duration = Duration::from_secs(1800);
const CIKTI_SINIRI: usize = 800;
const KARANTINA_AD_SINIRI: usize = 64;
const KB_ONEK_UZUNLUGU: usize = 2;

const GUNCELLEME_BETIGI: &str = r#"$ErrorActionPreference='Stop'; try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}; try { Get-HotFix | Select-Object @{N='kb';E={ if ($_.HotFixID -match '^KB[0-9]{6,8}$') { $_.HotFixID } else { $null } }},@{N='baslik';E={ "$($_.Description) ($($_.HotFixID))" }},@{N='tur';E={ [string]$_.Description }},@{N='tarih';E={ if ($_.InstalledOn) { $_.InstalledOn.ToString('yyyy-MM-dd') } else { $null } }},@{N='kaldirilabilir';E={ [bool]($_.HotFixID -match '^KB[0-9]{6,8}$') }} | ConvertTo-Json -Compress } catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }"#;

const BAGLANTI_BETIGI: &str = r#"$ErrorActionPreference='Stop'; try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}; try { $surecler=@{}; Get-Process -ErrorAction SilentlyContinue | ForEach-Object { $surecler[[int]$_.Id]=$_.ProcessName }; $tcp=@(Get-NetTCPConnection -ErrorAction Stop | ForEach-Object { [pscustomobject]@{ protokol='TCP'; yerel_adres=[string]$_.LocalAddress; yerel_port=[int]$_.LocalPort; uzak_adres=[string]$_.RemoteAddress; uzak_port=[int]$_.RemotePort; durum=[string]$_.State; pid=[int]$_.OwningProcess; surec=[string]$surecler[[int]$_.OwningProcess] } }); $udp=@(Get-NetUDPEndpoint -ErrorAction Stop | ForEach-Object { [pscustomobject]@{ protokol='UDP'; yerel_adres=[string]$_.LocalAddress; yerel_port=[int]$_.LocalPort; uzak_adres=''; uzak_port=0; durum=''; pid=[int]$_.OwningProcess; surec=[string]$surecler[[int]$_.OwningProcess] } }); ConvertTo-Json -InputObject (@($tcp)+@($udp)) -Compress } catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }"#;

#[derive(Clone)]
pub struct GelismisAracServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl GelismisAracServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    pub fn guncellemeleri_listele(&self) -> Sonuc<Vec<Guncelleme>> {
        let sonuc = self.powershell(
            GUNCELLEME_BETIGI,
            "Güncelleme listesi alınamadı",
            TARAMA_ZAMAN_ASIMI,
        )?;
        let mut guncellemeler = guncelleme_json_ayristir(&sonuc.stdout)?;
        guncellemeler.sort_by(|a, b| {
            b.tarih
                .cmp(&a.tarih)
                .then_with(|| a.baslik.to_lowercase().cmp(&b.baslik.to_lowercase()))
        });
        Ok(guncellemeler)
    }

    pub fn guncelleme_kaldir(&self, kb: &str) -> Sonuc<KomutSonucu> {
        let kb = kb.trim();
        if !kb_gecerli_mi(kb) {
            return Err(Hata::Dogrulama(format!(
                "Geçersiz güncelleme kimliği: {kb}"
            )));
        }
        let numara = &kb[KB_ONEK_UZUNLUGU..];
        self.calistirici.calistir(
            "wusa.exe",
            &[
                "/uninstall".to_string(),
                format!("/kb:{numara}"),
                "/quiet".to_string(),
                "/norestart".to_string(),
            ],
            KALDIRMA_ZAMAN_ASIMI,
        )
    }

    pub fn tarayici_uzantilari(&self) -> Sonuc<Vec<TarayiciUzantisi>> {
        let mut uzantilar = Vec::new();
        for (tarayici, veri, dogrudan) in chromium_kaynaklari() {
            uzantilar.extend(chromium_tara(tarayici, &veri, dogrudan));
        }
        if let Some(profil_koku) = firefox_profil_koku() {
            uzantilar.extend(firefox_uzantilari(&profil_koku));
        }
        sirala_uzantilar(&mut uzantilar);
        Ok(uzantilar)
    }

    pub fn uzanti_kaldir(&self, uzanti: &TarayiciUzantisi, karantina: &Path) -> Sonuc<()> {
        let kokler = bilinen_uzanti_kokleri();
        uzanti_kaldir_koklerle(uzanti, karantina, &kokler)
    }

    pub fn baglantilari_listele(&self) -> Sonuc<Vec<Baglanti>> {
        let sonuc = self.powershell(
            BAGLANTI_BETIGI,
            "Ağ bağlantıları alınamadı",
            TARAMA_ZAMAN_ASIMI,
        )?;
        baglanti_json_ayristir(&sonuc.stdout)
    }

    fn powershell(&self, betik: &str, baslik: &str, zaman_asimi: Duration) -> Sonuc<KomutSonucu> {
        let sonuc = self.calistirici.calistir(
            "powershell.exe",
            &[
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                betik.to_string(),
            ],
            zaman_asimi,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{baslik} (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(sonuc)
    }
}

fn chromium_kaynaklari() -> Vec<(&'static str, PathBuf, bool)> {
    let mut kaynaklar = Vec::new();
    if let Some(yerel) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        for (ad, goreli) in [
            ("Chrome", r"Google\Chrome\User Data"),
            ("Edge", r"Microsoft\Edge\User Data"),
            ("Brave", r"BraveSoftware\Brave-Browser\User Data"),
            ("Vivaldi", r"Vivaldi\User Data"),
            ("Yandex", r"Yandex\YandexBrowser\User Data"),
        ] {
            kaynaklar.push((ad, yerel.join(goreli), false));
        }
    }
    if let Some(gezgin) = std::env::var_os("APPDATA").map(PathBuf::from) {
        kaynaklar.push(("Opera", gezgin.join(r"Opera Software\Opera Stable"), true));
    }
    kaynaklar
}

fn firefox_profil_koku() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|kok| PathBuf::from(kok).join(r"Mozilla\Firefox\Profiles"))
}

fn chromium_tara(tarayici: &str, veri: &Path, dogrudan: bool) -> Vec<TarayiciUzantisi> {
    if dogrudan {
        let profil = profil_adi(veri);
        return chromium_profil_tara(tarayici, &profil, &veri.join("Extensions"));
    }
    let Ok(profiller) = std::fs::read_dir(veri) else {
        return Vec::new();
    };
    let mut uzantilar = Vec::new();
    for girdi in profiller.flatten() {
        if !girdi.file_type().map(|tur| tur.is_dir()).unwrap_or(false) {
            continue;
        }
        let profil = girdi.file_name().to_string_lossy().to_string();
        uzantilar.extend(chromium_profil_tara(
            tarayici,
            &profil,
            &girdi.path().join("Extensions"),
        ));
    }
    uzantilar
}

fn chromium_profil_tara(tarayici: &str, profil: &str, uzanti_koku: &Path) -> Vec<TarayiciUzantisi> {
    let Ok(girdiler) = std::fs::read_dir(uzanti_koku) else {
        return Vec::new();
    };
    let mut uzantilar = Vec::new();
    for girdi in girdiler.flatten() {
        if !girdi.file_type().map(|tur| tur.is_dir()).unwrap_or(false) {
            continue;
        }
        let kimlik = girdi.file_name().to_string_lossy().to_string();
        if let Some(uzanti) = chromium_uzantisi(tarayici, profil, &kimlik, &girdi.path()) {
            uzantilar.push(uzanti);
        }
    }
    uzantilar
}

fn chromium_uzantisi(
    tarayici: &str,
    profil: &str,
    kimlik: &str,
    uzanti_yolu: &Path,
) -> Option<TarayiciUzantisi> {
    let (surum, manifest, yol) = en_iyi_manifest(uzanti_yolu)?;
    let (ham_ad, _, dil) = manifest_alani(&manifest);
    let ham_ad = ham_ad.unwrap_or_else(|| kimlik.to_string());
    let ad = yerel_ad(&ham_ad, &yol, dil.as_deref()).unwrap_or_else(|| kimlik.to_string());
    Some(TarayiciUzantisi {
        kimlik: uzanti_kimlik_olustur(tarayici, profil, kimlik),
        tarayici: tarayici.to_string(),
        profil: profil.to_string(),
        ad,
        surum,
        yol: yol.to_string_lossy().to_string(),
        etkin: true,
    })
}

fn en_iyi_manifest(uzanti_yolu: &Path) -> Option<(String, String, PathBuf)> {
    let Ok(girdiler) = std::fs::read_dir(uzanti_yolu) else {
        return None;
    };
    let mut adaylar = Vec::new();
    for girdi in girdiler.flatten() {
        if !girdi.file_type().map(|tur| tur.is_dir()).unwrap_or(false) {
            continue;
        }
        let Ok(manifest) = std::fs::read_to_string(girdi.path().join("manifest.json")) else {
            continue;
        };
        let (_, surum, _) = manifest_alani(&manifest);
        let surum = surum.unwrap_or_else(|| girdi.file_name().to_string_lossy().to_string());
        adaylar.push((surum, manifest, girdi.path()));
    }
    adaylar.into_iter().max_by(|a, b| surum_kiyasla(&a.0, &b.0))
}

fn surum_kiyasla(a: &str, b: &str) -> std::cmp::Ordering {
    let parcalar = |metin: &str| {
        metin
            .split('.')
            .map(|parca| parca.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    parcalar(a).cmp(&parcalar(b))
}

fn yerel_ad(ham: &str, uzanti_yolu: &Path, dil: Option<&str>) -> Option<String> {
    if !ham.starts_with("__MSG_") || !ham.ends_with("__") {
        return Some(ham.to_string());
    }
    let mut diller = Vec::new();
    if let Some(dil) = dil {
        diller.push(dil.to_string());
        if let Some(kok) = dil.split('_').next() {
            if !diller.iter().any(|deger| deger == kok) {
                diller.push(kok.to_string());
            }
        }
    }
    diller.push("en".to_string());
    let yereller = uzanti_yolu.join("_locales");
    if let Ok(girdiler) = std::fs::read_dir(&yereller) {
        for girdi in girdiler.flatten() {
            let ad = girdi.file_name().to_string_lossy().to_string();
            if !diller.iter().any(|deger| deger.eq_ignore_ascii_case(&ad)) {
                diller.push(ad);
            }
        }
    }
    for dil in diller {
        let Ok(metin) = std::fs::read_to_string(yereller.join(&dil).join("messages.json")) else {
            continue;
        };
        if let Some(ad) = mesaj_adi_coz(&metin, ham) {
            return Some(ad);
        }
    }
    None
}

fn firefox_uzantilari(profil_koku: &Path) -> Vec<TarayiciUzantisi> {
    let Ok(profiller) = std::fs::read_dir(profil_koku) else {
        return Vec::new();
    };
    let mut uzantilar = Vec::new();
    for girdi in profiller.flatten() {
        if !girdi.file_type().map(|tur| tur.is_dir()).unwrap_or(false) {
            continue;
        }
        let Ok(metin) = std::fs::read_to_string(girdi.path().join("extensions.json")) else {
            continue;
        };
        let profil = girdi.file_name().to_string_lossy().to_string();
        uzantilar.extend(firefox_profil_uzantilari(&metin, &profil));
    }
    uzantilar
}

fn firefox_profil_uzantilari(json: &str, profil: &str) -> Vec<TarayiciUzantisi> {
    let metin = json.trim_start_matches('\u{feff}');
    let Ok(veri) = serde_json::from_str::<Value>(metin) else {
        return Vec::new();
    };
    let Some(eklentiler) = veri.get("addons").and_then(Value::as_array) else {
        return Vec::new();
    };
    eklentiler
        .iter()
        .filter_map(|eklenti| firefox_uzantisi(eklenti, profil))
        .collect()
}

fn firefox_uzantisi(eklenti: &Value, profil: &str) -> Option<TarayiciUzantisi> {
    if eklenti.get("type").and_then(Value::as_str) != Some("extension") {
        return None;
    }
    let kimlik = eklenti
        .get("id")
        .and_then(Value::as_str)
        .filter(|deger| !deger.is_empty())?;
    let yol = eklenti
        .get("path")
        .and_then(Value::as_str)
        .filter(|deger| !deger.is_empty())?;
    let ham_ad = eklenti
        .get("defaultLocale")
        .and_then(|yerel| yerel.get("name"))
        .and_then(Value::as_str)
        .or_else(|| eklenti.get("name").and_then(Value::as_str))
        .map(str::trim)
        .filter(|ad| !ad.is_empty())
        .unwrap_or(kimlik);
    Some(TarayiciUzantisi {
        kimlik: uzanti_kimlik_olustur("Firefox", profil, kimlik),
        tarayici: "Firefox".to_string(),
        profil: profil.to_string(),
        ad: yerel_ad(ham_ad, Path::new(yol), None).unwrap_or_else(|| kimlik.to_string()),
        surum: eklenti
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        yol: yol.to_string(),
        etkin: eklenti
            .get("active")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn sirala_uzantilar(uzantilar: &mut [TarayiciUzantisi]) {
    uzantilar.sort_by(|a, b| {
        a.tarayici
            .to_lowercase()
            .cmp(&b.tarayici.to_lowercase())
            .then_with(|| a.profil.to_lowercase().cmp(&b.profil.to_lowercase()))
            .then_with(|| a.ad.to_lowercase().cmp(&b.ad.to_lowercase()))
            .then_with(|| a.kimlik.cmp(&b.kimlik))
    });
}

fn profil_adi(veri: &Path) -> String {
    veri.file_name()
        .map(|ad| ad.to_string_lossy().to_string())
        .unwrap_or_else(|| "Varsayılan".to_string())
}

fn bilinen_uzanti_kokleri() -> Vec<PathBuf> {
    let mut kokler = Vec::new();
    for (_, veri, dogrudan) in chromium_kaynaklari() {
        if dogrudan {
            uzanti_koku_ekle(&veri.join("Extensions"), &mut kokler);
            continue;
        }
        let Ok(profiller) = std::fs::read_dir(&veri) else {
            continue;
        };
        for girdi in profiller.flatten() {
            if girdi.file_type().map(|tur| tur.is_dir()).unwrap_or(false) {
                uzanti_koku_ekle(&girdi.path().join("Extensions"), &mut kokler);
            }
        }
    }
    if let Some(profil_koku) = firefox_profil_koku() {
        if let Ok(profiller) = std::fs::read_dir(profil_koku) {
            for girdi in profiller.flatten() {
                if girdi.file_type().map(|tur| tur.is_dir()).unwrap_or(false) {
                    uzanti_koku_ekle(&girdi.path().join("extensions"), &mut kokler);
                }
            }
        }
    }
    kokler
}

fn uzanti_koku_ekle(yol: &Path, kokler: &mut Vec<PathBuf>) {
    if let Ok(kanonik) = std::fs::canonicalize(yol) {
        if kanonik.is_dir() {
            kokler.push(kanonik);
        }
    }
}

fn uzanti_kaldir_koklerle(
    uzanti: &TarayiciUzantisi,
    karantina: &Path,
    kokler: &[PathBuf],
) -> Sonuc<()> {
    let kaynak = std::fs::canonicalize(uzanti.yol.trim())
        .map_err(|hata| Hata::Dogrulama(format!("Uzantı yolu doğrulanamadı: {hata}")))?;
    if !kokler.iter().any(|kok| yol_kok_altinda(kok, &kaynak)) {
        return Err(Hata::Yetki(
            "Uzantı yolu bilinen tarayıcı uzantı köklerinin dışında.".into(),
        ));
    }
    let meta = std::fs::symlink_metadata(&kaynak)
        .map_err(|hata| Hata::DisArac(format!("Uzantı bilgisi okunamadı: {hata}")))?;
    std::fs::create_dir_all(karantina)
        .map_err(|hata| Hata::DisArac(format!("Karantina klasörü oluşturulamadı: {hata}")))?;
    let ad = kaynak
        .file_name()
        .map(|deger| deger.to_string_lossy().to_string())
        .unwrap_or_else(|| "uzanti".to_string());
    let hedef = benzersiz_hedef(karantina, &ad);
    tasi_veya_kopyala(&kaynak, &hedef, meta.is_dir())
}

fn benzersiz_hedef(karantina: &Path, ad: &str) -> PathBuf {
    let temel: String = ad
        .chars()
        .filter(|karakter| karakter.is_alphanumeric() || matches!(karakter, '.' | '-' | '_'))
        .take(KARANTINA_AD_SINIRI)
        .collect();
    let temel = if temel.is_empty() {
        "uzanti".to_string()
    } else {
        temel
    };
    let zaman = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|sure| sure.as_millis())
        .unwrap_or_default();
    let mut hedef = karantina.join(format!("{temel}_{}_{}", std::process::id(), zaman));
    let mut sayac = 2u32;
    while hedef.exists() {
        hedef = karantina.join(format!("{temel}_{}_{}_{sayac}", std::process::id(), zaman));
        sayac += 1;
    }
    hedef
}

fn tasi_veya_kopyala(kaynak: &Path, hedef: &Path, dizin: bool) -> Sonuc<()> {
    if std::fs::rename(kaynak, hedef).is_ok() {
        return Ok(());
    }
    if dizin {
        kopyala_dizin(kaynak, hedef)?;
        return std::fs::remove_dir_all(kaynak)
            .map_err(|hata| Hata::DisArac(format!("Uzantı klasörü kaldırılamadı: {hata}")));
    }
    std::fs::copy(kaynak, hedef)
        .map_err(|hata| Hata::DisArac(format!("Uzantı dosyası kopyalanamadı: {hata}")))?;
    std::fs::remove_file(kaynak)
        .map_err(|hata| Hata::DisArac(format!("Uzantı dosyası kaldırılamadı: {hata}")))
}

fn kopyala_dizin(kaynak: &Path, hedef: &Path) -> Sonuc<()> {
    std::fs::create_dir_all(hedef)
        .map_err(|hata| Hata::DisArac(format!("Karantina klasörü oluşturulamadı: {hata}")))?;
    let girdiler = std::fs::read_dir(kaynak)
        .map_err(|hata| Hata::DisArac(format!("Uzantı klasörü okunamadı: {hata}")))?;
    for girdi in girdiler.flatten() {
        let tur = girdi
            .file_type()
            .map_err(|hata| Hata::DisArac(format!("Uzantı dosyası okunamadı: {hata}")))?;
        if tur.is_symlink() {
            continue;
        }
        let hedef_yol = hedef.join(girdi.file_name());
        if tur.is_dir() {
            kopyala_dizin(&girdi.path(), &hedef_yol)?;
        } else if tur.is_file() {
            std::fs::copy(girdi.path(), &hedef_yol)
                .map_err(|hata| Hata::DisArac(format!("Uzantı dosyası kopyalanamadı: {hata}")))?;
        }
    }
    Ok(())
}

fn hata_ozeti(sonuc: &KomutSonucu) -> String {
    let metin = if sonuc.stderr.trim().is_empty() {
        sonuc.stdout.trim()
    } else {
        sonuc.stderr.trim()
    };
    let ilk = metin
        .lines()
        .map(str::trim)
        .find(|satir| !satir.is_empty())
        .unwrap_or("çıktı yok");
    ilk.chars().take(CIKTI_SINIRI).collect()
}

#[cfg(test)]
mod testler {
    use super::*;
    use sp_core::gelismis_arac::uc_nokta;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct SahteCalistirici {
        cevaplar: Mutex<VecDeque<KomutSonucu>>,
        cagrilar: Mutex<Vec<(String, Vec<String>, Duration)>>,
    }

    impl KomutCalistirici for SahteCalistirici {
        fn calistir(
            &self,
            program: &str,
            argumanlar: &[String],
            zaman_asimi: Duration,
        ) -> Sonuc<KomutSonucu> {
            self.cagrilar.lock().unwrap().push((
                program.to_string(),
                argumanlar.to_vec(),
                zaman_asimi,
            ));
            self.cevaplar
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| Hata::DisArac("test cevabı yok".into()))
        }
    }

    fn cevap(stdout: &str, kod: i32) -> KomutSonucu {
        KomutSonucu {
            komut: "test".into(),
            kod,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn servis(cevaplar: Vec<KomutSonucu>) -> (GelismisAracServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            cevaplar: Mutex::new(VecDeque::from(cevaplar)),
            cagrilar: Mutex::new(Vec::new()),
        });
        (GelismisAracServisi::yeni(calistirici.clone()), calistirici)
    }

    #[test]
    fn guncelleme_listesi_tarihe_gore_siralanir() {
        let json = r#"[
            {"kb":"KB5034441","baslik":"Eski","tur":"Update","tarih":"2025-12-01","kaldirilabilir":true},
            {"kb":"KB5030509","baslik":"Yeni","tur":"Security Update","tarih":"2026-03-05","kaldirilabilir":false},
            {"kb":null,"baslik":"Tarihsiz","tur":"Hotfix","tarih":null,"kaldirilabilir":false}
        ]"#;
        let (servis, calistirici) = servis(vec![cevap(json, 0)]);
        let liste = servis.guncellemeleri_listele().unwrap();
        assert_eq!(liste.len(), 3);
        assert_eq!(liste[0].kb.as_deref(), Some("KB5030509"));
        assert_eq!(liste[1].kb.as_deref(), Some("KB5034441"));
        assert_eq!(liste[2].baslik, "Tarihsiz");
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "powershell.exe");
        let betik = cagrilar[0].1.last().unwrap();
        assert!(betik.contains("Get-HotFix"));
        assert!(betik.contains("ConvertTo-Json"));
    }

    #[test]
    fn gecersiz_kb_kaldirma_komutu_calistirmaz() {
        let (servis, calistirici) = servis(Vec::new());
        for kb in [
            "",
            "KB12345",
            "KB123456789",
            "KB12345; calc",
            "KB5034441 && calc",
            "KB 5034441",
        ] {
            assert!(servis.guncelleme_kaldir(kb).is_err());
        }
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn guncelleme_kaldirma_wusa_ile_guvenli_argumanlar_kullanir() {
        let (servis, calistirici) = servis(vec![cevap("", 3010)]);
        let sonuc = servis.guncelleme_kaldir("kb5034441").unwrap();
        assert_eq!(sonuc.kod, 3010);
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "wusa.exe");
        assert_eq!(
            cagrilar[0].1,
            vec!["/uninstall", "/kb:5034441", "/quiet", "/norestart"]
        );
        assert_eq!(cagrilar[0].2, Duration::from_secs(1800));
    }

    #[test]
    fn baglanti_listesi_powershell_ciktisindan_okunur() {
        let json = r#"[
            {"protokol":"TCP","yerel_adres":"::1","yerel_port":443,"uzak_adres":"::1","uzak_port":51000,"durum":"Established","pid":4321,"surec":"chrome"},
            {"protokol":"UDP","yerel_adres":"0.0.0.0","yerel_port":"5353","uzak_adres":"","uzak_port":"0","durum":"","pid":"700","surec":"svchost"}
        ]"#;
        let (servis, calistirici) = servis(vec![cevap(json, 0)]);
        let liste = servis.baglantilari_listele().unwrap();
        assert_eq!(liste.len(), 2);
        assert_eq!(liste[0].pid, 4321);
        assert_eq!(
            uc_nokta(&liste[0].yerel_adres, liste[0].yerel_port),
            "[::1]:443"
        );
        assert_eq!(liste[1].yerel_port, 5353);
        assert_eq!(liste[1].surec, "svchost");
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        let betik = cagrilar[0].1.last().unwrap();
        assert!(betik.contains("Get-NetTCPConnection"));
        assert!(betik.contains("Get-NetUDPEndpoint"));
        assert!(betik.contains("Get-Process"));
    }

    #[test]
    fn powershell_hatasi_dis_arac_hatasina_donusur() {
        let mut hatali = cevap("", 1);
        hatali.stderr = "Get-NetTCPConnection : terim bulunamadı".into();
        let (servis, _calistirici) = servis(vec![hatali]);
        let hata = servis.baglantilari_listele().unwrap_err();
        assert!(matches!(hata, Hata::DisArac(_)));
        assert!(hata.to_string().contains("Get-NetTCPConnection"));
    }

    fn ornek_uzanti(yol: &Path) -> TarayiciUzantisi {
        TarayiciUzantisi {
            kimlik: "Chrome_Profil_test".into(),
            tarayici: "Chrome".into(),
            profil: "Profil".into(),
            ad: "Test".into(),
            surum: "1.0".into(),
            yol: yol.to_string_lossy().to_string(),
            etkin: true,
        }
    }

    #[test]
    fn uzanti_kaldirma_guvenli_kok_denetimi_yapar() {
        let kok = tempfile::tempdir().unwrap();
        let ext_koku = kok.path().join("Profil").join("Extensions");
        let surum = ext_koku
            .join("abcdefghijklmnopabcdefghijklmnop")
            .join("1.0_0");
        std::fs::create_dir_all(&surum).unwrap();
        std::fs::write(surum.join("manifest.json"), "{}").unwrap();
        let xpi = ext_koku.join("eklenti@ornek.xpi");
        std::fs::write(&xpi, b"xpi").unwrap();
        let karantina = tempfile::tempdir().unwrap();
        let kokler = vec![std::fs::canonicalize(&ext_koku).unwrap()];

        uzanti_kaldir_koklerle(&ornek_uzanti(&surum), karantina.path(), &kokler).unwrap();
        assert!(!surum.exists());
        uzanti_kaldir_koklerle(&ornek_uzanti(&xpi), karantina.path(), &kokler).unwrap();
        assert!(!xpi.exists());
        assert_eq!(std::fs::read_dir(karantina.path()).unwrap().count(), 2);

        let disarida = tempfile::tempdir().unwrap();
        let korunacak = disarida.path().join("korunacak.txt");
        std::fs::write(&korunacak, b"veri").unwrap();
        let hata = uzanti_kaldir_koklerle(&ornek_uzanti(&korunacak), karantina.path(), &kokler)
            .unwrap_err();
        assert!(matches!(hata, Hata::Yetki(_)));
        assert!(korunacak.exists());

        let hata = uzanti_kaldir_koklerle(&ornek_uzanti(&ext_koku), karantina.path(), &kokler)
            .unwrap_err();
        assert!(matches!(hata, Hata::Yetki(_)));
        let hata = uzanti_kaldir_koklerle(
            &ornek_uzanti(&ext_koku.join("yok")),
            karantina.path(),
            &kokler,
        )
        .unwrap_err();
        assert!(matches!(hata, Hata::Dogrulama(_)));
    }

    #[test]
    fn chromium_uzantilari_manifest_ve_yerel_mesajlari_cozer() {
        let kok = tempfile::tempdir().unwrap();
        let uzantilar = kok.path().join("Default").join("Extensions");
        let eski = uzantilar
            .join("abcdefghijklmnopabcdefghijklmnop")
            .join("1.0_0");
        let yeni = uzantilar
            .join("abcdefghijklmnopabcdefghijklmnop")
            .join("2.0_0");
        std::fs::create_dir_all(eski.join("_locales").join("en")).unwrap();
        std::fs::create_dir_all(yeni.join("_locales").join("tr")).unwrap();
        std::fs::write(
            eski.join("manifest.json"),
            r#"{"name":"Eski Ad","version":"1.0","default_locale":"en"}"#,
        )
        .unwrap();
        std::fs::write(
            yeni.join("manifest.json"),
            r#"{"name":"__MSG_eklenti_adi__","version":"2.0","default_locale":"tr"}"#,
        )
        .unwrap();
        std::fs::write(
            yeni.join("_locales").join("tr").join("messages.json"),
            r#"{"eklenti_adi":{"message":"Türkçe Ad"}}"#,
        )
        .unwrap();
        let adsiz = uzantilar
            .join("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
            .join("1.0_0");
        std::fs::create_dir_all(&adsiz).unwrap();
        std::fs::write(
            adsiz.join("manifest.json"),
            r#"{"name":"__MSG_yok__","version":"1.0"}"#,
        )
        .unwrap();

        let liste = chromium_tara("Chrome", kok.path(), false);
        assert_eq!(liste.len(), 2);
        let cozulen = liste
            .iter()
            .find(|uzanti| uzanti.kimlik.contains("abcdefghijklmnop"))
            .unwrap();
        assert_eq!(cozulen.ad, "Türkçe Ad");
        assert_eq!(cozulen.surum, "2.0");
        assert!(cozulen.yol.ends_with("2.0_0"));
        let kimlige_dusen = liste
            .iter()
            .find(|uzanti| uzanti.kimlik.contains("bbbbbbbb"))
            .unwrap();
        assert_eq!(kimlige_dusen.ad, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        assert!(liste.iter().all(|uzanti| uzanti.etkin));
    }

    #[test]
    fn firefox_eklentileri_yalnizca_etkin_uzantilari_listeler() {
        let kok = tempfile::tempdir().unwrap();
        let profil = kok.path().join("abc.default-release");
        let ext = profil.join("extensions");
        std::fs::create_dir_all(&ext).unwrap();
        let xpi = ext.join("eklenti@ornek.xpi");
        std::fs::write(&xpi, b"xpi").unwrap();
        let json = serde_json::json!({
            "addons": [
                {
                    "id": "eklenti@ornek",
                    "type": "extension",
                    "active": true,
                    "version": "3.1",
                    "path": xpi.to_string_lossy(),
                    "defaultLocale": {"name": "Örnek Eklenti"}
                },
                {
                    "id": "tema@ornek",
                    "type": "theme",
                    "active": true,
                    "version": "1",
                    "path": xpi.to_string_lossy()
                },
                {
                    "id": "pasif@ornek",
                    "type": "extension",
                    "active": false,
                    "version": "1",
                    "path": xpi.to_string_lossy()
                }
            ]
        })
        .to_string();
        std::fs::write(profil.join("extensions.json"), json).unwrap();

        let liste = firefox_uzantilari(kok.path());
        assert_eq!(liste.len(), 2);
        let eklenti = liste
            .iter()
            .find(|uzanti| uzanti.kimlik.contains("eklenti@ornek"))
            .unwrap();
        assert_eq!(eklenti.ad, "Örnek Eklenti");
        assert_eq!(eklenti.surum, "3.1");
        assert!(eklenti.etkin);
        assert!(eklenti.yol.ends_with("eklenti@ornek.xpi"));
        let pasif = liste
            .iter()
            .find(|uzanti| uzanti.kimlik.contains("pasif@ornek"))
            .unwrap();
        assert!(!pasif.etkin);
        assert!(liste.iter().all(|uzanti| uzanti.tarayici == "Firefox"));
    }
}
