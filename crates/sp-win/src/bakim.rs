use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::bakim::{
    appxleri_ayristir, ozellikleri_ayristir, suruculeri_ayristir, AppxPaketi, OzellikOgesi,
    SurucuOgesi,
};
use sp_core::error::{Hata, Sonuc};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

const BAKIM_ZAMAN_ASIMI: Duration = Duration::from_secs(1800);
const CIKTI_SINIRI: usize = 8000;
const DISM_PROGRAMI: &str = "Dism.exe";
const POWERSHELL_PROGRAMI: &str = "powershell";
const PNPUTIL_PROGRAMI: &str = "pnputil";
const APPX_BETIGI: &str = "$ErrorActionPreference='Stop'; Get-AppxPackage | Select-Object @{N='Ad';E={$_.Name}},@{N='TamAd';E={$_.PackageFullName}},@{N='Surum';E={$_.Version.ToString()}},@{N='Yayinci';E={$_.Publisher}} | ConvertTo-Json -Compress";
const OZELLIK_BETIGI: &str = "$ErrorActionPreference='Stop'; Get-WindowsOptionalFeature -Online | Select-Object @{N='Ad';E={$_.FeatureName}},@{N='Durum';E={$_.State.ToString()}} | ConvertTo-Json -Compress";
const SURUCU_BETIGI: &str = "$ErrorActionPreference='Stop'; Get-WindowsDriver -Online | Select-Object @{N='Ad';E={$_.Driver}},@{N='Yayinci';E={$_.ProviderName}},@{N='Sinif';E={$_.ClassName}},@{N='Surum';E={$_.Version.ToString()}},@{N='Tarih';E={$_.Date.ToString('yyyy-MM-dd')}},@{N='Inf';E={$_.Driver}} | ConvertTo-Json -Compress";

