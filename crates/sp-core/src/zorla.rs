use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

const DOSYA_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const AZAMI_ALT_ANAHTAR_UZUNLUGU: usize = 256;
const YASAKLI_KOK_DEGISKENLERI: [&str; 9] = [
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramData",
    "CommonProgramFiles",
    "CommonProgramFiles(x86)",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
    "PUBLIC",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZorlaSonucu {
    pub ad: String,
    pub klasor_tasindi: Option<String>,
    pub kayit_silindi: bool,
    pub yedek: Option<String>,
    pub hatalar: Vec<String>,
}

pub fn kimlik_coz(kimlik: &str) -> Option<(bool, u32, String)> {
    let parcalar = kimlik.split('|').collect::<Vec<_>>();
    if parcalar.len() != 3 {
        return None;
    }
    let (hklm_mi, gorunum) = match parcalar[0] {
        "HKLM 64-bit" => (true, 64),
        "HKLM 32-bit" => (true, 32),
        "HKCU 64-bit" => (false, 64),
        "HKCU 32-bit" => (false, 32),
        _ => return None,
    };
    let beklenen = if gorunum == 64 { "64" } else { "32" };
    if parcalar[1] != beklenen || !alt_anahtar_gecerli_mi(parcalar[2]) {
        return None;
    }
    Some((hklm_mi, gorunum, parcalar[2].to_string()))
}

pub fn kurulum_yolu_guvenli_mi(yol: &Path) -> bool {
    if !yol.is_absolute() || !yol.is_dir() {
        return false;
    }
    let Ok(meta) = fs::symlink_metadata(yol) else {
        return false;
    };
    if meta.file_attributes() & DOSYA_ATTRIBUTE_REPARSE_POINT != 0 || yol.file_name().is_none() {
        return false;
    }
    if yol
        .components()
        .any(|bilesen| matches!(bilesen, Component::ParentDir))
    {
        return false;
    }
    let normaller = yol
        .components()
        .filter_map(|bilesen| match bilesen {
            Component::Normal(parca) => Some(parca.to_string_lossy().to_lowercase()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if normaller.len() < 2 || normaller[0] == "windows" {
        return false;
    }
    !yasakli_kok_mu(yol)
}

fn alt_anahtar_gecerli_mi(alt: &str) -> bool {
    !alt.trim().is_empty()
        && alt.chars().count() < AZAMI_ALT_ANAHTAR_UZUNLUGU
        && !alt.contains("..")
        && !alt.starts_with('\\')
        && !alt.ends_with('\\')
        && !alt
            .chars()
            .any(|karakter| matches!(karakter, '\0' | '\r' | '\n' | '|'))
}

fn yasakli_kok_mu(yol: &Path) -> bool {
    let metin = yol_metni(yol);
    if let Some(sistem) = sistem_koku() {
        if metin == sistem || metin.starts_with(&format!("{sistem}\\")) {
            return true;
        }
    }
    YASAKLI_KOK_DEGISKENLERI.iter().any(|degisken| {
        std::env::var_os(degisken)
            .map(PathBuf::from)
            .map(|kok| yol_metni(&kok))
            .is_some_and(|kok| {
                !kok.is_empty() && (metin == kok || kok.starts_with(&format!("{metin}\\")))
            })
    })
}

fn sistem_koku() -> Option<String> {
    std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("windir"))
        .map(PathBuf::from)
        .map(|kok| yol_metni(&kok))
        .filter(|kok| !kok.is_empty())
}

fn yol_metni(yol: &Path) -> String {
    yol.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::kaldirici::{KuruluUygulama, UygulamaKaynagi};
    use std::env;

    fn gecici_uygulama() -> (tempfile::TempDir, PathBuf) {
        let gecici = tempfile::tempdir().unwrap();
        let uygulama = gecici.path().join("Uygulama");
        fs::create_dir_all(&uygulama).unwrap();
        (gecici, uygulama)
    }

    #[test]
    fn kimlik_coz_kok_ve_gorunum_dondurur() {
        assert_eq!(
            kimlik_coz("HKLM 64-bit|64|TestUygulama"),
            Some((true, 64, "TestUygulama".to_string()))
        );
        assert_eq!(
            kimlik_coz("HKCU 32-bit|32|TestUygulama"),
            Some((false, 32, "TestUygulama".to_string()))
        );
        let kimlik = KuruluUygulama::kimlik_olustur(UygulamaKaynagi::Hklm32, "TestApp");
        assert_eq!(kimlik_coz(&kimlik), Some((true, 32, "testapp".to_string())));
    }

    #[test]
    fn kimlik_coz_gecersiz_bicimleri_reddeder() {
        assert_eq!(kimlik_coz(""), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|32|Test"), None);
        assert_eq!(kimlik_coz("HKEY_LOCAL_MACHINE|64|Test"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|Test|Fazla"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|   "), None);
    }

    #[test]
    fn kimlik_coz_tehlikeli_alt_anahtarlari_reddeder() {
        assert_eq!(kimlik_coz("HKLM 64-bit|64|Uygulama\\..\\Sistem"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|.."), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|Test\nAlt"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|Test\0Alt"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|\\Test"), None);
        assert_eq!(kimlik_coz("HKLM 64-bit|64|Test\\"), None);
    }

    #[test]
    fn kimlik_coz_uzun_alt_anahtari_reddeder() {
        let sinirda = format!("HKLM 64-bit|64|{}", "a".repeat(255));
        let asan = format!("HKLM 64-bit|64|{}", "a".repeat(256));
        assert!(kimlik_coz(&sinirda).is_some());
        assert_eq!(kimlik_coz(&asan), None);
    }

    #[test]
    fn guvenli_yol_uygulama_klasorunu_kabul_eder() {
        let (_gecici, uygulama) = gecici_uygulama();
        assert!(kurulum_yolu_guvenli_mi(&uygulama));
    }

    #[test]
    fn guvenli_yol_sistem_koklerini_reddeder() {
        for degisken in [
            "ProgramFiles",
            "ProgramFiles(x86)",
            "ProgramData",
            "USERPROFILE",
        ] {
            let Some(kok) = env::var_os(degisken).map(PathBuf::from) else {
                continue;
            };
            if kok.is_dir() {
                assert!(
                    !kurulum_yolu_guvenli_mi(&kok),
                    "{degisken} kökü reddedilmeli"
                );
            }
        }
        if let Some(sistem) = env::var_os("SystemRoot").or_else(|| env::var_os("windir")) {
            let sistem = PathBuf::from(sistem);
            assert!(!kurulum_yolu_guvenli_mi(&sistem));
            assert!(!kurulum_yolu_guvenli_mi(&sistem.join("System32")));
        }
    }

    #[test]
    fn guvenli_yol_kullanici_profili_ustunu_reddeder() {
        let Some(profil) = env::var_os("USERPROFILE").map(PathBuf::from) else {
            return;
        };
        let appdata = profil.join("AppData");
        if appdata.is_dir() {
            assert!(!kurulum_yolu_guvenli_mi(&appdata));
        }
    }

    #[test]
    fn guvenli_yol_goreli_ve_eksik_yollari_reddeder() {
        assert!(!kurulum_yolu_guvenli_mi(Path::new("Uygulama")));
        assert!(!kurulum_yolu_guvenli_mi(Path::new(
            r"C:\WinNexusTestYok\Uygulama"
        )));
        assert!(!kurulum_yolu_guvenli_mi(Path::new(r"C:\")));
    }

    #[test]
    fn guvenli_yol_ust_dizin_ve_dosyalari_reddeder() {
        let (gecici, uygulama) = gecici_uygulama();
        let ust_dizin = uygulama.join("..").join("Uygulama");
        assert!(!kurulum_yolu_guvenli_mi(&ust_dizin));
        let dosya = gecici.path().join("dosya.txt");
        fs::write(&dosya, b"x").unwrap();
        assert!(!kurulum_yolu_guvenli_mi(&dosya));
    }

    #[test]
    fn guvenli_yol_windows_alt_dizinini_reddeder() {
        if let Some(sistem) = env::var_os("SystemRoot").or_else(|| env::var_os("windir")) {
            let alt = PathBuf::from(sistem).join("System32");
            assert!(!kurulum_yolu_guvenli_mi(&alt));
        }
    }
}
