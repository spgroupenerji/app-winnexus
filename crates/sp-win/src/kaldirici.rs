use crate::registry::WindowsAdaptoru;
use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::adaptor::SistemAdaptoru;
use sp_core::error::{Hata, Sonuc};
use sp_core::kaldirici::{
    komut_guvenli_mi, sessiz_kaldirma_komutu, KuruluUygulama, UygulamaKaynagi,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};
use winreg::RegKey;

const UNINSTALL_YOLU: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const KALDIRMA_ZAMAN_ASIMI: Duration = Duration::from_secs(1800);
const GERI_YUKLEME_ZAMAN_ASIMI: Duration = Duration::from_secs(180);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KaldirmaSonucu {
    pub komut: String,
    pub kod: i32,
    pub stdout: String,
    pub stderr: String,
}

impl KaldirmaSonucu {
    pub fn basarili(&self) -> bool {
        matches!(self.kod, 0 | 1641 | 3010)
    }

    pub fn ozet(&self) -> String {
        let cikti = if self.stderr.trim().is_empty() {
            self.stdout.trim()
        } else {
            self.stderr.trim()
        };
        if cikti.is_empty() {
            format!("Kod {}", self.kod)
        } else {
            format!("Kod {}: {}", self.kod, cikti)
        }
    }
}

#[derive(Clone)]
pub struct KaldiriciServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl KaldiriciServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    pub fn uygulamalari_tara(&self) -> Sonuc<Vec<KuruluUygulama>> {
        uygulamalari_tara()
    }

    pub fn kaldir(&self, uygulama: &KuruluUygulama, sessiz: bool) -> Sonuc<KaldirmaSonucu> {
        let komut = if sessiz {
            uygulama.sessiz_kaldirma_komutu.clone().ok_or_else(|| {
                Hata::Desteklenmiyor(format!(
                    "{} için sessiz kaldırma komutu bulunamadı.",
                    uygulama.ad
                ))
            })?
        } else {
            uygulama.kaldirma_komutu.clone().ok_or_else(|| {
                Hata::Desteklenmiyor(format!("{} için kaldırma komutu bulunamadı.", uygulama.ad))
            })?
        };
        if !komut_guvenli_mi(&komut) {
            return Err(Hata::Yetki(format!(
                "{} için kaldırma komutu güvenli değil.",
                uygulama.ad
            )));
        }
        let sonuc = self.calistirici.calistir(
            "cmd",
            &["/C".to_string(), komut.clone()],
            KALDIRMA_ZAMAN_ASIMI,
        )?;
        Ok(KaldirmaSonucu {
            komut,
            kod: sonuc.kod,
            stdout: sinirla(&sonuc.stdout),
            stderr: sinirla(&sonuc.stderr),
        })
    }

    pub fn geri_yukleme_noktasi(&self, aciklama: &str) -> Sonuc<KomutSonucu> {
        if !WindowsAdaptoru.yonetici_mi() {
            return Err(Hata::Yetki(
                "Geri yükleme noktası için yönetici izni gerekiyor.".into(),
            ));
        }
        let aciklama = aciklama
            .chars()
            .filter(|karakter| !matches!(karakter, '\'' | '\r' | '\n' | '\0'))
            .take(60)
            .collect::<String>();
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ Checkpoint-Computer -Description 'WinNexus: {aciklama}' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        );
        let sonuc = self.calistirici.calistir(
            "powershell",
            &[
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                betik,
            ],
            GERI_YUKLEME_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Geri yükleme noktası oluşturulamadı: {}",
                sinirla(&sonuc.stderr)
            )));
        }
        Ok(sonuc)
    }
}

fn sinirla(metin: &str) -> String {
    metin.trim().chars().take(2000).collect()
}

fn metin(anahtar: &RegKey, ad: &str) -> Option<String> {
    anahtar
        .get_value(ad)
        .ok()
        .map(|deger: String| deger.trim().to_string())
        .filter(|deger| !deger.is_empty())
}

