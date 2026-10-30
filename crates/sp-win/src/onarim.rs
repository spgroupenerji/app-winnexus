// Onarım ve gelişmiş temizlik Windows uygulaması.
use crate::registry::WindowsAdaptoru;
use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::adaptor::SistemAdaptoru;
use sp_core::error::{Hata, Sonuc};
use sp_core::onarim::{
    geri_yukleme_aciklamasi_temizle, geri_yukleme_noktalari_ayristir,
    geri_yukleme_sirasi_gecerli_mi, geri_yukleme_surucu_gecerli_mi, guncelleme_gecmisi_ayristir,
    kategoriler, sorun_giderici_kimligi_gecerli_mi, GeriYuklemeNoktasi, GuncellemeKaydi,
    KategoriTarama, OnarimTemizlikSonucu,
};
use sp_core::temizleyici::yol_kok_altinda_mi;
use std::fs::{self, Metadata};
use std::os::windows::fs::MetadataExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const REPARSE_NOKTASI: u32 = 0x0000_0400;
const POWERSHELL_PROGRAMI: &str = "powershell";
const SFC_PROGRAMI: &str = "sfc.exe";
const DISM_PROGRAMI: &str = "Dism.exe";
const CHROME_KAYIT_YOLU: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\Google Chrome";
const UZUN_ZAMAN_ASIMI: Duration = Duration::from_secs(3600);
const ORTA_ZAMAN_ASIMI: Duration = Duration::from_secs(600);
const KISA_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const SFC_CIKTI_SINIRI: usize = 12000;
const CIKTI_SINIRI: usize = 8000;
const AZAMI_HATA_SAYISI: usize = 50;
const GUNCELLEME_GECMISI_BETIGI: &str = "Get-HotFix | Select-Object @{N='Kimlik';E={$_.HotFixID}},@{N='Aciklama';E={$_.Description}},@{N='Tarih';E={if($_.InstalledOn){$_.InstalledOn.ToString('yyyy-MM-dd')}else{''}}} | ConvertTo-Json -Compress";
const GERI_YUKLEME_BETIGI: &str = "Get-ComputerRestorePoint | Select-Object @{N='Sira';E={$_.SequenceNumber}},@{N='Aciklama';E={$_.Description}},@{N='Tarih';E={if($_.CreationTime){$_.CreationTime.ToString('yyyy-MM-dd HH:mm')}else{''}}} | ConvertTo-Json -Compress";

#[derive(Clone)]
pub struct OnarimServisi {
    calistirici: Arc<dyn KomutCalistirici>,
    yonetici_denetimi: fn() -> bool,
}

