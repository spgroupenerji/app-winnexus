use crate::error::{Hata, Sonuc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Surum {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl Surum {
    pub fn ayristir(metin: &str) -> Option<Self> {
        let parcalar = metin
            .split('.')
            .map(|parca| parca.parse::<u32>().ok())
            .collect::<Option<Vec<_>>>()?;
        if !(2..=3).contains(&parcalar.len()) {
            return None;
        }
        Some(Self {
            major: parcalar[0],
            minor: parcalar[1],
            build: *parcalar.get(2).unwrap_or(&0),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Karsilastirma {
    Esit,
    Buyuk,
    BuyukEsit,
    Kucuk,
    KucukEsit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsKosulu {
    pub karsilastirma: Karsilastirma,
    pub surum: Surum,
}

impl OsKosulu {
    pub fn ayristir(ifade: &str) -> Sonuc<Self> {
        let ifade = ifade.trim();
        let (karsilastirma, kalan) = if let Some(kalan) = ifade.strip_prefix(">=") {
            (Karsilastirma::BuyukEsit, kalan)
        } else if let Some(kalan) = ifade.strip_prefix("<=") {
            (Karsilastirma::KucukEsit, kalan)
        } else if let Some(kalan) = ifade.strip_prefix('>') {
            (Karsilastirma::Buyuk, kalan)
        } else if let Some(kalan) = ifade.strip_prefix('<') {
            (Karsilastirma::Kucuk, kalan)
        } else if let Some(kalan) = ifade.strip_prefix('=') {
            (Karsilastirma::Esit, kalan)
        } else {
            (Karsilastirma::Esit, ifade)
        };

        let surum = Surum::ayristir(kalan.trim())
            .ok_or_else(|| Hata::KuralGecersiz(format!("Geçersiz OS koşulu: {ifade}")))?;

        Ok(Self {
            karsilastirma,
            surum,
        })
    }

    pub fn uygun(&self, mevcut: &Surum) -> bool {
        match self.karsilastirma {
            Karsilastirma::Esit => mevcut == &self.surum,
            Karsilastirma::Buyuk => mevcut > &self.surum,
            Karsilastirma::BuyukEsit => mevcut >= &self.surum,
            Karsilastirma::Kucuk => mevcut < &self.surum,
            Karsilastirma::KucukEsit => mevcut <= &self.surum,
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn surum_ayristirir() {
        assert_eq!(
            Surum::ayristir("10.0.22000"),
            Some(Surum {
                major: 10,
                minor: 0,
                build: 22000
            })
        );
        assert_eq!(
            Surum::ayristir("10.0"),
            Some(Surum {
                major: 10,
                minor: 0,
                build: 0
            })
        );
        assert_eq!(Surum::ayristir("10"), None);
    }

    #[test]
    fn karsilastirma_uygular() {
        let kosul = OsKosulu::ayristir(">=10.0.22000").unwrap();
        assert!(kosul.uygun(&Surum {
            major: 10,
            minor: 0,
            build: 26200
        }));
        assert!(!kosul.uygun(&Surum {
            major: 10,
            minor: 0,
            build: 19045
        }));
    }
}
