use crate::registry::WindowsAdaptoru;
use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::adaptor::SistemAdaptoru;
use sp_core::error::{Hata, Sonuc};
use sp_core::kaldirici::KuruluUygulama;
use sp_core::zorla::{kimlik_coz, kurulum_yolu_guvenli_mi, ZorlaSonucu};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const UNINSTALL_YOLU: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const REG_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const CIKTI_SINIRI: usize = 2000;
const KARANTINA_DENEME_SAYISI: u32 = 1000;
const DOSYA_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

pub struct ZorlaServisi {
    calistirici: Arc<dyn KomutCalistirici>,
    karantina_koku: PathBuf,
}

impl ZorlaServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>, karantina_koku: PathBuf) -> Self {
        Self {
            calistirici,
            karantina_koku,
        }
    }

    pub fn zorla_kaldir(&self, uygulama: &KuruluUygulama) -> Sonuc<ZorlaSonucu> {
        let (hklm_mi, gorunum, alt_anahtar) = kimlik_coz(&uygulama.kimlik).ok_or_else(|| {
            Hata::Dogrulama(format!(
                "{} için kayıt defteri kimliği geçersiz.",
                uygulama.ad
            ))
        })?;
        if !WindowsAdaptoru.yonetici_mi() {
            return Err(Hata::Yetki(
                "Zorla kaldırma için yönetici izni gerekiyor.".into(),
            ));
        }
        let karantina = self.karantina_dizini()?;
        let kayit_yolu = kayit_yolu(hklm_mi, &alt_anahtar);
        let (kayit_silindi, yedek, kayit_hatasi) =
            self.kayit_isle(&karantina, &kayit_yolu, gorunum);
        let (klasor_tasindi, klasor_hatasi) = self.klasor_tasi(uygulama, &karantina);
        let hatalar = [kayit_hatasi, klasor_hatasi]
            .into_iter()
            .flatten()
            .collect();
        Ok(ZorlaSonucu {
            ad: uygulama.ad.clone(),
            klasor_tasindi,
            kayit_silindi,
            yedek,
            hatalar,
        })
    }

    fn karantina_dizini(&self) -> Sonuc<PathBuf> {
        fs::create_dir_all(&self.karantina_koku).map_err(|hata| {
            Hata::Beklenmeyen(format!("Karantina dizini oluşturulamadı: {hata}"))
        })?;
        let ad = format!("zorla-{}", simdiki_saniye());
        for sira in 0..KARANTINA_DENEME_SAYISI {
            let aday = if sira == 0 {
                self.karantina_koku.join(&ad)
            } else {
                self.karantina_koku.join(format!("{ad}-{sira}"))
            };
            match fs::create_dir(&aday) {
                Ok(()) => return Ok(aday),
                Err(hata) if hata.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(hata) => {
                    return Err(Hata::Beklenmeyen(format!(
                        "Karantina dizini oluşturulamadı: {hata}"
                    )))
                }
            }
        }
        Err(Hata::Beklenmeyen(
            "Karantina dizini adı üretilemedi.".into(),
        ))
    }

    fn kayit_isle(
        &self,
        karantina: &Path,
        kayit_yolu: &str,
        gorunum: u32,
    ) -> (bool, Option<String>, Option<String>) {
        let yedek = karantina.join("kayit.reg");
        let yedek_metni = yedek.to_string_lossy().to_string();
        let yedek_sonucu =
            self.reg_calistir(&["export", kayit_yolu, yedek_metni.as_str(), "/y"], gorunum);
        match yedek_sonucu {
            Ok(sonuc) if sonuc.kod == 0 => {}
            Ok(sonuc) => {
                return (
                    false,
                    None,
                    Some(format!(
                        "Kayıt defteri yedeklenemedi: {}",
                        sinirla(&sonuc.stderr)
                    )),
                )
            }
            Err(hata) => {
                return (
                    false,
                    None,
                    Some(format!("Kayıt defteri yedeklenemedi: {hata}")),
                )
            }
        }
        match self.reg_calistir(&["delete", kayit_yolu, "/f"], gorunum) {
            Ok(sonuc) if sonuc.kod == 0 => (true, Some(yedek_metni), None),
            Ok(sonuc) => (
                false,
                Some(yedek_metni),
                Some(format!(
                    "Kayıt defteri silinemedi: {}",
                    sinirla(&sonuc.stderr)
                )),
            ),
            Err(hata) => (
                false,
                Some(yedek_metni),
                Some(format!("Kayıt defteri silinemedi: {hata}")),
            ),
        }
    }

    fn reg_calistir(&self, temel: &[&str], gorunum: u32) -> Sonuc<KomutSonucu> {
        let mut argumanlar = temel
            .iter()
            .map(|parca| parca.to_string())
            .collect::<Vec<_>>();
        argumanlar.push(if gorunum == 32 { "/reg:32" } else { "/reg:64" }.to_string());
        self.calistirici
            .calistir("reg.exe", &argumanlar, REG_ZAMAN_ASIMI)
    }

    fn klasor_tasi(
        &self,
        uygulama: &KuruluUygulama,
        karantina: &Path,
    ) -> (Option<String>, Option<String>) {
        let Some(konum) = uygulama
            .kurulum_konumu
            .as_deref()
            .map(str::trim)
            .map(|konum| konum.trim_matches('"'))
            .filter(|konum| !konum.is_empty())
        else {
            return (None, None);
        };
        let kaynak = PathBuf::from(konum);
        if !kaynak.exists() {
            return (None, None);
        }
        if !kurulum_yolu_guvenli_mi(&kaynak) {
            return (
                None,
                Some(format!(
                    "Kurulum konumu güvenli değil, klasör taşınmadı: {konum}"
                )),
            );
        }
        if bos_klasor_mu(&kaynak) {
            return (None, None);
        }
        let Some(hedef) = tasima_hedefi(&kaynak, karantina) else {
            return (
                None,
                Some(format!(
                    "Karantina hedefi kurulum klasörünün içinde: {konum}"
                )),
            );
        };
        match tasi(&kaynak, &hedef) {
            Ok(()) => (Some(hedef.to_string_lossy().to_string()), None),
            Err(hata) => (None, Some(format!("{konum} taşınamadı: {hata}"))),
        }
    }
}

