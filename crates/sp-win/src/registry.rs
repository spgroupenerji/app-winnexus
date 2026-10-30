use sp_core::adaptor::SistemAdaptoru;
use sp_core::error::{Hata, Sonuc};
use sp_core::model::{Kove, RegistryKonum, RegistryVeri};
use winreg::enums::{
    HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_READ,
    KEY_WOW64_32KEY, KEY_WOW64_64KEY, KEY_WRITE, REG_BINARY, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
};
use winreg::{RegKey, RegValue};

pub struct WindowsAdaptoru;

impl WindowsAdaptoru {
    fn kok(kove: Kove) -> winreg::HKEY {
        match kove {
            Kove::Hklm => HKEY_LOCAL_MACHINE,
            Kove::Hkcu => HKEY_CURRENT_USER,
            Kove::Hkcr => HKEY_CLASSES_ROOT,
            Kove::Hku => HKEY_USERS,
        }
    }

    fn bayraklar(gorunum: Option<u32>, yazma: bool) -> u32 {
        let mut bayrak = if yazma {
            KEY_READ | KEY_WRITE
        } else {
            KEY_READ
        };
        match gorunum {
            Some(32) => bayrak |= KEY_WOW64_32KEY,
            Some(64) => bayrak |= KEY_WOW64_64KEY,
            _ => {}
        }
        bayrak
    }

    fn regdeger(veri: &RegistryVeri) -> RegValue {
        match veri {
            RegistryVeri::Dword(deger) => RegValue {
                bytes: deger.to_le_bytes().to_vec(),
                vtype: REG_DWORD,
            },
            RegistryVeri::Sz(metin) => RegValue {
                bytes: utf16_bayt(metin),
                vtype: REG_SZ,
            },
            RegistryVeri::GenisSz(metin) => RegValue {
                bytes: utf16_bayt(metin),
                vtype: REG_EXPAND_SZ,
            },
            RegistryVeri::Binary(baytlar) => RegValue {
                bytes: baytlar.clone(),
                vtype: REG_BINARY,
            },
        }
    }

    fn degerden(ham: RegValue) -> RegistryVeri {
        match ham.vtype {
            REG_DWORD if ham.bytes.len() >= 4 => RegistryVeri::Dword(u32::from_le_bytes([
                ham.bytes[0],
                ham.bytes[1],
                ham.bytes[2],
                ham.bytes[3],
            ])),
            REG_SZ => RegistryVeri::Sz(utf16_metin(&ham.bytes)),
            REG_EXPAND_SZ => RegistryVeri::GenisSz(utf16_metin(&ham.bytes)),
            _ => RegistryVeri::Binary(ham.bytes),
        }
    }
}

fn utf16_bayt(metin: &str) -> Vec<u8> {
    metin
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn utf16_metin(baytlar: &[u8]) -> String {
    let birimler = baytlar
        .as_chunks::<2>()
        .0
        .iter()
        .map(|parca| u16::from_le_bytes(*parca))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&birimler)
        .trim_end_matches('\0')
        .to_string()
}

impl SistemAdaptoru for WindowsAdaptoru {
    fn registry_oku(&self, konum: &RegistryKonum) -> Sonuc<Option<RegistryVeri>> {
        let anahtar = match RegKey::predef(Self::kok(konum.kove))
            .open_subkey_with_flags(&konum.yol, Self::bayraklar(konum.gorunum, false))
        {
            Ok(anahtar) => anahtar,
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(hata) => return Err(Hata::Registry(format!("{} açılamadı: {hata}", konum.yol))),
        };

        match anahtar.get_raw_value(&konum.deger) {
            Ok(ham) => Ok(Some(Self::degerden(ham))),
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(hata) => Err(Hata::Registry(format!(
                "{}\\{} okunamadı: {hata}",
                konum.yol, konum.deger
            ))),
        }
    }

    fn registry_yaz(&self, konum: &RegistryKonum, veri: &RegistryVeri) -> Sonuc<()> {
        let (anahtar, _) = RegKey::predef(Self::kok(konum.kove))
            .create_subkey_with_flags(&konum.yol, Self::bayraklar(konum.gorunum, true))
            .map_err(|hata| Hata::Registry(format!("{} oluşturulamadı: {hata}", konum.yol)))?;
        anahtar
            .set_raw_value(&konum.deger, &Self::regdeger(veri))
            .map_err(|hata| {
                Hata::Registry(format!("{}\\{} yazılamadı: {hata}", konum.yol, konum.deger))
            })
    }

    fn registry_sil(&self, konum: &RegistryKonum) -> Sonuc<()> {
        let anahtar = match RegKey::predef(Self::kok(konum.kove))
            .open_subkey_with_flags(&konum.yol, Self::bayraklar(konum.gorunum, true))
        {
            Ok(anahtar) => anahtar,
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(hata) => return Err(Hata::Registry(format!("{} açılamadı: {hata}", konum.yol))),
        };

        match anahtar.delete_value(&konum.deger) {
            Ok(()) => Ok(()),
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(hata) => Err(Hata::Registry(format!(
                "{}\\{} silinemedi: {hata}",
                konum.yol, konum.deger
            ))),
        }
    }

    fn yonetici_mi(&self) -> bool {
        unsafe { windows_sys::Win32::UI::Shell::IsUserAnAdmin() != 0 }
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn konum(deger: &str) -> RegistryKonum {
        RegistryKonum {
            kove: Kove::Hkcu,
            yol: "Software\\SP GROUP\\WinNexus\\Test".to_string(),
            deger: deger.to_string(),
            gorunum: Some(64),
        }
    }

    #[test]
    fn dword_yaz_oku_sil() {
        let adaptor = WindowsAdaptoru;
        let konum = konum("DwordDeger");
        let _ = adaptor.registry_sil(&konum);
        adaptor
            .registry_yaz(&konum, &RegistryVeri::Dword(7))
            .unwrap();
        assert_eq!(
            adaptor.registry_oku(&konum).unwrap(),
            Some(RegistryVeri::Dword(7))
        );
        adaptor.registry_sil(&konum).unwrap();
        assert_eq!(adaptor.registry_oku(&konum).unwrap(), None);
    }

    #[test]
    fn metin_yaz_oku() {
        let adaptor = WindowsAdaptoru;
        let konum = konum("MetinDeger");
        let _ = adaptor.registry_sil(&konum);
        adaptor
            .registry_yaz(&konum, &RegistryVeri::Sz("Türkçe metin".into()))
            .unwrap();
        assert_eq!(
            adaptor.registry_oku(&konum).unwrap(),
            Some(RegistryVeri::Sz("Türkçe metin".into()))
        );
        adaptor.registry_sil(&konum).unwrap();
    }

    #[test]
    fn olmayan_deger_none() {
        let adaptor = WindowsAdaptoru;
        let konum = konum("OlmayanDeger");
        let _ = adaptor.registry_sil(&konum);
        assert_eq!(adaptor.registry_oku(&konum).unwrap(), None);
    }
}
