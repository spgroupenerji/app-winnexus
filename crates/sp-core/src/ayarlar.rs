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
}
