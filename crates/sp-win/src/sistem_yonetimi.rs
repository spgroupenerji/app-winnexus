use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::error::{Hata, Sonuc};
use sp_core::sistem_yonetimi::serde_json::{self, Map, Value};
use sp_core::sistem_yonetimi::{
    baslangic_yedekleri_ayristir, baslangic_yedekleri_metne, kimlik_olustur, yedek_cikar,
    yedek_dosya_adi, yedek_ekle, BaslangicKonumu, BaslangicOgesi, BaslangicYedegi, GorevOgesi,
    ServisOgesi, SurecOgesi,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY, KEY_WRITE,
    REG_EXPAND_SZ, REG_SZ,
};
use winreg::RegKey;

const RUN_YOLU: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUNONCE_YOLU: &str = r"Software\Microsoft\Windows\CurrentVersion\RunOnce";
const BASLANGIC_MENU_YOLU: &str = r"Microsoft\Windows\Start Menu\Programs\Startup";
const BASLANGIC_YEDEK_DOSYASI: &str = "baslangic_yedek.json";
const BASLANGIC_ALT_KLASORU: &str = "baslangic";
const KONUMLAR: [BaslangicKonumu; 7] = [
    BaslangicKonumu::HkcuRun,
    BaslangicKonumu::HkcuRunOnce,
    BaslangicKonumu::HklmRun,
    BaslangicKonumu::HklmRunOnce,
    BaslangicKonumu::HklmRun32,
    BaslangicKonumu::HkcuBaslangic,
    BaslangicKonumu::OrtakBaslangic,
];
const KISA_ZAMAN_ASIMI: Duration = Duration::from_secs(30);
const TARAMA_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const ISLEM_ZAMAN_ASIMI: Duration = Duration::from_secs(180);
const CIKTI_SINIRI: usize = 2000;
const SERVIS_ADI_AZAMI: usize = 256;
const GOREV_METNI_AZAMI: usize = 512;
const BAYT_MEGABAYT: u64 = 1024 * 1024;
const SC_ZATEN_CALISIYOR: i32 = 1056;
const SC_ZATEN_DURDU: i32 = 1062;
const GOREV_YOLU_VARSAYILAN: &str = "\\";

const SERVIS_BETIGI: &str = "$ErrorActionPreference='SilentlyContinue'; try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}; Get-Service | Select-Object @{Name='Ad';Expression={$_.Name}},@{Name='GorunenAd';Expression={$_.DisplayName}},@{Name='Durum';Expression={$_.Status.ToString()}},@{Name='Baslangic';Expression={$_.StartType.ToString()}} | ConvertTo-Json -Compress";

const GOREV_BETIGI: &str = "$ErrorActionPreference='SilentlyContinue'; try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}; Get-ScheduledTask | Select-Object @{Name='Ad';Expression={$_.TaskName}},@{Name='Yol';Expression={$_.TaskPath}},@{Name='Durum';Expression={$_.State.ToString()}},@{Name='Sonraki';Expression={(Get-ScheduledTaskInfo -TaskName $_.TaskName -TaskPath $_.TaskPath).NextRunTime.ToString('yyyy-MM-dd HH:mm')}} | ConvertTo-Json -Compress";

const SUREC_BETIGI: &str = "$ErrorActionPreference='SilentlyContinue'; try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}; Get-Process | Select-Object ProcessName,Id,WorkingSet64,CPU,@{Name='StartTime';Expression={if ($_.StartTime) { $_.StartTime.ToString('yyyy-MM-dd HH:mm:ss') } else { $null }}} -ErrorAction SilentlyContinue | ConvertTo-Json -Compress";

#[derive(Clone)]
pub struct SistemYonetimiServisi {
    calistirici: Arc<dyn KomutCalistirici>,
    yedek_dizini: PathBuf,
}

