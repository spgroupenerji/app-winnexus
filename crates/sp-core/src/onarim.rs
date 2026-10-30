// Onarım ve gelişmiş temizlik modelleri.
use crate::error::{Hata, Sonuc};
use serde_json::Value;

pub const BILINMIYOR_TARIHI: &str = "Bilinmiyor";
const AZAMI_KIMLIK_UZUNLUGU: usize = 64;
const AZAMI_GERI_YUKLEME_ACIKLAMASI: usize = 60;
const VARSAYILAN_GERI_YUKLEME_ACIKLAMASI: &str = "WinNexus geri yükleme noktası";

pub const SORUN_GIDERICI_KIMLIKLERI: &[(&str, &str)] = &[
    ("WindowsUpdateDiagnostic", "Windows Update"),
    ("NetworkDiagnostics", "Ağ bağdaştırıcısı"),
    ("InternetConnectionsDiagnostic", "İnternet bağlantıları"),
    ("ProgramCompatibilityDiagnostic", "Program uyumluluğu"),
    ("AudioPlaybackDiagnostic", "Ses kayıttan yürütme"),
    ("VideoPlaybackDiagnostic", "Video kayıttan yürütme"),
    ("PrinterDiagnostic", "Yazıcı"),
    ("PowerDiagnostic", "Güç"),
    ("SystemMaintenanceDiagnostic", "Sistem bakımı"),
    ("BlueScreenDiagnostic", "Mavi ekran"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemizlikKategorisi {
    pub kimlik: String,
    pub ad: String,
    pub aciklama: String,
    pub yonetici: bool,
    pub riskli: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KategoriTarama {
    pub kategori: TemizlikKategorisi,
    pub dosya_sayisi: u64,
    pub bayt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OnarimTemizlikSonucu {
    pub silinen_dosya: u64,
    pub silinen_bayt: u64,
    pub hatalar: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeriYuklemeNoktasi {
    pub sira: u32,
    pub aciklama: String,
    pub tarih: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuncellemeKaydi {
    pub kimlik: String,
    pub aciklama: String,
    pub tarih: String,
}

pub fn kategoriler() -> Vec<TemizlikKategorisi> {
    vec![
        kategori(
            "sembol_thumb_onbellegi",
            "Simge ve küçük resim önbelleği",
            "Bozuk simge ve küçük resim önbelleğini temizler; görsel hataları giderir.",
            false,
            false,
        ),
        kategori(
            "son_kullanilan",
            "Son kullanılan öğeler",
            "Dosya Gezgini'ndeki son kullanılan öğe kısayollarını siler.",
            false,
            false,
        ),
        kategori(
            "wer_raporlari",
            "Windows hata raporları",
            "Windows Hata Bildirimi kuyruğunu ve arşivini temizler.",
            true,
            false,
        ),
        kategori(
            "windows_loglari",
            "Windows günlük dosyaları",
            "CBS, DISM ve Windows Update günlük dosyalarını siler.",
            true,
            false,
        ),
        kategori(
            "windows_update_indirmeleri",
            "Windows Update indirmeleri",
            "İndirilmiş Windows Update dosyalarını siler; güncelleme hizmetleri yeniden başlatılır.",
            true,
            false,
        ),
        kategori(
            "teslimat_optimizasyonu",
            "Teslimat İyileştirme önbelleği",
            "Teslimat İyileştirme önbelleğini boşaltır.",
            true,
            false,
        ),
        kategori(
            "defender_tarama_gecmisi",
            "Defender tarama geçmişi",
            "Microsoft Defender tarama geçmişi kayıtlarını siler.",
            true,
            false,
        ),
        kategori(
            "gelistirici_onbellekleri",
            "Geliştirici paket önbellekleri",
            "NuGet, pip ve npm önbelleklerini temizler.",
            false,
            false,
        ),
        kategori(
            "visual_studio_kaynagi",
            "Visual Studio kurulum kaynağı",
            "Visual Studio kurulum ve paket önbelleğini siler; onarım için yeniden indirme gerekebilir.",
            true,
            true,
        ),
        kategori(
            "chrome_kurulum_onbellegi",
            "Chrome kurulum önbelleği",
            "Chrome kurulum önbelleğindeki artık dosyaları siler.",
            false,
            false,
        ),
        kategori(
            "inetcache",
            "Windows internet önbelleği",
            "Windows internet önbelleğini temizler.",
            false,
            false,
        ),
    ]
}

pub fn guncelleme_gecmisi_ayristir(json: &str) -> Sonuc<Vec<GuncellemeKaydi>> {
    let kayitlar = json_ogeler(json, "Güncelleme geçmişi")?;
    Ok(kayitlar
        .iter()
        .map(|kayit| GuncellemeKaydi {
            kimlik: alan(kayit, &["Kimlik", "HotFixID"]),
            aciklama: alan(kayit, &["Aciklama", "Description"]),
            tarih: tarih_metni(kayit.get("Tarih").or_else(|| kayit.get("InstalledOn"))),
        })
        .collect())
}

pub fn geri_yukleme_noktalari_ayristir(json: &str) -> Sonuc<Vec<GeriYuklemeNoktasi>> {
    let kayitlar = json_ogeler(json, "Geri yükleme noktaları")?;
    Ok(kayitlar
        .iter()
        .map(|kayit| GeriYuklemeNoktasi {
            sira: sayi(kayit.get("Sira").or_else(|| kayit.get("SequenceNumber"))),
            aciklama: alan(kayit, &["Aciklama", "Description"]),
            tarih: tarih_metni(kayit.get("Tarih").or_else(|| kayit.get("CreationTime"))),
        })
        .collect())
}

pub fn sorun_giderici_kimligi_gecerli_mi(kimlik: &str) -> bool {
    !kimlik.is_empty()
        && kimlik.len() <= AZAMI_KIMLIK_UZUNLUGU
        && kimlik
            .chars()
            .all(|karakter| karakter.is_ascii_alphabetic())
        && SORUN_GIDERICI_KIMLIKLERI
            .iter()
            .any(|kayit| kayit.0 == kimlik)
}

pub fn geri_yukleme_surucu_gecerli_mi(surucu: &str) -> bool {
    let baytlar = surucu.as_bytes();
    baytlar.len() == 3
        && baytlar[0].is_ascii_alphabetic()
        && baytlar[1] == b':'
        && matches!(baytlar[2], b'\\' | b'/')
}

pub fn geri_yukleme_aciklamasi_temizle(aciklama: &str) -> String {
    let temiz: String = aciklama
        .chars()
        .filter(|karakter| !matches!(karakter, '\'' | '"'))
        .map(|karakter| if karakter.is_control() { ' ' } else { karakter })
        .collect();
    let metin = temiz.split_whitespace().collect::<Vec<_>>().join(" ");
    let kisaltilmis: String = metin.chars().take(AZAMI_GERI_YUKLEME_ACIKLAMASI).collect();
    let kisaltilmis = kisaltilmis.trim_end();
    if kisaltilmis.is_empty() {
        VARSAYILAN_GERI_YUKLEME_ACIKLAMASI.to_string()
    } else {
        kisaltilmis.to_string()
    }
}

pub fn geri_yukleme_sirasi_gecerli_mi(sira: u32) -> bool {
    sira > 0
}

fn kategori(
    kimlik: &str,
    ad: &str,
    aciklama: &str,
    yonetici: bool,
    riskli: bool,
) -> TemizlikKategorisi {
    TemizlikKategorisi {
        kimlik: kimlik.to_string(),
        ad: ad.to_string(),
        aciklama: aciklama.to_string(),
        yonetici,
        riskli,
    }
}

fn json_ogeler(json: &str, etiket: &str) -> Sonuc<Vec<Value>> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    let deger: Value = serde_json::from_str(json)
        .map_err(|hata| Hata::Beklenmeyen(format!("{etiket} JSON çıktısı okunamadı: {hata}")))?;
    match deger {
        Value::Null => Ok(Vec::new()),
        Value::Array(liste) => Ok(liste),
        Value::Object(_) => Ok(vec![deger]),
        _ => Err(Hata::Beklenmeyen(format!(
            "{etiket} JSON çıktısı beklenen biçimde değil."
        ))),
    }
}

fn alan(kayit: &Value, adlar: &[&str]) -> String {
    adlar
        .iter()
        .find_map(|ad| kayit.get(*ad))
        .map(deger_metni)
        .unwrap_or_default()
}

fn deger_metni(deger: &Value) -> String {
    match deger {
        Value::String(metin) => metin.trim().to_string(),
        Value::Number(sayi) => sayi.to_string(),
        Value::Bool(dogru) => dogru.to_string(),
        _ => String::new(),
    }
}

fn sayi(deger: Option<&Value>) -> u32 {
    match deger {
        Some(Value::Number(deger)) => deger
            .as_u64()
            .map(|deger| u32::try_from(deger).unwrap_or(u32::MAX))
            .unwrap_or_default(),
        Some(Value::String(metin)) => metin.trim().parse().unwrap_or_default(),
        _ => u32::default(),
    }
}

fn tarih_metni(deger: Option<&Value>) -> String {
    match deger {
        Some(Value::String(metin)) => metin_tarihi(metin),
        Some(Value::Number(sayi)) => sayi
            .as_i64()
            .and_then(unix_ms_metni)
            .unwrap_or_else(|| BILINMIYOR_TARIHI.to_string()),
        _ => BILINMIYOR_TARIHI.to_string(),
    }
}

fn metin_tarihi(metin: &str) -> String {
    let metin = metin.trim();
    if metin.is_empty() || metin.eq_ignore_ascii_case("null") {
        return BILINMIYOR_TARIHI.to_string();
    }
    if let Some(sonuc) = tarih_ms(metin).and_then(unix_ms_metni) {
        return sonuc;
    }
    if let Some(sonuc) = iso_tarih(metin) {
        return sonuc;
    }
    metin.to_string()
}

fn tarih_ms(metin: &str) -> Option<i64> {
    let icerik = metin.strip_prefix("/Date(")?.strip_suffix(")/")?;
    let sayi: String = icerik
        .chars()
        .take_while(|karakter| karakter.is_ascii_digit() || *karakter == '-')
        .collect();
    sayi.parse().ok()
}

fn unix_ms_metni(ms: i64) -> Option<String> {
    let zaman = time::OffsetDateTime::from_unix_timestamp(ms.div_euclid(1000)).ok()?;
    Some(format!(
        "{:02}.{:02}.{:04}",
        zaman.day(),
        u8::from(zaman.month()),
        zaman.year()
    ))
}

fn iso_tarih(metin: &str) -> Option<String> {
    let govde = metin.split(['T', ' ']).next()?;
    let parcalar: Vec<&str> = govde.split('-').collect();
    if parcalar.len() != 3 {
        return None;
    }
    let yil: i32 = parcalar[0].parse().ok()?;
    let ay: u8 = parcalar[1].parse().ok()?;
    let gun: u8 = parcalar[2].parse().ok()?;
    time::Date::from_calendar_date(yil, time::Month::try_from(ay).ok()?, gun).ok()?;
    Some(format!("{gun:02}.{ay:02}.{yil:04}"))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn guncelleme_gecmisi_alanlari_ve_tarihleri_ayristirir() {
        let json = r#"[{"Kimlik":"KB5034441","Aciklama":"Security Update","Tarih":"2026-01-15"},{"HotFixID":"KB5030509","Description":"Update","InstalledOn":"/Date(1767225600000)/"}]"#;
        let kayitlar = guncelleme_gecmisi_ayristir(json).unwrap();
        assert_eq!(kayitlar.len(), 2);
        assert_eq!(
            kayitlar[0],
            GuncellemeKaydi {
                kimlik: "KB5034441".into(),
                aciklama: "Security Update".into(),
                tarih: "15.01.2026".into(),
            }
        );
        assert_eq!(kayitlar[1].kimlik, "KB5030509");
        assert_eq!(kayitlar[1].tarih, "01.01.2026");
    }

    #[test]
    fn eksik_guncelleme_tarihi_turkce_metne_duser() {
        let json = r#"{"Kimlik":"KB1","Aciklama":"","Tarih":null}"#;
        let kayitlar = guncelleme_gecmisi_ayristir(json).unwrap();
        assert_eq!(kayitlar.len(), 1);
        assert_eq!(kayitlar[0].tarih, BILINMIYOR_TARIHI);
    }

    #[test]
    fn geri_yukleme_noktalari_sira_ve_tarihle_ayristirilir() {
        let json = r#"[{"Sira":12,"Aciklama":"Bakım","Tarih":"2026-02-03 14:30"},{"SequenceNumber":"7","Description":"Kurulum","CreationTime":"/Date(1767225600000)/"}]"#;
        let noktalar = geri_yukleme_noktalari_ayristir(json).unwrap();
        assert_eq!(
            noktalar,
            vec![
                GeriYuklemeNoktasi {
                    sira: 12,
                    aciklama: "Bakım".into(),
                    tarih: "03.02.2026".into(),
                },
                GeriYuklemeNoktasi {
                    sira: 7,
                    aciklama: "Kurulum".into(),
                    tarih: "01.01.2026".into(),
                },
            ]
        );
    }

    #[test]
    fn bos_ve_gecersiz_json_islenir() {
        assert!(guncelleme_gecmisi_ayristir("").unwrap().is_empty());
        assert!(guncelleme_gecmisi_ayristir("null").unwrap().is_empty());
        assert!(geri_yukleme_noktalari_ayristir("[ ]").unwrap().is_empty());
        assert!(guncelleme_gecmisi_ayristir("{bozuk").is_err());
        assert!(geri_yukleme_noktalari_ayristir("42").is_err());
    }

    #[test]
    fn sorun_giderici_kimligi_denetlenir() {
        assert!(sorun_giderici_kimligi_gecerli_mi("WindowsUpdateDiagnostic"));
        assert!(sorun_giderici_kimligi_gecerli_mi("BlueScreenDiagnostic"));
        assert!(!sorun_giderici_kimligi_gecerli_mi(""));
        assert!(!sorun_giderici_kimligi_gecerli_mi(
            "windowsupdatediagnostic"
        ));
        assert!(!sorun_giderici_kimligi_gecerli_mi("Windows1Diagnostic"));
        assert!(!sorun_giderici_kimligi_gecerli_mi(
            "WindowsUpdateDiagnostic; calc"
        ));
        assert!(!sorun_giderici_kimligi_gecerli_mi(&"A".repeat(65)));
        assert!(!sorun_giderici_kimligi_gecerli_mi("UnknownDiagnostic"));
    }

    #[test]
    fn onbir_kategori_tanimlidir() {
        let kategoriler = kategoriler();
        assert_eq!(kategoriler.len(), 11);
        assert!(kategoriler
            .iter()
            .any(|kategori| kategori.kimlik == "sembol_thumb_onbellegi"
                && !kategori.yonetici
                && !kategori.riskli));
        assert!(kategoriler
            .iter()
            .any(|kategori| kategori.kimlik == "visual_studio_kaynagi"
                && kategori.yonetici
                && kategori.riskli));
        assert_eq!(
            kategoriler
                .iter()
                .filter(|kategori| kategori.yonetici)
                .count(),
            6
        );
    }

    #[test]
    fn geri_yukleme_surucusu_bicim_denetlenir() {
        assert!(geri_yukleme_surucu_gecerli_mi("C:\\"));
        assert!(geri_yukleme_surucu_gecerli_mi("d:/"));
        assert!(!geri_yukleme_surucu_gecerli_mi(""));
        assert!(!geri_yukleme_surucu_gecerli_mi("C:"));
        assert!(!geri_yukleme_surucu_gecerli_mi("C:\\Windows"));
        assert!(!geri_yukleme_surucu_gecerli_mi("1:\\"));
        assert!(!geri_yukleme_surucu_gecerli_mi("CC:\\"));
        assert!(!geri_yukleme_surucu_gecerli_mi("C|\\"));
        assert!(!geri_yukleme_surucu_gecerli_mi("Ç:\\"));
        assert!(!geri_yukleme_surucu_gecerli_mi("C:\\\n"));
    }

    #[test]
    fn geri_yukleme_aciklamasi_tirnak_ve_kontrol_karakterlerini_atar() {
        let temiz = geri_yukleme_aciklamasi_temizle(" Ayşe'nin \"bakım\"\r\nplanı\0 ");
        assert_eq!(temiz, "Ayşenin bakım planı");
        assert!(!temiz.contains('\0'));
        assert!(!temiz.contains('\''));
        assert!(!temiz.contains('"'));
    }

    #[test]
    fn geri_yukleme_aciklamasi_bosluklari_sadelestirir() {
        assert_eq!(
            geri_yukleme_aciklamasi_temizle("  bakım   ve\t\ntemizlik  "),
            "bakım ve temizlik"
        );
    }

    #[test]
    fn geri_yukleme_aciklamasi_altmis_karakterle_sinirlanir() {
        let uzun = "ş".repeat(80);
        assert_eq!(
            geri_yukleme_aciklamasi_temizle(&uzun),
            "ş".repeat(AZAMI_GERI_YUKLEME_ACIKLAMASI)
        );
        let sinirda_bosluk = format!("{} b", "a".repeat(59));
        let temiz = geri_yukleme_aciklamasi_temizle(&sinirda_bosluk);
        assert_eq!(temiz, "a".repeat(59));
    }

    #[test]
    fn geri_yukleme_aciklamasi_bossa_varsayilan_doner() {
        for girdi in ["", "   ", "\r\n\t", "\0", "'\"'"] {
            assert_eq!(
                geri_yukleme_aciklamasi_temizle(girdi),
                VARSAYILAN_GERI_YUKLEME_ACIKLAMASI
            );
        }
    }

    #[test]
    fn geri_yukleme_sirasi_sifiri_reddeder() {
        assert!(!geri_yukleme_sirasi_gecerli_mi(0));
        assert!(geri_yukleme_sirasi_gecerli_mi(1));
        assert!(geri_yukleme_sirasi_gecerli_mi(u32::MAX));
    }
}
