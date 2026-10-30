use crate::error::{Hata, Sonuc};
use crate::model::RegistryVeri;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GunlukKaydi {
    pub zaman: String,
    pub is_id: String,
    pub kural_id: String,
    pub adim: usize,
    pub eylem: String,
    pub onceki: Option<RegistryVeri>,
    pub sonuc: String,
    pub hata: Option<String>,
}

impl GunlukKaydi {
    fn simdi() -> String {
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
    }

    pub fn niyet(
        is_id: &str,
        kural_id: &str,
        adim: usize,
        eylem: &str,
        onceki: Option<&RegistryVeri>,
    ) -> Self {
        Self {
            zaman: Self::simdi(),
            is_id: is_id.to_string(),
            kural_id: kural_id.to_string(),
            adim,
            eylem: eylem.to_string(),
            onceki: onceki.cloned(),
            sonuc: "niyet".to_string(),
            hata: None,
        }
    }

    pub fn basarili(
        is_id: &str,
        kural_id: &str,
        adim: usize,
        eylem: &str,
        onceki: Option<&RegistryVeri>,
    ) -> Self {
        Self {
            sonuc: "basarili".to_string(),
            ..Self::niyet(is_id, kural_id, adim, eylem, onceki)
        }
    }

    pub fn hatali(
        is_id: &str,
        kural_id: &str,
        adim: usize,
        eylem: &str,
        onceki: Option<&RegistryVeri>,
        hata: &str,
    ) -> Self {
        Self {
            sonuc: "hata".to_string(),
            hata: Some(hata.to_string()),
            ..Self::niyet(is_id, kural_id, adim, eylem, onceki)
        }
    }
}

pub struct Gunluk {
    dosya: PathBuf,
}

impl Gunluk {
    pub fn yeni(dosya: PathBuf) -> Self {
        Self { dosya }
    }

    pub fn dosya(&self) -> &Path {
        &self.dosya
    }

    pub fn yaz(&self, kayit: &GunlukKaydi) -> Sonuc<()> {
        if let Some(anne) = self.dosya.parent() {
            fs::create_dir_all(anne).map_err(|e| Hata::Gunluk(e.to_string()))?;
        }
        let satir = serde_json::to_string(kayit).map_err(|e| Hata::Gunluk(e.to_string()))?;
        let mut dosya = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.dosya)
            .map_err(|e| Hata::Gunluk(e.to_string()))?;
        writeln!(dosya, "{satir}").map_err(|e| Hata::Gunluk(e.to_string()))?;
        dosya.flush().map_err(|e| Hata::Gunluk(e.to_string()))?;
        dosya.sync_all().map_err(|e| Hata::Gunluk(e.to_string()))?;
        Ok(())
    }

    pub fn son_kayitlar(&self, adet: usize) -> Sonuc<Vec<GunlukKaydi>> {
        if !self.dosya.exists() {
            return Ok(Vec::new());
        }
        let dosya = fs::File::open(&self.dosya).map_err(|e| Hata::Gunluk(e.to_string()))?;
        let okuyucu = BufReader::new(dosya);
        let mut kayitlar = Vec::new();
        for satir in okuyucu.lines() {
            let satir = satir.map_err(|e| Hata::Gunluk(e.to_string()))?;
            if satir.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<GunlukKaydi>(&satir) {
                Ok(kayit) => kayitlar.push(kayit),
                Err(hata) => tracing::warn!(%hata, "Günlük satırı atlandı"),
            }
        }
        let baslangic = kayitlar.len().saturating_sub(adet);
        Ok(kayitlar.split_off(baslangic))
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn yazar_ve_son_kayitlari_dondurur() {
        let dizin = tempfile::tempdir().unwrap();
        let gunluk = Gunluk::yeni(dizin.path().join("journal.jsonl"));
        let kayit = GunlukKaydi::niyet("is-1", "test.kural", 0, "test", None);
        gunluk.yaz(&kayit).unwrap();
        let kayitlar = gunluk.son_kayitlar(10).unwrap();
        assert_eq!(kayitlar.len(), 1);
        assert_eq!(kayitlar[0].is_id, "is-1");
    }

    #[test]
    fn bozuk_satiri_atlar() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("journal.jsonl");
        std::fs::write(&dosya, "bozuk\n").unwrap();
        let gunluk = Gunluk::yeni(dosya);
        let kayitlar = gunluk.son_kayitlar(10).unwrap();
        assert!(kayitlar.is_empty());
    }
}