impl SistemYonetimiServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>, yedek_dizini: PathBuf) -> Self {
        Self {
            calistirici,
            yedek_dizini,
        }
    }

    pub fn baslangic_ogeleri(&self) -> Sonuc<Vec<BaslangicOgesi>> {
        let mut ogeler = Vec::new();
        let mut hatalar = Vec::new();
        for konum in KONUMLAR {
            if let Some((kok, yol, gorunum)) = kayit_konumu(&konum) {
                match anahtar_ac(kok, yol, gorunum, false) {
                    Ok(anahtar) => ogeler.extend(kayit_ogeleri(&anahtar, &konum)),
                    Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => {}
                    Err(hata) => hatalar.push(format!("{}: {hata}", konum.metin())),
                }
                continue;
            }
            match baslangic_klasoru(&konum) {
                Ok(Some(klasor)) => ogeler.extend(klasor_ogeleri(&klasor, &konum)),
                Ok(None) => {}
                Err(hata) => hatalar.push(hata.to_string()),
            }
        }
        for yedek in self.yedekleri_oku()? {
            if ogeler.iter().any(|oge| oge.kimlik == yedek.kimlik) {
                continue;
            }
            ogeler.push(BaslangicOgesi {
                kimlik: yedek.kimlik,
                ad: yedek.ad,
                komut: yedek.komut,
                konum: yedek.konum,
                etkin: false,
            });
        }
        if ogeler.is_empty() && !hatalar.is_empty() {
            return Err(Hata::Registry(hatalar.join("; ")));
        }
        for hata in hatalar {
            tracing::warn!("Başlangıç taraması: {hata}");
        }
        ogeler.sort_by(|a, b| {
            a.ad.to_lowercase()
                .cmp(&b.ad.to_lowercase())
                .then_with(|| a.kimlik.cmp(&b.kimlik))
        });
        Ok(ogeler)
    }

    pub fn baslangic_etkinlestir(&self, oge: &BaslangicOgesi) -> Sonuc<()> {
        let mut yedekler = self.yedekleri_oku()?;
        let Some(yedek) = yedek_cikar(&mut yedekler, &oge.kimlik) else {
            return Ok(());
        };
        self.yedegi_geri_yaz(&yedek)?;
        self.yedekleri_yaz(&yedekler)
    }

    pub fn baslangic_devre_disi(&self, oge: &BaslangicOgesi) -> Sonuc<()> {
        let mut yedekler = self.yedekleri_oku()?;
        if !self.oge_var_mi(oge) {
            return if yedekler.iter().any(|yedek| yedek.kimlik == oge.kimlik) {
                Ok(())
            } else {
                Err(Hata::Beklenmeyen(format!("{} öğesi bulunamadı.", oge.ad)))
            };
        }
        let yedek = self.ogeyi_kaldir(oge)?;
        yedek_ekle(&mut yedekler, yedek.clone());
        if let Err(hata) = self.yedekleri_yaz(&yedekler) {
            if let Err(geri) = self.yedegi_geri_yaz(&yedek) {
                tracing::error!("Başlangıç öğesi geri alınamadı: {geri}");
            }
            return Err(hata);
        }
        Ok(())
    }

    pub fn servisler(&self) -> Sonuc<Vec<ServisOgesi>> {
        let sonuc = self.calistir_powershell(SERVIS_BETIGI, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Servis listesi alınamadı: {}",
                hata_metni(&sonuc)
            )));
        }
        servisleri_ayristir(&sonuc.stdout)
    }

    pub fn servis_baslat(&self, ad: &str) -> Sonuc<()> {
        servis_adini_dogrula(ad)?;
        let sonuc = self.calistirici.calistir(
            "sc.exe",
            &["start".to_string(), ad.to_string()],
            ISLEM_ZAMAN_ASIMI,
        )?;
        sonuc_kontrol(
            sonuc,
            &format!("{ad} servisi başlatılamadı"),
            &[SC_ZATEN_CALISIYOR],
        )
    }

    pub fn servis_durdur(&self, ad: &str) -> Sonuc<()> {
        servis_adini_dogrula(ad)?;
        let sonuc = self.calistirici.calistir(
            "sc.exe",
            &["stop".to_string(), ad.to_string()],
            ISLEM_ZAMAN_ASIMI,
        )?;
        sonuc_kontrol(
            sonuc,
            &format!("{ad} servisi durdurulamadı"),
            &[SC_ZATEN_DURDU],
        )
    }

    pub fn servis_baslangic_ayarla(&self, ad: &str, tur: &str) -> Sonuc<()> {
        servis_adini_dogrula(ad)?;
        let tur = tur.trim().to_ascii_lowercase();
        if !matches!(tur.as_str(), "auto" | "demand" | "disabled") {
            return Err(Hata::Dogrulama(
                "Servis başlangıç türü auto, demand veya disabled olmalı.".into(),
            ));
        }
        let sonuc = self.calistirici.calistir(
            "sc.exe",
            &[
                "config".to_string(),
                ad.to_string(),
                "start=".to_string(),
                tur,
            ],
            ISLEM_ZAMAN_ASIMI,
        )?;
        sonuc_kontrol(
            sonuc,
            &format!("{ad} servisinin başlangıç türü değiştirilemedi"),
            &[],
        )
    }

    pub fn gorevler(&self) -> Sonuc<Vec<GorevOgesi>> {
        let sonuc = self.calistir_powershell(GOREV_BETIGI, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Zamanlanmış görevler alınamadı: {}",
                hata_metni(&sonuc)
            )));
        }
        gorevleri_ayristir(&sonuc.stdout)
    }

    pub fn gorev_calistir(&self, ad: &str, yol: &str) -> Sonuc<()> {
        let (ad, yol) = gorev_hedefi(ad, yol)?;
        self.ps_mutasyon(
            format!("Start-ScheduledTask -TaskName '{ad}' -TaskPath '{yol}'"),
            "Görev başlatılamadı",
        )
    }

    pub fn gorev_etkinlestir(&self, ad: &str, yol: &str, etkin: bool) -> Sonuc<()> {
        let (ad, yol) = gorev_hedefi(ad, yol)?;
        let eylem = if etkin {
            "Enable-ScheduledTask"
        } else {
            "Disable-ScheduledTask"
        };
        let islem = if etkin {
            "Görev etkinleştirilemedi"
        } else {
            "Görev devre dışı bırakılamadı"
        };
        self.ps_mutasyon(
            format!("{eylem} -TaskName '{ad}' -TaskPath '{yol}' | Out-Null"),
            islem,
        )
    }

    pub fn surecler(&self) -> Sonuc<Vec<SurecOgesi>> {
        let sonuc = self.calistir_powershell(SUREC_BETIGI, TARAMA_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Süreç listesi alınamadı: {}",
                hata_metni(&sonuc)
            )));
        }
        surecleri_ayristir(&sonuc.stdout)
    }

    pub fn surec_sonlandir(&self, pid: u32) -> Sonuc<()> {
        if pid <= 4 || pid == std::process::id() {
            return Err(Hata::Dogrulama(format!(
                "{pid} numaralı süreç sonlandırılamaz: sistem veya uygulamanın kendi süreci."
            )));
        }
        let sonuc = self.calistirici.calistir(
            "taskkill.exe",
            &["/PID".to_string(), pid.to_string(), "/F".to_string()],
            KISA_ZAMAN_ASIMI,
        )?;
        sonuc_kontrol(
            sonuc,
            &format!("{pid} numaralı süreç sonlandırılamadı"),
            &[],
        )
    }

    fn calistir_powershell(&self, betik: &str, zaman_asimi: Duration) -> Sonuc<KomutSonucu> {
        self.calistirici.calistir(
            "powershell.exe",
            &[
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                betik.to_string(),
            ],
            zaman_asimi,
        )
    }

    fn ps_mutasyon(&self, cekirdek: String, islem: &str) -> Sonuc<()> {
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ {cekirdek}; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        );
        let sonuc = self.calistir_powershell(&betik, ISLEM_ZAMAN_ASIMI)?;
        sonuc_kontrol(sonuc, islem, &[])
    }

    fn yedek_dosyasi(&self) -> PathBuf {
        self.yedek_dizini.join(BASLANGIC_YEDEK_DOSYASI)
    }

    fn yedek_klasoru(&self) -> PathBuf {
        self.yedek_dizini.join(BASLANGIC_ALT_KLASORU)
    }

    fn yedekleri_oku(&self) -> Sonuc<Vec<BaslangicYedegi>> {
        match std::fs::read_to_string(self.yedek_dosyasi()) {
            Ok(metin) => baslangic_yedekleri_ayristir(&metin),
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(hata) => Err(Hata::Beklenmeyen(format!(
                "Başlangıç yedeği okunamadı: {hata}"
            ))),
        }
    }

    fn yedekleri_yaz(&self, yedekler: &[BaslangicYedegi]) -> Sonuc<()> {
        std::fs::create_dir_all(&self.yedek_dizini)
            .map_err(|hata| Hata::Beklenmeyen(format!("Yedek dizini oluşturulamadı: {hata}")))?;
        let metin = baslangic_yedekleri_metne(yedekler)?;
        let dosya = self.yedek_dosyasi();
        let gecici = dosya.with_extension("json.tmp");
        std::fs::write(&gecici, metin)
            .map_err(|hata| Hata::Beklenmeyen(format!("Başlangıç yedeği yazılamadı: {hata}")))?;
        std::fs::rename(&gecici, &dosya)
            .map_err(|hata| Hata::Beklenmeyen(format!("Başlangıç yedeği kaydedilemedi: {hata}")))
    }

    fn oge_var_mi(&self, oge: &BaslangicOgesi) -> bool {
        if let Some((kok, yol, gorunum)) = kayit_konumu(&oge.konum) {
            return anahtar_ac(kok, yol, gorunum, false)
                .ok()
                .map(|anahtar| {
                    let deger: std::io::Result<String> = anahtar.get_value(&oge.ad);
                    deger.is_ok()
                })
                .unwrap_or(false);
        }
        baslangic_klasoru(&oge.konum)
            .ok()
            .flatten()
            .and_then(|klasor| klasor_dosyasi(&klasor, &oge.ad))
            .is_some()
    }

    fn ogeyi_kaldir(&self, oge: &BaslangicOgesi) -> Sonuc<BaslangicYedegi> {
        if let Some((kok, yol, gorunum)) = kayit_konumu(&oge.konum) {
            let anahtar = anahtar_ac(kok, yol, gorunum, true).map_err(|hata| {
                Hata::Registry(format!("{} anahtarı açılamadı: {hata}", oge.konum.metin()))
            })?;
            let komut: String = anahtar
                .get_value(&oge.ad)
                .map_err(|hata| Hata::Registry(format!("{} değeri okunamadı: {hata}", oge.ad)))?;
            anahtar
                .delete_value(&oge.ad)
                .map_err(|hata| Hata::Registry(format!("{} değeri silinemedi: {hata}", oge.ad)))?;
            return Ok(BaslangicYedegi {
                kimlik: oge.kimlik.clone(),
                ad: oge.ad.clone(),
                komut: komut.trim().to_string(),
                konum: oge.konum.clone(),
            });
        }

        let klasor = baslangic_klasoru(&oge.konum)?
            .ok_or_else(|| Hata::Beklenmeyen("Başlangıç konumu tanınmadı.".into()))?;
        let dosya = klasor_dosyasi(&klasor, &oge.ad).ok_or_else(|| {
            Hata::Beklenmeyen(format!(
                "{} öğesi {} içinde bulunamadı.",
                oge.ad,
                oge.konum.metin()
            ))
        })?;
        let ad = dosya
            .file_name()
            .map(|ad| ad.to_string_lossy().to_string())
            .unwrap_or_else(|| oge.ad.clone());
        let hedef = self.yedek_klasoru().join(yedek_dosya_adi(&oge.kimlik));
        if hedef.exists() {
            std::fs::remove_file(&hedef).map_err(|hata| {
                Hata::Beklenmeyen(format!("Eski yedek dosya silinemedi: {hata}"))
            })?;
        }
        std::fs::create_dir_all(self.yedek_klasoru())
            .map_err(|hata| Hata::Beklenmeyen(format!("Yedek klasörü oluşturulamadı: {hata}")))?;
        tasi(&dosya, &hedef)?;
        Ok(BaslangicYedegi {
            kimlik: oge.kimlik.clone(),
            ad,
            komut: dosya.to_string_lossy().to_string(),
            konum: oge.konum.clone(),
        })
    }

    fn yedegi_geri_yaz(&self, yedek: &BaslangicYedegi) -> Sonuc<()> {
        if let Some((kok, yol, gorunum)) = kayit_konumu(&yedek.konum) {
            let mut bayrak = KEY_READ | KEY_WRITE;
            bayrak |= if gorunum == 32 {
                KEY_WOW64_32KEY
            } else {
                KEY_WOW64_64KEY
            };
            let (anahtar, _) = RegKey::predef(kok)
                .create_subkey_with_flags(yol, bayrak)
                .map_err(|hata| {
                    Hata::Registry(format!(
                        "{} anahtarı açılamadı: {hata}",
                        yedek.konum.metin()
                    ))
                })?;
            return anahtar.set_value(&yedek.ad, &yedek.komut).map_err(|hata| {
                Hata::Registry(format!("{} değeri geri yazılamadı: {hata}", yedek.ad))
            });
        }

        let klasor = baslangic_klasoru(&yedek.konum)?
            .ok_or_else(|| Hata::Beklenmeyen("Başlangıç konumu tanınmadı.".into()))?;
        if yedek.ad.is_empty() || yedek.ad.contains('\\') || yedek.ad.contains('/') {
            return Err(Hata::Beklenmeyen("Yedek dosya adı geçersiz.".into()));
        }
        let kaynak = self.yedek_klasoru().join(yedek_dosya_adi(&yedek.kimlik));
        let hedef = klasor.join(&yedek.ad);
        if hedef.is_file() {
            return Ok(());
        }
        if !kaynak.is_file() {
            return Err(Hata::Beklenmeyen(format!(
                "{} için yedek dosya bulunamadı.",
                yedek.ad
            )));
        }
        std::fs::create_dir_all(&klasor).map_err(|hata| {
            Hata::Beklenmeyen(format!("Başlangıç klasörü oluşturulamadı: {hata}"))
        })?;
        tasi(&kaynak, &hedef)
    }
}

