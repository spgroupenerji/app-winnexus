use crate::winget::{KomutCalistirici, KomutSonucu};
use adb_client::server::ADBServer;
use adb_client::usb::ADBUSBDevice;
use adb_client::ADBDeviceExt;
use sp_core::adb::seri_gecerli;
use sp_core::android::{
    cihaz_profilini_zenginlestir, cihazlari_ayristir, eylem_guvenli_mi, getprop_ayristir,
    paket_durumlari, paket_komut_argumanlari, paketleri_ayristir, uad_ayristir, AndroidCihaz,
    BaglantiDurumu, PaketDurumu, PaketEylemi, Risk, UadVeritabani, PLATFORM_TOOLS_URL, UAD_LISANS,
    UAD_URL,
};
use sp_core::error::{Hata, Sonuc};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ADB_PORTU: u16 = 5037;
const KISA_ZAMAN_ASIMI: Duration = Duration::from_secs(30);
const TARAMA_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const INDIRME_ZAMAN_ASIMI: Duration = Duration::from_secs(300);
const AZAMI_UAD_BAYT: u64 = 64 * 1024 * 1024;
const AZAMI_KABUK_KOMUTU: usize = 4096;
const PLATFORM_TOOLS_KLASORU: &str = "platform-tools";
const ADB_DOSYASI: &str = "adb.exe";
const UAD_DOSYASI: &str = "uad_lists.json";
const UAD_GECICI_DOSYASI: &str = "uad_lists.json.gecici";

const INDIRME_BETIGI: &str = r#"param(
    [string]$Uri,
    [string]$Dosya,
    [string]$Hedef
)
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Dosya
if ($Hedef -ne '') {
    Expand-Archive -LiteralPath $Dosya -DestinationPath $Hedef -Force
}
"#;

#[derive(Clone)]
pub struct AndroidServisi {
    calistirici: Arc<dyn KomutCalistirici>,
    kok: PathBuf,
}

#[derive(Debug, Clone)]
pub struct UadDurumu {
    pub veritabani: UadVeritabani,
    pub dosya: PathBuf,
    pub guncelleme: String,
    pub lisans: String,
}

