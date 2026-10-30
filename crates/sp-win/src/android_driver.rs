use crate::registry::WindowsAdaptoru;
use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::adaptor::SistemAdaptoru;
use sp_core::android::{vid_marka, GOOGLE_USB_DRIVER_URL};
use sp_core::donanim::serde_json::{self, Value};
use sp_core::error::{Hata, Sonuc};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

const POWERSHELL_PROGRAMI: &str = "powershell";
const PNPUTIL_PROGRAMI: &str = "pnputil.exe";
const WDI_ARACI: &str = "wdi-simple.exe";
const GOOGLE_INF_ADI: &str = "android_winusb.inf";
const TARAMA_ZAMAN_ASIMI: Duration = Duration::from_secs(90);
const IMZA_ZAMAN_ASIMI: Duration = Duration::from_secs(60);
const PNPUTIL_ZAMAN_ASIMI: Duration = Duration::from_secs(300);
const INDIRME_ZAMAN_ASIMI: Duration = Duration::from_secs(600);
const KURULUM_ZAMAN_ASIMI: Duration = Duration::from_secs(1800);
const USB_BETIGI: &str = "$ErrorActionPreference='Stop'; try { Get-CimInstance Win32_PnPEntity | Where-Object { $_.PNPDeviceID -like 'USB\\VID_*' } | Select-Object Name,Manufacturer,Status,PNPDeviceID,Service | ConvertTo-Json -Compress } catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbCihaz {
    pub vid: u16,
    pub pid: u16,
    pub ad: String,
    pub uretici: String,
    pub durum: String,
    pub surucu: String,
    pub adb_arayuzu: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurucuRaporu {
    pub cihazlar: Vec<UsbCihaz>,
    pub adb_hazir: bool,
    pub oneriler: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurucuSonucu {
    pub basarili: bool,
    pub mesaj: String,
    pub imza: String,
}

#[derive(Clone)]
pub struct SurucuServisi {
    calistirici: Arc<dyn KomutCalistirici>,
    yonetici_denetimi: fn() -> bool,
}

impl SurucuServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self {
            calistirici,
            yonetici_denetimi: || WindowsAdaptoru.yonetici_mi(),
        }
    }

    pub fn durumu_tara(&self) -> Sonuc<SurucuRaporu> {
        let sonuc =
            self.powershell_calistir(USB_BETIGI, "USB cihaz durumu okunamadı", TARAMA_ZAMAN_ASIMI)?;
        let cihazlar = pnp_json_ayristir(&sonuc.stdout)?;
        Ok(rapor_olustur(cihazlar))
    }

    pub fn inf_kur(&self, inf: &Path) -> Sonuc<SurucuSonucu> {
        self.yonetici_kontrolu()?;
        let inf = dosya_dogrula(inf, "inf", "Sürücü INF")?;
        let (imza, imza_gecerli) = self.imza_dogrula(&inf)?;
        if !imza_gecerli {
            return Ok(imza_reddi(
                format!("INF dosyasının dijital imzası geçerli değil ({imza}); sürücü kurulmadı."),
                imza,
            ));
        }
        let argumanlar = vec![
            "/add-driver".to_string(),
            komut_metni(&inf),
            "/install".to_string(),
        ];
        let sonuc =
            self.calistirici
                .calistir(PNPUTIL_PROGRAMI, &argumanlar, PNPUTIL_ZAMAN_ASIMI)?;
        let basarili = sonuc.kod == 0 && !cikti_hatali_mi(&sonuc);
        let mesaj = if basarili {
            format!("Sürücü paketi kuruldu: {}", komut_metni(&inf))
        } else {
            format!(
                "Sürücü paketi kurulamadı (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )
        };
        Ok(SurucuSonucu {
            basarili,
            mesaj,
            imza,
        })
    }

    pub fn samsung_kur(&self, kurulum: &Path) -> Sonuc<SurucuSonucu> {
        self.yonetici_kontrolu()?;
        let kurulum = dosya_dogrula(kurulum, "exe", "Samsung kurulum")?;
        let (imza, imza_gecerli) = self.imza_dogrula(&kurulum)?;
        if !imza_gecerli {
            return Err(Hata::Dogrulama(format!(
                "Samsung kurulum dosyasının dijital imzası geçerli değil ({imza}); kurulum yapılmadı."
            )));
        }
        let sonuc = self.calistirici.calistir(
            &komut_metni(&kurulum),
            &["/S".to_string()],
            KURULUM_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Samsung USB sürücüsü kurulamadı (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(SurucuSonucu {
            basarili: true,
            mesaj: "Samsung USB sürücüsü sessiz kurulumla kuruldu; gerekirse bilgisayarı yeniden başlatın.".to_string(),
            imza,
        })
    }

    pub fn google_usb_indir_kur(&self, hedef_zip: &Path) -> Sonuc<SurucuSonucu> {
        let hedef = zip_hedefi_hazirla(hedef_zip)?;
        self.google_indir(&hedef)?;
        let acilan = self.google_arsivi_ac(&hedef)?;
        let inf = android_inf_bul(&acilan).ok_or_else(|| {
            Hata::DisArac(format!("İndirilen arşivde {GOOGLE_INF_ADI} bulunamadı."))
        })?;
        self.inf_kur(&inf)
    }

    pub fn libwdi_kur(&self, arac: &Path, vid: u16, pid: u16) -> Sonuc<SurucuSonucu> {
        if !arac.exists() || !dosya_adi_eslesir(arac, WDI_ARACI) {
            return Err(Hata::Desteklenmiyor(
                "wdi-simple.exe bulunamadı; libwdi ile sürücü kurulumu için geçerli araç yolu gerekli."
                    .into(),
            ));
        }
        self.yonetici_kontrolu()?;
        let arac = dosya_dogrula(arac, "exe", "libwdi aracı")?;
        let (imza, imza_gecerli) = self.imza_dogrula(&arac)?;
        if !imza_gecerli {
            return Ok(imza_reddi(
                format!("libwdi aracının dijital imzası geçerli değil ({imza}); sürücü kurulmadı."),
                imza,
            ));
        }
        let sonuc = self.calistirici.calistir(
            &komut_metni(&arac),
            &libwdi_argumanlari(vid, pid),
            KURULUM_ZAMAN_ASIMI,
        )?;
        let basarili = sonuc.kod == 0 && !cikti_hatali_mi(&sonuc);
        let mesaj = if basarili {
            "Android ADB sürücüsü libwdi ile kuruldu.".to_string()
        } else {
            format!(
                "libwdi sürücü kurulumu başarısız (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )
        };
        Ok(SurucuSonucu {
            basarili,
            mesaj,
            imza,
        })
    }

    fn yonetici_kontrolu(&self) -> Sonuc<()> {
        if (self.yonetici_denetimi)() {
            Ok(())
        } else {
            Err(Hata::Yetki(
                "Sürücü kurulumu için WinNexus yönetici olarak çalıştırılmalı.".into(),
            ))
        }
    }

    fn imza_dogrula(&self, dosya: &Path) -> Sonuc<(String, bool)> {
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ Get-AuthenticodeSignature -LiteralPath '{yol}' | Select-Object Status,SignerCertificate | ConvertTo-Json -Compress }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}",
            yol = ps_metni(dosya)
        );
        let sonuc =
            self.powershell_calistir(&betik, "Dosya imzası doğrulanamadı", IMZA_ZAMAN_ASIMI)?;
        Ok((
            imza_durumu(&sonuc.stdout),
            imza_json_gecerli_mi(&sonuc.stdout),
        ))
    }

    fn google_indir(&self, hedef: &Path) -> Sonuc<()> {
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ Invoke-WebRequest -UseBasicParsing -Uri '{GOOGLE_USB_DRIVER_URL}' -OutFile '{yol}'; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}",
            yol = ps_metni(hedef)
        );
        self.powershell_calistir(
            &betik,
            "Google USB sürücü paketi indirilemedi",
            INDIRME_ZAMAN_ASIMI,
        )?;
        let boyut = std::fs::metadata(hedef)
            .map(|ustveri| ustveri.len())
            .unwrap_or(0);
        if boyut == 0 {
            return Err(Hata::DisArac(format!(
                "Google USB sürücü paketi indirilemedi veya boş: {}",
                komut_metni(hedef)
            )));
        }
        Ok(())
    }

    fn google_arsivi_ac(&self, zip: &Path) -> Sonuc<PathBuf> {
        let acilan = zip.with_extension("");
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ Expand-Archive -LiteralPath '{zip}' -DestinationPath '{hedef}' -Force; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}",
            zip = ps_metni(zip),
            hedef = ps_metni(&acilan)
        );
        self.powershell_calistir(
            &betik,
            "Google USB sürücü arşivi açılamadı",
            INDIRME_ZAMAN_ASIMI,
        )?;
        if !acilan.is_dir() {
            return Err(Hata::DisArac(format!(
                "Google USB sürücü arşivi açılamadı: {}",
                komut_metni(&acilan)
            )));
        }
        Ok(acilan)
    }

    fn powershell_calistir(
        &self,
        betik: &str,
        baslik: &str,
        zaman_asimi: Duration,
    ) -> Sonuc<KomutSonucu> {
        let argumanlar = [
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            betik,
        ]
        .map(str::to_string);
        let sonuc = self
            .calistirici
            .calistir(POWERSHELL_PROGRAMI, &argumanlar, zaman_asimi)?;
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

fn imza_reddi(mesaj: String, imza: String) -> SurucuSonucu {
    SurucuSonucu {
        basarili: false,
        mesaj,
        imza,
    }
}

pub fn pnp_json_ayristir(json: &str) -> Sonuc<Vec<UsbCihaz>> {
    let temiz = json.trim_start_matches('\u{feff}').trim();
    if temiz.is_empty() {
        return Ok(Vec::new());
    }
    let deger: Value = serde_json::from_str(temiz).map_err(|hata| {
        Hata::Beklenmeyen(format!("USB cihaz listesi JSON olarak okunamadı: {hata}"))
    })?;
    let kayitlar = match &deger {
        Value::Array(kayitlar) => kayitlar.as_slice(),
        Value::Object(_) => std::slice::from_ref(&deger),
        _ => {
            return Err(Hata::Beklenmeyen(
                "USB cihaz listesi beklenen JSON biçiminde değil.".into(),
            ));
        }
    };
    Ok(kayitlar.iter().filter_map(usb_cihazi_ayristir).collect())
}

fn usb_cihazi_ayristir(kayit: &Value) -> Option<UsbCihaz> {
    let kimlik = alan_metni(
        kayit,
        &["PNPDeviceID", "PnpDeviceID", "PnpDeviceId", "Kimlik"],
    )?;
    if !kimlik.to_ascii_uppercase().starts_with("USB\\VID_") {
        return None;
    }
    let (vid, pid) = vid_pid_ayristir(&kimlik)?;
    let ad = alan_metni(kayit, &["Name", "Ad"]).unwrap_or_else(|| "Bilinmeyen aygıt".to_string());
    let uretici = alan_metni(kayit, &["Manufacturer", "Üretici", "Uretici"]).unwrap_or_default();
    let durum = alan_metni(kayit, &["Status", "Durum"]).unwrap_or_default();
    let surucu = alan_metni(kayit, &["Service", "Hizmet"]).unwrap_or_default();
    let adb_arayuzu = adb_metni_mi(&ad) || adb_metni_mi(&uretici);
    Some(UsbCihaz {
        vid,
        pid,
        ad,
        uretici,
        durum,
        surucu,
        adb_arayuzu,
    })
}

fn adb_metni_mi(metin: &str) -> bool {
    let kucuk = metin.to_lowercase();
    kucuk.contains("adb") || kucuk.contains("android")
}

fn alan_metni(nesne: &Value, adaylar: &[&str]) -> Option<String> {
    adaylar.iter().find_map(|ad| {
        let metin = nesne.get(*ad)?.as_str()?.trim();
        (!metin.is_empty()).then(|| metin.to_string())
    })
}

pub fn vid_pid_ayristir(kimlik: &str) -> Option<(u16, u16)> {
    let kimlik = kimlik.to_ascii_uppercase();
    Some((hex_dort(&kimlik, "VID_")?, hex_dort(&kimlik, "PID_")?))
}

fn hex_dort(metin: &str, on_ek: &str) -> Option<u16> {
    let bas = metin.find(on_ek)? + on_ek.len();
    let deger = metin.get(bas..bas + 4)?;
    if !deger.bytes().all(|bayt| bayt.is_ascii_hexdigit()) {
        return None;
    }
    let devam = metin.as_bytes().get(bas + 4).copied();
    if devam.is_some_and(|bayt| bayt.is_ascii_hexdigit()) {
        return None;
    }
    u16::from_str_radix(deger, 16).ok()
}

pub fn imza_json_gecerli_mi(json: &str) -> bool {
    let durum = imza_durumu(json);
    durum.eq_ignore_ascii_case("valid") || durum.eq_ignore_ascii_case("geçerli")
}

fn imza_durumu(json: &str) -> String {
    let Some(deger) = json_degeri(json) else {
        return "Bilinmiyor".to_string();
    };
    let Some(durum) = deger.get("Status").or_else(|| deger.get("Durum")) else {
        return "Bilinmiyor".to_string();
    };
    if let Some(metin) = durum
        .as_str()
        .map(str::trim)
        .filter(|metin| !metin.is_empty())
    {
        return metin.to_string();
    }
    match durum.as_i64() {
        Some(0) => "Valid".to_string(),
        Some(2) => "NotSigned".to_string(),
        Some(kod) => format!("imza kodu {kod}"),
        None => "Bilinmiyor".to_string(),
    }
}

fn rapor_olustur(cihazlar: Vec<UsbCihaz>) -> SurucuRaporu {
    let adb_hazir = cihazlar.iter().any(|cihaz| cihaz.adb_arayuzu);
    let mut oneriler = Vec::new();
    for cihaz in &cihazlar {
        let Some(marka) = vid_marka(cihaz.vid) else {
            continue;
        };
        let oneri = format!(
            "{marka} cihazı algılandı (VID {:04X}); üreticinin resmî USB sürücüsünü kurmayı deneyin.",
            cihaz.vid
        );
        if !oneriler.contains(&oneri) {
            oneriler.push(oneri);
        }
    }
    if !adb_hazir {
        oneriler.push(
            "ADB arayüzü bulunamadı; cihazda USB hata ayıklamayı açın ve sürücünün kurulu olduğundan emin olun."
                .to_string(),
        );
    }
    SurucuRaporu {
        cihazlar,
        adb_hazir,
        oneriler,
    }
}

fn dosya_dogrula(yol: &Path, uzanti: &str, tur: &str) -> Sonuc<PathBuf> {
    let kanonik = std::fs::canonicalize(yol).map_err(|hata| {
        Hata::Dogrulama(format!(
            "{tur} dosyası bulunamadı ({}): {hata}",
            yol.display()
        ))
    })?;
    if !kanonik.is_file() {
        return Err(Hata::Dogrulama(format!(
            "{tur} yolu bir dosya değil: {}",
            yol.display()
        )));
    }
    let dogru_uzanti = kanonik
        .extension()
        .and_then(|deger| deger.to_str())
        .is_some_and(|deger| deger.eq_ignore_ascii_case(uzanti));
    if !dogru_uzanti {
        return Err(Hata::Dogrulama(format!(
            "{tur} dosyası .{uzanti} uzantılı olmalı: {}",
            yol.display()
        )));
    }
    Ok(kanonik)
}

fn zip_hedefi_hazirla(hedef: &Path) -> Sonuc<PathBuf> {
    let uzanti_gecerli = hedef
        .extension()
        .and_then(|uzanti| uzanti.to_str())
        .is_some_and(|uzanti| uzanti.eq_ignore_ascii_case("zip"));
    if !uzanti_gecerli {
        return Err(Hata::Dogrulama(
            "Google sürücü arşivi .zip dosyası olmalı.".into(),
        ));
    }
    if hedef.is_dir() {
        return Err(Hata::Dogrulama(format!(
            "Hedef ZIP yolu bir dizin: {}",
            hedef.display()
        )));
    }
    let dosya_adi = hedef
        .file_name()
        .ok_or_else(|| Hata::Dogrulama("Hedef ZIP dosya adı geçersiz.".into()))?;
    let ust = match hedef.parent() {
        Some(ust) if !ust.as_os_str().is_empty() => ust.to_path_buf(),
        _ => std::env::current_dir()
            .map_err(|hata| Hata::DisArac(format!("Çalışma dizini okunamadı: {hata}")))?,
    };
    std::fs::create_dir_all(&ust)
        .map_err(|hata| Hata::DisArac(format!("İndirme dizini oluşturulamadı: {hata}")))?;
    let ust = std::fs::canonicalize(&ust)
        .map_err(|hata| Hata::DisArac(format!("İndirme dizini doğrulanamadı: {hata}")))?;
    Ok(ust.join(dosya_adi))
}

fn android_inf_bul(kok: &Path) -> Option<PathBuf> {
    let kayitlar = std::fs::read_dir(kok).ok()?;
    let mut altlar = Vec::new();
    for kayit in kayitlar.flatten() {
        let Ok(tur) = kayit.file_type() else {
            continue;
        };
        if tur.is_symlink() {
            continue;
        }
        let yol = kayit.path();
        if tur.is_file() {
            if dosya_adi_eslesir(&yol, GOOGLE_INF_ADI) {
                return Some(yol);
            }
        } else if tur.is_dir() {
            altlar.push(yol);
        }
    }
    altlar.iter().find_map(|alt| android_inf_bul(alt))
}

fn dosya_adi_eslesir(yol: &Path, ad: &str) -> bool {
    yol.file_name()
        .and_then(|dosya| dosya.to_str())
        .is_some_and(|dosya| dosya.eq_ignore_ascii_case(ad))
}

fn json_degeri(json: &str) -> Option<Value> {
    let temiz = json.trim_start_matches('\u{feff}').trim();
    if temiz.is_empty() {
        return None;
    }
    serde_json::from_str(temiz).ok()
}

fn ps_metni(yol: &Path) -> String {
    komut_metni(yol).replace('\'', "''")
}

fn komut_metni(yol: &Path) -> String {
    let metin = yol.to_string_lossy();
    if let Some(kalan) = metin.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{kalan}")
    } else if let Some(kalan) = metin.strip_prefix(r"\\?\") {
        kalan.to_string()
    } else {
        metin.into_owned()
    }
}