fn kayit_konumu(konum: &BaslangicKonumu) -> Option<(winreg::HKEY, &'static str, u32)> {
    match konum {
        BaslangicKonumu::HkcuRun => Some((HKEY_CURRENT_USER, RUN_YOLU, 64)),
        BaslangicKonumu::HkcuRunOnce => Some((HKEY_CURRENT_USER, RUNONCE_YOLU, 64)),
        BaslangicKonumu::HklmRun => Some((HKEY_LOCAL_MACHINE, RUN_YOLU, 64)),
        BaslangicKonumu::HklmRunOnce => Some((HKEY_LOCAL_MACHINE, RUNONCE_YOLU, 64)),
        BaslangicKonumu::HklmRun32 => Some((HKEY_LOCAL_MACHINE, RUN_YOLU, 32)),
        BaslangicKonumu::HkcuBaslangic | BaslangicKonumu::OrtakBaslangic => None,
    }
}

fn baslangic_klasoru(konum: &BaslangicKonumu) -> Sonuc<Option<PathBuf>> {
    let degisken = match konum {
        BaslangicKonumu::HkcuBaslangic => "APPDATA",
        BaslangicKonumu::OrtakBaslangic => "PROGRAMDATA",
        _ => return Ok(None),
    };
    let kok = std::env::var_os(degisken)
        .ok_or_else(|| Hata::Beklenmeyen(format!("{degisken} ortam değişkeni bulunamadı.")))?;
    Ok(Some(PathBuf::from(kok).join(BASLANGIC_MENU_YOLU)))
}

