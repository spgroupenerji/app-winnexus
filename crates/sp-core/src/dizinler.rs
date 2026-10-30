use crate::error::{Hata, Sonuc};
use std::fs;
use std::path::{Path, PathBuf};

const VERI_KLASORU: &str = "veri";

#[derive(Debug, Clone)]
pub struct Dizinler {
    pub veri: PathBuf,
    pub yerel: PathBuf,
}

impl Dizinler {
    pub fn varsayilan() -> Sonuc<Self> {
        let exe = std::env::current_exe()
            .map_err(|hata| Hata::Ayar(format!("Uygulama yolu bulunamadı: {hata}")))?;
        Ok(Self::exe_yaninda(&exe))
    }

    pub fn exe_yaninda(exe: &Path) -> Self {
        let kok = exe
            .parent()
            .filter(|anne| !anne.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            veri: kok.clone(),
            yerel: kok.join(VERI_KLASORU),
        }
    }

    pub fn yeni(veri: PathBuf, yerel: PathBuf) -> Self {
        Self { veri, yerel }
    }

    pub fn olustur(&self) -> Sonuc<()> {
        fs::create_dir_all(&self.veri).map_err(|e| Hata::Ayar(e.to_string()))?;
        fs::create_dir_all(&self.yerel).map_err(|e| Hata::Ayar(e.to_string()))?;
        Ok(())
    }

    pub fn ayarlar_dosyasi(&self) -> PathBuf {
        self.veri.join("settings.toml")
    }

    pub fn gunluk_dosyasi(&self) -> PathBuf {
        let simdi = time::OffsetDateTime::now_utc();
        self.yerel.join(format!(
            "journal-{:04}{:02}.jsonl",
            simdi.year(),
            u8::from(simdi.month())
        ))
    }

    pub fn log_dizini(&self) -> PathBuf {
        self.yerel.join("logs")
    }

    pub fn karantina_dizini(&self) -> PathBuf {
        self.yerel.join("quarantine")
    }

    pub fn eski_ayar_dosyasi() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|appdata| {
            PathBuf::from(appdata)
                .join("SP GROUP")
                .join("WinNexus")
                .join("settings.toml")
        })
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn gunluk_dosyasi_aylik() {
        let dizin = Dizinler::yeni(PathBuf::from("veri"), PathBuf::from("yerel"));
        let ad = dizin
            .gunluk_dosyasi()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert!(ad.starts_with("journal-"));
        assert!(ad.ends_with(".jsonl"));
    }

    #[test]
    fn dizinleri_olusturur() {
        let gecici = tempfile::tempdir().unwrap();
        let dizin = Dizinler::yeni(gecici.path().join("veri"), gecici.path().join("yerel"));
        dizin.olustur().unwrap();
        assert!(dizin.veri.exists());
        assert!(dizin.yerel.exists());
    }

    #[test]
    fn ayarlar_ve_veri_exe_yaninda_tutulur() {
        let exe = Path::new(r"C:\Taşınabilir\WinNexus\WinNexus.exe");
        let dizin = Dizinler::exe_yaninda(exe);
        assert_eq!(dizin.veri, PathBuf::from(r"C:\Taşınabilir\WinNexus"));
        assert_eq!(
            dizin.ayarlar_dosyasi(),
            PathBuf::from(r"C:\Taşınabilir\WinNexus\settings.toml")
        );
        assert_eq!(dizin.yerel, PathBuf::from(r"C:\Taşınabilir\WinNexus\veri"));
    }

    #[test]
    fn exe_anne_dizini_yoksa_gecerli_dizin_kullanilir() {
        let dizin = Dizinler::exe_yaninda(Path::new("WinNexus.exe"));
        assert_eq!(dizin.veri, PathBuf::from("."));
        assert_eq!(dizin.yerel, PathBuf::from("./veri"));
    }
}