fn tasima_hedefi(kaynak: &Path, karantina: &Path) -> Option<PathBuf> {
    let hedef = karantina.join(kaynak.file_name()?);
    (!hedef.starts_with(kaynak)).then_some(hedef)
}

fn kayit_yolu(hklm_mi: bool, alt_anahtar: &str) -> String {
    format!(
        "{}\\{UNINSTALL_YOLU}\\{alt_anahtar}",
        if hklm_mi { "HKLM" } else { "HKCU" }
    )
}

fn simdiki_saniye() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn sinirla(metin: &str) -> String {
    metin.trim().chars().take(CIKTI_SINIRI).collect()
}

fn bos_klasor_mu(yol: &Path) -> bool {
    fs::read_dir(yol)
        .map(|mut girisler| girisler.next().is_none())
        .unwrap_or(false)
}

fn baglanti_mi(meta: &fs::Metadata) -> bool {
    meta.file_attributes() & DOSYA_ATTRIBUTE_REPARSE_POINT != 0
}

fn tasi(kaynak: &Path, hedef: &Path) -> std::io::Result<()> {
    if fs::rename(kaynak, hedef).is_ok() {
        return Ok(());
    }
    kopyala(kaynak, hedef)?;
    sil(kaynak)
}

fn kopyala(kaynak: &Path, hedef: &Path) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(kaynak)?;
    if baglanti_mi(&meta) {
        return Ok(());
    }
    if meta.is_dir() {
        fs::create_dir_all(hedef)?;
        for giris in fs::read_dir(kaynak)? {
            let giris = giris?;
            kopyala(&giris.path(), &hedef.join(giris.file_name()))?;
        }
        Ok(())
    } else {
        fs::copy(kaynak, hedef).map(|_| ())
    }
}

