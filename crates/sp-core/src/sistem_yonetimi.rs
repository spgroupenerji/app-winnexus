use crate::error::{Hata, Sonuc};
use serde::{Deserialize, Serialize};

// sp-win, kendi Cargo.toml'unda serde_json bağımlılığı olmadan bu yardımcıları kullanabilsin.
pub use serde_json;

const KIMLIK_AYIRICI: char = '|';
const YEDEK_DOSYA_ONEKI: &str = "oge-";
const YEDEK_ADI_TEMEL_SINIRI: usize = 96;
const FNV_BASLANGIC: u32 = 0x811c_9dc5;
const FNV_CARPAN: u32 = 0x0100_0193;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BaslangicKonumu {
    HkcuRun,
    HkcuRunOnce,
    HklmRun,
    HklmRunOnce,
    HklmRun32,
    HkcuBaslangic,
    OrtakBaslangic,
}

impl BaslangicKonumu {
    pub fn metin(&self) -> &'static str {
        match self {
            Self::HkcuRun => "HKCU Run (kullanıcı)",
            Self::HkcuRunOnce => "HKCU RunOnce (kullanıcı)",
            Self::HklmRun => "HKLM Run (makine)",
            Self::HklmRunOnce => "HKLM RunOnce (makine)",
            Self::HklmRun32 => "HKLM Run (32-bit)",
            Self::HkcuBaslangic => "Kullanıcı Başlangıç klasörü",
            Self::OrtakBaslangic => "Ortak Başlangıç klasörü",
        }
    }

    pub fn kimlik(&self) -> &'static str {
        match self {
            Self::HkcuRun => "hkcu-run",
            Self::HkcuRunOnce => "hkcu-runonce",
            Self::HklmRun => "hklm-run",
            Self::HklmRunOnce => "hklm-runonce",
            Self::HklmRun32 => "hklm-run32",
            Self::HkcuBaslangic => "hkcu-baslangic",
            Self::OrtakBaslangic => "ortak-baslangic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaslangicOgesi {
    pub kimlik: String,
    pub ad: String,
    pub komut: String,
    pub konum: BaslangicKonumu,
    pub etkin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServisOgesi {
    pub ad: String,
    pub gorunen_ad: String,
    pub durum: String,
    pub baslangic: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GorevOgesi {
    pub ad: String,
    pub yol: String,
    pub durum: String,
    pub sonraki: String,
    pub kimlik: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurecOgesi {
    pub ad: String,
    pub pid: u32,
    pub bellek_mb: u64,
    pub cpu_saniye: Option<f64>,
    pub kullanici: Option<String>,
    pub baslama: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaslangicYedegi {
    pub kimlik: String,
    pub ad: String,
    pub komut: String,
    pub konum: BaslangicKonumu,
}

pub fn kimlik_olustur(konum: &BaslangicKonumu, ad: &str) -> String {
    format!(
        "{}{}{}",
        konum.kimlik(),
        KIMLIK_AYIRICI,
        ad.trim().to_ascii_lowercase()
    )
}

pub fn yedek_dosya_adi(kimlik: &str) -> String {
    let temel: String = kimlik
        .chars()
        .map(|karakter| {
            if karakter.is_ascii_alphanumeric() || matches!(karakter, '-' | '_' | '.') {
                karakter.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .take(YEDEK_ADI_TEMEL_SINIRI)
        .collect();
    let ozet = kimlik.bytes().fold(FNV_BASLANGIC, |ozet, bayt| {
        (ozet ^ u32::from(bayt)).wrapping_mul(FNV_CARPAN)
    });
    format!("{}{temel}-{ozet:08x}", YEDEK_DOSYA_ONEKI)
}

pub fn baslangic_yedekleri_ayristir(json: &str) -> Sonuc<Vec<BaslangicYedegi>> {
    let metin = json.trim();
    if metin.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(metin)
        .map_err(|hata| Hata::Beklenmeyen(format!("Başlangıç yedeği çözümlenemedi: {hata}")))
}

pub fn baslangic_yedekleri_metne(yedekler: &[BaslangicYedegi]) -> Sonuc<String> {
    serde_json::to_string_pretty(yedekler)
        .map_err(|hata| Hata::Beklenmeyen(format!("Başlangıç yedeği yazılamadı: {hata}")))
}

pub fn yedek_ekle(yedekler: &mut Vec<BaslangicYedegi>, yedek: BaslangicYedegi) {
    yedekler.retain(|mevcut| mevcut.kimlik != yedek.kimlik);
    yedekler.push(yedek);
    yedekler.sort_by(|a, b| a.kimlik.cmp(&b.kimlik));
}

pub fn yedek_cikar(yedekler: &mut Vec<BaslangicYedegi>, kimlik: &str) -> Option<BaslangicYedegi> {
    let sira = yedekler.iter().position(|yedek| yedek.kimlik == kimlik)?;
    Some(yedekler.remove(sira))
}

#[cfg(test)]
mod testler {
    use super::*;

    const TUM_KONUMLAR: [BaslangicKonumu; 7] = [
        BaslangicKonumu::HkcuRun,
        BaslangicKonumu::HkcuRunOnce,
        BaslangicKonumu::HklmRun,
        BaslangicKonumu::HklmRunOnce,
        BaslangicKonumu::HklmRun32,
        BaslangicKonumu::HkcuBaslangic,
        BaslangicKonumu::OrtakBaslangic,
    ];

    #[test]
    fn konum_metinleri_ve_kimlikleri_dolu() {
        for konum in &TUM_KONUMLAR {
            assert!(!konum.metin().trim().is_empty());
            assert!(!konum.kimlik().trim().is_empty());
            assert!(konum
                .kimlik()
                .chars()
                .all(|karakter| karakter.is_ascii_lowercase()
                    || karakter.is_ascii_digit()
                    || karakter == '-'));
        }
    }

    #[test]
    fn kimlik_konum_ve_adi_birlestirir() {
        let kimlik = kimlik_olustur(&BaslangicKonumu::HklmRun32, "  TestAraç  ");
        assert_eq!(kimlik, "hklm-run32|testaraç");
        assert_ne!(
            kimlik_olustur(&BaslangicKonumu::HklmRun, "TestAraç"),
            kimlik_olustur(&BaslangicKonumu::HklmRunOnce, "TestAraç")
        );
    }

    #[test]
    fn yedek_json_gidis_donus_yapar() {
        let yedek = BaslangicYedegi {
            kimlik: "hkcu-run|test".into(),
            ad: "Test".into(),
            komut: r"C:\Test\test.exe".into(),
            konum: BaslangicKonumu::HkcuRun,
        };
        let metin = baslangic_yedekleri_metne(std::slice::from_ref(&yedek)).unwrap();
        assert_eq!(baslangic_yedekleri_ayristir(&metin).unwrap(), vec![yedek]);
        assert!(baslangic_yedekleri_ayristir("  ").unwrap().is_empty());
        assert!(baslangic_yedekleri_ayristir("{bozuk").is_err());
    }

    #[test]
    fn yedek_ekleme_ve_cikarma_tek_kayit_tutar() {
        let mut yedekler = Vec::new();
        let eski = BaslangicYedegi {
            kimlik: "hkcu-run|a".into(),
            ad: "A".into(),
            komut: "a.exe".into(),
            konum: BaslangicKonumu::HkcuRun,
        };
        let yeni = BaslangicYedegi {
            komut: "yeni.exe".into(),
            ..eski.clone()
        };
        yedek_ekle(&mut yedekler, eski);
        yedek_ekle(&mut yedekler, yeni.clone());
        assert_eq!(yedekler.len(), 1);
        assert_eq!(yedekler[0].komut, "yeni.exe");
        assert_eq!(yedek_cikar(&mut yedekler, &yeni.kimlik), Some(yeni));
        assert!(yedek_cikar(&mut yedekler, "yok").is_none());
    }

    #[test]
    fn yedek_dosya_adi_guvenli_ve_benzersiz() {
        let ad = yedek_dosya_adi("hkcu-run|C:\\Program Files\\A|B");
        assert!(ad.starts_with("oge-"));
        assert!(ad.chars().all(
            |karakter| karakter.is_ascii_alphanumeric() || matches!(karakter, '-' | '_' | '.')
        ));
        assert_ne!(
            yedek_dosya_adi("hkcu-run|a_b"),
            yedek_dosya_adi("hkcu-run|a|b")
        );
    }
}
