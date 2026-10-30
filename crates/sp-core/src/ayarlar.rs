use crate::error::{Hata, Sonuc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Tema {
    #[default]
    Sistem,
    Acik,
    Koyu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ayarlar {
    #[serde(default)]
    pub tema: Tema,
    #[serde(default = "varsayilan_dil")]
    pub dil: String,
    #[serde(default)]
    pub winget_lisans_onayi: bool,
    #[serde(default)]
    pub winget_kaynak_onayi: bool,
    #[serde(default)]
    pub adb_yolu: Option<String>,
    #[serde(default)]
    pub winget_eslesmeler: BTreeMap<String, String>,
    #[serde(default)]
    pub yasal_uyari_kabul_edildi: bool,
}

fn varsayilan_dil() -> String {
    "tr".to_string()
}

impl Default for Ayarlar {
    fn default() -> Self {
        Self {
            tema: Tema::Sistem,
            dil: varsayilan_dil(),
            winget_lisans_onayi: false,
            winget_kaynak_onayi: false,
            adb_yolu: None,
            winget_eslesmeler: BTreeMap::new(),
            yasal_uyari_kabul_edildi: false,
        }
    }
}

const UYGULAMA_ADI: &str = "WinNexus";
const TERCIH_SURUMU: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct TercihDosyasi {
    pub uygulama: String,
    pub surum: u32,
    pub ayarlar: Ayarlar,
}

impl Ayarlar {
    pub fn yukle(dosya: &Path) -> Sonuc<Self> {
        if !dosya.exists() {
            return Ok(Self::default());
        }
        let metin = fs::read_to_string(dosya).map_err(|e| Hata::Ayar(e.to_string()))?;
        toml::from_str(&metin).map_err(|e| Hata::Ayar(format!("Ayarlar okunamadı: {e}")))
    }

    pub fn kaydet(&self, dosya: &Path) -> Sonuc<()> {
        if let Some(anne) = dosya.parent() {
            fs::create_dir_all(anne).map_err(|e| Hata::Ayar(e.to_string()))?;
        }
        let metin = toml::to_string_pretty(self).map_err(|e| Hata::Ayar(e.to_string()))?;
        fs::write(dosya, metin).map_err(|e| Hata::Ayar(e.to_string()))
    }

    pub fn disa_aktar(&self, dosya: &Path) -> Sonuc<()> {
        if let Some(anne) = dosya.parent() {
            fs::create_dir_all(anne)
                .map_err(|e| Hata::Ayar(format!("Tercih dizini oluşturulamadı: {e}")))?;
        }
        let sarmal = TercihDosyasi {
            uygulama: UYGULAMA_ADI.to_string(),
            surum: TERCIH_SURUMU,
            ayarlar: self.clone(),
        };
        let metin = serde_json::to_string_pretty(&sarmal)
            .map_err(|e| Hata::Ayar(format!("Tercihler JSON biçimine dönüştürülemedi: {e}")))?;
        fs::write(dosya, metin).map_err(|e| Hata::Ayar(format!("Tercihler yazılamadı: {e}")))
    }

    pub fn ice_aktar(dosya: &Path) -> Sonuc<Self> {
        let metin = fs::read_to_string(dosya)
            .map_err(|e| Hata::Ayar(format!("Tercih dosyası okunamadı: {e}")))?;
        let sarmal: TercihDosyasi = serde_json::from_str(&metin)
            .map_err(|e| Hata::Ayar(format!("Tercih dosyası geçersiz JSON içeriyor: {e}")))?;
        if sarmal.uygulama != UYGULAMA_ADI {
            return Err(Hata::Ayar(format!(
                "Tercih dosyası bu uygulamaya ait değil: beklenen \"{UYGULAMA_ADI}\", bulunan \"{}\".", sarmal.uygulama
            )));
        }
        if sarmal.surum > TERCIH_SURUMU {
            return Err(Hata::Ayar(format!(
                "Tercih dosyası sürümü desteklenmiyor: en fazla {TERCIH_SURUMU}, bulunan {}.",
                sarmal.surum
            )));
        }
        Ok(sarmal.ayarlar)
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kaydet_ve_yukle() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("settings.toml");
        let ayarlar = Ayarlar {
            tema: Tema::Koyu,
            winget_lisans_onayi: true,
            ..Ayarlar::default()
        };
        ayarlar.kaydet(&dosya).unwrap();
        let okunan = Ayarlar::yukle(&dosya).unwrap();
        assert_eq!(okunan.tema, Tema::Koyu);
        assert!(okunan.winget_lisans_onayi);
        assert_eq!(okunan.dil, "tr");
        assert!(!okunan.yasal_uyari_kabul_edildi);
    }

    #[test]
    fn eksik_dosya_varsayilan() {
        let dizin = tempfile::tempdir().unwrap();
        let ayarlar = Ayarlar::yukle(&dizin.path().join("yok.toml")).unwrap();
        assert_eq!(ayarlar.tema, Tema::Sistem);
    }

    #[test]
    fn bozuk_dosya_hata() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("settings.toml");
        std::fs::write(&dosya, "bu = [geçersiz").unwrap();
        assert!(Ayarlar::yukle(&dosya).is_err());
    }

    #[test]
    fn tercih_roundtrip() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("tercihler.json");
        let kaynak = Ayarlar {
            tema: Tema::Koyu,
            winget_lisans_onayi: true,
            adb_yolu: Some("C:\\adb\\adb.exe".to_string()),
            ..Ayarlar::default()
        };
        kaynak.disa_aktar(&dosya).unwrap();
        let okunan = Ayarlar::ice_aktar(&dosya).unwrap();
        assert_eq!(okunan.tema, Tema::Koyu);
        assert_eq!(okunan.winget_lisans_onayi, kaynak.winget_lisans_onayi);
        assert_eq!(okunan.adb_yolu, kaynak.adb_yolu);
        assert_eq!(okunan.dil, "tr");
    }

    #[test]
    fn gecersiz_json_hata() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("tercihler.json");
        std::fs::write(&dosya, "{ bu geçerli JSON değil").unwrap();
        assert!(Ayarlar::ice_aktar(&dosya).is_err());
    }

    #[test]
    fn eksik_alan_varsayilan() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("tercihler.json");
        std::fs::write(&dosya, r#"{"uygulama":"WinNexus","surum":1,"ayarlar":{}}"#).unwrap();
        let okunan = Ayarlar::ice_aktar(&dosya).unwrap();
        assert_eq!(okunan.tema, Tema::Sistem);
        assert_eq!(okunan.dil, "tr");
        assert!(!okunan.winget_lisans_onayi);
        assert_eq!(okunan.adb_yolu, None);
    }

    #[test]
    fn yanlis_uygulama_adi_hata() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("tercihler.json");
        std::fs::write(
            &dosya,
            r#"{"uygulama":"BaskaUygulama","surum":1,"ayarlar":{}}"#,
        )
        .unwrap();
        assert!(Ayarlar::ice_aktar(&dosya).is_err());
    }

    #[test]
    fn gelecek_surum_hata() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("tercihler.json");
        std::fs::write(&dosya, r#"{"uygulama":"WinNexus","surum":2,"ayarlar":{}}"#).unwrap();
        assert!(Ayarlar::ice_aktar(&dosya).is_err());
    }

    #[test]
    fn yazilamayan_yol_hata() {
        let dizin = tempfile::tempdir().unwrap();
        let engel = dizin.path().join("engel");
        std::fs::write(&engel, "dosya").unwrap();
        let dosya = engel.join("alt").join("tercihler.json");
        assert!(Ayarlar::default().disa_aktar(&dosya).is_err());
    }
}