fn sil(yol: &Path) -> std::io::Result<()> {
    let meta = fs::symlink_metadata(yol)?;
    if baglanti_mi(&meta) {
        return fs::remove_dir(yol).or_else(|_| fs::remove_file(yol));
    }
    if meta.is_dir() {
        fs::remove_dir_all(yol)
    } else {
        fs::remove_file(yol)
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use sp_core::kaldirici::UygulamaKaynagi;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct SahteCalistirici {
        cagrilar: Mutex<Vec<(String, Vec<String>)>>,
        kodlar: Mutex<VecDeque<i32>>,
    }

    impl SahteCalistirici {
        fn yeni(kodlar: &[i32]) -> Self {
            Self {
                cagrilar: Mutex::new(Vec::new()),
                kodlar: Mutex::new(kodlar.iter().copied().collect()),
            }
        }
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
            let kod = self.kodlar.lock().unwrap().pop_front().unwrap_or(0);
            Ok(KomutSonucu {
                komut: program.to_string(),
                kod,
                stdout: String::new(),
                stderr: "hata".to_string(),
            })
        }
    }

    fn servis(gecici: &tempfile::TempDir, kodlar: &[i32]) -> (ZorlaServisi, Arc<SahteCalistirici>) {
        let sahte = Arc::new(SahteCalistirici::yeni(kodlar));
        let servis = ZorlaServisi::yeni(sahte.clone(), gecici.path().join("karantina"));
        (servis, sahte)
    }

    fn uygulama(kurulum_konumu: Option<&str>) -> KuruluUygulama {
        KuruluUygulama {
            kimlik: "HKLM 64-bit|64|TestUygulama".to_string(),
            ad: "Test Uygulama".to_string(),
            surum: None,
            yayinci: None,
            kurulum_tarihi: None,
            boyut_kb: None,
            kurulum_konumu: kurulum_konumu.map(str::to_string),
            kaldirma_komutu: None,
            sessiz_kaldirma_komutu: None,
            kaynak: UygulamaKaynagi::Hklm64,
        }
    }

    const KAYIT: &str = r"HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall\TestUygulama";

    #[test]
    fn kayit_yedeklenir_ve_silinir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, sahte) = servis(&gecici, &[0, 0]);
        let karantina = gecici.path().join("zorla-1");
        fs::create_dir_all(&karantina).unwrap();
        let (silindi, yedek, hata) = servis.kayit_isle(&karantina, KAYIT, 64);
        assert!(silindi);
        assert!(hata.is_none());
        assert_eq!(
            yedek,
            Some(karantina.join("kayit.reg").to_string_lossy().to_string())
        );
        let cagrilar = sahte.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 2);
        assert_eq!(cagrilar[0].0, "reg.exe");
        assert_eq!(cagrilar[0].1[0], "export");
        assert_eq!(cagrilar[0].1[1], KAYIT);
        assert_eq!(
            cagrilar[0].1[2],
            karantina.join("kayit.reg").to_string_lossy().to_string()
        );
        assert_eq!(cagrilar[0].1[3], "/y");
        assert_eq!(cagrilar[0].1[4], "/reg:64");
        assert_eq!(cagrilar[1].1, vec!["delete", KAYIT, "/f", "/reg:64"]);
    }

    #[test]
    fn otuz_iki_bit_gorunum_bayragi_kullanilir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, sahte) = servis(&gecici, &[0, 0]);
        let karantina = gecici.path().join("zorla-1");
        fs::create_dir_all(&karantina).unwrap();
        let (silindi, _, hata) = servis.kayit_isle(&karantina, KAYIT, 32);
        assert!(silindi);
        assert!(hata.is_none());
        let cagrilar = sahte.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].1[4], "/reg:32");
        assert_eq!(cagrilar[1].1[3], "/reg:32");
    }

    #[test]
    fn yedek_basarisizsa_kayit_silinmez() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, sahte) = servis(&gecici, &[1]);
        let karantina = gecici.path().join("zorla-1");
        fs::create_dir_all(&karantina).unwrap();
        let (silindi, yedek, hata) = servis.kayit_isle(&karantina, KAYIT, 64);
        assert!(!silindi);
        assert!(yedek.is_none());
        assert!(hata.unwrap().contains("yedeklenemedi"));
        assert_eq!(sahte.cagrilar.lock().unwrap().len(), 1);
    }

    #[test]
    fn silme_basarisizsa_yedek_korunur() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, _) = servis(&gecici, &[0, 1]);
        let karantina = gecici.path().join("zorla-1");
        fs::create_dir_all(&karantina).unwrap();
        let (silindi, yedek, hata) = servis.kayit_isle(&karantina, KAYIT, 64);
        assert!(!silindi);
        assert!(yedek.is_some());
        assert!(hata.unwrap().contains("silinemedi"));
    }

    #[test]
    fn klasor_karantinaya_tasinir() {
        let gecici = tempfile::tempdir().unwrap();
        let kaynak = gecici.path().join("Uygulama");
        fs::create_dir_all(kaynak.join("alt")).unwrap();
        fs::write(kaynak.join("alt").join("veri.txt"), b"x").unwrap();
        let karantina = gecici.path().join("karantina");
        fs::create_dir_all(&karantina).unwrap();
        let (servis, _) = servis(&gecici, &[]);
        let metin = kaynak.to_string_lossy().to_string();
        let uygulama = uygulama(Some(&metin));
        let (tasindi, hata) = servis.klasor_tasi(&uygulama, &karantina);
        assert!(hata.is_none());
        let hedef = PathBuf::from(tasindi.unwrap());
        assert_eq!(hedef.file_name().unwrap().to_string_lossy(), "Uygulama");
        assert!(hedef.join("alt").join("veri.txt").exists());
        assert!(!kaynak.exists());
    }

    #[test]
    fn tehlikeli_ve_eksik_yollar_tasinmaz() {
        let gecici = tempfile::tempdir().unwrap();
        let karantina = gecici.path().join("karantina");
        fs::create_dir_all(&karantina).unwrap();
        let (servis, _) = servis(&gecici, &[]);
        let sistem = uygulama(Some(r"C:\Windows"));
        let (tasindi, hata) = servis.klasor_tasi(&sistem, &karantina);
        assert!(tasindi.is_none());
        assert!(hata.unwrap().contains("güvenli değil"));
        let eksik = uygulama(Some(r"C:\WinNexusTestYok\Uygulama"));
        assert_eq!(servis.klasor_tasi(&eksik, &karantina), (None, None));
    }

    #[test]
    fn gecersiz_kimlik_hicbir_islem_yapmaz() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, sahte) = servis(&gecici, &[]);
        let mut bozuk = uygulama(None);
        bozuk.kimlik = "bozuk".to_string();
        assert!(servis.zorla_kaldir(&bozuk).is_err());
        assert!(sahte.cagrilar.lock().unwrap().is_empty());
    }
}
