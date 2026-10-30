// WinNexus gelişmiş araç servisi güncelleme, tarayıcı uzantısı ve ağ bağlantısı modelleri.
use crate::error::{Hata, Sonuc};
use serde_json::{Map, Value};
use std::path::Path;
use time::{Date, Month, OffsetDateTime};

const KB_ONEK: &str = "KB";
const KB_ASGARI_RAKAM: usize = 6;
const KB_AZAMI_RAKAM: usize = 8;
const BILINMEYEN_BASLIK: &str = "Bilinmeyen güncelleme";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Guncelleme {
    pub kb: Option<String>,
    pub baslik: String,
    pub tur: String,
    pub tarih: Option<String>,
    pub kaldirilabilir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TarayiciUzantisi {
    pub kimlik: String,
    pub tarayici: String,
    pub profil: String,
    pub ad: String,
    pub surum: String,
    pub yol: String,
    pub etkin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Baglanti {
    pub protokol: String,
    pub yerel_adres: String,
    pub yerel_port: u16,
    pub uzak_adres: String,
    pub uzak_port: u16,
    pub durum: String,
    pub pid: u32,
    pub surec: String,
}

pub fn kb_gecerli_mi(kb: &str) -> bool {
    if kb.len() < KB_ONEK.len() + KB_ASGARI_RAKAM || kb.len() > KB_ONEK.len() + KB_AZAMI_RAKAM {
        return false;
    }
    let (onek, rakamlar) = kb.split_at(KB_ONEK.len());
    onek.eq_ignore_ascii_case(KB_ONEK) && rakamlar.bytes().all(|bayt| bayt.is_ascii_digit())
}

pub fn guncelleme_json_ayristir(json: &str) -> Sonuc<Vec<Guncelleme>> {
    let mut guncellemeler = Vec::new();
    for kayit in json_nesneleri(json, "Güncelleme listesi")? {
        if let Some(guncelleme) = guncelleme_oku(&kayit) {
            guncellemeler.push(guncelleme);
        }
    }
    Ok(guncellemeler)
}

pub fn uzanti_kimlik_olustur(tarayici: &str, profil: &str, uzanti: &str) -> String {
    format!("{}_{}_{}", tarayici.trim(), profil.trim(), uzanti.trim())
}

pub fn manifest_alani(manifest: &str) -> (Option<String>, Option<String>, Option<String>) {
    let metin = manifest.trim_start_matches('\u{feff}').trim();
    let Ok(deger) = serde_json::from_str::<Value>(metin) else {
        return (None, None, None);
    };
    let alan = |ad: &str| -> Option<String> {
        deger
            .get(ad)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|metin| !metin.is_empty())
            .map(str::to_string)
    };
    (alan("name"), alan("version"), alan("default_locale"))
}

pub fn mesaj_adi_coz(mesajlar_json: &str, anahtar: &str) -> Option<String> {
    let metin = mesajlar_json.trim_start_matches('\u{feff}').trim();
    let deger: Value = serde_json::from_str(metin).ok()?;
    let nesne = deger.as_object()?;
    let anahtar = mesaj_anahtari(anahtar);
    let kayit = nesne.get(anahtar).or_else(|| {
        nesne
            .iter()
            .find(|(ad, _)| ad.eq_ignore_ascii_case(anahtar))
            .map(|(_, deger)| deger)
    })?;
    let mesaj = kayit.get("message")?.as_str()?.trim();
    (!mesaj.is_empty()).then(|| mesaj.to_string())
}

pub fn baglanti_json_ayristir(json: &str) -> Sonuc<Vec<Baglanti>> {
    let mut baglantilar = Vec::new();
    for kayit in json_nesneleri(json, "Bağlantı listesi")? {
        if let Some(baglanti) = baglanti_oku(&kayit) {
            baglantilar.push(baglanti);
        }
    }
    Ok(baglantilar)
}

pub fn uc_nokta(adres: &str, port: u16) -> String {
    let adres = adres.trim();
    if adres.is_empty() && port == 0 {
        return "-".to_string();
    }
    let gosterim = if adres.is_empty() {
        "*".to_string()
    } else if adres.contains(':') {
        format!("[{adres}]")
    } else {
        adres.to_string()
    };
    if port == 0 {
        format!("{gosterim}:*")
    } else {
        format!("{gosterim}:{port}")
    }
}

pub fn yol_kok_altinda(kok: &Path, aday: &Path) -> bool {
    let (Ok(kok), Ok(aday)) = (kok.canonicalize(), aday.canonicalize()) else {
        return false;
    };
    aday != kok && aday.starts_with(kok)
}

fn guncelleme_oku(kayit: &Map<String, Value>) -> Option<Guncelleme> {
    let kb = metin_alan(kayit, &["kb", "Kimlik", "HotFixID"])
        .map(|deger| deger.to_ascii_uppercase())
        .filter(|deger| kb_gecerli_mi(deger));
    let baslik = metin_alan(
        kayit,
        &["baslik", "Baslik", "Ad", "Title", "Description", "Aciklama"],
    );
    let tur =
        metin_alan(kayit, &["tur", "Tur", "Type", "Description", "Aciklama"]).unwrap_or_default();
    if kb.is_none() && baslik.is_none() && tur.is_empty() {
        return None;
    }
    let baslik = baslik
        .or_else(|| kb.clone())
        .unwrap_or_else(|| BILINMEYEN_BASLIK.to_string());
    let kaldirilabilir = mantiksal_alan(kayit, &["kaldirilabilir", "Kaldirilabilir", "Removable"])
        .unwrap_or(kb.is_some());
    Some(Guncelleme {
        kb,
        baslik,
        tur,
        tarih: tarih_alan(kayit),
        kaldirilabilir,
    })
}

fn baglanti_oku(kayit: &Map<String, Value>) -> Option<Baglanti> {
    let protokol = metin_alan(kayit, &["protokol", "Protokol", "Protocol"])
        .unwrap_or_default()
        .to_ascii_uppercase();
    let yerel_adres =
        metin_alan(kayit, &["yerel_adres", "YerelAdres", "LocalAddress"]).unwrap_or_default();
    let uzak_adres =
        metin_alan(kayit, &["uzak_adres", "UzakAdres", "RemoteAddress"]).unwrap_or_default();
    let yerel_port = port_alan(kayit, &["yerel_port", "YerelPort", "LocalPort"]);
    let uzak_port = port_alan(kayit, &["uzak_port", "UzakPort", "RemotePort"]);
    let pid = tamsayi_alan(kayit, &["pid", "Pid", "PID", "OwningProcess"])
        .and_then(|deger| u32::try_from(deger).ok())
        .unwrap_or(0);
    let durum = metin_alan(kayit, &["durum", "Durum", "State"]).unwrap_or_default();
    let surec = metin_alan(kayit, &["surec", "Surec", "ProcessName"]).unwrap_or_default();
    let bos = protokol.is_empty()
        && yerel_adres.is_empty()
        && uzak_adres.is_empty()
        && yerel_port == 0
        && uzak_port == 0
        && pid == 0;
    if bos {
        return None;
    }
    Some(Baglanti {
        protokol,
        yerel_adres,
        yerel_port,
        uzak_adres,
        uzak_port,
        durum,
        pid,
        surec,
    })
}

fn json_nesneleri(json: &str, etiket: &str) -> Sonuc<Vec<Map<String, Value>>> {
    let metin = json.trim_start_matches('\u{feff}').trim();
    if metin.is_empty() {
        return Ok(Vec::new());
    }
    let deger: Value = serde_json::from_str(metin)
        .map_err(|hata| Hata::DisArac(format!("{etiket} JSON çıktısı çözümlenemedi: {hata}")))?;
    match deger {
        Value::Null => Ok(Vec::new()),
        Value::Array(liste) => Ok(liste
            .into_iter()
            .filter_map(|oge| match oge {
                Value::Object(nesne) => Some(nesne),
                _ => None,
            })
            .collect()),
        Value::Object(nesne) => Ok(vec![nesne]),
        _ => Err(Hata::DisArac(format!(
            "{etiket} JSON çıktısı beklenen biçimde değil."
        ))),
    }
}

fn alan<'a>(nesne: &'a Map<String, Value>, adlar: &[&str]) -> Option<&'a Value> {
    adlar.iter().find_map(|ad| {
        nesne
            .iter()
            .find(|(anahtar, _)| anahtar.eq_ignore_ascii_case(ad))
            .map(|(_, deger)| deger)
    })
}