fn sayi(anahtar: &RegKey, ad: &str) -> Option<u32> {
    anahtar.get_value(ad).ok()
}

fn guncelleme_mi(anahtar: &RegKey) -> bool {
    if metin(anahtar, "ParentKeyName").is_some() {
        return true;
    }
    metin(anahtar, "ReleaseType")
        .map(|tur| {
            let tur = tur.to_ascii_lowercase();
            tur.contains("update")
                || tur.contains("hotfix")
                || tur.contains("security")
                || tur.contains("servicepack")
        })
        .unwrap_or(false)
}

fn uygulama_oku(
    anahtar: &RegKey,
    alt_anahtar: &str,
    kaynak: UygulamaKaynagi,
) -> Option<KuruluUygulama> {
    let ad = metin(anahtar, "DisplayName")?;
    if sayi(anahtar, "SystemComponent") == Some(1) || guncelleme_mi(anahtar) {
        return None;
    }
    if sayi(anahtar, "NoRemove") == Some(1) {
        return None;
    }
    let msi = sayi(anahtar, "WindowsInstaller") == Some(1);
    let urun_kodu = metin(anahtar, "ProductCode");
    let kaldirma = metin(anahtar, "UninstallString")
        .or_else(|| urun_kodu.as_ref().map(|kod| format!("msiexec.exe /X{kod}")));
    let sessiz = sessiz_kaldirma_komutu(
        kaldirma.as_deref(),
        metin(anahtar, "QuietUninstallString").as_deref(),
        msi,
    );
    Some(KuruluUygulama {
        kimlik: KuruluUygulama::kimlik_olustur(kaynak, alt_anahtar),
        ad,
        surum: metin(anahtar, "DisplayVersion"),
        yayinci: metin(anahtar, "Publisher"),
        kurulum_tarihi: metin(anahtar, "InstallDate")
            .and_then(|tarih| sp_core::kaldirici::tarih_metni(&tarih)),
        boyut_kb: sayi(anahtar, "EstimatedSize").map(u64::from),
        kurulum_konumu: metin(anahtar, "InstallLocation"),
        kaldirma_komutu: kaldirma,
        sessiz_kaldirma_komutu: sessiz,
        kaynak,
    })
}

fn kok_tara(
    kok: winreg::HKEY,
    kaynak: UygulamaKaynagi,
    sonuclar: &mut Vec<KuruluUygulama>,
    benzersiz: &mut HashSet<String>,
    hatalar: &mut Vec<String>,
) {
    let bayrak = KEY_READ
        | match kaynak.gorunum() {
            32 => KEY_WOW64_32KEY,
            _ => KEY_WOW64_64KEY,
        };
    let anahtar = match RegKey::predef(kok).open_subkey_with_flags(UNINSTALL_YOLU, bayrak) {
        Ok(anahtar) => anahtar,
        Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return,
        Err(hata) => {
            hatalar.push(format!("{}: {hata}", kaynak.metin()));
            return;
        }
    };
    for alt_anahtar in anahtar.enum_keys().flatten() {
        let Ok(uygulama_anahtari) = anahtar.open_subkey_with_flags(&alt_anahtar, bayrak) else {
            continue;
        };
        let Some(uygulama) = uygulama_oku(&uygulama_anahtari, &alt_anahtar, kaynak) else {
            continue;
        };
        let anahtar_metni = format!(
            "{}|{}|{}",
            uygulama.ad.to_ascii_lowercase(),
            uygulama.surum.as_deref().unwrap_or_default(),
            uygulama
                .yayinci
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
        );
        if benzersiz.insert(anahtar_metni) {
            sonuclar.push(uygulama);
        }
    }
}

