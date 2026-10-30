use sp_core::error::{Hata, Sonuc};
use sp_core::winget::{
    arama_tablosu_ayristir, guncelleme_tablosu_ayristir, ihracat_json_ayristir,
    kaynak_adi_gecerli_mi, kaynak_adresi_gecerli_mi, kaynak_tablosu_ayristir, liste_json_olustur,
    liste_metni_ayristir, paket_kimligi_dogrula, pin_tablosu_ayristir, AramaSonucu,
    GuncellemeSatiri, Kaynak, PinKaydi, WingetPaketi,
};
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const KISA_ZAMAN_ASIMI: Duration = Duration::from_secs(30);
const KAYNAK_ZAMAN_ASIMI: Duration = Duration::from_secs(60);
const TARAMA_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const ISLEM_ZAMAN_ASIMI: Duration = Duration::from_secs(1800);
const AZAMI_ICE_AKTARMA_BAYT: u64 = 1024 * 1024;
const AZAMI_ARAMA_ADEDI: u32 = 100;

fn ilk_satir(metin: &str) -> &str {
    metin
        .lines()
        .map(str::trim)
        .find(|satir| !satir.is_empty())
        .unwrap_or("")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KomutSonucu {
    pub komut: String,
    pub kod: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait KomutCalistirici: Send + Sync {
    fn calistir(
        &self,
        program: &str,
        argumanlar: &[String],
        zaman_asimi: Duration,
    ) -> Sonuc<KomutSonucu>;
}

pub struct WindowsKomutCalistirici;

impl KomutCalistirici for WindowsKomutCalistirici {
    fn calistir(
        &self,
        program: &str,
        argumanlar: &[String],
        zaman_asimi: Duration,
    ) -> Sonuc<KomutSonucu> {
        let komut = komut_metni(program, argumanlar);
        let mut komut_sureci = Command::new(program);
        if program.eq_ignore_ascii_case("cmd")
            && argumanlar.len() == 2
            && argumanlar[0].eq_ignore_ascii_case("/C")
        {
            komut_sureci.arg("/C").raw_arg(&argumanlar[1]);
        } else {
            komut_sureci.args(argumanlar);
        }
        let mut cocuk = komut_sureci
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|hata| Hata::DisArac(format!("{komut} başlatılamadı: {hata}")))?;

        let baslangic = Instant::now();
        loop {
            match cocuk.try_wait() {
                Ok(Some(durum)) => {
                    let mut stdout = Vec::new();
                    if let Some(mut kanal) = cocuk.stdout.take() {
                        kanal.read_to_end(&mut stdout).map_err(|hata| {
                            Hata::DisArac(format!("{komut} çıktısı okunamadı: {hata}"))
                        })?;
                    }
                    let mut stderr = Vec::new();
                    if let Some(mut kanal) = cocuk.stderr.take() {
                        kanal.read_to_end(&mut stderr).map_err(|hata| {
                            Hata::DisArac(format!("{komut} hatası okunamadı: {hata}"))
                        })?;
                    }
                    return Ok(KomutSonucu {
                        komut,
                        kod: durum.code().unwrap_or(-1),
                        stdout: metne_cevir(&stdout),
                        stderr: metne_cevir(&stderr),
                    });
                }
                Ok(None) => {}
                Err(hata) => {
                    return Err(Hata::DisArac(format!("{komut} beklenemedi: {hata}")));
                }
            }

            if baslangic.elapsed() > zaman_asimi {
                let _ = cocuk.kill();
                let _ = cocuk.wait();
                return Err(Hata::DisArac(format!("{komut} zaman aşımına uğradı.")));
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

fn komut_metni(program: &str, argumanlar: &[String]) -> String {
    let mut parcalar = Vec::with_capacity(argumanlar.len() + 1);
    parcalar.push(program.to_string());
    for arguman in argumanlar {
        if arguman.contains(' ') {
            parcalar.push(format!("\"{arguman}\""));
        } else {
            parcalar.push(arguman.clone());
        }
    }
    parcalar.join(" ")
}

fn metne_cevir(baytlar: &[u8]) -> String {
    match std::str::from_utf8(baytlar) {
        Ok(metin) => metin.to_string(),
        Err(_) => oem_metne_cevir(baytlar)
            .unwrap_or_else(|| encoding_rs::WINDOWS_1254.decode(baytlar).0.into_owned()),
    }
}

fn oem_metne_cevir(baytlar: &[u8]) -> Option<String> {
    use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP};
    if baytlar.is_empty() {
        return Some(String::new());
    }
    let uzunluk = unsafe {
        MultiByteToWideChar(
            CP_OEMCP,
            0,
            baytlar.as_ptr(),
            baytlar.len() as i32,
            std::ptr::null_mut(),
            0,
        )
    };
    if uzunluk <= 0 {
        return None;
    }
    let mut tampon = vec![0u16; uzunluk as usize];
    let yazilan = unsafe {
        MultiByteToWideChar(
            CP_OEMCP,
            0,
            baytlar.as_ptr(),
            baytlar.len() as i32,
            tampon.as_mut_ptr(),
            uzunluk,
        )
    };
    if yazilan <= 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&tampon[..yazilan as usize]))
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WingetAyarlari {
    pub lisans_onayi: bool,
    pub kaynak_onayi: bool,
}

#[derive(Clone)]
pub struct WingetServisi {
    calistirici: Arc<dyn KomutCalistirici>,
    ayarlar: WingetAyarlari,
    komutlar: Arc<Mutex<Vec<String>>>,
}

impl WingetServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>, ayarlar: WingetAyarlari) -> Self {
        Self {
            calistirici,
            ayarlar,
            komutlar: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn ayarlari_guncelle(&mut self, ayarlar: WingetAyarlari) {
        self.ayarlar = ayarlar;
    }

    pub fn komut_gecmisi(&self) -> Vec<String> {
        self.komutlar.lock().unwrap().clone()
    }

    fn argumanlar(&self, temel: &[&str], paket_onayi: bool) -> Vec<String> {
        let mut argumanlar = temel
            .iter()
            .map(|parca| parca.to_string())
            .collect::<Vec<_>>();
        if paket_onayi && self.ayarlar.lisans_onayi {
            argumanlar.push("--accept-package-agreements".to_string());
        }
        if self.ayarlar.kaynak_onayi {
            argumanlar.push("--accept-source-agreements".to_string());
        }
        argumanlar
    }

    fn calistir(&self, argumanlar: Vec<String>, zaman_asimi: Duration) -> Sonuc<KomutSonucu> {
        let komut = komut_metni("winget", &argumanlar);
        self.komutlar.lock().unwrap().push(komut);
        self.calistirici
            .calistir("winget", &argumanlar, zaman_asimi)
    }

    pub fn surum(&self) -> Sonuc<String> {
        let sonuc = self.calistir(vec!["--version".to_string()], KISA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget sürümü okunamadı: {}",
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc.stdout.trim().to_string())
    }

    pub fn kaynak_guncelle(&self) -> Sonuc<KomutSonucu> {
        let argumanlar = self.argumanlar(&["source", "update", "--disable-interactivity"], false);
        let sonuc = self.calistir(argumanlar, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget kaynakları güncellenemedi: {}",
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc)
    }

    pub fn guncellemeler(&self) -> Sonuc<Vec<GuncellemeSatiri>> {
        let sonuc = self.calistir(
            vec![
                "list".to_string(),
                "--upgrade-available".to_string(),
                "--include-unknown".to_string(),
                "--include-pinned".to_string(),
                "--disable-interactivity".to_string(),
            ],
            TARAMA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget güncellemeleri okunamadı: {}",
                sonuc.stderr.trim()
            )));
        }
        guncelleme_tablosu_ayristir(&sonuc.stdout)
    }

    pub fn pinler(&self) -> Sonuc<Vec<PinKaydi>> {
        let sonuc = self.calistir(
            vec![
                "pin".to_string(),
                "list".to_string(),
                "--disable-interactivity".to_string(),
            ],
            KISA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget pin listesi okunamadı: {}",
                sonuc.stderr.trim()
            )));
        }
        pin_tablosu_ayristir(&sonuc.stdout)
    }

    pub fn kurulu_paketler(&self) -> Sonuc<Vec<WingetPaketi>> {
        let dosya = std::env::temp_dir().join(format!(
            "winnexus-winget-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let dosya_metni = dosya.to_string_lossy().to_string();
        let argumanlar = self.argumanlar(
            &[
                "export",
                "-o",
                &dosya_metni,
                "--include-versions",
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, TARAMA_ZAMAN_ASIMI);
        let okuma = std::fs::read(&dosya);
        let _ = std::fs::remove_file(&dosya);
        let sonuc = sonuc?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget export başarısız: {}",
                sonuc.stderr.trim()
            )));
        }
        let baytlar = okuma
            .map_err(|hata| Hata::DisArac(format!("winget export dosyası okunamadı: {hata}")))?;
        ihracat_json_ayristir(&metne_cevir(&baytlar))
    }

    pub fn paket_var_mi(&self, id: &str) -> Sonuc<bool> {
        paket_kimligi_dogrula(id)?;
        let argumanlar = self.argumanlar(
            &["show", "--id", id, "--exact", "--disable-interactivity"],
            false,
        );
        let sonuc = self.calistir(argumanlar, TARAMA_ZAMAN_ASIMI)?;
        Ok(sonuc.kod == 0)
    }

    pub fn ara(&self, ad: &str) -> Sonuc<Vec<AramaSonucu>> {
        if ad.trim().is_empty() {
            return Ok(Vec::new());
        }
        let argumanlar = self.argumanlar(
            &[
                "search",
                "--name",
                ad,
                "--source",
                "winget",
                "--count",
                "10",
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget araması başarısız: {}",
                sonuc.stderr.trim()
            )));
        }
        arama_tablosu_ayristir(&sonuc.stdout)
    }

    pub fn kaynaklar(&self) -> Sonuc<Vec<Kaynak>> {
        let argumanlar = self.argumanlar(&["source", "list", "--disable-interactivity"], false);
        let sonuc = self.calistir(argumanlar, KISA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget kaynak listesi okunamadı (kod {}): {}",
                sonuc.kod,
                ilk_satir(&sonuc.stderr)
            )));
        }
        kaynak_tablosu_ayristir(&sonuc.stdout)
    }

    pub fn kaynak_ekle(&self, ad: &str, adres: &str) -> Sonuc<KomutSonucu> {
        if !kaynak_adi_gecerli_mi(ad) {
            return Err(Hata::Beklenmeyen(format!("Geçersiz kaynak adı: {ad}")));
        }
        if !kaynak_adresi_gecerli_mi(adres) {
            return Err(Hata::Beklenmeyen(format!(
                "Geçersiz kaynak adresi (https:// olmalı): {adres}"
            )));
        }
        let argumanlar = self.argumanlar(
            &[
                "source",
                "add",
                "--name",
                ad,
                "--arg",
                adres,
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, KAYNAK_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{ad} kaynağı eklenemedi (kod {}): {}",
                sonuc.kod,
                ilk_satir(&sonuc.stderr)
            )));
        }
        Ok(sonuc)
    }

    pub fn kaynak_kaldir(&self, ad: &str) -> Sonuc<KomutSonucu> {
        if !kaynak_adi_gecerli_mi(ad) {
            return Err(Hata::Beklenmeyen(format!("Geçersiz kaynak adı: {ad}")));
        }
        let argumanlar = self.argumanlar(
            &["source", "remove", "--name", ad, "--disable-interactivity"],
            false,
        );
        let sonuc = self.calistir(argumanlar, KAYNAK_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{ad} kaynağı kaldırılamadı (kod {}): {}",
                sonuc.kod,
                ilk_satir(&sonuc.stderr)
            )));
        }
        Ok(sonuc)
    }

    pub fn ara_kaynak(&self, ad: &str, kaynak: &str, adet: u32) -> Sonuc<Vec<AramaSonucu>> {
        if !kaynak_adi_gecerli_mi(kaynak) {
            return Err(Hata::Beklenmeyen(format!("Geçersiz kaynak adı: {kaynak}")));
        }
        if !(1..=AZAMI_ARAMA_ADEDI).contains(&adet) {
            return Err(Hata::Beklenmeyen(format!(
                "Arama sonuç adedi 1 ile {AZAMI_ARAMA_ADEDI} arasında olmalı."
            )));
        }
        let ad = ad.trim();
        if ad.starts_with('-') || ad.chars().any(char::is_control) {
            return Err(Hata::Beklenmeyen("Geçersiz arama sorgusu.".into()));
        }
        let adet_metni = adet.to_string();
        let mut temel = vec!["search"];
        // Boş sorguda --name eklenmez; winget tüm kaynağı listeler.
        if !ad.is_empty() {
            temel.push("--name");
            temel.push(ad);
        }
        temel.extend([
            "--source",
            kaynak,
            "--count",
            &adet_metni,
            "--disable-interactivity",
        ]);
        let argumanlar = self.argumanlar(&temel, false);
        let sonuc = self.calistir(argumanlar, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "winget araması başarısız (kod {}): {}",
                sonuc.kod,
                ilk_satir(&sonuc.stderr)
            )));
        }
        let mut sonuclar = arama_tablosu_ayristir(&sonuc.stdout)?;
        for sonuc in &mut sonuclar {
            if sonuc.kaynak.is_none() {
                sonuc.kaynak = Some(kaynak.to_string());
            }
        }
        Ok(sonuclar)
    }

    pub fn kaldir(&self, id: &str) -> Sonuc<KomutSonucu> {
        paket_kimligi_dogrula(id)?;
        let argumanlar = self.argumanlar(
            &[
                "uninstall",
                "--id",
                id,
                "--exact",
                "--silent",
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, ISLEM_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{id} kaldırılamadı (kod {}): {}",
                sonuc.kod,
                ilk_satir(&sonuc.stderr)
            )));
        }
        Ok(sonuc)
    }

    pub fn ice_aktar(&self, dosya: &Path) -> Sonuc<Vec<WingetPaketi>> {
        let ustveri = std::fs::metadata(dosya)
            .map_err(|hata| Hata::Beklenmeyen(format!("Liste dosyası okunamadı: {hata}")))?;
        if !ustveri.is_file() {
            return Err(Hata::Beklenmeyen(format!(
                "Liste yolu bir dosya değil: {}",
                dosya.display()
            )));
        }
        if ustveri.len() > AZAMI_ICE_AKTARMA_BAYT {
            return Err(Hata::Beklenmeyen(
                "Liste dosyası 1 MiB sınırını aşıyor.".into(),
            ));
        }
        let baytlar = std::fs::read(dosya)
            .map_err(|hata| Hata::Beklenmeyen(format!("Liste dosyası okunamadı: {hata}")))?;
        let metin = metne_cevir(&baytlar);
        let metin = metin.trim_start_matches('\u{feff}');
        if metin.trim_start().starts_with('{') {
            return ihracat_json_ayristir(metin);
        }
        let idler = liste_metni_ayristir(metin)?;
        Ok(idler
            .into_iter()
            .map(|id| WingetPaketi {
                id,
                ad: None,
                kaynak: Some("winget".into()),
                surum: None,
            })
            .collect())
    }

    pub fn disa_aktar(&self, idler: &[String], dosya: &Path) -> Sonuc<()> {
        let json = liste_json_olustur(idler)?;
        if dosya.is_dir() {
            return Err(Hata::Beklenmeyen(format!(
                "Hedef yol bir dizin: {}",
                dosya.display()
            )));
        }
        if let Some(ust) = dosya.parent() {
            if !ust.as_os_str().is_empty() {
                std::fs::create_dir_all(ust).map_err(|hata| {
                    Hata::Beklenmeyen(format!("Hedef dizin oluşturulamadı: {hata}"))
                })?;
            }
        }
        std::fs::write(dosya, json)
            .map_err(|hata| Hata::Beklenmeyen(format!("Liste dosyası yazılamadı: {hata}")))
    }

    pub fn guncelle(&self, id: &str) -> Sonuc<KomutSonucu> {
        paket_kimligi_dogrula(id)?;
        if !self.ayarlar.lisans_onayi {
            return Err(Hata::Yetki(
                "Paket lisans sözleşmesi onayı kapalı. Winget ayarlarından açın.".into(),
            ));
        }
        let argumanlar = self.argumanlar(
            &[
                "upgrade",
                "--id",
                id,
                "--exact",
                "--silent",
                "--disable-interactivity",
            ],
            true,
        );
        let sonuc = self.calistir(argumanlar, ISLEM_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{} güncellenemedi (kod {}): {}",
                id,
                sonuc.kod,
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc)
    }

    pub fn kur(&self, id: &str) -> Sonuc<KomutSonucu> {
        paket_kimligi_dogrula(id)?;
        if !self.ayarlar.lisans_onayi {
            return Err(Hata::Yetki(
                "Paket lisans sözleşmesi onayı kapalı. Winget ayarlarından açın.".into(),
            ));
        }
        let argumanlar = self.argumanlar(
            &[
                "install",
                "--id",
                id,
                "--exact",
                "--silent",
                "--disable-interactivity",
            ],
            true,
        );
        let sonuc = self.calistir(argumanlar, ISLEM_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{} kurulamadı (kod {}): {}",
                id,
                sonuc.kod,
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc)
    }

    pub fn asla_guncelle(&self, id: &str) -> Sonuc<KomutSonucu> {
        paket_kimligi_dogrula(id)?;
        let argumanlar = self.argumanlar(
            &[
                "pin",
                "add",
                "--id",
                id,
                "--exact",
                "--blocking",
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, KISA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{} için blocking pin eklenemedi: {}",
                id,
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc)
    }

    pub fn pin_kaldir(&self, id: &str) -> Sonuc<KomutSonucu> {
        paket_kimligi_dogrula(id)?;
        let argumanlar = self.argumanlar(
            &[
                "pin",
                "remove",
                "--id",
                id,
                "--exact",
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, KISA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{} pin kaydı kaldırılamadı: {}",
                id,
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc)
    }

    pub fn surum_sabitle(&self, id: &str, surum: &str) -> Sonuc<KomutSonucu> {
        paket_kimligi_dogrula(id)?;
        if surum.trim().is_empty()
            || surum.starts_with('-')
            || surum.chars().any(char::is_whitespace)
        {
            return Err(Hata::Beklenmeyen("Geçersiz sürüm değeri.".into()));
        }
        let argumanlar = self.argumanlar(
            &[
                "pin",
                "add",
                "--id",
                id,
                "--exact",
                "--version",
                surum,
                "--disable-interactivity",
            ],
            false,
        );
        let sonuc = self.calistir(argumanlar, KISA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{} sürümü sabitlenemedi: {}",
                id,
                sonuc.stderr.trim()
            )));
        }
        Ok(sonuc)
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn oem_turkce_cikti_cozulur() {
        let baytlar = [
            0x98, 0x73, 0x74, 0x65, 0x6E, 0x65, 0x6E, 0x20, 0x69, 0x9F, 0x6C, 0x65, 0x6D, 0x20,
            0x69, 0x87, 0x69, 0x6E, 0x20, 0x79, 0x81, 0x6B, 0x73, 0x65, 0x6C, 0x74, 0x6D, 0x65,
            0x20, 0x67, 0x65, 0x72, 0x65, 0x6B, 0x69, 0x79, 0x6F, 0x72, 0x2E,
        ];
        assert_eq!(
            metne_cevir(&baytlar),
            "İstenen işlem için yükseltme gerekiyor."
        );
    }

    struct SahteCalistirici {
        cevap: KomutSonucu,
        cagrilar: Mutex<Vec<Vec<String>>>,
    }

    impl KomutCalistirici for SahteCalistirici {
        fn calistir(
            &self,
            _program: &str,
            argumanlar: &[String],
            _zaman_asimi: Duration,
        ) -> Sonuc<KomutSonucu> {
            self.cagrilar.lock().unwrap().push(argumanlar.to_vec());
            Ok(self.cevap.clone())
        }
    }

    fn servis(stdout: &str, lisans: bool, kaynak: bool) -> (WingetServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            cevap: KomutSonucu {
                komut: "winget".into(),
                kod: 0,
                stdout: stdout.into(),
                stderr: String::new(),
            },
            cagrilar: Mutex::new(Vec::new()),
        });
        (
            WingetServisi::yeni(
                calistirici.clone(),
                WingetAyarlari {
                    lisans_onayi: lisans,
                    kaynak_onayi: kaynak,
                },
            ),
            calistirici,
        )
    }

    #[test]
    fn guncellemeleri_ayristirir() {
        let metin = "Name                  Id                     Version   Available   Source\nBlender               BlenderFoundation.Blender 5.1.2   5.2.3   winget\n";
        let (servis, calistirici) = servis(metin, false, false);
        let sonuc = servis.guncellemeler().unwrap();
        assert_eq!(sonuc.len(), 1);
        assert_eq!(sonuc[0].id, "BlenderFoundation.Blender");
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(cagri.contains(&"--upgrade-available".to_string()));
        assert!(cagri.contains(&"--include-pinned".to_string()));
    }

    #[test]
    fn lisans_onayi_yoksa_guncellemez() {
        let (servis, calistirici) = servis("", false, true);
        assert!(servis.guncelle("Git.Git").is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn guncellemede_onay_bayraklari() {
        let (servis, calistirici) = servis("", true, true);
        servis.guncelle("Git.Git").unwrap();
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(cagri.contains(&"--accept-package-agreements".to_string()));
        assert!(cagri.contains(&"--accept-source-agreements".to_string()));
        assert!(cagri.contains(&"--silent".to_string()));
    }

    #[test]
    fn asla_guncelle_blocking_pin_ekler() {
        let (servis, calistirici) = servis("", false, true);
        servis.asla_guncelle("Git.Git").unwrap();
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(cagri.contains(&"pin".to_string()));
        assert!(cagri.contains(&"--blocking".to_string()));
    }

    #[test]
    fn gecersiz_kimlik_reddedilir() {
        let (servis, calistirici) = servis("", true, true);
        assert!(servis.guncelle("kötü paket").is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn komut_gecmisi_tutar() {
        let (servis, _calistirici) = servis("", false, false);
        let _ = servis.surum();
        let gecmis = servis.komut_gecmisi();
        assert_eq!(gecmis.len(), 1);
        assert!(gecmis[0].starts_with("winget --version"));
    }

    #[test]
    fn kaynak_listesini_okur() {
        let metin = "Name        Argument                                      Explicit\n------------------------------------------------------------------\nmsstore     https://storeedgefd.dsx.mp.microsoft.com/v9.0 false\nwinget      https://cdn.winget.microsoft.com/cache        false\n";
        let (servis, calistirici) = servis(metin, false, false);
        let kaynaklar = servis.kaynaklar().unwrap();
        assert_eq!(kaynaklar.len(), 2);
        assert_eq!(kaynaklar[1].ad, "winget");
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert_eq!(cagri, &["source", "list", "--disable-interactivity"]);
    }

    #[test]
    fn kaynak_ekle_argumanlari() {
        let (servis, calistirici) = servis("", false, true);
        servis
            .kaynak_ekle("test-kaynak", "https://ornek.test/api")
            .unwrap();
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(cagri.contains(&"source".to_string()));
        assert!(cagri.contains(&"add".to_string()));
        assert!(cagri.contains(&"--name".to_string()));
        assert!(cagri.contains(&"test-kaynak".to_string()));
        assert!(cagri.contains(&"--arg".to_string()));
        assert!(cagri.contains(&"https://ornek.test/api".to_string()));
        assert!(cagri.contains(&"--accept-source-agreements".to_string()));
        assert!(!cagri.contains(&"--accept-package-agreements".to_string()));
    }

    #[test]
    fn gecersiz_kaynak_reddedilir() {
        let (servis, calistirici) = servis("", true, true);
        assert!(servis.kaynak_ekle("kötü ad", "https://ornek.test").is_err());
        assert!(servis.kaynak_ekle("ad", "http://ornek.test").is_err());
        assert!(servis.kaynak_kaldir("kötü ad").is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn kaynak_kaldirma_argumanlari() {
        let (servis, calistirici) = servis("", false, false);
        servis.kaynak_kaldir("winget").unwrap();
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert_eq!(
            cagri,
            &[
                "source",
                "remove",
                "--name",
                "winget",
                "--disable-interactivity"
            ]
        );
    }

    #[test]
    fn kaynaktan_arama_yapar() {
        let metin = "Name      Id             Version\n-----------------------------------\nGit       Git.Git        2.56.0.3\n";
        let (servis, calistirici) = servis(metin, false, false);
        let sonuclar = servis.ara_kaynak("git", "winget", 5).unwrap();
        assert_eq!(sonuclar.len(), 1);
        assert_eq!(sonuclar[0].id, "Git.Git");
        assert_eq!(sonuclar[0].kaynak.as_deref(), Some("winget"));
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(cagri.contains(&"--name".to_string()));
        assert!(cagri.contains(&"git".to_string()));
        assert!(cagri.contains(&"--source".to_string()));
        assert!(cagri.contains(&"--count".to_string()));
        assert!(cagri.contains(&"5".to_string()));
    }

    #[test]
    fn bos_sorguda_adi_atlar_ve_sinirlari_dogrular() {
        let (servis, calistirici) = servis("", false, false);
        servis.ara_kaynak("", "winget", 10).unwrap();
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(!cagri.contains(&"--name".to_string()));
        assert!(servis.ara_kaynak("git", "winget", 0).is_err());
        assert!(servis.ara_kaynak("git", "winget", 101).is_err());
        assert!(servis.ara_kaynak("git", "kötü ad", 5).is_err());
        assert!(servis.ara_kaynak("-tehlikeli", "winget", 5).is_err());
    }

    #[test]
    fn kaldirma_argumanlari() {
        let (servis, calistirici) = servis("", false, true);
        servis.kaldir("Git.Git").unwrap();
        let cagri = &calistirici.cagrilar.lock().unwrap()[0];
        assert!(cagri.contains(&"uninstall".to_string()));
        assert!(cagri.contains(&"Git.Git".to_string()));
        assert!(cagri.contains(&"--exact".to_string()));
        assert!(cagri.contains(&"--silent".to_string()));
        assert!(cagri.contains(&"--accept-source-agreements".to_string()));
        assert!(!cagri.contains(&"--accept-package-agreements".to_string()));
    }

    #[test]
    fn kaldirma_lisans_onayi_istemez() {
        let (servis, _calistirici) = servis("", false, false);
        assert!(servis.kaldir("Git.Git").is_ok());
    }

    #[test]
    fn liste_dosyasindan_ice_aktarir() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("liste.txt");
        std::fs::write(
            &dosya,
            "# yorum\nGit.Git\nMicrosoft.VCRedist.2015+.x64\nGit.Git\n",
        )
        .unwrap();
        let (servis, calistirici) = servis("", false, false);
        let paketler = servis.ice_aktar(&dosya).unwrap();
        assert_eq!(paketler.len(), 2);
        assert_eq!(paketler[0].id, "Git.Git");
        assert_eq!(paketler[0].kaynak.as_deref(), Some("winget"));
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn json_dosyasindan_ice_aktarir() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("liste.json");
        std::fs::write(
            &dosya,
            r#"{"Sources":[{"Packages":[{"PackageIdentifier":"Git.Git"}],"SourceDetails":{"Name":"winget"}}]}"#,
        )
        .unwrap();
        let (servis, _calistirici) = servis("", false, false);
        let paketler = servis.ice_aktar(&dosya).unwrap();
        assert_eq!(paketler.len(), 1);
        assert_eq!(paketler[0].id, "Git.Git");
    }

    #[test]
    fn ice_aktarma_hatalarini_reddeder() {
        let dizin = tempfile::tempdir().unwrap();
        let (servis, _calistirici) = servis("", false, false);
        assert!(servis.ice_aktar(&dizin.path().join("yok.txt")).is_err());
        assert!(servis.ice_aktar(dizin.path()).is_err());
        let buyuk = dizin.path().join("buyuk.txt");
        std::fs::write(&buyuk, vec![b'a'; 1024 * 1024 + 1]).unwrap();
        assert!(servis.ice_aktar(&buyuk).is_err());
        let bozuk = dizin.path().join("bozuk.txt");
        std::fs::write(&bozuk, "Git Git.Git 2.56.0 winget\n").unwrap();
        assert!(servis.ice_aktar(&bozuk).is_err());
    }

    #[test]
    fn listeyi_disa_aktarir_ve_geri_okur() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("alt").join("liste.json");
        let (servis, _calistirici) = servis("", false, false);
        let idler = vec!["Git.Git".to_string(), "Git.Git".to_string()];
        servis.disa_aktar(&idler, &dosya).unwrap();
        let metin = std::fs::read_to_string(&dosya).unwrap();
        let paketler = ihracat_json_ayristir(&metin).unwrap();
        assert_eq!(paketler.len(), 1);
        assert_eq!(paketler[0].id, "Git.Git");
    }

    #[test]
    fn disa_aktarma_hatalarini_reddeder() {
        let dizin = tempfile::tempdir().unwrap();
        let (servis, _calistirici) = servis("", false, false);
        assert!(servis
            .disa_aktar(&[], &dizin.path().join("x.json"))
            .is_err());
        assert!(servis
            .disa_aktar(&["kötü paket".to_string()], &dizin.path().join("x.json"))
            .is_err());
        assert!(servis
            .disa_aktar(&["Git.Git".to_string()], dizin.path())
            .is_err());
    }
}