fn metin_alan(nesne: &Map<String, Value>, adlar: &[&str]) -> Option<String> {
    match alan(nesne, adlar)? {
        Value::String(metin) => {
            let metin = metin.trim();
            (!metin.is_empty()).then(|| metin.to_string())
        }
        Value::Number(sayi) => Some(sayi.to_string()),
        _ => None,
    }
}

fn mantiksal_alan(nesne: &Map<String, Value>, adlar: &[&str]) -> Option<bool> {
    match alan(nesne, adlar)? {
        Value::Bool(deger) => Some(*deger),
        Value::String(metin) => match metin.trim().to_ascii_lowercase().as_str() {
            "true" | "evet" | "1" => Some(true),
            "false" | "hayir" | "0" => Some(false),
            _ => None,
        },
        Value::Number(sayi) => sayi.as_i64().map(|deger| deger != 0),
        _ => None,
    }
}

fn tamsayi_alan(nesne: &Map<String, Value>, adlar: &[&str]) -> Option<u64> {
    match alan(nesne, adlar)? {
        Value::Number(sayi) => sayi.as_u64().or_else(|| {
            sayi.as_f64()
                .filter(|deger| *deger >= 0.0 && deger.fract() == 0.0)
                .map(|deger| deger as u64)
        }),
        Value::String(metin) => metin.trim().parse().ok(),
        _ => None,
    }
}

