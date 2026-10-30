use crate::registry::WindowsAdaptoru;
use crate::winget::{KomutCalistirici, WindowsKomutCalistirici};
use sp_core::adaptor::SistemAdaptoru;
use sp_core::error::{Hata, Sonuc};
use sp_core::model::{Kove, RegistryKonum, RegistryVeri};
use std::path::Path;
use std::time::Duration;

const ISLEM_ZAMAN_ASIMI: Duration = Duration::from_secs(600);

fn sahiplik_konum(yol: &str, deger: &str) -> RegistryKonum {
    RegistryKonum {
        kove: Kove::Hkcr,
        yol: yol.to_string(),
        deger: deger.to_string(),
        gorunum: None,
    }
}

fn varsayilan_yaz(yol: &str, metin: &str) -> Sonuc<()> {
    WindowsAdaptoru.registry_yaz(
        &sahiplik_konum(yol, ""),
        &RegistryVeri::Sz(metin.to_string()),
    )
}

fn anahtar_sil(yol: &str) -> Sonuc<()> {
    use winreg::enums::HKEY_CLASSES_ROOT;
    use winreg::RegKey;
    match RegKey::predef(HKEY_CLASSES_ROOT).delete_subkey_all(yol) {
        Ok(()) => Ok(()),
        Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(hata) => Err(Hata::Registry(format!("{yol} silinemedi: {hata}"))),
    }
}

pub fn komut_metni(exe: &Path, yer_tutucu: &str) -> String {
    format!("\"{}\" --sahipligi-al \"{}\"", exe.display(), yer_tutucu)
}

const MENU_ANAHTARLARI: [(&str, &str); 3] = [
    (r"*\shell\WinNexus.SahipligiAl", "%1"),
    (r"Directory\shell\WinNexus.SahipligiAl", "%1"),
    (r"Directory\Background\shell\WinNexus.SahipligiAl", "%V"),
];

pub fn menu_ekle(exe: &Path) -> Sonuc<()> {
    for (anahtar, yer_tutucu) in MENU_ANAHTARLARI {
        varsayilan_yaz(anahtar, "Sahipliği Al")?;
        varsayilan_yaz(
            &format!(r"{anahtar}\command"),
            &komut_metni(exe, yer_tutucu),
        )?;
    }
    Ok(())
}

pub fn menu_kaldir() -> Sonuc<()> {
    for (anahtar, _) in MENU_ANAHTARLARI {
        anahtar_sil(anahtar)?;
    }
    Ok(())
}

pub fn sahipligi_al(yol: &Path) -> Sonuc<()> {
    if !yol.exists() {
        return Err(Hata::Beklenmeyen(format!(
            "Yol bulunamadı: {}",
            yol.display()
        )));
    }
    let kanonik = yol
        .canonicalize()
        .map_err(|hata| Hata::Beklenmeyen(format!("Yol doğrulanamadı: {hata}")))?;
    let metin = kanonik.to_string_lossy().to_string();
    let calistirici = WindowsKomutCalistirici;

    let takeown = calistirici.calistir(
        "takeown",
        &[
            "/f".to_string(),
            metin.clone(),
            "/r".to_string(),
            "/a".to_string(),
        ],
        ISLEM_ZAMAN_ASIMI,
    )?;
    if takeown.kod != 0 {
        return Err(Hata::DisArac(format!(
            "takeown başarısız: {}",
            takeown.stderr.trim()
        )));
    }

    let icacls = calistirici.calistir(
        "icacls",
        &[
            metin,
            "/grant".to_string(),
            "*S-1-5-32-544:F".to_string(),
            "/t".to_string(),
            "/c".to_string(),
        ],
        ISLEM_ZAMAN_ASIMI,
    )?;
    if icacls.kod != 0 {
        return Err(Hata::DisArac(format!(
            "icacls başarısız: {}",
            icacls.stderr.trim()
        )));
    }
    Ok(())
}

pub fn hata_kaydet(hata: &str) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(klasor) = exe.parent() else {
        return;
    };
    let dizin = klasor.join("veri").join("gunluk");
    let _ = std::fs::create_dir_all(&dizin);
    let dosya = dizin.join("sahiplik.log");
    if let Ok(mut dosya) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dosya)
    {
        use std::io::Write;
        let _ = writeln!(dosya, "{hata}");
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn komut_metni_tirnaklar() {
        let metin = komut_metni(Path::new(r"C:\Program Files\WinNexus.exe"), "%1");
        assert!(metin.starts_with('"'));
        assert!(metin.contains("--sahipligi-al"));
        assert!(metin.ends_with("\"%1\""));
    }

    #[test]
    fn menu_anahtarlari_yerlesik_fiilleri_ezmez() {
        for (anahtar, _) in MENU_ANAHTARLARI {
            assert!(anahtar.ends_with("WinNexus.SahipligiAl"));
            assert!(!anahtar.contains(r"\runas"));
        }
    }
}