impl AndroidServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>, kok: PathBuf) -> Self {
        Self { calistirici, kok }
    }

    pub fn adb_yolu(&self) -> Option<PathBuf> {
        let yerel = self.kok.join(PLATFORM_TOOLS_KLASORU).join(ADB_DOSYASI);
        if yerel.is_file() {
            return Some(yerel);
        }
        if let Some(sdk_koku) = std::env::var_os("LOCALAPPDATA") {
            let sdk = PathBuf::from(sdk_koku)
                .join("Android")
                .join("Sdk")
                .join(PLATFORM_TOOLS_KLASORU)
                .join(ADB_DOSYASI);
            if sdk.is_file() {
                return Some(sdk);
            }
        }
        path_te_bul(ADB_DOSYASI).or_else(|| path_te_bul("adb"))
    }

    pub fn platform_tools_indir(&self) -> Sonuc<PathBuf> {
        std::fs::create_dir_all(&self.kok).map_err(|hata| {
            Hata::Beklenmeyen(format!("Platform Tools dizini oluşturulamadı: {hata}"))
        })?;
        let arsiv = gecici_dosya("platform-tools", "zip");
        // Arşiv kendi platform-tools klasörünü taşıdığı için kök dizine açılır.
        let indirme = self.indir(PLATFORM_TOOLS_URL, &arsiv, Some(&self.kok));
        let _ = std::fs::remove_file(&arsiv);
        indirme?;
        let adb = self.kok.join(PLATFORM_TOOLS_KLASORU).join(ADB_DOSYASI);
        if adb.is_file() {
            return Ok(adb);
        }
        Err(Hata::DisArac(format!(
            "Platform Tools arşivinde adb.exe bulunamadı: {}",
            adb.display()
        )))
    }

    pub fn cihazlari_tara(&self, adb: &Path) -> Sonuc<Vec<AndroidCihaz>> {
        let sonuc = self.calistir(
            adb,
            &["devices".to_string(), "-l".to_string()],
            KISA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "adb devices başarısız (kod {}): {}",
                sonuc.kod,
                arac_hatasi(&sonuc)
            )));
        }
        let mut cihazlar = cihazlari_ayristir(&sonuc.stdout);
        for cihaz in &mut cihazlar {
            if cihaz.durum != BaglantiDurumu::Bagli {
                continue;
            }
            let degerler = self.getprop_oku(adb, &cihaz.seri)?;
            cihaz_profilini_zenginlestir(cihaz, &degerler);
        }
        Ok(cihazlar)
    }

    pub fn paketleri_tara(&self, adb: &Path, seri: &str) -> Sonuc<Vec<PaketDurumu>> {
        seri_dogrula(seri)?;
        let etkin = self.paket_listesi(adb, seri, "-e")?;
        let devre_disi = self.paket_listesi(adb, seri, "-d")?;
        let tumu = self.paket_listesi(adb, seri, "-u")?;
        let sistem = self.paket_listesi(adb, seri, "-s")?;
        let kullanici = self.paket_listesi(adb, seri, "-3")?;
        Ok(paket_durumlari(
            &etkin,
            &devre_disi,
            &tumu,
            &sistem,
            &kullanici,
        ))
    }

    pub fn paket_eylemi(
        &self,
        adb: &Path,
        seri: &str,
        paket: &str,
        eylem: PaketEylemi,
        risk: Risk,
        uzman_onayi: bool,
    ) -> Sonuc<KomutSonucu> {
        eylem_guvenli_mi(risk, eylem, uzman_onayi)?;
        let argumanlar = paket_komut_argumanlari(eylem, paket, seri)?;
        let sonuc = self.calistir(adb, &argumanlar, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{paket} için işlem başarısız (kod {}): {}",
                sonuc.kod,
                arac_hatasi(&sonuc)
            )));
        }
        match eylem {
            PaketEylemi::Kaldir { .. } if !sonuc.stdout.contains("Success") => Err(Hata::DisArac(
                format!("{paket} kaldırılamadı: {}", arac_hatasi(&sonuc)),
            )),
            PaketEylemi::GeriYukle if sonuc.stdout.contains("Failure") => Err(Hata::DisArac(
                format!("{paket} geri yüklenemedi: {}", sonuc.stdout.trim()),
            )),
            _ => Ok(sonuc),
        }
    }

    pub fn uad_onbellek_yukle(&self) -> Sonuc<Option<UadDurumu>> {
        let dosya = self.uad_dosyasi();
        let metin = match std::fs::read_to_string(&dosya) {
            Ok(metin) => metin,
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(hata) => {
                return Err(Hata::Beklenmeyen(format!(
                    "UAD önbelleği okunamadı: {hata}"
                )))
            }
        };
        let veritabani = uad_ayristir(metin.trim_start_matches('\u{feff}'))?;
        Ok(Some(uad_durumu(veritabani, dosya)))
    }

    pub fn uad_indir(&self) -> Sonuc<UadDurumu> {
        let gecici = gecici_dosya("uad", "json");
        let sonuc = self
            .indir(UAD_URL, &gecici, None)
            .and_then(|()| self.uad_ice_aktar(&gecici));
        let _ = std::fs::remove_file(&gecici);
        sonuc
    }

    pub fn uad_yerel_yukle(&self, dosya: &Path) -> Sonuc<UadDurumu> {
        self.uad_ice_aktar(dosya)
    }

    pub fn saf_rust_kabuk(&self, adb: &Path, seri: &str, komut: &str) -> Sonuc<String> {
        seri_dogrula(seri)?;
        kabuk_komutunu_dogrula(komut)?;
        let adb_yolu = Some(adb.to_string_lossy().to_string());
        let adres = SocketAddrV4::new(Ipv4Addr::LOCALHOST, ADB_PORTU);
        let mut sunucu = ADBServer::new_from_path(adres, adb_yolu.clone());
        let cihaz = match sunucu.get_device_by_name(seri) {
            Ok(cihaz) => cihaz,
            Err(ilk_hata) => {
                ADBServer::start(&HashMap::new(), &adb_yolu);
                sunucu.get_device_by_name(seri).map_err(|hata| {
                    Hata::DisArac(format!(
                        "{seri} cihazına TCP ADB sunucusu üzerinden ulaşılamadı: {hata} (ilk deneme: {ilk_hata})"
                    ))
                })?
            }
        };
        kabuk_ciktisi(cihaz, komut, seri)
    }

    pub fn saf_rust_usb_kabuk(&self, komut: &str) -> Sonuc<String> {
        kabuk_komutunu_dogrula(komut)?;
        let cihaz = ADBUSBDevice::autodetect().map_err(|hata| {
            Hata::DisArac(format!("USB üzerinden ADB cihazı bulunamadı: {hata}"))
        })?;
        kabuk_ciktisi(cihaz, komut, "USB")
    }

    fn calistir(
        &self,
        adb: &Path,
        argumanlar: &[String],
        zaman_asimi: Duration,
    ) -> Sonuc<KomutSonucu> {
        self.calistirici
            .calistir(adb.to_string_lossy().as_ref(), argumanlar, zaman_asimi)
    }

    fn getprop_oku(&self, adb: &Path, seri: &str) -> Sonuc<HashMap<String, String>> {
        seri_dogrula(seri)?;
        let sonuc = self.calistir(
            adb,
            &[
                "-s".to_string(),
                seri.to_string(),
                "shell".to_string(),
                "getprop".to_string(),
            ],
            KISA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{seri} cihaz özellikleri okunamadı (kod {}): {}",
                sonuc.kod,
                arac_hatasi(&sonuc)
            )));
        }
        Ok(getprop_ayristir(&sonuc.stdout))
    }

    fn paket_listesi(&self, adb: &Path, seri: &str, bayrak: &str) -> Sonuc<Vec<String>> {
        let sonuc = self.calistir(
            adb,
            &[
                "-s".to_string(),
                seri.to_string(),
                "shell".to_string(),
                "pm".to_string(),
                "list".to_string(),
                "packages".to_string(),
                bayrak.to_string(),
            ],
            TARAMA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{seri} paket listesi ({bayrak}) alınamadı (kod {}): {}",
                sonuc.kod,
                arac_hatasi(&sonuc)
            )));
        }
        Ok(paketleri_ayristir(&sonuc.stdout))
    }

    fn uad_ice_aktar(&self, kaynak: &Path) -> Sonuc<UadDurumu> {
        let metin = uad_metni_oku(kaynak)?;
        let veritabani = uad_ayristir(&metin)?;
        self.uad_onbellege_yaz(&metin)?;
        Ok(uad_durumu(veritabani, self.uad_dosyasi()))
    }

    fn uad_dosyasi(&self) -> PathBuf {
        self.kok.join(UAD_DOSYASI)
    }

    fn uad_onbellege_yaz(&self, metin: &str) -> Sonuc<()> {
        std::fs::create_dir_all(&self.kok)
            .map_err(|hata| Hata::Beklenmeyen(format!("UAD dizini oluşturulamadı: {hata}")))?;
        let gecici = self.kok.join(UAD_GECICI_DOSYASI);
        std::fs::write(&gecici, metin)
            .map_err(|hata| Hata::Beklenmeyen(format!("UAD önbelleği yazılamadı: {hata}")))?;
        std::fs::rename(&gecici, self.uad_dosyasi()).map_err(|hata| {
            let _ = std::fs::remove_file(&gecici);
            Hata::Beklenmeyen(format!("UAD önbelleği kaydedilemedi: {hata}"))
        })
    }

    fn indir(&self, url: &str, dosya: &Path, hedef: Option<&Path>) -> Sonuc<()> {
        let betik = gecici_dosya("indirme", "ps1");
        std::fs::write(&betik, INDIRME_BETIGI)
            .map_err(|hata| Hata::Beklenmeyen(format!("İndirme betiği yazılamadı: {hata}")))?;
        let argumanlar = vec![
            "-NoProfile".to_string(),
            "-NonInteractive".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-File".to_string(),
            betik.to_string_lossy().to_string(),
            "-Uri".to_string(),
            url.to_string(),
            "-Dosya".to_string(),
            dosya.to_string_lossy().to_string(),
            "-Hedef".to_string(),
            hedef
                .map(|yol| yol.to_string_lossy().to_string())
                .unwrap_or_default(),
        ];
        let sonuc = self
            .calistirici
            .calistir("powershell.exe", &argumanlar, INDIRME_ZAMAN_ASIMI);
        let _ = std::fs::remove_file(&betik);
        let sonuc = sonuc?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "İndirme başarısız (kod {}): {}",
                sonuc.kod,
                arac_hatasi(&sonuc)
            )));
        }
        if dosya.is_file() {
            return Ok(());
        }
        Err(Hata::DisArac(format!(
            "İndirme tamamlandı ancak dosya oluşmadı: {}",
            dosya.display()
        )))
    }
}

