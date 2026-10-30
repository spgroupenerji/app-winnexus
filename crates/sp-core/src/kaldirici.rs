use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UygulamaKaynagi {
    Hklm64,
    Hklm32,
    Hkcu64,
    Hkcu32,
}

impl UygulamaKaynagi {
    pub fn metin(self) -> &'static str {
        match self {
            Self::Hklm64 => "HKLM 64-bit",
            Self::Hklm32 => "HKLM 32-bit",
            Self::Hkcu64 => "HKCU 64-bit",
            Self::Hkcu32 => "HKCU 32-bit",
        }
    }

    pub fn kove(self) -> &'static str {
        match self {
            Self::Hklm64 | Self::Hklm32 => "HKLM",
            Self::Hkcu64 | Self::Hkcu32 => "HKCU",
        }
    }

    pub fn gorunum(self) -> u32 {
        match self {
            Self::Hklm64 | Self::Hkcu64 => 64,
            Self::Hklm32 | Self::Hkcu32 => 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KuruluUygulama {
    pub kimlik: String,
    pub ad: String,
    pub surum: Option<String>,
    pub yayinci: Option<String>,
    pub kurulum_tarihi: Option<String>,
    pub boyut_kb: Option<u64>,
    pub kurulum_konumu: Option<String>,
    pub kaldirma_komutu: Option<String>,
    pub sessiz_kaldirma_komutu: Option<String>,
    pub kaynak: UygulamaKaynagi,
}

impl KuruluUygulama {
    pub fn kaldirilabilir(&self) -> bool {
        self.kaldirma_komutu
            .as_deref()
            .is_some_and(komut_guvenli_mi)
    }

    pub fn sessiz_kaldirilabilir(&self) -> bool {
        self.sessiz_kaldirma_komutu
            .as_deref()
            .is_some_and(komut_guvenli_mi)
    }

    pub fn boyut_metni(&self) -> Option<String> {
        self.boyut_kb.map(boyut_metni)
    }

    pub fn kimlik_olustur(kaynak: UygulamaKaynagi, alt_anahtar: &str) -> String {
        format!(
            "{}|{}|{}",
            kaynak.metin(),
            kaynak.gorunum(),
            alt_anahtar.to_ascii_lowercase()
        )
    }
}

pub fn komut_guvenli_mi(komut: &str) -> bool {
    let komut = komut.trim();
    !komut.is_empty()
        && komut.len() <= 4096
        && !komut
            .chars()
            .any(|karakter| matches!(karakter, '\0' | '\r' | '\n'))
}

pub fn sessiz_kaldirma_komutu(
    kaldirma: Option<&str>,
    sessiz: Option<&str>,
    msi: bool,
) -> Option<String> {
    if let Some(sessiz) = sessiz.map(str::trim).filter(|metin| !metin.is_empty()) {
        return Some(sessiz.to_string());
    }
    if msi
        || kaldirma
            .map(|metin| metin.to_ascii_lowercase().contains("msiexec"))
            .unwrap_or(false)
    {
        if let Some(guid) = msi_guid(kaldirma.unwrap_or_default()) {
            return Some(format!("msiexec.exe /X{guid} /qn /norestart"));
        }
    }
    None
}

pub fn msi_guid(komut: &str) -> Option<String> {
    let bas = komut.find('{')?;
    let son = komut[bas..].find('}')? + bas;
    let guid = &komut[bas..=son];
    (guid.len() == 38).then(|| guid.to_string())
}

pub fn tarih_metni(ham: &str) -> Option<String> {
    let ham = ham.trim();
    if ham.is_empty() {
        return None;
    }
    if ham.len() == 8 && ham.chars().all(|karakter| karakter.is_ascii_digit()) {
        return Some(format!("{}.{}.{}", &ham[6..8], &ham[4..6], &ham[0..4]));
    }
    Some(ham.to_string())
}

pub fn boyut_metni(kb: u64) -> String {
    let (deger, birim) = if kb >= 1024 * 1024 {
        (kb as f64 / 1024.0 / 1024.0, "GB")
    } else if kb >= 1024 {
        (kb as f64 / 1024.0, "MB")
    } else {
        (kb as f64, "KB")
    };
    format!("{deger:.1} {birim}").replace('.', ",")
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sessiz_komut_quiettakes_precedence() {
        let komut = sessiz_kaldirma_komutu(
            Some("setup.exe /remove"),
            Some("setup.exe /remove /quiet"),
            false,
        );
        assert_eq!(komut.as_deref(), Some("setup.exe /remove /quiet"));
    }

    #[test]
    fn msi_komutu_uretilir() {
        let guid = "{12345678-1234-1234-1234-123456789012}";
        let komut = sessiz_kaldirma_komutu(Some(&format!("MsiExec.exe /I{guid}")), None, true);
        assert_eq!(
            komut.as_deref(),
            Some("msiexec.exe /X{12345678-1234-1234-1234-123456789012} /qn /norestart")
        );
    }

    #[test]
    fn tehlikeli_komut_reddedilir() {
        assert!(!komut_guvenli_mi(""));
        assert!(!komut_guvenli_mi("setup.exe\nrm -rf"));
        assert!(komut_guvenli_mi("setup.exe /S"));
    }

    #[test]
    fn tarih_ve_boyut_bicimlenir() {
        assert_eq!(tarih_metni("20261030").as_deref(), Some("30.10.2026"));
        assert_eq!(boyut_metni(2048), "2,0 MB");
        assert_eq!(boyut_metni(2 * 1024 * 1024), "2,0 GB");
    }

    #[test]
    fn kimlik_kaynak_ve_anahtar_icerir() {
        let kimlik = KuruluUygulama::kimlik_olustur(UygulamaKaynagi::Hklm64, "TestApp");
        assert_eq!(kimlik, "HKLM 64-bit|64|testapp");
    }
}