pub fn uygulamalari_tara() -> Sonuc<Vec<KuruluUygulama>> {
    let mut sonuclar = Vec::new();
    let mut benzersiz = HashSet::new();
    let mut hatalar = Vec::new();
    for (kok, kaynak) in [
        (HKEY_LOCAL_MACHINE, UygulamaKaynagi::Hklm64),
        (HKEY_LOCAL_MACHINE, UygulamaKaynagi::Hklm32),
        (HKEY_CURRENT_USER, UygulamaKaynagi::Hkcu64),
        (HKEY_CURRENT_USER, UygulamaKaynagi::Hkcu32),
    ] {
        kok_tara(kok, kaynak, &mut sonuclar, &mut benzersiz, &mut hatalar);
    }
    if sonuclar.is_empty() && !hatalar.is_empty() {
        return Err(Hata::Registry(hatalar.join("; ")));
    }
    for hata in hatalar {
        tracing::warn!("Kaldırıcı taraması: {hata}");
    }
    sonuclar.sort_by_key(|uygulama| uygulama.ad.to_lowercase());
    Ok(sonuclar)
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::sync::Mutex;

    struct SahteCalistirici {
        cagrilar: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl KomutCalistirici for SahteCalistirici {
        fn calistir(
            &self,
            program: &str,
            argumanlar: &[String],
            _zaman_asimi: Duration,
        ) -> Sonuc<KomutSonucu> {
            self.cagrilar
                .lock()
                .unwrap()
                .push((program.to_string(), argumanlar.to_vec()));
            Ok(KomutSonucu {
                komut: program.to_string(),
                kod: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    fn uygulama(komut: Option<&str>) -> KuruluUygulama {
        KuruluUygulama {
            kimlik: "test".into(),
            ad: "Test".into(),
            surum: None,
            yayinci: None,
            kurulum_tarihi: None,
            boyut_kb: None,
            kurulum_konumu: None,
            kaldirma_komutu: komut.map(str::to_string),
            sessiz_kaldirma_komutu: None,
            kaynak: UygulamaKaynagi::Hklm64,
        }
    }

    #[test]
    fn kaldirma_komutu_cmd_ile_calistirilir() {
        let sahte = Arc::new(SahteCalistirici {
            cagrilar: Mutex::new(Vec::new()),
        });
        let servis = KaldiriciServisi::yeni(sahte.clone());
        let sonuc = servis
            .kaldir(&uygulama(Some("test.exe /remove")), false)
            .unwrap();
        assert!(sonuc.basarili());
        let cagrilar = sahte.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "cmd");
        assert_eq!(cagrilar[0].1, vec!["/C", "test.exe /remove"]);
    }

    #[test]
    fn komut_yoksa_kaldirmaz() {
        let sahte = Arc::new(SahteCalistirici {
            cagrilar: Mutex::new(Vec::new()),
        });
        let servis = KaldiriciServisi::yeni(sahte);
        assert!(servis.kaldir(&uygulama(None), false).is_err());
    }

    #[test]
    fn tirnakli_komut_cmd_ile_calisir() {
        let calistirici = crate::winget::WindowsKomutCalistirici;
        let sonuc = calistirici
            .calistir(
                "cmd",
                &[
                    "/C".to_string(),
                    r#""C:\Windows\System32\cmd.exe" /C exit 0"#.to_string(),
                ],
                Duration::from_secs(10),
            )
            .unwrap();
        assert_eq!(sonuc.kod, 0, "stderr={}", sonuc.stderr);
    }

    #[test]
    fn tarama_paniklemez() {
        let sonuc = uygulamalari_tara();
        assert!(sonuc.is_ok());
    }

    #[test]
    fn tarama_kimlikleri_benzersizdir() {
        let uygulamalar = uygulamalari_tara().unwrap();
        let mut kimlikler = HashSet::new();
        for uygulama in &uygulamalar {
            assert!(
                kimlikler.insert(uygulama.kimlik.clone()),
                "yinelenen kimlik: {}",
                uygulama.kimlik
            );
        }
    }

    #[test]
    fn sinirla_uzun_ciktiyi_kirpar() {
        let metin = "a".repeat(3000);
        assert_eq!(sinirla(&metin).len(), 2000);
    }
}