fn port_alan(nesne: &Map<String, Value>, adlar: &[&str]) -> u16 {
    tamsayi_alan(nesne, adlar)
        .and_then(|deger| u16::try_from(deger).ok())
        .unwrap_or(0)
}

fn tarih_alan(nesne: &Map<String, Value>) -> Option<String> {
    match alan(nesne, &["tarih", "Tarih", "InstalledOn", "Date"])? {
        Value::String(metin) => tarih_coz(metin),
        Value::Number(sayi) => sayi.as_i64().and_then(unix_ms_tarih),
        _ => None,
    }
}

fn tarih_coz(metin: &str) -> Option<String> {
    let metin = metin.trim();
    if let Some(ms) = tarih_ms(metin) {
        return unix_ms_tarih(ms);
    }
    iso_tarih(metin)
}

fn tarih_ms(metin: &str) -> Option<i64> {
    let icerik = metin.strip_prefix("/Date(")?.strip_suffix(")/")?;
    let sayi: String = icerik
        .chars()
        .take_while(|karakter| karakter.is_ascii_digit() || *karakter == '-')
        .collect();
    if sayi.is_empty() || sayi == "-" {
        return None;
    }
    sayi.parse().ok()
}

fn unix_ms_tarih(ms: i64) -> Option<String> {
    let zaman = OffsetDateTime::from_unix_timestamp(ms.div_euclid(1000)).ok()?;
    Some(format!(
        "{:04}-{:02}-{:02}",
        zaman.year(),
        u8::from(zaman.month()),
        zaman.day()
    ))
}

fn iso_tarih(metin: &str) -> Option<String> {
    let govde = metin.split(['T', 't', ' ']).next()?;
    let parcalar: Vec<&str> = govde.split('-').collect();
    if parcalar.len() != 3 {
        return None;
    }
    let yil: i32 = parcalar[0].parse().ok()?;
    let ay: u8 = parcalar[1].parse().ok()?;
    let gun: u8 = parcalar[2].parse().ok()?;
    Date::from_calendar_date(yil, Month::try_from(ay).ok()?, gun).ok()?;
    Some(format!("{yil:04}-{ay:02}-{gun:02}"))
}