fn libwdi_argumanlari(vid: u16, pid: u16) -> Vec<String> {
    vec![
        "--vid".to_string(),
        format!("0x{vid:04X}"),
        "--pid".to_string(),
        format!("0x{pid:04X}"),
        "--type".to_string(),
        "0".to_string(),
        "--name".to_string(),
        "Android ADB Interface".to_string(),
        "--silent".to_string(),
    ]
}

fn cikti_hatali_mi(sonuc: &KomutSonucu) -> bool {
    let metin = format!("{}\n{}", sonuc.stdout, sonuc.stderr).to_lowercase();
    [
        "error",
        "failed",
        "unable",
        "could not",
        "cannot",
        "hata",
        "başarısız",
        "eklenemedi",
        "kurulamadı",
        "yüklenemedi",
    ]
    .iter()
    .any(|ifade| metin.contains(ifade))
}

fn hata_ozeti(sonuc: &KomutSonucu) -> String {
    let metin = if sonuc.stderr.trim().is_empty() {
        sonuc.stdout.trim()
    } else {
        sonuc.stderr.trim()
    };
    let Some(ilk) = metin.lines().map(str::trim).find(|satir| !satir.is_empty()) else {
        return "çıktı yok".to_string();
    };
    let kucuk = ilk.to_lowercase();
    if [
        "yükseltme",
        "elevation",
        "access is denied",
        "erişim engellendi",
        "0x80070005",
    ]
    .iter()
    .any(|ifade| kucuk.contains(ifade))
    {
        return "Bu işlem için yönetici izni gerekiyor.".to_string();
    }
    ilk.chars().take(300).collect()
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    const IMZA_GECERLI: &str = r#"{"Status":"Valid","SignerCertificate":{"Subject":"CN=Test"}}"#;
    const IMZA_GECERSIZ: &str = r#"{"Status":"NotSigned","SignerCertificate":null}"#;

    struct SahteCalistirici {
        cevaplar: Mutex<VecDeque<KomutSonucu>>,
        cagrilar: Mutex<Vec<(String, Vec<String>, Duration)>>,
    }

    impl KomutCalistirici for SahteCalistirici {
        fn calistir(
            &self,
            program: &str,
            argumanlar: &[String],
            zaman_asimi: Duration,
        ) -> Sonuc<KomutSonucu> {
            self.cagrilar.lock().unwrap().push((
                program.to_string(),
                argumanlar.to_vec(),
                zaman_asimi,
            ));
            self.cevaplar
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| Hata::DisArac("test cevabı yok".into()))
        }
    }

    fn cevap(stdout: &str) -> KomutSonucu {
        KomutSonucu {
            komut: "test".into(),
            kod: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn test_servisi(
        cevaplar: Vec<KomutSonucu>,
        yonetici: bool,
    ) -> (SurucuServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            cevaplar: Mutex::new(VecDeque::from(cevaplar)),
            cagrilar: Mutex::new(Vec::new()),
        });
        let servis = SurucuServisi {
            calistirici: calistirici.clone(),
            yonetici_denetimi: if yonetici { || true } else { || false },
        };
        (servis, calistirici)
    }

    fn cagrilar(calistirici: &SahteCalistirici) -> Vec<(String, Vec<String>, Duration)> {
        calistirici.cagrilar.lock().unwrap().clone()
    }

    fn gecici_dosya(ad: &str, icerik: &[u8]) -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dizin = tempfile::tempdir().unwrap();
        let yol = dizin.path().join(ad);
        std::fs::write(&yol, icerik).unwrap();
        let kanonik = std::fs::canonicalize(&yol).unwrap();
        (dizin, yol, kanonik)
    }

    #[test]
    fn komut_yolundan_verbatim_onek_kaldirilir() {
        assert_eq!(
            komut_metni(Path::new(r"\\?\C:\surucu\android_winusb.inf")),
            r"C:\surucu\android_winusb.inf"
        );
        assert_eq!(
            komut_metni(Path::new(r"\\?\UNC\sunucu\pay\surucu.inf")),
            r"\\sunucu\pay\surucu.inf"
        );
        assert_eq!(
            komut_metni(Path::new(r"C:\surucu\surucu.inf")),
            r"C:\surucu\surucu.inf"
        );
    }

    #[test]
    fn vid_pid_ayristirilir() {
        assert_eq!(
            vid_pid_ayristir(r"USB\VID_04E8&PID_6860\R58M123"),
            Some((0x04E8, 0x6860))
        );
        assert_eq!(
            vid_pid_ayristir(r"usb\vid_18d1&pid_4ee7&mi_01"),
            Some((0x18D1, 0x4EE7))
        );
        assert_eq!(vid_pid_ayristir(r"USB\VID_04E8&PID_68601"), None);
        assert_eq!(
            vid_pid_ayristir(r"HID\VID_046D&PID_C31C"),
            Some((0x046D, 0xC31C))
        );
        assert_eq!(vid_pid_ayristir("VID_ZZZZ&PID_0001"), None);
        assert_eq!(vid_pid_ayristir(""), None);
    }

    #[test]
    fn pnp_json_ingilizce_ve_turkce_alanlarla_ayristirilir() {
        let json = r#"[
            {"Name":"Samsung Android ADB Interface","Manufacturer":"Samsung Electronics Co., Ltd.","Status":"OK","PNPDeviceID":"USB\\VID_04E8&PID_6860&MI_02\\6&1","Service":"WinUSB"},
            {"Ad":"Pixel ADB Interface","Üretici":"Google Inc.","Durum":"OK","PNPDeviceID":"USB\\VID_18D1&PID_4EE7&MI_01\\7&2","Hizmet":"WinUSB"},
            {"Name":"Klavye","Manufacturer":"Logitech","Status":"OK","PNPDeviceID":"HID\\VID_046D&PID_C31C","Service":"kbdhid"},
            {"Name":"Eksik Kimlik","PNPDeviceID":"ROOT\\USB\\0000"}
        ]"#;
        let cihazlar = pnp_json_ayristir(json).unwrap();
        assert_eq!(cihazlar.len(), 2);
        assert_eq!(cihazlar[0].vid, 0x04E8);
        assert_eq!(cihazlar[0].pid, 0x6860);
        assert_eq!(cihazlar[0].surucu, "WinUSB");
        assert!(cihazlar[0].adb_arayuzu);
        assert_eq!(cihazlar[1].ad, "Pixel ADB Interface");
        assert_eq!(cihazlar[1].uretici, "Google Inc.");
        assert_eq!(cihazlar[1].durum, "OK");
        assert_eq!(pnp_json_ayristir("").unwrap(), Vec::new());
        assert!(pnp_json_ayristir("{bozuk").is_err());
    }

    #[test]
    fn tek_pnp_nesnesi_ayristirilir() {
        let json = r#"{"Name":"Telefon","Manufacturer":"Üretici","Status":"OK","PNPDeviceID":"USB\\VID_2717&PID_FF48\\abc","Service":"usbccgp"}"#;
        let cihazlar = pnp_json_ayristir(json).unwrap();
        assert_eq!(cihazlar.len(), 1);
        assert_eq!(cihazlar[0].vid, 0x2717);
        assert!(!cihazlar[0].adb_arayuzu);
    }

    #[test]
    fn imza_jsonu_gecerliligi_denetlenir() {
        assert!(imza_json_gecerli_mi(IMZA_GECERLI));
        assert!(imza_json_gecerli_mi(
            r#"{"Status":0,"SignerCertificate":{}}"#
        ));
        assert!(!imza_json_gecerli_mi(IMZA_GECERSIZ));
        assert!(!imza_json_gecerli_mi(
            r#"{"Status":2,"SignerCertificate":null}"#
        ));
        assert!(!imza_json_gecerli_mi(r#"{"Status":"NotTrusted"}"#));
        assert!(!imza_json_gecerli_mi("{bozuk"));
    }

    #[test]
    fn durumu_tara_rapor_uretir() {
        let json = r#"{"Name":"Samsung Android ADB Interface","Manufacturer":"Samsung","Status":"OK","PNPDeviceID":"USB\\VID_04E8&PID_6860\\1","Service":"WinUSB"}"#;
        let (servis, calistirici) = test_servisi(vec![cevap(json)], false);
        let rapor = servis.durumu_tara().unwrap();
        assert!(rapor.adb_hazir);
        assert_eq!(rapor.cihazlar.len(), 1);
        assert!(rapor.oneriler.iter().any(|oneri| oneri.contains("Samsung")));
        let cagri = cagrilar(&calistirici);
        assert_eq!(cagri[0].0, "powershell");
        let betik = cagri[0].1.last().unwrap();
        assert!(betik.contains("Get-CimInstance Win32_PnPEntity"));
        assert!(betik.contains("ConvertTo-Json -Compress"));
        assert!(!betik.contains("Get-PnpDevice"));
    }

    #[test]
    fn durumu_tara_hatasini_turkce_bildirir() {
        let mut hatali = cevap("");
        hatali.kod = 1;
        hatali.stderr = "Erişim engellendi.".into();
        let (servis, _calistirici) = test_servisi(vec![hatali], false);
        let hata = servis.durumu_tara().unwrap_err().to_string();
        assert!(hata.contains("USB cihaz durumu okunamadı"));
        assert!(hata.contains("yönetici izni"));
    }

    #[test]
    fn inf_kur_pnputil_argumanlari_ve_zaman_asimi() {
        let (_dizin, inf, kanonik) = gecici_dosya("android_winusb.inf", b"[Version]\n");
        let (servis, calistirici) = test_servisi(
            vec![
                cevap(IMZA_GECERLI),
                cevap("Driver package added successfully."),
            ],
            true,
        );
        let sonuc = servis.inf_kur(&inf).unwrap();
        assert!(sonuc.basarili);
        assert_eq!(sonuc.imza, "Valid");
        let cagri = cagrilar(&calistirici);
        assert_eq!(cagri.len(), 2);
        assert!(cagri[0]
            .1
            .last()
            .unwrap()
            .contains("Get-AuthenticodeSignature"));
        assert!(!cagri[0].1.last().unwrap().contains(r"\\?\"));
        assert_eq!(cagri[1].0, "pnputil.exe");
        assert_eq!(
            cagri[1].1,
            vec![
                "/add-driver".to_string(),
                komut_metni(&kanonik),
                "/install".to_string()
            ]
        );
        assert!(!cagri[1].1[1].starts_with(r"\\?\"));
        assert_eq!(cagri[1].2, Duration::from_secs(300));
    }

    #[test]
    fn inf_kur_yonetici_degilse_komut_calistirmaz() {
        let (servis, calistirici) = test_servisi(Vec::new(), false);
        let hata = servis
            .inf_kur(Path::new(r"C:\surucu\android_winusb.inf"))
            .unwrap_err();
        assert!(matches!(hata, Hata::Yetki(_)));
        assert!(cagrilar(&calistirici).is_empty());
    }

    #[test]
    fn inf_kur_gecersiz_imzayla_kurmaz() {
        let (_dizin, inf, _kanonik) = gecici_dosya("surucu.inf", b"[Version]\n");
        let (servis, calistirici) = test_servisi(vec![cevap(IMZA_GECERSIZ)], true);
        let sonuc = servis.inf_kur(&inf).unwrap();
        assert!(!sonuc.basarili);
        assert_eq!(sonuc.imza, "NotSigned");
        assert_eq!(cagrilar(&calistirici).len(), 1);
    }

    #[test]
    fn samsung_kur_sessiz_calisir_ve_imza_ister() {
        let (_dizin, exe, kanonik) = gecici_dosya("Samsung_USB_Driver.exe", b"exe");
        let (servis, calistirici) = test_servisi(vec![cevap(IMZA_GECERLI), cevap("")], true);
        let sonuc = servis.samsung_kur(&exe).unwrap();
        assert!(sonuc.basarili);
        let cagri = cagrilar(&calistirici);
        assert_eq!(cagri[1].0, komut_metni(&kanonik));
        assert_eq!(cagri[1].1, vec!["/S".to_string()]);
        let (gecersiz_servis, gecersiz_calistirici) =
            test_servisi(vec![cevap(IMZA_GECERSIZ)], true);
        assert!(gecersiz_servis.samsung_kur(&exe).is_err());
        assert_eq!(cagrilar(&gecersiz_calistirici).len(), 1);
    }

    #[test]
    fn libwdi_argumanlari_dogru_kurulur() {
        let (_dizin, arac, _kanonik) = gecici_dosya("wdi-simple.exe", b"exe");
        let (servis, calistirici) = test_servisi(vec![cevap(IMZA_GECERLI), cevap("Success")], true);
        let sonuc = servis.libwdi_kur(&arac, 0x04E8, 0x6860).unwrap();
        assert!(sonuc.basarili);
        let cagri = cagrilar(&calistirici);
        assert_eq!(
            cagri[1].1,
            vec![
                "--vid".to_string(),
                "0x04E8".to_string(),
                "--pid".to_string(),
                "0x6860".to_string(),
                "--type".to_string(),
                "0".to_string(),
                "--name".to_string(),
                "Android ADB Interface".to_string(),
                "--silent".to_string()
            ]
        );
    }

    #[test]
    fn libwdi_araci_yoksa_desteklenmiyor_doner() {
        let dizin = tempfile::tempdir().unwrap();
        let (servis, calistirici) = test_servisi(Vec::new(), true);
        let hata = servis
            .libwdi_kur(&dizin.path().join("wdi-simple.exe"), 1, 2)
            .unwrap_err();
        assert!(matches!(hata, Hata::Desteklenmiyor(_)));
        assert!(cagrilar(&calistirici).is_empty());
    }

    #[test]
    fn google_usb_akisi_indirir_acar_ve_inf_kurar() {
        let dizin = tempfile::tempdir().unwrap();
        let zip = dizin.path().join("google-usb.zip");
        std::fs::write(&zip, b"PK").unwrap();
        let acilan = dizin.path().join("google-usb").join("usb_driver");
        std::fs::create_dir_all(&acilan).unwrap();
        std::fs::write(acilan.join(GOOGLE_INF_ADI), b"[Version]\n").unwrap();
        let (servis, calistirici) = test_servisi(
            vec![
                cevap(""),
                cevap(""),
                cevap(IMZA_GECERLI),
                cevap("Driver package added successfully."),
            ],
            true,
        );
        let sonuc = servis.google_usb_indir_kur(&zip).unwrap();
        assert!(sonuc.basarili);
        let cagri = cagrilar(&calistirici);
        assert!(cagri[0].1.last().unwrap().contains(GOOGLE_USB_DRIVER_URL));
        assert!(cagri[0]
            .1
            .last()
            .unwrap()
            .contains("Invoke-WebRequest -UseBasicParsing"));
        assert!(cagri[1].1.last().unwrap().contains("Expand-Archive"));
        assert_eq!(cagri[3].0, "pnputil.exe");
        assert!(GOOGLE_USB_DRIVER_URL.starts_with("https://"));
    }

    #[test]
    fn google_usb_gecersiz_hedefi_reddeder() {
        let dizin = tempfile::tempdir().unwrap();
        let (servis, calistirici) = test_servisi(Vec::new(), true);
        assert!(servis
            .google_usb_indir_kur(&dizin.path().join("surucu.inf"))
            .is_err());
        assert!(cagrilar(&calistirici).is_empty());
    }
}