#[derive(Clone)]
pub struct BakimServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl BakimServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    pub fn dism_analiz(&self) -> Sonuc<String> {
        self.dism_calistir(
            &["/Online", "/Cleanup-Image", "/AnalyzeComponentStore"],
            "Bileşen deposu analizi",
        )
    }

    pub fn dism_temizle(&self) -> Sonuc<String> {
        self.dism_calistir(
            &["/Online", "/Cleanup-Image", "/StartComponentCleanup"],
            "Bileşen deposu temizliği",
        )
    }

    pub fn dism_saglik_tara(&self) -> Sonuc<String> {
        self.dism_calistir(
            &["/Online", "/Cleanup-Image", "/ScanHealth"],
            "Bileşen deposu sağlık taraması",
        )
    }

    pub fn dism_onar(&self) -> Sonuc<String> {
        self.dism_calistir(
            &["/Online", "/Cleanup-Image", "/RestoreHealth"],
            "Bileşen deposu onarımı",
        )
    }

    pub fn appx_listele(&self) -> Sonuc<Vec<AppxPaketi>> {
        let sonuc = self.powershell_calistir(APPX_BETIGI, "Appx paketleri listelenemedi")?;
        appxleri_ayristir(&sonuc.stdout)
    }

    pub fn appx_kaldir(&self, tam_ad: &str) -> Sonuc<()> {
        let tam_ad = tam_ad.trim();
        if !appx_tam_ad_gecerli_mi(tam_ad) {
            return Err(Hata::Dogrulama("Geçersiz Appx paket tam adı.".into()));
        }
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ Remove-AppxPackage -Package '{tam_ad}'; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        );
        self.powershell_calistir(&betik, "Appx paketi kaldırılamadı")?;
        Ok(())
    }

    pub fn ozellikler(&self) -> Sonuc<Vec<OzellikOgesi>> {
        let sonuc =
            self.powershell_calistir(OZELLIK_BETIGI, "Windows özellikleri listelenemedi")?;
        ozellikleri_ayristir(&sonuc.stdout)
    }

    pub fn ozellik_ayarla(&self, ad: &str, etkin: bool) -> Sonuc<String> {
        let ad = ad.trim();
        if !ozellik_adi_gecerli_mi(ad) {
            return Err(Hata::Dogrulama("Geçersiz Windows özelliği adı.".into()));
        }
        let eylem = if etkin {
            "Enable-WindowsOptionalFeature"
        } else {
            "Disable-WindowsOptionalFeature"
        };
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ {eylem} -Online -FeatureName '{ad}' -NoRestart | Out-String -Width 4096; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        );
        let sonuc = self.powershell_calistir(&betik, "Windows özelliği ayarlanamadı")?;
        Ok(cikti_metni(&sonuc))
    }

    pub fn suruculer(&self) -> Sonuc<Vec<SurucuOgesi>> {
        let sonuc = self.powershell_calistir(SURUCU_BETIGI, "Sürücüler listelenemedi")?;
        suruculeri_ayristir(&sonuc.stdout)
    }

    pub fn surucu_disa_aktar(&self, inf: &str, hedef: &Path) -> Sonuc<()> {
        let inf = inf.trim();
        if !surucu_inf_gecerli_mi(inf) {
            return Err(Hata::Dogrulama("Geçersiz sürücü INF adı.".into()));
        }
        if hedef.as_os_str().is_empty() {
            return Err(Hata::Dogrulama("Sürücü hedef dizini boş olamaz.".into()));
        }
        std::fs::create_dir_all(hedef)
            .map_err(|hata| Hata::DisArac(format!("Sürücü hedef dizini oluşturulamadı: {hata}")))?;
        let sonuc = self.calistirici.calistir(
            PNPUTIL_PROGRAMI,
            &[
                "/export-driver".to_string(),
                inf.to_string(),
                hedef.to_string_lossy().to_string(),
            ],
            BAKIM_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Sürücü dışa aktarılamadı (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(())
    }

    fn dism_calistir(&self, argumanlar: &[&str], baslik: &str) -> Sonuc<String> {
        let argumanlar = argumanlar
            .iter()
            .map(|parca| parca.to_string())
            .collect::<Vec<_>>();
        let sonuc = self
            .calistirici
            .calistir(DISM_PROGRAMI, &argumanlar, BAKIM_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{baslik} başarısız (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(cikti_metni(&sonuc))
    }

    fn powershell_calistir(&self, betik: &str, baslik: &str) -> Sonuc<KomutSonucu> {
        let sonuc = self.calistirici.calistir(
            POWERSHELL_PROGRAMI,
            &[
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                betik.to_string(),
            ],
            BAKIM_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{baslik} (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(sonuc)
    }
}

fn appx_tam_ad_gecerli_mi(tam_ad: &str) -> bool {
    !tam_ad.is_empty()
        && tam_ad.len() <= 256
        && tam_ad.chars().all(|karakter| {
            karakter.is_ascii_alphanumeric() || matches!(karakter, '.' | '_' | '-' | '+')
        })
}

fn ozellik_adi_gecerli_mi(ad: &str) -> bool {
    !ad.is_empty()
        && ad.len() <= 128
        && ad
            .chars()
            .all(|karakter| karakter.is_ascii_alphanumeric() || matches!(karakter, '.' | '_' | '-'))
}

fn surucu_inf_gecerli_mi(inf: &str) -> bool {
    if inf.len() <= 4 || !inf.is_ascii() {
        return false;
    }
    let (govde, uzanti) = inf.split_at(inf.len() - 4);
    uzanti.eq_ignore_ascii_case(".inf")
        && !govde.is_empty()
        && govde
            .chars()
            .all(|karakter| karakter.is_ascii_alphanumeric() || matches!(karakter, '.' | '_' | '-'))
}

fn cikti_metni(sonuc: &KomutSonucu) -> String {
    let mut metin = sonuc.stdout.trim().to_string();
    let hata = sonuc.stderr.trim();
    if !hata.is_empty() {
        if !metin.is_empty() {
            metin.push('\n');
        }
        metin.push_str(hata);
    }
    metin.chars().take(CIKTI_SINIRI).collect()
}

fn hata_ozeti(sonuc: &KomutSonucu) -> String {
    let metin = if sonuc.stderr.trim().is_empty() {
        sonuc.stdout.trim()
    } else {
        sonuc.stderr.trim()
    };
    let ilk = metin.lines().map(str::trim).find(|satir| !satir.is_empty());
    let Some(ilk) = ilk else {
        return "çıktı yok".to_string();
    };
    let kucuk = ilk.to_lowercase();
    if kucuk.contains("yükseltme")
        || kucuk.contains("elevation")
        || kucuk.contains("access is denied")
        || kucuk.contains("erişim engellendi")
        || kucuk.contains("0x80070005")
    {
        return "Bu işlem için yönetici izni gerekiyor.".to_string();
    }
    ilk.chars().take(300).collect()
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::sync::Mutex;

    struct SahteCalistirici {
        kod: i32,
        stdout: String,
        stderr: String,
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
                kod: self.kod,
                stdout: self.stdout.clone(),
                stderr: self.stderr.clone(),
            })
        }
    }

    fn servis(kod: i32, stdout: &str) -> (BakimServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            kod,
            stdout: stdout.to_string(),
            stderr: "hata ayrıntısı".to_string(),
            cagrilar: Mutex::new(Vec::new()),
        });
        (BakimServisi::yeni(calistirici.clone()), calistirici)
    }

    #[test]
    fn ozellikler_yetki_hatasi_ham_yigin_icermez() {
        let servis = BakimServisi::yeni(Arc::new(crate::winget::WindowsKomutCalistirici));
        if let Err(hata) = servis.ozellikler() {
            let metin = hata.to_string();
            assert!(!metin.contains("At line:"), "{metin}");
            assert!(!metin.contains("CategoryInfo"), "{metin}");
        }
    }

    #[test]
    fn yukseltme_hatasi_ozetlenir() {
        let sonuc = KomutSonucu {
            komut: "powershell".to_string(),
            kod: 1,
            stdout: String::new(),
            stderr: "Get-WindowsOptionalFeature : İstenen işlem için yükseltme gerekiyor.\nAt line:1 char:32\n+ CategoryInfo : NotSpecified".to_string(),
        };
        assert_eq!(hata_ozeti(&sonuc), "Bu işlem için yönetici izni gerekiyor.");
    }

    #[test]
    fn dism_analizi_dogru_argumanlarla_calisir() {
        let (servis, calistirici) = servis(0, "analiz tamam");
        let cikti = servis.dism_analiz().unwrap();
        assert_eq!(cikti, "analiz tamam\nhata ayrıntısı");
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "Dism.exe");
        assert_eq!(
            cagrilar[0].1,
            vec!["/Online", "/Cleanup-Image", "/AnalyzeComponentStore"]
        );
    }

    #[test]
    fn dism_hatali_kod_turkce_hata_dondurur() {
        let (servis, _calistirici) = servis(87, "");
        let hata = servis.dism_onar().unwrap_err();
        assert!(hata.to_string().contains("onarımı başarısız"));
    }

    #[test]
    fn appx_listesi_tek_nesneden_ayristirilir() {
        let (servis, calistirici) = servis(
            0,
            r#"{"Ad":"A","TamAd":"A_1.0_x64__8wekyb3d8bbwe","Surum":"1.0","Yayinci":"X"}"#,
        );
        let paketler = servis.appx_listele().unwrap();
        assert_eq!(paketler.len(), 1);
        assert_eq!(paketler[0].surum, "1.0");
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "powershell");
        assert!(cagrilar[0].1.last().unwrap().contains("Get-AppxPackage"));
    }

    #[test]
    fn gecersiz_appx_tam_adi_komut_calistirmaz() {
        let (servis, calistirici) = servis(0, "");
        assert!(servis.appx_kaldir("A_1.0'; calc; '").is_err());
        assert!(servis.appx_kaldir("A_1.0 x64").is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
        assert!(servis.appx_kaldir("A_1.0_x64__8wekyb3d8bbwe").is_ok());
    }

    #[test]
    fn ozellik_ayari_acma_ve_kapama_betigi_uretir() {
        let (servis, calistirici) = servis(0, "işlem tamam");
        servis
            .ozellik_ayarla("Microsoft-Windows-Subsystem-Linux", true)
            .unwrap();
        servis
            .ozellik_ayarla("Microsoft-Windows-Subsystem-Linux", false)
            .unwrap();
        {
            let cagrilar = calistirici.cagrilar.lock().unwrap();
            assert!(cagrilar[0]
                .1
                .last()
                .unwrap()
                .contains("Enable-WindowsOptionalFeature"));
            assert!(cagrilar[1]
                .1
                .last()
                .unwrap()
                .contains("Disable-WindowsOptionalFeature"));
        }
        assert!(servis.ozellik_ayarla("kötü ad; rm", true).is_err());
    }

    #[test]
    fn gecersiz_inf_reddedilir() {
        let (servis, calistirici) = servis(0, "");
        let hedef = std::env::temp_dir();
        assert!(servis.surucu_disa_aktar("../oem1.inf", &hedef).is_err());
        assert!(servis.surucu_disa_aktar("oem1.sys", &hedef).is_err());
        assert!(servis.surucu_disa_aktar("oem1.inf /y", &hedef).is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn surucu_disa_aktarma_hedef_dizini_olusturur() {
        let (servis, calistirici) = servis(0, "");
        let gecici = tempfile::tempdir().unwrap();
        let hedef = gecici.path().join("surucu-yedek");
        servis.surucu_disa_aktar("oem12.inf", &hedef).unwrap();
        assert!(hedef.is_dir());
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "pnputil");
        assert_eq!(cagrilar[0].1[0], "/export-driver");
        assert_eq!(cagrilar[0].1[1], "oem12.inf");
    }

    #[test]
    fn uzantilar_dogru_dogrulanir() {
        assert!(surucu_inf_gecerli_mi("OEM12.INF"));
        assert!(appx_tam_ad_gecerli_mi("Name_1.0.0.0_x64__8wekyb3d8bbwe"));
        assert!(!ozellik_adi_gecerli_mi(""));
    }
}
