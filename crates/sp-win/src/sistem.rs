use crate::registry::WindowsAdaptoru;
use sp_core::adaptor::{SistemAdaptoru, SistemBilgisi};
use sp_core::error::{Hata, Sonuc};
use sp_core::model::{Kove, RegistryKonum, RegistryVeri};
use sp_core::os::Surum;

pub const GOD_MODE_CLSID: &str = "shell:::{ED7BA470-8E54-465E-825C-99712043E01C}";

pub fn baslik_cubugu_karanlik_ayarla(ham_tutamak: isize, karanlik: bool) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};

    let hwnd: HWND = ham_tutamak as HWND;
    let deger: i32 = i32::from(karanlik);
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
            std::ptr::addr_of!(deger).cast(),
            std::mem::size_of::<i32>() as u32,
        );
    }
}

pub fn god_mode_ac() -> Sonuc<()> {
    std::process::Command::new("explorer.exe")
        .arg(GOD_MODE_CLSID)
        .spawn()
        .map(|_| ())
        .map_err(|hata| Hata::DisArac(format!("God Mode açılamadı: {hata}")))
}

pub fn sistem_bilgisi() -> SistemBilgisi {
    let adaptor = WindowsAdaptoru;
    let surum = surum_oku(&adaptor).unwrap_or(Surum {
        major: 10,
        minor: 0,
        build: 0,
    });
    SistemBilgisi {
        surum,
        yonetici: adaptor.yonetici_mi(),
    }
}

pub fn animasyonlar_kapali_mi() -> bool {
    let adaptor = WindowsAdaptoru;
    let konum = RegistryKonum {
        kove: Kove::Hkcu,
        yol: "Control Panel\\Desktop\\WindowMetrics".to_string(),
        deger: "MinAnimate".to_string(),
        gorunum: Some(64),
    };
    match adaptor.registry_oku(&konum) {
        Ok(Some(RegistryVeri::Sz(deger))) | Ok(Some(RegistryVeri::GenisSz(deger))) => {
            deger.trim() == "0"
        }
        Ok(Some(RegistryVeri::Dword(deger))) => deger == 0,
        _ => false,
    }
}

fn surum_oku(adaptor: &WindowsAdaptoru) -> Option<Surum> {
    let oku = |deger: &str| {
        adaptor
            .registry_oku(&RegistryKonum {
                kove: Kove::Hklm,
                yol: "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion".to_string(),
                deger: deger.to_string(),
                gorunum: Some(64),
            })
            .ok()
            .flatten()
    };

    let major = match oku("CurrentMajorVersionNumber")? {
        RegistryVeri::Dword(deger) => deger,
        _ => return None,
    };
    let minor = match oku("CurrentMinorVersionNumber")? {
        RegistryVeri::Dword(deger) => deger,
        _ => return None,
    };
    let build = match oku("CurrentBuildNumber")? {
        RegistryVeri::Sz(deger) | RegistryVeri::GenisSz(deger) => deger.parse().ok()?,
        _ => return None,
    };

    Some(Surum {
        major,
        minor,
        build,
    })
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sistem_surumunu_okur() {
        let bilgi = sistem_bilgisi();
        assert!(bilgi.surum.major >= 10);
        assert!(bilgi.surum.build > 0);
    }

    #[test]
    fn god_mode_shell_protokolunu_kullanir() {
        assert_eq!(
            GOD_MODE_CLSID,
            "shell:::{ED7BA470-8E54-465E-825C-99712043E01C}"
        );
        assert!(GOD_MODE_CLSID.starts_with("shell:::"));
    }
}