fn anahtar_ac(kok: winreg::HKEY, yol: &str, gorunum: u32, yazma: bool) -> std::io::Result<RegKey> {
    let mut bayrak = if yazma {
        KEY_READ | KEY_WRITE
    } else {
        KEY_READ
    };
    bayrak |= if gorunum == 32 {
        KEY_WOW64_32KEY
    } else {
        KEY_WOW64_64KEY
    };
    RegKey::predef(kok).open_subkey_with_flags(yol, bayrak)
}

fn kayit_ogeleri(anahtar: &RegKey, konum: &BaslangicKonumu) -> Vec<BaslangicOgesi> {
    anahtar
        .enum_values()
        .flatten()
        .filter_map(|(ad, deger)| {
            if !matches!(deger.vtype, REG_SZ | REG_EXPAND_SZ) {
                return None;
            }
            let komut = deger.to_string().trim().to_string();
            if komut.is_empty() {
                return None;
            }
            Some(BaslangicOgesi {
                kimlik: kimlik_olustur(konum, &ad),
                ad,
                komut,
                konum: konum.clone(),
                etkin: true,
            })
        })
        .collect()
}

fn klasor_ogeleri(klasor: &Path, konum: &BaslangicKonumu) -> Vec<BaslangicOgesi> {
    let Ok(girdiler) = std::fs::read_dir(klasor) else {
        return Vec::new();
    };
    girdiler
        .flatten()
        .filter_map(|girdi| {
            if !girdi.file_type().map(|tur| tur.is_file()).unwrap_or(false) {
                return None;
            }
            let ad = girdi.file_name().to_string_lossy().to_string();
            Some(BaslangicOgesi {
                kimlik: kimlik_olustur(konum, &ad),
                ad,
                komut: girdi.path().to_string_lossy().to_string(),
                konum: konum.clone(),
                etkin: true,
            })
        })
        .collect()
}

