use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Risk {
    Dusuk,
    Orta,
    Yuksek,
    Kritik,
}

impl Risk {
    pub fn metin(self) -> &'static str {
        match self {
            Self::Dusuk => "Düşük",
            Self::Orta => "Orta",
            Self::Yuksek => "Yüksek",
            Self::Kritik => "Kritik",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KuralDurumu {
    Uygulandi,
    Uygulanmadi,
    Kismi,
    Desteklenmiyor,
    Bilinmiyor,
    Hata,
}

impl KuralDurumu {
    pub fn metin(&self) -> &'static str {
        match self {
            Self::Uygulandi => "Uygulandı",
            Self::Uygulanmadi => "Uygulanmadı",
            Self::Kismi => "Kısmi",
            Self::Desteklenmiyor => "Desteklenmiyor",
            Self::Bilinmiyor => "Bilinmiyor",
            Self::Hata => "Hata",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kove {
    #[serde(rename = "HKLM")]
    Hklm,
    #[serde(rename = "HKCU")]
    Hkcu,
    #[serde(rename = "HKCR")]
    Hkcr,
    #[serde(rename = "HKU")]
    Hku,
}

impl Kove {
    pub fn metin(self) -> &'static str {
        match self {
            Self::Hklm => "HKLM",
            Self::Hkcu => "HKCU",
            Self::Hkcr => "HKCR",
            Self::Hku => "HKU",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryVeri {
    Dword(u32),
    Sz(String),
    GenisSz(String),
    Binary(Vec<u8>),
}

impl fmt::Display for RegistryVeri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dword(v) => write!(f, "DWORD({v})"),
            Self::Sz(v) => write!(f, "SZ({v})"),
            Self::GenisSz(v) => write!(f, "EXPAND_SZ({v})"),
            Self::Binary(v) => write!(f, "BINARY({} bayt)", v.len()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryKonum {
    pub kove: Kove,
    pub yol: String,
    pub deger: String,
    pub gorunum: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tur", rename_all = "snake_case")]
pub enum Algilama {
    Registry {
        kove: Kove,
        yol: String,
        deger: String,
        #[serde(default)]
        gorunum: Option<u32>,
    },
}

impl Algilama {
    pub fn konum(&self) -> RegistryKonum {
        match self {
            Self::Registry {
                kove,
                yol,
                deger,
                gorunum,
            } => RegistryKonum {
                kove: *kove,
                yol: yol.clone(),
                deger: deger.clone(),
                gorunum: *gorunum,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tur", rename_all = "snake_case")]
pub enum Dogrulama {
    Registry {
        kove: Kove,
        yol: String,
        deger: String,
        #[serde(default)]
        gorunum: Option<u32>,
        #[serde(default)]
        beklenen: Option<RegistryVeri>,
    },
}

impl Dogrulama {
    pub fn konum(&self) -> RegistryKonum {
        match self {
            Self::Registry {
                kove,
                yol,
                deger,
                gorunum,
                ..
            } => RegistryKonum {
                kove: *kove,
                yol: yol.clone(),
                deger: deger.clone(),
                gorunum: *gorunum,
            },
        }
    }

    pub fn beklenen(&self) -> Option<&RegistryVeri> {
        match self {
            Self::Registry { beklenen, .. } => beklenen.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tur", rename_all = "snake_case")]
pub enum Eylem {
    RegistryYaz {
        kove: Kove,
        yol: String,
        deger: String,
        #[serde(default)]
        gorunum: Option<u32>,
        veri: RegistryVeri,
    },
    RegistrySil {
        kove: Kove,
        yol: String,
        deger: String,
        #[serde(default)]
        gorunum: Option<u32>,
    },
}

impl Eylem {
    pub fn konum(&self) -> RegistryKonum {
        match self {
            Self::RegistryYaz {
                kove,
                yol,
                deger,
                gorunum,
                ..
            }
            | Self::RegistrySil {
                kove,
                yol,
                deger,
                gorunum,
            } => RegistryKonum {
                kove: *kove,
                yol: yol.clone(),
                deger: deger.clone(),
                gorunum: *gorunum,
            },
        }
    }

    pub fn aciklama(&self) -> String {
        match self {
            Self::RegistryYaz {
                kove,
                yol,
                deger,
                gorunum,
                veri,
            } => format!(
                "{}:\\{}\\{} = {}{}",
                kove.metin(),
                yol,
                deger,
                veri,
                gorunum.map(|g| format!(" [{g} bit]")).unwrap_or_default()
            ),
            Self::RegistrySil {
                kove,
                yol,
                deger,
                gorunum,
            } => format!(
                "{}:\\{}\\{} sil{}",
                kove.metin(),
                yol,
                deger,
                gorunum.map(|g| format!(" [{g} bit]")).unwrap_or_default()
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kural {
    pub id: String,
    pub baslik: String,
    pub ozet: String,
    pub kategori: String,
    pub risk: Risk,
    #[serde(default)]
    pub yonetici: bool,
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub yeniden_baslatma: bool,
    pub kanit: String,
    #[serde(default)]
    pub algila: Vec<Algilama>,
    #[serde(default)]
    pub uygula: Vec<Eylem>,
    #[serde(default)]
    pub dogrula: Vec<Dogrulama>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KuralDosyasi {
    #[serde(default)]
    pub rule: Vec<Kural>,
}