fn seri_dogrula(seri: &str) -> Sonuc<()> {
    if seri_gecerli(seri) {
        Ok(())
    } else {
        Err(Hata::Beklenmeyen("Geçersiz ADB cihaz serisi.".into()))
    }
}

fn kabuk_komutunu_dogrula(komut: &str) -> Sonuc<()> {
    if komut.trim().is_empty() {
        return Err(Hata::Beklenmeyen("ADB kabuk komutu boş olamaz.".into()));
    }
    if komut.len() > AZAMI_KABUK_KOMUTU || komut.chars().any(char::is_control) {
        return Err(Hata::Beklenmeyen("ADB kabuk komutu geçersiz.".into()));
    }
    Ok(())
}

fn kabuk_ciktisi(mut cihaz: impl ADBDeviceExt, komut: &str, seri: &str) -> Sonuc<String> {
    let mut cikti = Vec::new();
    cihaz
        .shell_command(&komut, Some(&mut cikti), None)
        .map_err(|hata| Hata::DisArac(format!("{seri} cihazında komut çalıştırılamadı: {hata}")))?;
    String::from_utf8(cikti).map_err(|hata| {
        Hata::DisArac(format!(
            "{seri} cihazındaki komut çıktısı çözülemedi: {hata}"
        ))
    })
}