fn klasor_dosyasi(klasor: &Path, ad: &str) -> Option<PathBuf> {
    if ad.is_empty() || ad.contains('\\') || ad.contains('/') {
        return None;
    }
    let tam = klasor.join(ad);
    if tam.is_file() {
        return Some(tam);
    }
    let aranan = ad.to_ascii_lowercase();
    std::fs::read_dir(klasor)
        .ok()?
        .flatten()
        .find(|girdi| {
            girdi.file_type().map(|tur| tur.is_file()).unwrap_or(false)
                && girdi.file_name().to_string_lossy().to_ascii_lowercase() == aranan
        })
        .map(|girdi| girdi.path())
}

fn tasi(kaynak: &Path, hedef: &Path) -> Sonuc<()> {
    if std::fs::rename(kaynak, hedef).is_ok() {
        return Ok(());
    }
    std::fs::copy(kaynak, hedef)
        .map_err(|hata| Hata::Beklenmeyen(format!("Dosya kopyalanamadı: {hata}")))?;
    std::fs::remove_file(kaynak)
        .map_err(|hata| Hata::Beklenmeyen(format!("Dosya taşınamadı: {hata}")))
}

fn hata_metni(sonuc: &KomutSonucu) -> String {
    let cikti = if sonuc.stderr.trim().is_empty() {
        sonuc.stdout.trim()
    } else {
        sonuc.stderr.trim()
    };
    cikti.chars().take(CIKTI_SINIRI).collect()
}

fn sonuc_kontrol(sonuc: KomutSonucu, islem: &str, kabul_edilen_kodlar: &[i32]) -> Sonuc<()> {
    if sonuc.kod == 0 || kabul_edilen_kodlar.contains(&sonuc.kod) {
        return Ok(());
    }
    let metin = hata_metni(&sonuc);
    Err(Hata::DisArac(if metin.is_empty() {
        format!("{islem} (kod {}).", sonuc.kod)
    } else {
        format!("{islem} (kod {}): {metin}", sonuc.kod)
    }))
}