impl OnarimServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self {
            calistirici,
            yonetici_denetimi: || WindowsAdaptoru.yonetici_mi(),
        }
    }

    pub fn kategorileri_tara(&self) -> Sonuc<Vec<KategoriTarama>> {
        Ok(kategoriler()
            .into_iter()
            .map(|kategori| {
                let sayim = tarama_yap(&kategori_yollari(&kategori.kimlik));
                KategoriTarama {
                    kategori,
                    dosya_sayisi: sayim.dosya,
                    bayt: sayim.bayt,
                }
            })
            .collect())
    }

    pub fn kategori_temizle(&self, kimlik: &str) -> Sonuc<OnarimTemizlikSonucu> {
        if !kategoriler()
            .iter()
            .any(|kategori| kategori.kimlik == kimlik)
        {
            return Err(Hata::Dogrulama(format!(
                "Bilinmeyen temizlik kategorisi: {kimlik}"
            )));
        }
        match kimlik {
            "windows_update_indirmeleri" => self.windows_update_temizle(),
            "defender_tarama_gecmisi" => self.defender_gecmisi_temizle(),
            _ => Ok(dizinleri_temizle(&kategori_yollari(kimlik))),
        }
    }

    pub fn sfc_calistir(&self) -> Sonuc<String> {
        let sonuc =
            self.calistirici
                .calistir(SFC_PROGRAMI, &["/scannow".to_string()], UZUN_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "SFC taraması başarısız (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(cikti_metni(&sonuc, SFC_CIKTI_SINIRI))
    }

    pub fn dism_reset_base(&self) -> Sonuc<String> {
        let argumanlar = [
            "/Online",
            "/Cleanup-Image",
            "/StartComponentCleanup",
            "/ResetBase",
        ]
        .map(str::to_string);
        let sonuc = self
            .calistirici
            .calistir(DISM_PROGRAMI, &argumanlar, UZUN_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Bileşen temizliği başarısız (kod {}): {}",
                sonuc.kod,
                hata_ozeti(&sonuc)
            )));
        }
        Ok(cikti_metni(&sonuc, CIKTI_SINIRI))
    }

    pub fn appx_onar(&self, tam_ad: &str) -> Sonuc<String> {
        let tam_ad = tam_ad.trim();
        if !appx_tam_ad_gecerli_mi(tam_ad) {
            return Err(Hata::Dogrulama("Geçersiz Appx paket tam adı.".into()));
        }
        let betik = format!(
            "$ErrorActionPreference='Stop'; $p=Get-AppxPackage -Package '{tam_ad}'; if(-not $p){{throw 'Paket bulunamadı.'}}; Add-AppxPackage -DisableDevelopmentMode -Register (Join-Path $p.InstallLocation 'AppxManifest.xml')"
        );
        let sonuc = self.powershell(&betik, "Appx paketi onarılamadı", ORTA_ZAMAN_ASIMI)?;
        Ok(cikti_metni(&sonuc, CIKTI_SINIRI))
    }

    pub fn guncelleme_gecmisi(&self) -> Sonuc<Vec<GuncellemeKaydi>> {
        let sonuc = self.powershell(
            GUNCELLEME_GECMISI_BETIGI,
            "Güncelleme geçmişi okunamadı",
            KISA_ZAMAN_ASIMI,
        )?;
        guncelleme_gecmisi_ayristir(&sonuc.stdout)
    }

    pub fn guncelleme_gecmisi_temizle(&self) -> Sonuc<()> {
        let kok = sistem_koku()
            .ok_or_else(|| Hata::Beklenmeyen("SystemRoot değişkeni bulunamadı.".into()))?;
        let veri = ps_metni(&kok.join("SoftwareDistribution").join("DataStore"));
        let loglar = ps_metni(&kok.join("SoftwareDistribution").join("Logs"));
        // Güncelleme veritabanı yalnızca hizmetler durdurularak komutla sıfırlanır.
        let betik = format!(
            "$ErrorActionPreference='SilentlyContinue'; Stop-Service wuauserv,bits,cryptsvc -Force -ErrorAction SilentlyContinue; Remove-Item '{veri}' -Recurse -Force -ErrorAction SilentlyContinue; Remove-Item '{loglar}' -Recurse -Force -ErrorAction SilentlyContinue; Start-Service wuauserv,bits -ErrorAction SilentlyContinue; if(Test-Path -LiteralPath '{veri}'){{ [Console]::Error.WriteLine('Güncelleme veritabanı silinemedi; yönetici izni gerekiyor olabilir.'); exit 1 }}"
        );
        self.powershell(&betik, "Güncelleme geçmişi temizlenemedi", ORTA_ZAMAN_ASIMI)?;
        Ok(())
    }

    pub fn geri_yukleme_noktalari(&self) -> Sonuc<Vec<GeriYuklemeNoktasi>> {
        let sonuc = self.powershell(
            GERI_YUKLEME_BETIGI,
            "Geri yükleme noktaları okunamadı",
            KISA_ZAMAN_ASIMI,
        )?;
        geri_yukleme_noktalari_ayristir(&sonuc.stdout)
    }

    pub fn eski_geri_yukleme_noktalarini_sil(&self) -> Sonuc<u32> {
        let noktalar = self.geri_yukleme_noktalari()?;
        if noktalar.len() < 2 {
            return Ok(0);
        }
        let en_yuksek = noktalar
            .iter()
            .map(|nokta| nokta.sira)
            .max()
            .unwrap_or_default();
        let silinecekler: Vec<String> = noktalar
            .iter()
            .filter(|nokta| nokta.sira != en_yuksek)
            .map(|nokta| nokta.sira.to_string())
            .collect();
        if silinecekler.is_empty() {
            return Ok(0);
        }
        let betik = format!(
            "$ErrorActionPreference='Stop'; $sr=[wmiclass]'root\\default:SystemRestore'; $sayi=0; foreach($sira in @({})){{ $sr.DeleteRestorePoint($sira) | Out-Null; $sayi++ }}; Write-Output $sayi",
            silinecekler.join(",")
        );
        let sonuc = self.powershell(
            &betik,
            "Eski geri yükleme noktaları silinemedi",
            ORTA_ZAMAN_ASIMI,
        )?;
        son_sayi(&sonuc.stdout)
            .ok_or_else(|| Hata::DisArac("Silinen geri yükleme noktası sayısı okunamadı.".into()))
    }

    pub fn geri_yukleme_olustur(&self, aciklama: &str) -> Sonuc<String> {
        self.yonetici_kontrolu()?;
        let aciklama = geri_yukleme_aciklamasi_temizle(aciklama);
        let betik = format!(
            "$ErrorActionPreference='Stop'; try {{ Checkpoint-Computer -Description 'WinNexus: {aciklama}' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop; exit 0 }} catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
        );
        match self.powershell(
            &betik,
            "Geri yükleme noktası oluşturulamadı",
            ORTA_ZAMAN_ASIMI,
        ) {
            Ok(_) => {}
            Err(hata) if geri_yukleme_siklik_hatasi_mi(&hata.to_string()) => {
                return Err(Hata::DisArac(
                    "Windows 24 saat içinde yalnızca bir geri yükleme noktası oluşturur; son nokta henüz bir günlük değil. Daha sonra yeniden deneyin."
                        .into(),
                ));
            }
            Err(hata) => return Err(hata),
        }
        Ok(format!("Geri yükleme noktası oluşturuldu: {aciklama}"))
    }

    pub fn geri_yukle(&self, sira: u32) -> Sonuc<String> {
        if !geri_yukleme_sirasi_gecerli_mi(sira) {
            return Err(Hata::Dogrulama(
                "Geri yükleme noktası sırası sıfır olamaz.".into(),
            ));
        }
        self.yonetici_kontrolu()?;
        let betik =
            format!("Restore-Computer -RestorePoint {sira} -Confirm:$false -ErrorAction Stop");
        self.powershell(&betik, "Geri yükleme başlatılamadı", UZUN_ZAMAN_ASIMI)?;
        Ok(format!(
            "Geri yükleme başlatıldı (sıra {sira}); işlem tamamlanınca bilgisayar yeniden başlatılacak."
        ))
    }

    pub fn geri_yukleme_etkinlestir(&self, surucu: &str, etkin: bool) -> Sonuc<String> {
        if !geri_yukleme_surucu_gecerli_mi(surucu) {
            return Err(Hata::Dogrulama(
                "Geçersiz sürücü biçimi; 'C:\\' veya 'C:/' bekleniyor.".into(),
            ));
        }
        self.yonetici_kontrolu()?;
        let komut = if etkin {
            "Enable-ComputerRestore"
        } else {
            "Disable-ComputerRestore"
        };
        let betik = format!("$ErrorActionPreference='Stop'; {komut} -Drive '{surucu}'");
        self.powershell(
            &betik,
            "Geri yükleme ayarı değiştirilemedi",
            KISA_ZAMAN_ASIMI,
        )?;
        let durum = if etkin {
            "etkinleştirildi"
        } else {
            "devre dışı bırakıldı"
        };
        Ok(format!("{surucu} sürücüsünde sistem koruması {durum}."))
    }

    pub fn ikon_onbellek_onar(&self) -> Sonuc<()> {
        let yerel = yerel_veri()
            .ok_or_else(|| Hata::Beklenmeyen("LOCALAPPDATA değişkeni bulunamadı.".into()))?;
        let klasor = ps_metni(&yerel.join("Microsoft\\Windows\\Explorer"));
        let betik = format!(
            "$ErrorActionPreference='SilentlyContinue'; Stop-Process -Name explorer -Force; Remove-Item -Path '{klasor}\\iconcache_*.db','{klasor}\\thumbcache_*.db' -Force -ErrorAction SilentlyContinue; Start-Process explorer"
        );
        self.powershell(&betik, "Simge önbelleği onarılamadı", KISA_ZAMAN_ASIMI)?;
        Ok(())
    }

    pub fn sorun_giderici_baslat(&self, kimlik: &str) -> Sonuc<()> {
        if !sorun_giderici_kimligi_gecerli_mi(kimlik) {
            return Err(Hata::Dogrulama("Geçersiz sorun giderici kimliği.".into()));
        }
        let msdt = sistem_koku()
            .map(|kok| kok.join("System32").join("msdt.exe"))
            .filter(|yol| yol.is_file());
        let sonuc = match msdt {
            Some(msdt) => Command::new(msdt).args(["/id", kimlik]).spawn(),
            None => Command::new("cmd")
                .args(["/C", "start", "", "ms-settings:troubleshoot"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn(),
        };
        sonuc
            .map(|_| ())
            .map_err(|hata| Hata::DisArac(format!("Sorun giderici başlatılamadı: {hata}")))
    }

    pub fn chrome_eski_surum_tara(&self) -> Sonuc<Vec<String>> {
        let guncel = chrome_surumu()
            .ok_or_else(|| Hata::Dogrulama("Chrome sürümü kayıt defterinden okunamadı.".into()))?;
        Ok(eski_surumler(&chrome_klasorleri(), &guncel))
    }

    pub fn chrome_eski_surumleri_sil(&self, yollar: &[String]) -> Sonuc<OnarimTemizlikSonucu> {
        let guncel = chrome_surumu().ok_or_else(|| {
            Hata::Dogrulama("Chrome sürümü okunamadığı için eski sürümler silinmedi.".into())
        })?;
        let kokler: Vec<PathBuf> = chrome_klasorleri()
            .iter()
            .filter_map(|kok| fs::canonicalize(kok).ok())
            .collect();
        let mut sonuc = OnarimTemizlikSonucu::default();
        for yol in yollar {
            match chrome_yolu_dogrula(Path::new(yol), &kokler, &guncel) {
                Ok(kanonik) => {
                    let ayar = SilmeAyari {
                        kok: &kanonik,
                        izinli: &kokler,
                        suzgec: Suzgec::Tumu,
                        kok_sil: true,
                    };
                    dizin_temizle(&ayar, &mut sonuc);
                }
                Err(hata) => hata_ekle(&mut sonuc, format!("{yol}: {hata}")),
            }
        }
        Ok(sonuc)
    }

    fn windows_update_temizle(&self) -> Sonuc<OnarimTemizlikSonucu> {
        let kok = sistem_koku()
            .ok_or_else(|| Hata::Beklenmeyen("SystemRoot değişkeni bulunamadı.".into()))?;
        let dizin = ps_metni(&kok.join("SoftwareDistribution").join("Download"));
        let betik = format!(
            "Stop-Service wuauserv,bits -Force -ErrorAction SilentlyContinue; Remove-Item '{dizin}' -Recurse -Force -ErrorAction SilentlyContinue; Start-Service wuauserv,bits -ErrorAction SilentlyContinue"
        );
        self.powershell(
            &betik,
            "Windows Update indirmeleri temizlenemedi",
            ORTA_ZAMAN_ASIMI,
        )?;
        Ok(OnarimTemizlikSonucu::default())
    }

    fn defender_gecmisi_temizle(&self) -> Sonuc<OnarimTemizlikSonucu> {
        let kok = program_verisi()
            .ok_or_else(|| Hata::Beklenmeyen("ProgramData değişkeni bulunamadı.".into()))?;
        let dizin = ps_metni(
            &kok.join("Microsoft\\Windows Defender")
                .join("Scans\\History\\Results"),
        );
        let betik = format!(
            "$ErrorActionPreference='SilentlyContinue'; if(-not (Test-Path -LiteralPath '{dizin}')){{ exit 0 }}; Remove-Item -Path '{dizin}\\*' -Recurse -Force -ErrorAction SilentlyContinue; $kalan=@(Get-ChildItem -LiteralPath '{dizin}' -Force -ErrorAction Stop).Count; if($kalan -gt 0){{ exit 1 }} else {{ exit 0 }}"
        );
        self.powershell(
            &betik,
            "Defender tarama geçmişi silinemedi",
            ORTA_ZAMAN_ASIMI,
        )?;
        Ok(OnarimTemizlikSonucu::default())
    }

    fn powershell(&self, betik: &str, baslik: &str, zaman_asimi: Duration) -> Sonuc<KomutSonucu> {
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
            zaman_asimi,
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

    fn yonetici_kontrolu(&self) -> Sonuc<()> {
        if (self.yonetici_denetimi)() {
            Ok(())
        } else {
            Err(Hata::Yetki(
                "Sistem geri yükleme işlemleri yönetici olarak çalıştırılmalı.".into(),
            ))
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Suzgec {
    #[default]
    Tumu,
    Simge,
}

impl Suzgec {
    fn uygun(self, ad: &str) -> bool {
        match self {
            Self::Tumu => true,
            Self::Simge => {
                let ad = ad.to_ascii_lowercase();
                (ad.starts_with("iconcache_") || ad.starts_with("thumbcache_"))
                    && ad.ends_with(".db")
            }
        }
    }
}

#[derive(Default)]
struct KategoriYollari {
    klasorler: Vec<PathBuf>,
    dosyalar: Vec<PathBuf>,
    suzgec: Suzgec,
}

#[derive(Default)]
struct Sayim {
    dosya: u64,
    bayt: u64,
}

#[derive(Clone, Copy)]
struct SilmeAyari<'a> {
    kok: &'a Path,
    izinli: &'a [PathBuf],
    suzgec: Suzgec,
    kok_sil: bool,
}

fn kategori_yollari(kimlik: &str) -> KategoriYollari {
    let mut yollar = KategoriYollari::default();
    match kimlik {
        "sembol_thumb_onbellegi" => {
            yollar.suzgec = Suzgec::Simge;
            if let Some(yerel) = yerel_veri() {
                yollar
                    .klasorler
                    .push(yerel.join("Microsoft\\Windows\\Explorer"));
                yollar.dosyalar.push(yerel.join("IconCache.db"));
            }
        }
        "son_kullanilan" => yollari_ekle(
            &mut yollar,
            gezgin_veri(),
            &["Microsoft\\Windows\\Recent"],
        ),
        "wer_raporlari" => yollari_ekle(
            &mut yollar,
            program_verisi(),
            &[
                "Microsoft\\Windows\\WER\\ReportQueue",
                "Microsoft\\Windows\\WER\\ReportArchive",
                "Microsoft\\Windows\\WER\\Temp",
            ],
        ),
        "windows_loglari" => yollari_ekle(
            &mut yollar,
            sistem_koku(),
            &["Logs\\CBS", "Logs\\DISM", "Logs\\WindowsUpdate"],
        ),
        "windows_update_indirmeleri" => yollari_ekle(
            &mut yollar,
            sistem_koku(),
            &["SoftwareDistribution\\Download"],
        ),
        "teslimat_optimizasyonu" => yollari_ekle(
            &mut yollar,
            sistem_koku(),
            &["ServiceProfiles\\NetworkService\\AppData\\Local\\Microsoft\\Windows\\DeliveryOptimization\\Cache"],
        ),
        "defender_tarama_gecmisi" => yollari_ekle(
            &mut yollar,
            program_verisi(),
            &["Microsoft\\Windows Defender\\Scans\\History\\Results"],
        ),
        "gelistirici_onbellekleri" => {
            yollari_ekle(
                &mut yollar,
                yerel_veri(),
                &["NuGet\\v3-cache", "pip\\Cache"],
            );
            yollari_ekle(&mut yollar, gezgin_veri(), &["npm-cache"]);
        }
        "visual_studio_kaynagi" => yollari_ekle(
            &mut yollar,
            program_verisi(),
            &["Microsoft\\VisualStudio\\Packages", "Package Cache"],
        ),
        "chrome_kurulum_onbellegi" => yollari_ekle(
            &mut yollar,
            yerel_veri(),
            &["Google\\Chrome\\Installer"],
        ),
        "inetcache" => yollari_ekle(
            &mut yollar,
            yerel_veri(),
            &["Microsoft\\Windows\\INetCache"],
        ),
        _ => {}
    }
    yollar
}

fn yollari_ekle(yollar: &mut KategoriYollari, kok: Option<PathBuf>, altlar: &[&str]) {
    if let Some(kok) = kok {
        yollar
            .klasorler
            .extend(altlar.iter().map(|alt| kok.join(alt)));
    }
}

fn tarama_yap(yollar: &KategoriYollari) -> Sayim {
    let mut sayim = Sayim::default();
    for kok in &yollar.klasorler {
        dizin_tara(kok, yollar.suzgec, &mut sayim);
    }
    for dosya in &yollar.dosyalar {
        if let Ok(meta) = fs::symlink_metadata(dosya) {
            if meta.is_file() && !yeniden_ayristirma_mi(&meta) {
                sayim.dosya += 1;
                sayim.bayt = sayim.bayt.saturating_add(meta.len());
            }
        }
    }
    sayim
}

fn dizin_tara(dizin: &Path, suzgec: Suzgec, sayim: &mut Sayim) {
    let Ok(kayitlar) = fs::read_dir(dizin) else {
        return;
    };
    for kayit in kayitlar.flatten() {
        let yol = kayit.path();
        let Ok(meta) = fs::symlink_metadata(&yol) else {
            continue;
        };
        if yeniden_ayristirma_mi(&meta) {
            continue;
        }
        if meta.is_dir() {
            dizin_tara(&yol, suzgec, sayim);
        } else if meta.is_file() && suzgec.uygun(&dosya_adi(&yol)) {
            sayim.dosya += 1;
            sayim.bayt = sayim.bayt.saturating_add(meta.len());
        }
    }
}

fn dizinleri_temizle(yollar: &KategoriYollari) -> OnarimTemizlikSonucu {
    let izinli = izinli_yollar(yollar);
    let mut sonuc = OnarimTemizlikSonucu::default();
    for kok in &yollar.klasorler {
        let Ok(meta) = fs::symlink_metadata(kok) else {
            continue;
        };
        if !meta.is_dir() || yeniden_ayristirma_mi(&meta) {
            continue;
        }
        let Ok(kanonik) = fs::canonicalize(kok) else {
            continue;
        };
        let ayar = SilmeAyari {
            kok: &kanonik,
            izinli: &izinli,
            suzgec: yollar.suzgec,
            kok_sil: false,
        };
        dizin_temizle(&ayar, &mut sonuc);
    }
    for dosya in &yollar.dosyalar {
        if !dosya.exists() {
            continue;
        }
        match dosya_sil(dosya, &izinli) {
            Ok(bayt) => {
                sonuc.silinen_dosya += 1;
                sonuc.silinen_bayt = sonuc.silinen_bayt.saturating_add(bayt);
            }
            Err(hata) => hata_ekle(&mut sonuc, hata),
        }
    }
    sonuc
}

fn izinli_yollar(yollar: &KategoriYollari) -> Vec<PathBuf> {
    let mut izinli: Vec<PathBuf> = yollar
        .klasorler
        .iter()
        .filter_map(|kok| fs::canonicalize(kok).ok())
        .collect();
    izinli.extend(
        yollar
            .dosyalar
            .iter()
            .filter_map(|dosya| fs::canonicalize(dosya).ok()),
    );
    izinli
}

fn dizin_temizle(ayar: &SilmeAyari, sonuc: &mut OnarimTemizlikSonucu) {
    let mut klasorler = vec![ayar.kok.to_path_buf()];
    let mut bekleyen = vec![ayar.kok.to_path_buf()];
    while let Some(dizin) = bekleyen.pop() {
        let Ok(kayitlar) = fs::read_dir(&dizin) else {
            hata_ekle(sonuc, format!("{} okunamadı.", dizin.display()));
            continue;
        };
        for kayit in kayitlar.flatten() {
            let yol = kayit.path();
            let Ok(meta) = fs::symlink_metadata(&yol) else {
                continue;
            };
            if yeniden_ayristirma_mi(&meta) {
                continue;
            }
            if meta.is_dir() {
                klasorler.push(yol.clone());
                bekleyen.push(yol);
            } else if meta.is_file() && ayar.suzgec.uygun(&dosya_adi(&yol)) {
                match dosya_sil(&yol, ayar.izinli) {
                    Ok(bayt) => {
                        sonuc.silinen_dosya += 1;
                        sonuc.silinen_bayt = sonuc.silinen_bayt.saturating_add(bayt);
                    }
                    Err(hata) => hata_ekle(sonuc, hata),
                }
            }
        }
    }
    klasorler.sort_by_key(|dizin| std::cmp::Reverse(dizin.components().count()));
    for dizin in klasorler {
        if !ayar.kok_sil && dizin.as_path() == ayar.kok {
            continue;
        }
        if bos_mu(&dizin) {
            let _ = fs::remove_dir(&dizin);
        }
    }
}

#[allow(clippy::permissions_set_readonly_false)]
fn dosya_sil(yol: &Path, izinli: &[PathBuf]) -> Result<u64, String> {
    let kanonik = fs::canonicalize(yol).map_err(|hata| format!("{}: {hata}", yol.display()))?;
    if !izinli.iter().any(|kok| yol_kok_altinda_mi(kok, &kanonik)) {
        return Err(format!("{} izinli kökün dışında.", yol.display()));
    }
    let meta =
        fs::symlink_metadata(&kanonik).map_err(|hata| format!("{}: {hata}", yol.display()))?;
    if !meta.is_file() || yeniden_ayristirma_mi(&meta) {
        return Err(format!("{} normal bir dosya değil.", yol.display()));
    }
    if meta.permissions().readonly() {
        let mut izinler = meta.permissions();
        izinler.set_readonly(false);
        fs::set_permissions(&kanonik, izinler).map_err(|hata| {
            format!(
                "{} salt okunur bayrağı kaldırılamadı: {hata}",
                yol.display()
            )
        })?;
    }
    fs::remove_file(&kanonik).map_err(|hata| format!("{} silinemedi: {hata}", yol.display()))?;
    Ok(meta.len())
}

fn bos_mu(dizin: &Path) -> bool {
    fs::read_dir(dizin)
        .map(|mut kayitlar| kayitlar.next().is_none())
        .unwrap_or(false)
}

fn yeniden_ayristirma_mi(meta: &Metadata) -> bool {
    meta.file_attributes() & REPARSE_NOKTASI != 0
}

fn dosya_adi(yol: &Path) -> String {
    yol.file_name()
        .map(|ad| ad.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn hata_ekle(sonuc: &mut OnarimTemizlikSonucu, mesaj: String) {
    if sonuc.hatalar.len() < AZAMI_HATA_SAYISI {
        sonuc.hatalar.push(mesaj);
    }
}

fn appx_tam_ad_gecerli_mi(tam_ad: &str) -> bool {
    !tam_ad.is_empty()
        && tam_ad.len() <= 256
        && tam_ad.chars().all(|karakter| {
            karakter.is_ascii_alphanumeric() || matches!(karakter, '.' | '_' | '-' | '+')
        })
}

fn eski_surumler(kokler: &[PathBuf], guncel: &str) -> Vec<String> {
    let mut sonuclar = Vec::new();
    for kok in kokler {
        let Ok(kayitlar) = fs::read_dir(kok) else {
            continue;
        };
        for kayit in kayitlar.flatten() {
            let yol = kayit.path();
            let ad = dosya_adi(&yol);
            if !surum_benzeri_mi(&ad) || ad == guncel {
                continue;
            }
            let uygun = fs::symlink_metadata(&yol)
                .map(|meta| meta.is_dir() && !yeniden_ayristirma_mi(&meta))
                .unwrap_or(false);
            if uygun {
                sonuclar.push(yol.to_string_lossy().into_owned());
            }
        }
    }
    sonuclar.sort();
    sonuclar
}

fn chrome_yolu_dogrula(yol: &Path, kokler: &[PathBuf], guncel: &str) -> Result<PathBuf, String> {
    let kanonik = fs::canonicalize(yol).map_err(|hata| format!("bulunamadı: {hata}"))?;
    let meta = fs::symlink_metadata(&kanonik).map_err(|hata| format!("okunamadı: {hata}"))?;
    if !meta.is_dir() || yeniden_ayristirma_mi(&meta) {
        return Err("klasör değil.".into());
    }
    let ust = kanonik
        .parent()
        .and_then(|ust| fs::canonicalize(ust).ok())
        .ok_or_else(|| "üst klasör doğrulanamadı.".to_string())?;
    if !kokler.contains(&ust) {
        return Err("Chrome uygulama klasörünün dışında.".into());
    }
    let ad = dosya_adi(&kanonik);
    if !surum_benzeri_mi(&ad) {
        return Err("sürüm klasörü değil.".into());
    }
    if ad == guncel {
        return Err("geçerli Chrome sürümü silinemez.".into());
    }
    Ok(kanonik)
}

fn surum_benzeri_mi(ad: &str) -> bool {
    ad.starts_with(|karakter: char| karakter.is_ascii_digit())
        && ad.contains('.')
        && ad
            .chars()
            .all(|karakter| karakter.is_ascii_digit() || karakter == '.')
}

fn chrome_surumu() -> Option<String> {
    let konumlar = [
        (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, KEY_READ | KEY_WOW64_32KEY),
        (HKEY_CURRENT_USER, KEY_READ | KEY_WOW64_64KEY),
    ];
    konumlar.iter().find_map(|(kok, bayrak)| {
        let anahtar = RegKey::predef(*kok)
            .open_subkey_with_flags(CHROME_KAYIT_YOLU, *bayrak)
            .ok()?;
        let surum: String = anahtar.get_value("DisplayVersion").ok()?;
        let surum = surum.trim().to_string();
        (!surum.is_empty()).then_some(surum)
    })
}

fn chrome_klasorleri() -> Vec<PathBuf> {
    ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .copied()
        .filter_map(std::env::var_os)
        .map(|kok| {
            PathBuf::from(kok)
                .join("Google")
                .join("Chrome")
                .join("Application")
        })
        .collect()
}

fn son_sayi(metin: &str) -> Option<u32> {
    metin
        .lines()
        .rev()
        .map(str::trim)
        .find(|satir| !satir.is_empty())?
        .parse()
        .ok()
}

fn cikti_metni(sonuc: &KomutSonucu, sinir: usize) -> String {
    let mut metin = sonuc.stdout.trim().to_string();
    let hata = sonuc.stderr.trim();
    if !hata.is_empty() {
        if !metin.is_empty() {
            metin.push('\n');
        }
        metin.push_str(hata);
    }
    metin.chars().take(sinir).collect()
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

fn geri_yukleme_siklik_hatasi_mi(mesaj: &str) -> bool {
    let mesaj = mesaj.to_lowercase();
    [
        "within the past 24 hours",
        "already been created",
        "24 saat",
    ]
    .iter()
    .any(|ifade| mesaj.contains(ifade))
}

fn ps_metni(yol: &Path) -> String {
    yol.to_string_lossy().replace('\'', "''")
}

fn yerel_veri() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

fn gezgin_veri() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

fn program_verisi() -> Option<PathBuf> {
    std::env::var_os("ProgramData").map(PathBuf::from)
}

fn sistem_koku() -> Option<PathBuf> {
    std::env::var_os("SystemRoot").map(PathBuf::from)
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

    fn cevap(stdout: &str) -> KomutSonucu {
        KomutSonucu {
            komut: "test".into(),
            kod: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn servis(cevaplar: Vec<KomutSonucu>) -> (OnarimServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            cevaplar: Mutex::new(VecDeque::from(cevaplar)),
            cagrilar: Mutex::new(Vec::new()),
        });
        (OnarimServisi::yeni(calistirici.clone()), calistirici)
    }

    fn cagrilar(calistirici: &SahteCalistirici) -> Vec<(String, Vec<String>)> {
        calistirici.cagrilar.lock().unwrap().clone()
    }

    fn denetimli_servis(
        cevaplar: Vec<KomutSonucu>,
        yonetici: bool,
    ) -> (OnarimServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            cevaplar: Mutex::new(VecDeque::from(cevaplar)),
            cagrilar: Mutex::new(Vec::new()),
        });
        let servis = OnarimServisi {
            calistirici: calistirici.clone(),
            yonetici_denetimi: if yonetici { || true } else { || false },
        };
        (servis, calistirici)
    }

    #[test]
    fn sfc_ve_dism_dogru_argumanlarla_calisir() {
        let (servis, calistirici) = servis(vec![cevap("tarama tamam"), cevap("temizlik tamam")]);
        assert_eq!(servis.sfc_calistir().unwrap(), "tarama tamam");
        assert_eq!(servis.dism_reset_base().unwrap(), "temizlik tamam");
        let cagri = cagrilar(&calistirici);
        assert_eq!(cagri[0].0, "sfc.exe");
        assert_eq!(cagri[0].1, vec!["/scannow"]);
        assert_eq!(cagri[1].0, "Dism.exe");
        assert_eq!(
            cagri[1].1,
            vec![
                "/Online",
                "/Cleanup-Image",
                "/StartComponentCleanup",
                "/ResetBase"
            ]
        );
    }

    #[test]
    fn sfc_hatali_kod_turkce_hata_dondurur() {
        let mut hatali = cevap("bozuk");
        hatali.kod = 5;
        hatali.stderr = "SFC başarısız oldu.\nayrıntı".into();
        let (servis, _calistirici) = servis(vec![hatali]);
        let hata = servis.sfc_calistir().unwrap_err();
        assert!(hata.to_string().contains("SFC taraması başarısız"));
        assert!(hata.to_string().contains("SFC başarısız oldu."));
    }

    #[test]
    fn appx_onar_betik_uretir_ve_gecersiz_adi_reddeder() {
        let (servis, calistirici) = servis(vec![cevap("")]);
        let tam_ad = "Microsoft.WindowsCalculator_11.0_x64__8wekyb3d8bbwe";
        servis.appx_onar(tam_ad).unwrap();
        let cagri = cagrilar(&calistirici);
        assert_eq!(cagri[0].0, "powershell");
        let betik = cagri[0].1.last().unwrap();
        assert!(betik.contains(&format!("Get-AppxPackage -Package '{tam_ad}'")));
        assert!(betik.contains("Add-AppxPackage -DisableDevelopmentMode -Register"));
        assert!(betik.contains("AppxManifest.xml"));
        assert!(servis.appx_onar("A_1.0'; calc; '").is_err());
        assert_eq!(cagrilar(&calistirici).len(), 1);
    }

    #[test]
    fn guncelleme_gecmisi_ve_geri_yukleme_ayristirilir() {
        let (servis, calistirici) = servis(vec![
            cevap(r#"[{"Kimlik":"KB1","Aciklama":"Güncelleme","Tarih":"2026-01-15"}]"#),
            cevap(r#"{"Sira":3,"Aciklama":"Nokta","Tarih":"2026-02-03 14:30"}"#),
        ]);
        let kayitlar = servis.guncelleme_gecmisi().unwrap();
        assert_eq!(kayitlar[0].tarih, "15.01.2026");
        let noktalar = servis.geri_yukleme_noktalari().unwrap();
        assert_eq!(noktalar[0].sira, 3);
        let cagri = cagrilar(&calistirici);
        assert!(cagri[0].1.last().unwrap().contains("Get-HotFix"));
        assert!(cagri[1]
            .1
            .last()
            .unwrap()
            .contains("Get-ComputerRestorePoint"));
    }

    #[test]
    fn tek_geri_yukleme_noktasi_silinmez() {
        let (tek_servis, tek_calistirici) = servis(vec![cevap(r#"[{"Sira":5,"Aciklama":"Tek"}]"#)]);
        assert_eq!(tek_servis.eski_geri_yukleme_noktalarini_sil().unwrap(), 0);
        assert_eq!(cagrilar(&tek_calistirici).len(), 1);
        let (bos_servis, bos_calistirici) = servis(vec![cevap("[]")]);
        assert_eq!(bos_servis.eski_geri_yukleme_noktalarini_sil().unwrap(), 0);
        assert_eq!(cagrilar(&bos_calistirici).len(), 1);
    }

    #[test]
    fn eski_geri_yukleme_noktalari_silinir_ve_sayi_okunur() {
        let (servis, calistirici) = servis(vec![
            cevap(r#"[{"Sira":2},{"Sira":5},{"Sira":9}]"#),
            cevap("2\n"),
        ]);
        assert_eq!(servis.eski_geri_yukleme_noktalarini_sil().unwrap(), 2);
        let cagri = cagrilar(&calistirici);
        let betik = cagri[1].1.last().unwrap();
        assert!(betik.contains("DeleteRestorePoint"));
        assert!(betik.contains("@(2,5)"));
        assert!(!betik.contains("@(9)"));
    }

    #[test]
    fn desteklenmeyen_sorun_giderici_komut_calistirmaz() {
        let (servis, calistirici) = servis(Vec::new());
        assert!(servis
            .sorun_giderici_baslat("WindowsUpdateDiagnostic; calc")
            .is_err());
        assert!(servis.sorun_giderici_baslat("Bilinmeyen").is_err());
        assert!(cagrilar(&calistirici).is_empty());
    }

    #[test]
    fn guncelleme_gecmisi_temizleme_betigi_hizmetleri_yonetir() {
        let (servis, calistirici) = servis(vec![cevap("")]);
        servis.guncelleme_gecmisi_temizle().unwrap();
        let cagri = cagrilar(&calistirici);
        let betik = cagri[0].1.last().unwrap();
        assert!(betik.contains("Stop-Service wuauserv,bits,cryptsvc"));
        assert!(betik.contains("SoftwareDistribution"));
        assert!(betik.contains("DataStore"));
        assert!(betik.contains("Start-Service wuauserv,bits"));
    }

    #[test]
    fn bilinmeyen_kategori_reddedilir() {
        let (servis, calistirici) = servis(Vec::new());
        assert!(servis.kategori_temizle("bilinmeyen").is_err());
        assert!(cagrilar(&calistirici).is_empty());
    }

    fn silme_testi(kok: &Path, suzgec: Suzgec) -> OnarimTemizlikSonucu {
        let kanonik = fs::canonicalize(kok).unwrap();
        let ayar = SilmeAyari {
            kok: &kanonik,
            izinli: std::slice::from_ref(&kanonik),
            suzgec,
            kok_sil: false,
        };
        let mut sonuc = OnarimTemizlikSonucu::default();
        dizin_temizle(&ayar, &mut sonuc);
        sonuc
    }

    #[test]
    fn dizin_temizle_dosyalari_ve_bos_klasorleri_siler() {
        let kok = tempfile::tempdir().unwrap();
        let alt = kok.path().join("a").join("b");
        fs::create_dir_all(&alt).unwrap();
        fs::write(alt.join("veri.bin"), b"12345").unwrap();
        let salt = kok.path().join("salt.txt");
        fs::write(&salt, b"abc").unwrap();
        let mut izinler = fs::metadata(&salt).unwrap().permissions();
        izinler.set_readonly(true);
        fs::set_permissions(&salt, izinler).unwrap();
        let sonuc = silme_testi(kok.path(), Suzgec::Tumu);
        assert_eq!(sonuc.silinen_dosya, 2);
        assert_eq!(sonuc.silinen_bayt, 8);
        assert!(sonuc.hatalar.is_empty(), "{:?}", sonuc.hatalar);
        assert!(kok.path().exists());
        assert!(!kok.path().join("a").exists());
        assert_eq!(fs::read_dir(kok.path()).unwrap().count(), 0);
    }

    #[test]
    fn sembol_suzgeci_yalnizca_onbellek_dosyalarini_siler() {
        let kok = tempfile::tempdir().unwrap();
        fs::write(kok.path().join("iconcache_16.db"), b"1").unwrap();
        fs::write(kok.path().join("thumbcache_32.db"), b"22").unwrap();
        fs::write(kok.path().join("korunacak.txt"), b"333").unwrap();
        let sonuc = silme_testi(kok.path(), Suzgec::Simge);
        assert_eq!(sonuc.silinen_dosya, 2);
        assert_eq!(sonuc.silinen_bayt, 3);
        assert!(kok.path().join("korunacak.txt").exists());
    }

    #[test]
    fn sembolik_baglanti_izlenmez() {
        let kok = tempfile::tempdir().unwrap();
        let hedef = tempfile::tempdir().unwrap();
        fs::write(hedef.path().join("korunacak.txt"), b"veri").unwrap();
        let bag = kok.path().join("bag");
        if std::os::windows::fs::symlink_dir(hedef.path(), &bag).is_err() {
            return;
        }
        let sonuc = silme_testi(kok.path(), Suzgec::Tumu);
        assert_eq!(sonuc.silinen_dosya, 0);
        assert!(hedef.path().join("korunacak.txt").exists());
    }

    #[test]
    fn eski_chrome_surumleri_bulunur() {
        let kok = tempfile::tempdir().unwrap();
        for ad in ["120.0.6099.130", "121.0.6167.85", "SetupMetrics"] {
            fs::create_dir(kok.path().join(ad)).unwrap();
        }
        let bulunan = eski_surumler(&[kok.path().to_path_buf()], "121.0.6167.85");
        assert_eq!(
            bulunan,
            vec![kok
                .path()
                .join("120.0.6099.130")
                .to_string_lossy()
                .into_owned()]
        );
    }

    #[test]
    fn chrome_silme_yolu_guvenlik_denetimleri() {
        let kok = tempfile::tempdir().unwrap();
        let eski = kok.path().join("120.0.6099.130");
        let guncel = kok.path().join("121.0.6167.85");
        let diger = kok.path().join("SetupMetrics");
        for yol in [&eski, &guncel, &diger] {
            fs::create_dir(yol).unwrap();
        }
        let kokler = vec![fs::canonicalize(kok.path()).unwrap()];
        assert!(chrome_yolu_dogrula(&eski, &kokler, "121.0.6167.85").is_ok());
        assert!(chrome_yolu_dogrula(&guncel, &kokler, "121.0.6167.85").is_err());
        assert!(chrome_yolu_dogrula(&diger, &kokler, "121.0.6167.85").is_err());
        assert!(chrome_yolu_dogrula(&kok.path().join("yok"), &kokler, "121.0.6167.85").is_err());
        let disarida = std::env::temp_dir();
        assert!(chrome_yolu_dogrula(&disarida, &kokler, "121.0.6167.85").is_err());
    }

    #[test]
    fn geri_yukleme_olustur_aciklamayi_temizler_ve_betik_uretir() {
        let (servis, calistirici) = denetimli_servis(vec![cevap("")], true);
        let sonuc = servis.geri_yukleme_olustur("  Bakım 'test'\r\n").unwrap();
        assert!(sonuc.contains("Geri yükleme noktası oluşturuldu"));
        assert!(sonuc.contains("Bakım test"));
        let cagri = cagrilar(&calistirici);
        assert_eq!(cagri[0].0, "powershell");
        let betik = cagri[0].1.last().unwrap();
        assert!(betik.contains("Checkpoint-Computer -Description 'WinNexus: Bakım test'"));
        assert!(betik.contains("-RestorePointType 'MODIFY_SETTINGS'"));
        assert!(betik.contains("-ErrorAction Stop"));
    }

    #[test]
    fn geri_yukleme_olustur_24_saat_sinirini_anlamli_iletir() {
        let mut hatali = cevap("");
        hatali.kod = 1;
        hatali.stderr = "A new system restore point cannot be created because one has already been created within the past 24 hours. Please try again later.".into();
        let (servis, _calistirici) = denetimli_servis(vec![hatali], true);
        let hata = servis.geri_yukleme_olustur("deneme").unwrap_err();
        assert!(hata.to_string().contains("24 saat"));
        assert!(hata.to_string().contains("Daha sonra yeniden deneyin"));
    }

    #[test]
    fn geri_yukle_komutu_ve_yeniden_baslatma_bildirimini_uretir() {
        let (servis, calistirici) = denetimli_servis(vec![cevap("")], true);
        let mesaj = servis.geri_yukle(7).unwrap();
        assert!(mesaj.contains("yeniden başlatılacak"));
        let cagri = cagrilar(&calistirici);
        let betik = cagri[0].1.last().unwrap();
        assert!(betik.contains("Restore-Computer -RestorePoint 7"));
        assert!(betik.contains("-Confirm:$false"));
        assert!(betik.contains("-ErrorAction Stop"));
    }

    #[test]
    fn geri_yukle_sifir_sirayi_reddeder_ve_komut_calistirmaz() {
        let (servis, calistirici) = denetimli_servis(Vec::new(), true);
        let hata = servis.geri_yukle(0).unwrap_err();
        assert!(hata.to_string().contains("sıfır olamaz"));
        assert!(cagrilar(&calistirici).is_empty());
    }

    #[test]
    fn geri_yukleme_korumasi_etkinlestirilir_ve_kapatilir() {
        let (servis, calistirici) = denetimli_servis(vec![cevap(""), cevap("")], true);
        let mesaj = servis.geri_yukleme_etkinlestir("C:\\", true).unwrap();
        assert!(mesaj.contains("etkinleştirildi"));
        let mesaj = servis.geri_yukleme_etkinlestir("D:/", false).unwrap();
        assert!(mesaj.contains("devre dışı bırakıldı"));
        let cagri = cagrilar(&calistirici);
        assert!(cagri[0]
            .1
            .last()
            .unwrap()
            .contains("Enable-ComputerRestore -Drive 'C:\\'"));
        assert!(cagri[1]
            .1
            .last()
            .unwrap()
            .contains("Disable-ComputerRestore -Drive 'D:/'"));
    }

    #[test]
    fn gecersiz_surucu_ve_yetkisiz_kullanici_komut_calistirmaz() {
        let (servis, calistirici) = denetimli_servis(Vec::new(), true);
        assert!(servis.geri_yukleme_etkinlestir("C:", true).is_err());
        assert!(servis
            .geri_yukleme_etkinlestir("C:\\Windows", false)
            .is_err());
        assert!(cagrilar(&calistirici).is_empty());
        let (yetkisiz, yetkisiz_calistirici) = denetimli_servis(Vec::new(), false);
        assert!(yetkisiz.geri_yukleme_olustur("deneme").is_err());
        assert!(yetkisiz.geri_yukle(3).is_err());
        assert!(cagrilar(&yetkisiz_calistirici).is_empty());
    }

    #[test]
    fn geri_yukle_hatali_kodu_turkce_ve_sinirli_bildirir() {
        let mut hatali = cevap("");
        hatali.kod = 5;
        hatali.stderr = "Restore başarısız.\nayrıntı".into();
        let (servis, _calistirici) = denetimli_servis(vec![hatali], true);
        let hata = servis.geri_yukle(4).unwrap_err().to_string();
        assert!(hata.contains("Geri yükleme başlatılamadı"));
        assert!(hata.contains("kod 5"));
        assert!(hata.contains("Restore başarısız."));
        assert!(!hata.contains("ayrıntı"));
    }
}