fn uad_metni_oku(dosya: &Path) -> Sonuc<String> {
    let ustveri = std::fs::metadata(dosya)
        .map_err(|hata| Hata::Beklenmeyen(format!("UAD dosyası okunamadı: {hata}")))?;
    if !ustveri.is_file() {
        return Err(Hata::Beklenmeyen(format!(
            "UAD yolu bir dosya değil: {}",
            dosya.display()
        )));
    }
    if ustveri.len() > AZAMI_UAD_BAYT {
        return Err(Hata::Beklenmeyen(format!(
            "UAD dosyası {} MiB sınırını aşıyor.",
            AZAMI_UAD_BAYT / (1024 * 1024)
        )));
    }
    let metin = std::fs::read_to_string(dosya)
        .map_err(|hata| Hata::Beklenmeyen(format!("UAD dosyası okunamadı: {hata}")))?;
    Ok(metin.trim_start_matches('\u{feff}').to_string())
}

fn uad_durumu(veritabani: UadVeritabani, dosya: PathBuf) -> UadDurumu {
    let guncelleme = std::fs::metadata(&dosya)
        .and_then(|ustveri| ustveri.modified())
        .map(zaman_metni)
        .unwrap_or_else(|_| zaman_metni(SystemTime::now()));
    UadDurumu {
        veritabani,
        dosya,
        guncelleme,
        lisans: UAD_LISANS.to_string(),
    }
}

fn zaman_metni(zaman: SystemTime) -> String {
    let saniye = zaman
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (yil, ay, gun) = tarih(saniye / 86_400);
    let kalan = saniye % 86_400;
    format!(
        "{yil:04}-{ay:02}-{gun:02} {:02}:{:02}",
        kalan / 3_600,
        (kalan % 3_600) / 60
    )
}

// Gün sayısını UTC tarihine çeviren Howard Hinnant algoritması.
fn tarih(gun_sayisi: u64) -> (i64, u64, u64) {
    let z = gun_sayisi as i64 + 719_468;
    let donem = z / 146_097;
    let yil_gunu = z - donem * 146_097;
    let yil0 = (yil_gunu - yil_gunu / 1_460 + yil_gunu / 36_524 - yil_gunu / 146_096) / 365;
    let yil_basi = 365 * yil0 + yil0 / 4 - yil0 / 100;
    let kalan = yil_gunu - yil_basi;
    let mp = (5 * kalan + 2) / 153;
    let gun = kalan - (153 * mp + 2) / 5 + 1;
    let ay = if mp < 10 { mp + 3 } else { mp - 9 };
    let yil = if ay <= 2 {
        yil0 + donem * 400 + 1
    } else {
        yil0 + donem * 400
    };
    (yil, ay as u64, gun as u64)
}