fn servis_adini_dogrula(ad: &str) -> Sonuc<()> {
    let gecerli = !ad.is_empty()
        && ad.len() <= SERVIS_ADI_AZAMI
        && ad.chars().all(|karakter| {
            karakter.is_ascii_alphanumeric() || matches!(karakter, '.' | '_' | '-')
        });
    if gecerli {
        Ok(())
    } else {
        Err(Hata::Dogrulama(format!("Geçersiz servis adı: {ad}")))
    }
}

fn gorev_metnini_dogrula(deger: &str, alan: &str) -> Sonuc<()> {
    if deger.len() > GOREV_METNI_AZAMI {
        return Err(Hata::Dogrulama(format!("{alan} çok uzun.")));
    }
    if deger
        .chars()
        .any(|karakter| matches!(karakter, '\'' | '"' | '\r' | '\n' | '\0'))
    {
        return Err(Hata::Dogrulama(format!(
            "{alan} geçersiz karakter içeriyor: tırnak veya satır sonu kullanılamaz."
        )));
    }
    Ok(())
}

fn gorev_hedefi(ad: &str, yol: &str) -> Sonuc<(String, String)> {
    let ad = ad.trim();
    if ad.is_empty() {
        return Err(Hata::Dogrulama("Görev adı boş olamaz.".into()));
    }
    gorev_metnini_dogrula(ad, "Görev adı")?;
    let yol = yol.trim();
    let yol = if yol.is_empty() {
        GOREV_YOLU_VARSAYILAN
    } else {
        yol
    };
    gorev_metnini_dogrula(yol, "Görev yolu")?;
    if !yol.starts_with('\\') {
        return Err(Hata::Dogrulama("Görev yolu '\\' ile başlamalı.".into()));
    }
    Ok((ad.to_string(), yol.to_string()))
}

fn kayitlar(json: &str) -> Sonuc<Vec<Map<String, Value>>> {
    let metin = json.trim();
    if metin.is_empty() {
        return Ok(Vec::new());
    }
    let deger: Value = serde_json::from_str(metin)
        .map_err(|hata| Hata::Beklenmeyen(format!("JSON çıktısı çözümlenemedi: {hata}")))?;
    Ok(match deger {
        Value::Array(kayitlar) => kayitlar
            .into_iter()
            .filter_map(|kayit| match kayit {
                Value::Object(nesne) => Some(nesne),
                _ => None,
            })
            .collect(),
        Value::Object(nesne) => vec![nesne],
        Value::Null => Vec::new(),
        _ => {
            return Err(Hata::Beklenmeyen(
                "JSON çıktısı beklenen biçimde değil.".into(),
            ))
        }
    })
}

fn alan_metni(nesne: &Map<String, Value>, ad: &str) -> Option<String> {
    match nesne.get(ad)? {
        Value::String(metin) => {
            let metin = metin.trim();
            (!metin.is_empty()).then(|| metin.to_string())
        }
        Value::Number(sayi) => Some(sayi.to_string()),
        Value::Bool(dogru) => Some(dogru.to_string()),
        _ => None,
    }
}

fn alan_sayisi(nesne: &Map<String, Value>, ad: &str) -> Option<f64> {
    match nesne.get(ad)? {
        Value::Number(sayi) => sayi.as_f64(),
        Value::String(metin) => {
            let metin = metin.trim();
            metin
                .parse::<f64>()
                .ok()
                .or_else(|| metin.replace(',', ".").parse::<f64>().ok())
        }
        _ => None,
    }
}

fn alan_tamsayi(nesne: &Map<String, Value>, ad: &str) -> Option<u64> {
    match nesne.get(ad)? {
        Value::Number(sayi) => sayi.as_u64().or_else(|| {
            sayi.as_f64()
                .filter(|deger| *deger >= 0.0)
                .map(|deger| deger as u64)
        }),
        Value::String(metin) => metin.trim().parse::<u64>().ok(),
        _ => None,
    }
}

pub fn servisleri_ayristir(json: &str) -> Sonuc<Vec<ServisOgesi>> {
    let mut servisler = Vec::new();
    for nesne in kayitlar(json)? {
        let Some(ad) = alan_metni(&nesne, "Ad") else {
            continue;
        };
        servisler.push(ServisOgesi {
            gorunen_ad: alan_metni(&nesne, "GorunenAd").unwrap_or_else(|| ad.clone()),
            durum: alan_metni(&nesne, "Durum").unwrap_or_default(),
            baslangic: alan_metni(&nesne, "Baslangic").unwrap_or_default(),
            ad,
        });
    }
    Ok(servisler)
}

