use crate::adb::paket_dogrula;
use crate::error::{Hata, Sonuc};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

pub const UAD_URL: &str = "https://raw.githubusercontent.com/Universal-Debloater-Alliance/universal-android-debloater-next-generation/main/resources/assets/uad_lists.json";
pub const UAD_LISANS: &str = "GPL-3.0";
pub const PLATFORM_TOOLS_URL: &str =
    "https://dl.google.com/android/repository/platform-tools-latest-windows.zip";
pub const GOOGLE_USB_DRIVER_URL: &str =
    "https://dl.google.com/android/repository/usb_driver_r13-windows.zip";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    Recommended,
    Advanced,
    Expert,
    Unsafe,
    Bilinmiyor,
}

impl Risk {
    pub fn uad_metninden(metin: &str) -> Self {
        match metin.trim().to_ascii_lowercase().as_str() {
            "recommended" => Self::Recommended,
            "advanced" => Self::Advanced,
            "expert" => Self::Expert,
            "unsafe" => Self::Unsafe,
            _ => Self::Bilinmiyor,
        }
    }

    pub fn metin(self) -> &'static str {
        match self {
            Self::Recommended => "Önerilen",
            Self::Advanced => "Gelişmiş",
            Self::Expert => "Uzman",
            Self::Unsafe => "Güvensiz",
            Self::Bilinmiyor => "Bilinmiyor",
        }
    }

    pub fn toplu_islem_uygun(self) -> bool {
        matches!(self, Self::Recommended | Self::Advanced)
    }

    pub fn uzman_onayi_gerekir(self) -> bool {
        matches!(self, Self::Expert | Self::Bilinmiyor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UadKaydi {
    #[serde(default)]
    list: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default, rename = "neededBy")]
    needed_by: Vec<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    removal: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaketBilgisi {
    pub id: String,
    pub liste: String,
    pub aciklama: String,
    pub bagimliliklar: Vec<String>,
    pub ihtiyac_duyanlar: Vec<String>,
    pub etiketler: Vec<String>,
    pub risk: Risk,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UadVeritabani {
    pub paketler: HashMap<String, PaketBilgisi>,
    pub gecersiz_kayit: usize,
}

impl UadVeritabani {
    pub fn bilgi(&self, id: &str) -> Option<&PaketBilgisi> {
        self.paketler.get(id)
    }
}

pub fn uad_ayristir(json: &str) -> Sonuc<UadVeritabani> {
    let ham: HashMap<String, UadKaydi> = serde_json::from_str(json)
        .map_err(|hata| Hata::Beklenmeyen(format!("UAD veritabanı ayrıştırılamadı: {hata}")))?;
    let mut veritabani = UadVeritabani::default();
    for (id, kayit) in ham {
        if paket_dogrula(&id).is_err() {
            veritabani.gecersiz_kayit += 1;
            continue;
        }
        veritabani.paketler.insert(
            id.clone(),
            PaketBilgisi {
                id,
                liste: kayit.list.trim().to_string(),
                aciklama: kayit.description.trim().to_string(),
                bagimliliklar: temiz_liste(kayit.dependencies),
                ihtiyac_duyanlar: temiz_liste(kayit.needed_by),
                etiketler: temiz_liste(kayit.labels),
                risk: Risk::uad_metninden(&kayit.removal),
            },
        );
    }
    if veritabani.paketler.is_empty() {
        return Err(Hata::Beklenmeyen(
            "UAD veritabanında geçerli paket bulunamadı.".into(),
        ));
    }
    Ok(veritabani)
}

fn temiz_liste(degerler: Vec<String>) -> Vec<String> {
    let mut gorulen = HashSet::new();
    degerler
        .into_iter()
        .map(|deger| deger.trim().to_string())
        .filter(|deger| !deger.is_empty() && gorulen.insert(deger.clone()))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaglantiDurumu {
    Bagli,
    Yetkisiz,
    Cevrimdisi,
    Bilinmiyor,
}

impl BaglantiDurumu {
    pub fn adb_metninden(metin: &str) -> Self {
        match metin {
            "device" => Self::Bagli,
            "unauthorized" => Self::Yetkisiz,
            "offline" => Self::Cevrimdisi,
            _ => Self::Bilinmiyor,
        }
    }

    pub fn metin(self) -> &'static str {
        match self {
            Self::Bagli => "Bağlı",
            Self::Yetkisiz => "Yetkisiz",
            Self::Cevrimdisi => "Çevrimdışı",
            Self::Bilinmiyor => "Bilinmiyor",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidCihaz {
    pub seri: String,
    pub durum: BaglantiDurumu,
    pub model: Option<String>,
    pub marka: Option<String>,
    pub uretici: Option<String>,
    pub android_surumu: Option<String>,
    pub sdk: Option<u32>,
}

impl AndroidCihaz {
    pub fn adb_hazir(&self) -> bool {
        self.durum == BaglantiDurumu::Bagli
    }

    pub fn baslik(&self) -> String {
        self.model
            .clone()
            .or_else(|| self.marka.clone())
            .unwrap_or_else(|| self.seri.clone())
    }
}

pub fn cihazlari_ayristir(stdout: &str) -> Vec<AndroidCihaz> {
    let mut cihazlar = Vec::new();
    for satir in stdout.lines() {
        let satir = satir.trim();
        if satir.is_empty() || satir.starts_with("List of devices") || satir.starts_with('*') {
            continue;
        }
        let parcalar = satir.split_whitespace().collect::<Vec<_>>();
        if parcalar.len() < 2 || !crate::adb::seri_gecerli(parcalar[0]) {
            continue;
        }
        let durum = BaglantiDurumu::adb_metninden(parcalar[1]);
        if durum == BaglantiDurumu::Bilinmiyor {
            continue;
        }
        let model = parcalar
            .iter()
            .find_map(|parca| parca.strip_prefix("model:"))
            .map(|model| model.replace('_', " "));
        cihazlar.push(AndroidCihaz {
            seri: parcalar[0].to_string(),
            durum,
            model,
            marka: None,
            uretici: None,
            android_surumu: None,
            sdk: None,
        });
    }
    cihazlar
}

pub fn getprop_ayristir(stdout: &str) -> HashMap<String, String> {
    let mut degerler = HashMap::new();
    for satir in stdout.lines() {
        let Some(kalan) = satir.trim().strip_prefix('[') else {
            continue;
        };
        let Some(ayrac) = kalan.find("]: [") else {
            continue;
        };
        let anahtar = &kalan[..ayrac];
        let deger = &kalan[ayrac + 4..];
        let Some(deger) = deger.strip_suffix(']') else {
            continue;
        };
        if !anahtar.is_empty() && !deger.is_empty() {
            degerler.insert(anahtar.to_string(), deger.to_string());
        }
    }
    degerler
}

pub fn cihaz_profilini_zenginlestir(cihaz: &mut AndroidCihaz, degerler: &HashMap<String, String>) {
    cihaz.marka = degerler.get("ro.product.brand").cloned();
    cihaz.uretici = degerler.get("ro.product.manufacturer").cloned();
    cihaz.android_surumu = degerler.get("ro.build.version.release").cloned();
    cihaz.sdk = degerler
        .get("ro.build.version.sdk")
        .and_then(|sdk| sdk.parse::<u32>().ok());
    if cihaz.model.is_none() {
        cihaz.model = degerler.get("ro.product.model").cloned();
    }
}

pub fn marka_uyarilari(marka: Option<&str>, android_surumu: Option<&str>) -> Vec<String> {
    let marka = marka.unwrap_or_default().to_ascii_lowercase();
    let mut uyarilar =
        vec!["Telefon ekranındaki RSA USB hata ayıklama iznini onaylayın.".to_string()];
    if marka.contains("xiaomi") || marka.contains("redmi") || marka.contains("poco") {
        uyarilar.push(
            "Xiaomi/HyperOS: Geliştirici seçeneklerinde USB hata ayıklama (Güvenlik Ayarları) ve USB üzerinden yükle iznini açın; Xiaomi hesabı ve SIM gerekebilir."
                .to_string(),
        );
    }
    if marka.contains("oppo")
        || marka.contains("realme")
        || marka.contains("oneplus")
        || marka.contains("oplus")
    {
        uyarilar.push(
            "Oppo/Realme/OnePlus: Geliştirici seçeneklerinin altındaki İzin İzlemeyi Devre Dışı Bırak seçeneğini açın."
                .to_string(),
        );
    }
    if marka.contains("samsung") && android_surumu.and_then(android_major).unwrap_or(0) >= 14 {
        uyarilar.push(
            "Samsung Android 14+: USB hata ayıklamayı engelleyen Otomatik Engelleyici (Auto Blocker) ayarını kapatın."
                .to_string(),
        );
    }
    uyarilar
}

fn android_major(surum: &str) -> Option<u32> {
    surum
        .trim()
        .split('.')
        .next()
        .and_then(|parca| parca.parse().ok())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaketDurumu {
    pub id: String,
    pub sistem: bool,
    pub kullanici: bool,
    pub devre_disi: bool,
    pub kaldirilmis: bool,
}

pub fn paketleri_ayristir(stdout: &str) -> Vec<String> {
    let mut paketler = stdout
        .lines()
        .filter_map(|satir| satir.trim().strip_prefix("package:"))
        .filter(|paket| paket_dogrula(paket).is_ok())
        .map(str::to_string)
        .collect::<Vec<_>>();
    paketler.sort();
    paketler.dedup();
    paketler
}

pub fn paket_durumlari(
    etkin: &[String],
    devre_disi: &[String],
    kaldirilmis: &[String],
    sistem: &[String],
    kullanici: &[String],
) -> Vec<PaketDurumu> {
    let etkin = etkin.iter().cloned().collect::<HashSet<_>>();
    let devre_disi = devre_disi.iter().cloned().collect::<HashSet<_>>();
    let kaldirilmis = kaldirilmis.iter().cloned().collect::<HashSet<_>>();
    let sistem = sistem.iter().cloned().collect::<HashSet<_>>();
    let kullanici = kullanici.iter().cloned().collect::<HashSet<_>>();
    let mut kimlikler = etkin
        .union(&devre_disi)
        .chain(kaldirilmis.iter())
        .cloned()
        .collect::<Vec<_>>();
    kimlikler.sort();
    kimlikler.dedup();
    kimlikler
        .into_iter()
        .map(|id| PaketDurumu {
            sistem: sistem.contains(&id),
            kullanici: kullanici.contains(&id),
            devre_disi: devre_disi.contains(&id),
            kaldirilmis: kaldirilmis.contains(&id)
                && !etkin.contains(&id)
                && !devre_disi.contains(&id),
            id,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaketEylemi {
    Kaldir { verileri_koru: bool },
    DevreDisi,
    Etkinlestir,
    GeriYukle,
}

pub fn eylem_guvenli_mi(risk: Risk, eylem: PaketEylemi, uzman_onayi: bool) -> Sonuc<()> {
    if matches!(eylem, PaketEylemi::Etkinlestir | PaketEylemi::GeriYukle) {
        return Ok(());
    }
    if risk == Risk::Unsafe {
        return Err(Hata::Yetki(
            "Güvensiz olarak işaretlenen paket kaldırılamaz veya devre dışı bırakılamaz.".into(),
        ));
    }
    if risk.uzman_onayi_gerekir() && !uzman_onayi {
        return Err(Hata::Yetki("Bu paket için uzman onayı gerekiyor.".into()));
    }
    Ok(())
}

pub fn toplu_eylem_uygun_mu(risk: Risk) -> bool {
    risk.toplu_islem_uygun()
}

pub fn paket_komut_argumanlari(eylem: PaketEylemi, paket: &str, seri: &str) -> Sonuc<Vec<String>> {
    paket_dogrula(paket)?;
    if !crate::adb::seri_gecerli(seri) {
        return Err(Hata::Beklenmeyen("Geçersiz ADB cihaz serisi.".into()));
    }
    let mut argumanlar = vec!["-s".to_string(), seri.to_string(), "shell".to_string()];
    match eylem {
        PaketEylemi::Kaldir { verileri_koru } => {
            argumanlar.extend(["pm", "uninstall"].map(str::to_string));
            if verileri_koru {
                argumanlar.push("-k".to_string());
            }
            argumanlar.extend(["--user".to_string(), "0".to_string(), paket.to_string()]);
        }
        PaketEylemi::DevreDisi => {
            argumanlar.extend(["pm", "disable-user", "--user", "0", paket].map(str::to_string))
        }
        PaketEylemi::Etkinlestir => argumanlar.extend(["pm", "enable", paket].map(str::to_string)),
        PaketEylemi::GeriYukle => {
            argumanlar.extend(["cmd", "package", "install-existing", paket].map(str::to_string))
        }
    }
    Ok(argumanlar)
}

pub fn bagimlilik_uyarilari(
    bilgi: &PaketBilgisi,
    cihaz_paketleri: &HashSet<String>,
) -> Vec<String> {
    let mut uyarilar = Vec::new();
    for bagimlilik in &bilgi.bagimliliklar {
        if cihaz_paketleri.contains(bagimlilik) {
            uyarilar.push(format!("Bu paket {bagimlilik} paketine bağlı."));
        }
    }
    for ihtiyac in &bilgi.ihtiyac_duyanlar {
        if cihaz_paketleri.contains(ihtiyac) {
            uyarilar.push(format!("{ihtiyac} bu pakete ihtiyaç duyuyor."));
        }
    }
    uyarilar
}

pub fn vid_marka(vid: u16) -> Option<&'static str> {
    match vid {
        0x04E8 => Some("Samsung"),
        0x2717 => Some("Xiaomi/Poco/Redmi"),
        0x18D1 => Some("Google"),
        0x22D9 => Some("Oppo/Realme/OnePlus"),
        0x12D1 => Some("Huawei/Honor"),
        0x22B8 => Some("Motorola"),
        _ => None,
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    const FIKSTUR: &str = r#"{
        "com.miui.analytics": {"list":"Oem","description":"Analytics","dependencies":["com.miui.core"],"neededBy":[],"labels":["Analytics"],"removal":"Recommended"},
        "com.miui.core": {"list":"Oem","description":"Core","dependencies":[],"neededBy":["com.miui.analytics"],"labels":[],"removal":"Advanced"},
        "com.android.shared": {"list":"Aosp","description":"Shared","dependencies":[],"neededBy":[],"labels":[],"removal":"Unsafe"},
        "kotu paket": {"list":"Oem","description":"x","dependencies":[],"neededBy":[],"labels":[],"removal":"Recommended"}
    }"#;

    #[test]
    fn uad_veritabani_ayristirilir() {
        let db = uad_ayristir(FIKSTUR).unwrap();
        assert_eq!(db.paketler.len(), 3);
        assert_eq!(db.gecersiz_kayit, 1);
        assert_eq!(
            db.bilgi("com.miui.analytics").unwrap().risk,
            Risk::Recommended
        );
        assert_eq!(
            db.bilgi("com.miui.analytics").unwrap().bagimliliklar,
            vec!["com.miui.core"]
        );
        assert_eq!(db.bilgi("com.android.shared").unwrap().risk, Risk::Unsafe);
    }

    #[test]
    fn bilinmeyen_risk_guvenli_tarafa_duser() {
        let db = uad_ayristir(
            r#"{"com.ornek":{"list":"Oem","description":"","dependencies":[],"neededBy":[],"labels":[],"removal":"Bilinmeyen"}}"#,
        )
        .unwrap();
        assert_eq!(db.bilgi("com.ornek").unwrap().risk, Risk::Bilinmiyor);
    }

    #[test]
    fn risk_politikasi_uygulanir() {
        assert!(eylem_guvenli_mi(
            Risk::Recommended,
            PaketEylemi::Kaldir {
                verileri_koru: true
            },
            false
        )
        .is_ok());
        assert!(eylem_guvenli_mi(
            Risk::Expert,
            PaketEylemi::Kaldir {
                verileri_koru: false
            },
            false
        )
        .is_err());
        assert!(eylem_guvenli_mi(
            Risk::Expert,
            PaketEylemi::Kaldir {
                verileri_koru: false
            },
            true
        )
        .is_ok());
        assert!(eylem_guvenli_mi(
            Risk::Unsafe,
            PaketEylemi::Kaldir {
                verileri_koru: true
            },
            true
        )
        .is_err());
        assert!(eylem_guvenli_mi(Risk::Unsafe, PaketEylemi::GeriYukle, false).is_ok());
        assert!(toplu_eylem_uygun_mu(Risk::Recommended));
        assert!(!toplu_eylem_uygun_mu(Risk::Expert));
    }

    #[test]
    fn paket_komutlari_dogru_kurulur() {
        let kaldir = paket_komut_argumanlari(
            PaketEylemi::Kaldir {
                verileri_koru: true,
            },
            "com.miui.analytics",
            "ABC123",
        )
        .unwrap();
        assert_eq!(
            kaldir,
            vec![
                "-s",
                "ABC123",
                "shell",
                "pm",
                "uninstall",
                "-k",
                "--user",
                "0",
                "com.miui.analytics"
            ]
        );
        let geri = paket_komut_argumanlari(PaketEylemi::GeriYukle, "com.miui.analytics", "ABC123")
            .unwrap();
        assert!(geri.contains(&"install-existing".to_string()));
        assert!(paket_komut_argumanlari(PaketEylemi::DevreDisi, "kotu paket", "ABC123").is_err());
    }

    #[test]
    fn getprop_ayristirilir() {
        let degerler = getprop_ayristir(
            "[ro.product.brand]: [samsung]\n[ro.product.model]: [SM-S918B]\n[ro.build.version.release]: [14]\n[ro.build.version.sdk]: [34]\n",
        );
        assert_eq!(degerler.get("ro.product.brand").unwrap(), "samsung");
        assert_eq!(degerler.get("ro.build.version.sdk").unwrap(), "34");
    }

    #[test]
    fn cihaz_profili_zenginlestirilir() {
        let mut cihaz = AndroidCihaz {
            seri: "ABC".into(),
            durum: BaglantiDurumu::Bagli,
            model: None,
            marka: None,
            uretici: None,
            android_surumu: None,
            sdk: None,
        };
        let degerler = getprop_ayristir(
            "[ro.product.brand]: [Xiaomi]\n[ro.product.model]: [2211133G]\n[ro.build.version.release]: [14]\n[ro.build.version.sdk]: [34]\n",
        );
        cihaz_profilini_zenginlestir(&mut cihaz, &degerler);
        assert_eq!(cihaz.marka.as_deref(), Some("Xiaomi"));
        assert_eq!(cihaz.android_surumu.as_deref(), Some("14"));
        assert!(cihaz.adb_hazir());
    }

    #[test]
    fn cihaz_listesi_ayristirilir() {
        let cihazlar = cihazlari_ayristir(
            "List of devices attached\nABC\tdevice product:x model:Mi_10\nXYZ\tunauthorized\n",
        );
        assert_eq!(cihazlar.len(), 2);
        assert_eq!(cihazlar[0].model.as_deref(), Some("Mi 10"));
        assert_eq!(cihazlar[1].durum, BaglantiDurumu::Yetkisiz);
    }

    #[test]
    fn paket_durumlari_birlestirilir() {
        let etkin = vec!["com.a".to_string(), "com.b".to_string()];
        let devre = vec!["com.b".to_string()];
        let kaldirilmis = vec!["com.c".to_string()];
        let sistem = vec!["com.a".to_string(), "com.b".to_string()];
        let durumlar = paket_durumlari(&etkin, &devre, &kaldirilmis, &sistem, &[]);
        assert_eq!(durumlar.len(), 3);
        assert!(durumlar.iter().any(|p| p.id == "com.c" && p.kaldirilmis));
        assert!(durumlar.iter().any(|p| p.id == "com.b" && p.devre_disi));
    }

    #[test]
    fn marka_uyarilari_uretilir() {
        assert!(marka_uyarilari(Some("Xiaomi"), Some("14"))
            .iter()
            .any(|uyari| uyari.contains("Xiaomi")));
        assert!(marka_uyarilari(Some("samsung"), Some("14"))
            .iter()
            .any(|uyari| uyari.contains("Otomatik Engelleyici")));
        assert_eq!(marka_uyarilari(Some("Google"), Some("13")).len(), 1);
    }

    #[test]
    fn vid_marka_eslesir() {
        assert_eq!(vid_marka(0x04E8), Some("Samsung"));
        assert_eq!(vid_marka(0x2717), Some("Xiaomi/Poco/Redmi"));
        assert_eq!(vid_marka(0xFFFF), None);
    }

    #[test]
    fn bagimlilik_uyarilari_cihaza_gore_uretilir() {
        let db = uad_ayristir(FIKSTUR).unwrap();
        let bilgi = db.bilgi("com.miui.analytics").unwrap();
        let cihaz = ["com.miui.core".to_string()].into_iter().collect();
        let uyarilar = bagimlilik_uyarilari(bilgi, &cihaz);
        assert!(uyarilar.iter().any(|uyari| uyari.contains("com.miui.core")));
    }
}