fn arac_hatasi(sonuc: &KomutSonucu) -> String {
    let stderr = sonuc.stderr.trim();
    if stderr.is_empty() {
        sonuc.stdout.trim().to_string()
    } else {
        stderr.to_string()
    }
}

fn path_te_bul(ad: &str) -> Option<PathBuf> {
    let yol = std::env::var_os("PATH")?;
    std::env::split_paths(&yol)
        .map(|dizin| dizin.join(ad))
        .find(|aday| aday.is_file())
}

fn gecici_dosya(ad: &str, uzanti: &str) -> PathBuf {
    let sira = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "winnexus-{ad}-{}-{sira}.{uzanti}",
        std::process::id()
    ))
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::sync::Mutex;

    const FIKSTUR: &str = r#"{
        "com.miui.analytics": {"list":"Oem","description":"Analytics","dependencies":["com.miui.core"],"neededBy":[],"labels":[],"removal":"Recommended"},
        "com.android.shared": {"list":"Aosp","description":"Shared","dependencies":[],"neededBy":[],"labels":[],"removal":"Unsafe"}
    }"#;

    type Yanit = Arc<dyn Fn(&[String]) -> Sonuc<KomutSonucu> + Send + Sync>;

    struct SahteCalistirici {
        yanit: Yanit,
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
            (self.yanit)(argumanlar)
        }
    }

    fn servis(kok: PathBuf, yanit: Yanit) -> (AndroidServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            yanit,
            cagrilar: Mutex::new(Vec::new()),
        });
        (AndroidServisi::yeni(calistirici.clone(), kok), calistirici)
    }

    fn basarili(stdout: &str) -> Sonuc<KomutSonucu> {
        Ok(KomutSonucu {
            komut: "adb".into(),
            kod: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        })
    }

    fn hatali(stderr: &str) -> Sonuc<KomutSonucu> {
        Ok(KomutSonucu {
            komut: "adb".into(),
            kod: 1,
            stdout: String::new(),
            stderr: stderr.into(),
        })
    }

    fn arg_degeri(argumanlar: &[String], ad: &str) -> Option<String> {
        argumanlar
            .iter()
            .position(|arguman| arguman == ad)
            .and_then(|sira| argumanlar.get(sira + 1))
            .cloned()
    }

    fn anahtar(argumanlar: &[String]) -> &str {
        argumanlar.first().map(String::as_str).unwrap_or("")
    }

    #[test]
    fn adb_yolu_platform_tools_altinda_bulunur() {
        let gecici = tempfile::tempdir().unwrap();
        let adb = gecici.path().join(PLATFORM_TOOLS_KLASORU).join(ADB_DOSYASI);
        std::fs::create_dir_all(adb.parent().unwrap()).unwrap();
        std::fs::write(&adb, b"x").unwrap();
        let (servis, _) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|_: &[String]| basarili("")),
        );
        assert_eq!(servis.adb_yolu(), Some(adb));
    }

    #[test]
    fn cihazlar_taranir_ve_getprop_zenginlestirilir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, calistirici) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|argumanlar: &[String]| {
                if anahtar(argumanlar) == "devices" {
                    return basarili(
                        "List of devices attached\nABC123\tdevice model:Mi_10\n192.168.1.5:5555\tunauthorized\n",
                    );
                }
                basarili("[ro.product.brand]: [Xiaomi]\n[ro.product.model]: [2211133G]\n[ro.build.version.release]: [14]\n[ro.build.version.sdk]: [34]\n")
            }),
        );
        let cihazlar = servis.cihazlari_tara(Path::new("adb.exe")).unwrap();
        assert_eq!(cihazlar.len(), 2);
        assert_eq!(cihazlar[0].marka.as_deref(), Some("Xiaomi"));
        assert_eq!(cihazlar[0].android_surumu.as_deref(), Some("14"));
        assert_eq!(cihazlar[0].sdk, Some(34));
        assert_eq!(cihazlar[1].durum, BaglantiDurumu::Yetkisiz);
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 2);
        assert_eq!(cagrilar[0], ["devices", "-l"]);
        assert!(cagrilar[1].contains(&"ABC123".to_string()));
        assert!(cagrilar[1].contains(&"getprop".to_string()));
        assert!(!cagrilar
            .iter()
            .any(|cagri| cagri.contains(&"192.168.1.5:5555".to_string())));
    }

    #[test]
    fn getprop_hatasi_taramayi_durdurur() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, _) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|argumanlar: &[String]| {
                if anahtar(argumanlar) == "devices" {
                    return basarili("List of devices attached\nABC123\tdevice\n");
                }
                hatali("cihaz yanıt vermiyor")
            }),
        );
        let hata = servis.cihazlari_tara(Path::new("adb.exe")).unwrap_err();
        assert!(hata.to_string().contains("ABC123"));
        assert!(hata.to_string().contains("cihaz yanıt vermiyor"));
    }

    #[test]
    fn paket_listeleri_sirayla_calistirilir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, calistirici) = servis(
            gecici.path().to_path_buf(),
            Arc::new(
                |argumanlar: &[String]| match argumanlar.last().map(String::as_str) {
                    Some("-e") => basarili("package:com.a\npackage:com.b\n"),
                    Some("-d") => basarili("package:com.b\n"),
                    Some("-u") => basarili("package:com.a\npackage:com.b\npackage:com.c\n"),
                    Some("-s") => basarili("package:com.a\npackage:com.b\n"),
                    Some("-3") => basarili("package:com.c\n"),
                    _ => hatali("beklenmeyen bayrak"),
                },
            ),
        );
        let durumlar = servis
            .paketleri_tara(Path::new("adb.exe"), "ABC123")
            .unwrap();
        assert_eq!(durumlar.len(), 3);
        let bul = |id: &str| durumlar.iter().find(|paket| paket.id == id).unwrap();
        assert!(bul("com.a").sistem);
        assert!(bul("com.b").devre_disi);
        assert!(bul("com.c").kullanici);
        assert!(bul("com.c").kaldirilmis);
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 5);
        for (cagri, bayrak) in cagrilar.iter().zip(["-e", "-d", "-u", "-s", "-3"]) {
            assert_eq!(cagri[..4], ["-s", "ABC123", "shell", "pm"]);
            assert_eq!(cagri.last().map(String::as_str), Some(bayrak));
        }
    }

    #[test]
    fn paket_eylemi_politikayi_ve_komutu_uygular() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, calistirici) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|_: &[String]| basarili("Success\n")),
        );
        let adb = Path::new("adb.exe");
        assert!(servis
            .paket_eylemi(
                adb,
                "ABC123",
                "com.miui.analytics",
                PaketEylemi::Kaldir {
                    verileri_koru: true
                },
                Risk::Unsafe,
                true,
            )
            .is_err());
        assert!(servis
            .paket_eylemi(
                adb,
                "ABC123",
                "com.miui.analytics",
                PaketEylemi::DevreDisi,
                Risk::Expert,
                false,
            )
            .is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
        servis
            .paket_eylemi(
                adb,
                "ABC123",
                "com.miui.analytics",
                PaketEylemi::Kaldir {
                    verileri_koru: true,
                },
                Risk::Recommended,
                false,
            )
            .unwrap();
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 1);
        assert_eq!(cagrilar[0][..4], ["-s", "ABC123", "shell", "pm"]);
        assert!(cagrilar[0].contains(&"uninstall".to_string()));
        assert!(cagrilar[0].contains(&"--user".to_string()));
    }

    #[test]
    fn kaldirma_success_olmadan_reddedilir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, _) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|_: &[String]| basarili("Failure [DELETE_FAILED_INTERNAL_ERROR]\n")),
        );
        assert!(servis
            .paket_eylemi(
                Path::new("adb.exe"),
                "ABC123",
                "com.miui.analytics",
                PaketEylemi::Kaldir {
                    verileri_koru: false
                },
                Risk::Recommended,
                false,
            )
            .is_err());
    }

    #[test]
    fn uad_onbellegi_yuklenir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, _) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|_: &[String]| basarili("")),
        );
        assert!(servis.uad_onbellek_yukle().unwrap().is_none());
        let onbellek = gecici.path().join(UAD_DOSYASI);
        std::fs::write(&onbellek, FIKSTUR).unwrap();
        let durum = servis.uad_onbellek_yukle().unwrap().unwrap();
        assert_eq!(durum.veritabani.paketler.len(), 2);
        assert_eq!(
            durum.veritabani.bilgi("com.miui.analytics").unwrap().risk,
            Risk::Recommended
        );
        assert_eq!(durum.dosya, onbellek);
        assert_eq!(durum.lisans, UAD_LISANS);
        assert_eq!(durum.guncelleme.len(), 16);
        assert!(durum.guncelleme.contains('-'));
        assert!(durum.guncelleme.contains(':'));
        std::fs::write(&onbellek, b"{").unwrap();
        assert!(servis.uad_onbellek_yukle().is_err());
    }

    #[test]
    fn uad_yerel_dosyadan_ice_aktarilir() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("veri");
        let kaynak = gecici.path().join("uad.json");
        std::fs::write(&kaynak, FIKSTUR).unwrap();
        let (servis, calistirici) = servis(kok.clone(), Arc::new(|_: &[String]| basarili("")));
        let durum = servis.uad_yerel_yukle(&kaynak).unwrap();
        assert_eq!(durum.veritabani.paketler.len(), 2);
        assert_eq!(durum.dosya, kok.join(UAD_DOSYASI));
        assert!(kok.join(UAD_DOSYASI).is_file());
        assert_eq!(durum.lisans, UAD_LISANS);
        assert!(servis
            .uad_yerel_yukle(&gecici.path().join("yok.json"))
            .is_err());
        let bozuk = gecici.path().join("bozuk.json");
        std::fs::write(&bozuk, b"{").unwrap();
        assert!(servis.uad_yerel_yukle(&bozuk).is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn uad_acik_cagrida_indirilir() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("veri");
        let (servis, calistirici) = servis(
            kok.clone(),
            Arc::new(|argumanlar: &[String]| {
                let dosya = arg_degeri(argumanlar, "-Dosya").unwrap();
                std::fs::write(dosya, FIKSTUR).unwrap();
                basarili("")
            }),
        );
        let durum = servis.uad_indir().unwrap();
        assert_eq!(durum.veritabani.paketler.len(), 2);
        assert!(kok.join(UAD_DOSYASI).is_file());
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 1);
        assert!(cagrilar[0].contains(&UAD_URL.to_string()));
        assert_eq!(arg_degeri(&cagrilar[0], "-Hedef"), Some(String::new()));
    }

    #[test]
    fn platform_tools_indirme_argumanlari_dogrulanir() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("veri");
        let (servis, calistirici) = servis(
            kok.clone(),
            Arc::new(|argumanlar: &[String]| {
                let dosya = arg_degeri(argumanlar, "-Dosya").unwrap();
                let hedef = arg_degeri(argumanlar, "-Hedef").unwrap();
                std::fs::write(dosya, b"zip").unwrap();
                let adb = PathBuf::from(hedef)
                    .join(PLATFORM_TOOLS_KLASORU)
                    .join(ADB_DOSYASI);
                std::fs::create_dir_all(adb.parent().unwrap()).unwrap();
                std::fs::write(adb, b"x").unwrap();
                basarili("")
            }),
        );
        let adb = servis.platform_tools_indir().unwrap();
        assert_eq!(adb, kok.join(PLATFORM_TOOLS_KLASORU).join(ADB_DOSYASI));
        assert!(adb.is_file());
        assert!(INDIRME_BETIGI.contains("Invoke-WebRequest -UseBasicParsing"));
        assert!(INDIRME_BETIGI.contains("Expand-Archive"));
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 1);
        assert_eq!(cagrilar[0][0], "-NoProfile");
        assert!(cagrilar[0].contains(&PLATFORM_TOOLS_URL.to_string()));
        assert_eq!(
            arg_degeri(&cagrilar[0], "-Hedef").as_deref(),
            Some(kok.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn saf_rust_kabuk_gecersiz_seriyi_reddeder() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, calistirici) = servis(
            gecici.path().to_path_buf(),
            Arc::new(|_: &[String]| basarili("")),
        );
        assert!(servis
            .saf_rust_kabuk(Path::new("adb.exe"), "kötü seri", "getprop")
            .is_err());
        assert!(servis
            .saf_rust_kabuk(Path::new("adb.exe"), "ABC123", "")
            .is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn zaman_metni_utc_bicimlenir() {
        assert_eq!(zaman_metni(UNIX_EPOCH), "1970-01-01 00:00");
        assert_eq!(
            zaman_metni(UNIX_EPOCH + Duration::from_secs(1_709_164_800)),
            "2024-02-29 00:00"
        );
    }
}