pub fn gorevleri_ayristir(json: &str) -> Sonuc<Vec<GorevOgesi>> {
    let mut gorevler = Vec::new();
    for nesne in kayitlar(json)? {
        let Some(ad) = alan_metni(&nesne, "Ad") else {
            continue;
        };
        let yol = alan_metni(&nesne, "Yol").unwrap_or_else(|| GOREV_YOLU_VARSAYILAN.to_string());
        gorevler.push(GorevOgesi {
            kimlik: format!("{yol}|{ad}"),
            ad,
            yol,
            durum: alan_metni(&nesne, "Durum").unwrap_or_default(),
            sonraki: alan_metni(&nesne, "Sonraki").unwrap_or_default(),
        });
    }
    Ok(gorevler)
}

pub fn surecleri_ayristir(json: &str) -> Sonuc<Vec<SurecOgesi>> {
    let mut surecler = Vec::new();
    for nesne in kayitlar(json)? {
        let Some(ad) = alan_metni(&nesne, "ProcessName") else {
            continue;
        };
        let Some(pid) = alan_tamsayi(&nesne, "Id").and_then(|deger| u32::try_from(deger).ok())
        else {
            continue;
        };
        surecler.push(SurecOgesi {
            ad,
            pid,
            bellek_mb: alan_tamsayi(&nesne, "WorkingSet64")
                .map(|bayt| bayt / BAYT_MEGABAYT)
                .unwrap_or(0),
            cpu_saniye: alan_sayisi(&nesne, "CPU"),
            kullanici: alan_metni(&nesne, "Kullanici"),
            baslama: alan_metni(&nesne, "StartTime"),
        });
    }
    Ok(surecler)
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct SahteCalistirici {
        cevaplar: Mutex<VecDeque<KomutSonucu>>,
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
            self.cevaplar
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| Hata::DisArac("test cevabı yok".into()))
        }
    }

    fn cevap(stdout: &str, kod: i32) -> KomutSonucu {
        KomutSonucu {
            komut: "test".into(),
            kod,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn servis(cevaplar: Vec<KomutSonucu>) -> (SistemYonetimiServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            cevaplar: Mutex::new(VecDeque::from(cevaplar)),
            cagrilar: Mutex::new(Vec::new()),
        });
        (
            SistemYonetimiServisi::yeni(calistirici.clone(), PathBuf::from("yok")),
            calistirici,
        )
    }

    #[test]
    fn servisleri_dizi_olarak_ayristirir() {
        let json = r#"[
            {"Ad":"Spooler","GorunenAd":"Yazdırma Biriktiricisi","Durum":"Running","Baslangic":"Automatic"},
            {"Ad":"Bilinmeyen"}
        ]"#;
        let servisler = servisleri_ayristir(json).unwrap();
        assert_eq!(servisler.len(), 2);
        assert_eq!(servisler[0].ad, "Spooler");
        assert_eq!(servisler[0].gorunen_ad, "Yazdırma Biriktiricisi");
        assert_eq!(servisler[0].durum, "Running");
        assert_eq!(servisler[1].gorunen_ad, "Bilinmeyen");
        assert_eq!(servisler[1].baslangic, "");
    }

    #[test]
    fn servisleri_tek_nesne_olarak_ayristirir() {
        let json = r#"{"Ad":"W32Time","GorunenAd":"Windows Saati","Durum":"Stopped","Baslangic":"Manual"}"#;
        let servisler = servisleri_ayristir(json).unwrap();
        assert_eq!(servisler.len(), 1);
        assert_eq!(servisler[0].ad, "W32Time");
    }

    #[test]
    fn gorevleri_yol_ve_kimlikle_ayristirir() {
        let json = r#"[{"Ad":"Temizlik","Yol":"\\Microsoft\\Windows\\Gorev","Durum":"Ready","Sonraki":"2026-11-01 03:00"}]"#;
        let gorevler = gorevleri_ayristir(json).unwrap();
        assert_eq!(gorevler.len(), 1);
        assert_eq!(gorevler[0].yol, "\\Microsoft\\Windows\\Gorev");
        assert_eq!(gorevler[0].sonraki, "2026-11-01 03:00");
        assert_eq!(gorevler[0].kimlik, "\\Microsoft\\Windows\\Gorev|Temizlik");
    }

    #[test]
    fn gorevlerde_bos_yol_kok_yol_olur() {
        let json = r#"{"Ad":"Tek","Yol":"","Durum":"Ready","Sonraki":null}"#;
        let gorevler = gorevleri_ayristir(json).unwrap();
        assert_eq!(gorevler[0].yol, "\\");
        assert_eq!(gorevler[0].sonraki, "");
    }

    #[test]
    fn surecleri_sayi_ve_metin_alanlariyla_ayristirir() {
        let json = r#"[
            {"ProcessName":"explorer","Id":1234,"WorkingSet64":104857600,"CPU":12.5,"StartTime":"2026-10-30 06:00:00"},
            {"ProcessName":"csrss","Id":"700","WorkingSet64":"52428800","CPU":null,"StartTime":null},
            {"ProcessName":"bozuk","Id":"pid","WorkingSet64":0}
        ]"#;
        let surecler = surecleri_ayristir(json).unwrap();
        assert_eq!(surecler.len(), 2);
        assert_eq!(surecler[0].bellek_mb, 100);
        assert_eq!(surecler[0].cpu_saniye, Some(12.5));
        assert_eq!(surecler[0].baslama.as_deref(), Some("2026-10-30 06:00:00"));
        assert_eq!(surecler[1].pid, 700);
        assert_eq!(surecler[1].bellek_mb, 50);
        assert!(surecler[1].cpu_saniye.is_none());
        assert!(surecler[1].baslama.is_none());
    }

    #[test]
    fn bos_ve_bozuk_json_toleransli() {
        assert!(servisleri_ayristir("  ").unwrap().is_empty());
        assert!(gorevleri_ayristir("null").unwrap().is_empty());
        assert!(surecleri_ayristir("{bozuk").is_err());
    }

    #[test]
    fn servis_baslangic_turu_sc_ile_ayarlanir() {
        let (servis, calistirici) = servis(vec![cevap("", 0)]);
        servis.servis_baslangic_ayarla("Spooler", "AUTO").unwrap();
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "sc.exe");
        assert_eq!(cagrilar[0].1.join(" "), "config Spooler start= auto");
    }

    #[test]
    fn gecersiz_servis_adi_ve_turu_reddedilir() {
        let (servis, calistirici) = servis(vec![]);
        assert!(servis.servis_baslat("Spooler; calc").is_err());
        assert!(servis
            .servis_baslangic_ayarla("Spooler", "otomatik")
            .is_err());
        assert!(servis.servis_durdur("").is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn zaten_calisan_servis_hata_vermez() {
        let (servis, _calistirici) = servis(vec![cevap("", SC_ZATEN_CALISIYOR)]);
        assert!(servis.servis_baslat("Spooler").is_ok());
    }

    #[test]
    fn surec_koruma_kurallari_uygulanir() {
        let (servis, calistirici) = servis(vec![cevap("", 0)]);
        assert!(servis.surec_sonlandir(0).is_err());
        assert!(servis.surec_sonlandir(4).is_err());
        assert!(servis.surec_sonlandir(std::process::id()).is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
        assert!(servis.surec_sonlandir(4321).is_ok());
        assert_eq!(
            calistirici.cagrilar.lock().unwrap()[0].1.join(" "),
            "/PID 4321 /F"
        );
    }

    #[test]
    fn gorev_komutlari_powershell_ile_kurulur() {
        let (servis, calistirici) = servis(vec![cevap("", 0), cevap("", 0)]);
        servis.gorev_calistir("Temizlik", "").unwrap();
        servis
            .gorev_etkinlestir("Temizlik", "\\Microsoft", false)
            .unwrap();
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "powershell.exe");
        assert!(cagrilar[0].1.contains(&"-Command".to_string()));
        let betik = cagrilar[0].1.last().unwrap();
        assert!(betik.contains("Start-ScheduledTask -TaskName 'Temizlik' -TaskPath '\\'"));
        assert!(cagrilar[1]
            .1
            .last()
            .unwrap()
            .contains("Disable-ScheduledTask"));
    }

    #[test]
    fn gorev_adinda_tirnak_reddedilir() {
        let (servis, calistirici) = servis(vec![]);
        assert!(servis.gorev_calistir("Test' -TaskPath '\\", "\\").is_err());
        assert!(servis.gorev_etkinlestir("Satır\nSonu", "\\", true).is_err());
        assert!(servis.gorev_calistir("Ad", "yolsuz").is_err());
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn servis_listesi_calistirici_ciktisindan_okunur() {
        let (servis, _calistirici) = servis(vec![cevap(
            r#"{"Ad":"Spooler","GorunenAd":"Yazdırma","Durum":"Running","Baslangic":"Automatic"}"#,
            0,
        )]);
        let liste = servis.servisler().unwrap();
        assert_eq!(liste.len(), 1);
        assert_eq!(liste[0].durum, "Running");
    }

    #[test]
    fn basarisiz_komut_dis_arac_hatasi_verir() {
        let (servis, _calistirici) = servis(vec![KomutSonucu {
            komut: "sc.exe".into(),
            kod: 5,
            stdout: String::new(),
            stderr: "Erişim engellendi.".into(),
        }]);
        let hata = servis.servis_durdur("Spooler").unwrap_err();
        assert!(matches!(hata, Hata::DisArac(_)));
        assert!(hata.to_string().contains("Erişim engellendi."));
    }
}