fn mesaj_anahtari(anahtar: &str) -> &str {
    let anahtar = anahtar.trim();
    anahtar
        .strip_prefix("__MSG_")
        .and_then(|kalan| kalan.strip_suffix("__"))
        .unwrap_or(anahtar)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kb_kimligi_yalnizca_onek_ve_rakamlari_kabul_eder() {
        assert!(kb_gecerli_mi("KB5034441"));
        assert!(kb_gecerli_mi("kb503444"));
        assert!(kb_gecerli_mi("Kb12345678"));
        assert!(!kb_gecerli_mi("KB12345"));
        assert!(!kb_gecerli_mi("KB123456789"));
        assert!(!kb_gecerli_mi("KB12345A"));
        assert!(!kb_gecerli_mi("123456"));
        assert!(!kb_gecerli_mi(" KB123456"));
        assert!(!kb_gecerli_mi("KB 123456"));
        assert!(!kb_gecerli_mi("KB-123456"));
        assert!(!kb_gecerli_mi(""));
    }

    #[test]
    fn guncelleme_dizisi_alanlari_ve_tarihleri_cozer() {
        let json = r#"[
            {"kb":"kb5034441","baslik":"Güvenlik Güncellemesi","tur":"Security Update","tarih":"2026-01-15T10:20:30Z","kaldirilabilir":true},
            {"HotFixID":"KB5030509","Description":"Update","InstalledOn":"/Date(1767225600000)/"}
        ]"#;
        let liste = guncelleme_json_ayristir(json).unwrap();
        assert_eq!(liste.len(), 2);
        assert_eq!(liste[0].kb.as_deref(), Some("KB5034441"));
        assert_eq!(liste[0].baslik, "Güvenlik Güncellemesi");
        assert_eq!(liste[0].tur, "Security Update");
        assert_eq!(liste[0].tarih.as_deref(), Some("2026-01-15"));
        assert!(liste[0].kaldirilabilir);
        assert_eq!(liste[1].kb.as_deref(), Some("KB5030509"));
        assert_eq!(liste[1].tur, "Update");
        assert_eq!(liste[1].tarih.as_deref(), Some("2026-01-01"));
        assert!(liste[1].kaldirilabilir);
    }

    #[test]
    fn tekil_guncelleme_ve_eksik_alanlar_cozulur() {
        let json = r#"{"kb":"KB123456","baslik":"","tur":"","tarih":"","kaldirilabilir":false}"#;
        let liste = guncelleme_json_ayristir(json).unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].baslik, "KB123456");
        assert!(liste[0].tarih.is_none());
        assert!(!liste[0].kaldirilabilir);
    }

    #[test]
    fn bos_null_ve_gecersiz_guncelleme_ciktisi_ayrisir() {
        assert!(guncelleme_json_ayristir("").unwrap().is_empty());
        assert!(guncelleme_json_ayristir("null").unwrap().is_empty());
        assert!(guncelleme_json_ayristir("[]").unwrap().is_empty());
        assert!(guncelleme_json_ayristir("{bozuk").is_err());
        assert!(guncelleme_json_ayristir("42").is_err());
    }

    #[test]
    fn tarih_bicimleri_yyyy_aa_gg_uretir() {
        let json = r#"[
            {"kb":"KB123456","tarih":"/Date(1767225600000+0300)/"},
            {"kb":"KB123457","tarih":"2026-02-29"},
            {"kb":"KB123458","tarih":"2026-12-31T23:59:59+03:00"},
            {"kb":"KB123459","tarih":"bozuk"}
        ]"#;
        let liste = guncelleme_json_ayristir(json).unwrap();
        assert_eq!(liste[0].tarih.as_deref(), Some("2026-01-01"));
        assert!(liste[1].tarih.is_none());
        assert_eq!(liste[2].tarih.as_deref(), Some("2026-12-31"));
        assert!(liste[3].tarih.is_none());
    }

    #[test]
    fn manifest_alanlari_okunur_ve_bozuk_json_guvenli() {
        let manifest = r#"{"name":"__MSG_adi__","version":"1.2.3","default_locale":"tr"}"#;
        let (ad, surum, dil) = manifest_alani(manifest);
        assert_eq!(ad.as_deref(), Some("__MSG_adi__"));
        assert_eq!(surum.as_deref(), Some("1.2.3"));
        assert_eq!(dil.as_deref(), Some("tr"));
        assert_eq!(manifest_alani("{bozuk"), (None, None, None));
        assert_eq!(manifest_alani(""), (None, None, None));
        assert_eq!(manifest_alani("null"), (None, None, None));
    }

    #[test]
    fn mesaj_adi_anahtari_cozer() {
        let mesajlar = r#"{
            "extensionName":{"message":"Eklenti Adı"},
            "description":{"message":"Açıklama"},
            "bos":{"message":"  "}
        }"#;
        assert_eq!(
            mesaj_adi_coz(mesajlar, "__MSG_extensionName__").as_deref(),
            Some("Eklenti Adı")
        );
        assert_eq!(
            mesaj_adi_coz(mesajlar, "extensionName").as_deref(),
            Some("Eklenti Adı")
        );
        assert_eq!(
            mesaj_adi_coz(mesajlar, "EXTENSIONNAME").as_deref(),
            Some("Eklenti Adı")
        );
        assert_eq!(mesaj_adi_coz(mesajlar, "__MSG_yok__"), None);
        assert_eq!(mesaj_adi_coz(mesajlar, "bos"), None);
        assert_eq!(mesaj_adi_coz("{bozuk", "extensionName"), None);
    }

    #[test]
    fn uzanti_kimligi_karsiliklidir() {
        assert_eq!(
            uzanti_kimlik_olustur("Chrome", "Default", "abc"),
            "Chrome_Default_abc"
        );
        assert_eq!(
            uzanti_kimlik_olustur(" Chrome ", " Default ", " abc "),
            "Chrome_Default_abc"
        );
    }

    #[test]
    fn baglantilar_sayi_ve_metin_portlariyla_ayristirilir() {
        let json = r#"[
            {"protokol":"tcp","yerel_adres":"127.0.0.1","yerel_port":443,"uzak_adres":"10.0.0.5","uzak_port":"51000","durum":"Established","pid":1234,"surec":"chrome"},
            {"Protocol":"UDP","LocalAddress":"::","LocalPort":"5353","RemoteAddress":"","RemotePort":"0","OwningProcess":"700","ProcessName":"svchost"}
        ]"#;
        let liste = baglanti_json_ayristir(json).unwrap();
        assert_eq!(liste.len(), 2);
        assert_eq!(liste[0].protokol, "TCP");
        assert_eq!(liste[0].yerel_port, 443);
        assert_eq!(liste[0].uzak_port, 51000);
        assert_eq!(liste[0].pid, 1234);
        assert_eq!(liste[0].surec, "chrome");
        assert_eq!(liste[1].protokol, "UDP");
        assert_eq!(liste[1].yerel_adres, "::");
        assert_eq!(liste[1].yerel_port, 5353);
        assert_eq!(liste[1].pid, 700);
        assert_eq!(liste[1].surec, "svchost");
    }

    #[test]
    fn tekil_baglanti_ve_bos_cikti_ayristirilir() {
        let json = r#"{"protokol":"TCP","yerel_adres":"0.0.0.0","yerel_port":80,"uzak_adres":"","uzak_port":0,"durum":"Listen","pid":4,"surec":"System"}"#;
        let liste = baglanti_json_ayristir(json).unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].durum, "Listen");
        assert_eq!(liste[0].uzak_port, 0);
        assert!(baglanti_json_ayristir("").unwrap().is_empty());
        assert!(baglanti_json_ayristir("null").unwrap().is_empty());
        assert!(baglanti_json_ayristir("[]").unwrap().is_empty());
        assert!(baglanti_json_ayristir("{bozuk").is_err());
        assert!(baglanti_json_ayristir("true").is_err());
    }

    #[test]
    fn uc_nokta_ipv6_ve_bos_degerleri_bicimler() {
        assert_eq!(uc_nokta("::1", 443), "[::1]:443");
        assert_eq!(uc_nokta("fe80::1%12", 80), "[fe80::1%12]:80");
        assert_eq!(uc_nokta("127.0.0.1", 80), "127.0.0.1:80");
        assert_eq!(uc_nokta("", 0), "-");
        assert_eq!(uc_nokta("", 443), "*:443");
        assert_eq!(uc_nokta("0.0.0.0", 0), "0.0.0.0:*");
        assert_eq!(uc_nokta("::", 0), "[::]:*");
    }

    #[test]
    fn yol_kok_altinda_guvenli_alt_yol_denetimi() {
        let kok = tempfile::tempdir().unwrap();
        let alt = kok.path().join("profil").join("uzanti");
        std::fs::create_dir_all(&alt).unwrap();
        let kardes = tempfile::tempdir().unwrap();
        assert!(yol_kok_altinda(kok.path(), &alt));
        assert!(!yol_kok_altinda(kok.path(), kok.path()));
        assert!(!yol_kok_altinda(kok.path(), kardes.path()));
        assert!(!yol_kok_altinda(kok.path(), &kok.path().join("yok")));
        assert!(!yol_kok_altinda(&kok.path().join("yok"), &alt));
    }
}
