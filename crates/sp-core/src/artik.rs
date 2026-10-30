use crate::error::{Hata, Sonuc};

const MIN_AD_UZUNLUGU: usize = 3;
const AZAMI_KAYIT_ADI: usize = 128;
const AZAMI_PAKET_ADI: usize = 256;
const YAPI_PARCALARI: [&str; 10] = [
    "x64", "x86", "x32", "win32", "win64", "arm64", "32-bit", "64-bit", "32bit", "64bit",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtikTuru {
    Registry,
    Dosya,
    Klasor,
}

impl ArtikTuru {
    pub fn metin(self) -> &'static str {
        match self {
            Self::Registry => "Kayıt defteri",
            Self::Dosya => "Dosya",
            Self::Klasor => "Klasör",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtikOgesi {
    pub kimlik: String,
    pub tur: ArtikTuru,
    pub ad: String,
    pub yol: String,
    pub boyut_bayt: Option<u64>,
}

impl ArtikOgesi {
    pub fn kimlik_olustur(tur: ArtikTuru, yol: &str) -> String {
        format!("{}|{}", tur.metin(), yol_normalize(yol))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorePaketi {
    pub ad: String,
    pub tam_ad: String,
    pub surum: String,
    pub yayinci: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KarantinaSonucu {
    pub tasinan: u64,
    pub hatalar: Vec<String>,
}

pub fn normalize_adi(metin: &str) -> Option<String> {
    let parantezsiz = parantezleri_kaldir(&metin.to_lowercase());
    let temiz = parantezsiz
        .split_whitespace()
        .filter(|parca| !gurultu_parcasi_mi(parca))
        .collect::<Vec<_>>()
        .join(" ");
    let temiz = temiz
        .trim_matches(|karakter: char| {
            karakter.is_whitespace() || matches!(karakter, '-' | '_' | '.' | ',' | ';' | ':')
        })
        .to_string();
    (temiz.chars().count() >= MIN_AD_UZUNLUGU).then_some(temiz)
}

pub fn ad_eslesiyor_mu(aday: &str, hedefler: &[String]) -> bool {
    match normalize_adi(aday) {
        Some(normal) => hedefler.iter().any(|hedef| hedef == &normal),
        None => false,
    }
}

pub fn kayit_adi_uygun_mu(ad: &str) -> bool {
    let ad = ad.trim();
    !ad.is_empty()
        && ad.len() <= AZAMI_KAYIT_ADI
        && !ad
            .chars()
            .any(|karakter| matches!(karakter, '\\' | '/' | '\0' | '\r' | '\n'))
}

pub fn paket_tam_adi_gecerli_mi(tam_ad: &str) -> bool {
    !tam_ad.is_empty()
        && tam_ad.len() <= AZAMI_PAKET_ADI
        && tam_ad.chars().all(|karakter| {
            karakter.is_ascii_alphanumeric() || matches!(karakter, '_' | '.' | '+' | '-')
        })
}

pub fn store_paketlerini_ayristir(json: &str) -> Sonuc<Vec<StorePaketi>> {
    let metin = json.trim().trim_start_matches('\u{feff}').trim();
    if metin.is_empty() {
        return Ok(Vec::new());
    }
    let deger: serde_json::Value = serde_json::from_str(metin).map_err(|hata| {
        Hata::Beklenmeyen(format!("Store paket listesi ayrıştırılamadı: {hata}"))
    })?;
    let satirlar = match &deger {
        serde_json::Value::Array(satirlar) => satirlar.iter().collect::<Vec<_>>(),
        serde_json::Value::Object(_) => vec![&deger],
        serde_json::Value::Null => Vec::new(),
        _ => {
            return Err(Hata::Beklenmeyen(
                "Store paket listesi beklenen biçimde değil.".into(),
            ))
        }
    };
    Ok(satirlar
        .iter()
        .filter_map(|satir| paket_oku(satir))
        .collect())
}

fn paket_oku(satir: &serde_json::Value) -> Option<StorePaketi> {
    let alan = |ad: &str| {
        satir
            .get(ad)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|metin| !metin.is_empty())
            .map(str::to_string)
    };
    Some(StorePaketi {
        ad: alan("Ad")?,
        tam_ad: alan("TamAd")?,
        surum: alan("Surum").unwrap_or_default(),
        yayinci: alan("Yayinci").unwrap_or_default(),
    })
}

fn yol_normalize(yol: &str) -> String {
    yol.trim()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

fn parantezleri_kaldir(metin: &str) -> String {
    let mut sonuc = String::with_capacity(metin.len());
    let mut derinlik = 0usize;
    for karakter in metin.chars() {
        match karakter {
            '(' | '[' | '{' => derinlik += 1,
            ')' | ']' | '}' => derinlik = derinlik.saturating_sub(1),
            _ if derinlik == 0 => sonuc.push(karakter),
            _ => {}
        }
    }
    sonuc
}

fn gurultu_parcasi_mi(parca: &str) -> bool {
    let parca = parca.trim_matches(|karakter: char| {
        matches!(karakter, '-' | '_' | '.' | ',' | ';' | ':' | '(' | ')')
    });
    if parca.is_empty() || YAPI_PARCALARI.contains(&parca) {
        return true;
    }
    let govde = parca.strip_prefix('v').unwrap_or(parca);
    !govde.is_empty()
        && govde.chars().any(|karakter| karakter.is_ascii_digit())
        && govde
            .chars()
            .all(|karakter| karakter.is_ascii_digit() || karakter == '.')
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn tek_nesne_ayristirilir() {
        let json = r#"{"Ad":"Microsoft.WindowsStore","TamAd":"Microsoft.WindowsStore_22410.1401.2.0_x64__8wekyb3d8bbwe","Surum":"22410.1401.2.0","Yayinci":"CN=Microsoft Corporation"}"#;
        let paketler = store_paketlerini_ayristir(json).unwrap();
        assert_eq!(paketler.len(), 1);
        assert_eq!(paketler[0].ad, "Microsoft.WindowsStore");
        assert_eq!(paketler[0].surum, "22410.1401.2.0");
        assert_eq!(paketler[0].yayinci, "CN=Microsoft Corporation");
    }

    #[test]
    fn dizi_ayristirilir() {
        let json = r#"[
            {"Ad":"A.B","TamAd":"A.B_1.0.0.0_x64__abc","Surum":"1.0.0.0","Yayinci":"A"},
            {"Ad":"C.D","TamAd":"C.D_2.0.0.0_x86__def","Surum":"2.0.0.0","Yayinci":"C"}
        ]"#;
        let paketler = store_paketlerini_ayristir(json).unwrap();
        assert_eq!(paketler.len(), 2);
        assert_eq!(paketler[1].tam_ad, "C.D_2.0.0.0_x86__def");
    }

    #[test]
    fn bos_ve_bozuk_girdi() {
        assert!(store_paketlerini_ayristir("").unwrap().is_empty());
        assert!(store_paketlerini_ayristir("   ").unwrap().is_empty());
        assert!(store_paketlerini_ayristir("null").unwrap().is_empty());
        assert!(store_paketlerini_ayristir("bozuk").is_err());
        assert!(store_paketlerini_ayristir("42").is_err());
    }

    #[test]
    fn eksik_alanli_satir_atlanir() {
        let json = r#"[{"Ad":"Eksik"},{"Ad":null,"TamAd":"X"},{"TamAd":"Y"}]"#;
        assert!(store_paketlerini_ayristir(json).unwrap().is_empty());
    }

    #[test]
    fn ad_normalize_edilir() {
        assert_eq!(
            normalize_adi("Google Chrome (64-bit)").as_deref(),
            Some("google chrome")
        );
        assert_eq!(normalize_adi("7-Zip 23.01 (x64)").as_deref(), Some("7-zip"));
        assert_eq!(
            normalize_adi("  Notepad++   v8.6 ").as_deref(),
            Some("notepad++")
        );
        assert_eq!(normalize_adi("A").as_deref(), None);
        assert_eq!(normalize_adi("(x64)").as_deref(), None);
        assert_eq!(normalize_adi("   ").as_deref(), None);
    }

    #[test]
    fn yalnizca_tam_eslesme_kabul_edilir() {
        let hedefler = vec!["adobe".to_string(), "google llc".to_string()];
        assert!(ad_eslesiyor_mu("Adobe", &hedefler));
        assert!(ad_eslesiyor_mu("ADOBE (x64)", &hedefler));
        assert!(!ad_eslesiyor_mu("Adobe Inc.", &hedefler));
        assert!(!ad_eslesiyor_mu("Adobe Acrobat", &hedefler));
        assert!(ad_eslesiyor_mu("Google LLC", &hedefler));
    }

    #[test]
    fn paket_adi_dogrulanir() {
        assert!(paket_tam_adi_gecerli_mi(
            "Microsoft.WindowsStore_22410.1401.2.0_x64__8wekyb3d8bbwe"
        ));
        assert!(!paket_tam_adi_gecerli_mi(""));
        assert!(!paket_tam_adi_gecerli_mi("paket'; Remove-Item C:\\"));
        assert!(!paket_tam_adi_gecerli_mi("paket\nad"));
        assert!(!paket_tam_adi_gecerli_mi("paket ad"));
    }

    #[test]
    fn kayit_adi_yol_karakteri_icermez() {
        assert!(kayit_adi_uygun_mu("Google"));
        assert!(!kayit_adi_uygun_mu("Google\\Chrome"));
        assert!(!kayit_adi_uygun_mu(""));
    }

    #[test]
    fn kimlik_tur_ve_normal_yol_icerir() {
        let kimlik = ArtikOgesi::kimlik_olustur(ArtikTuru::Klasor, "C:\\Users\\Test\\AppData\\");
        assert_eq!(kimlik, "Klasör|c:\\users\\test\\appdata");
    }
}
