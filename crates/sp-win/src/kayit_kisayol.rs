use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::error::{Hata, Sonuc};
use sp_core::kayit_kisayol::{
    clsid_gecerli_mi, kayit_degeri_guvenli_mi, kayit_yolu_guvenli_mi, kisayol_json_ayristir,
    mui_uygulama_yolu_ayikla, ses_olayi_hedef_yolu, uninstall_yolu_ayikla,
    yardim_dosyasi_hedef_yolu, yazi_tipi_hedef_yolu, BaglamOgesi, KayitDefteriBulgu,
    KayitTemizlikSonucu, KisayolBulgu, KATEGORI_KALDIRMA, KATEGORI_MUI, KATEGORI_PAYLASILAN_DLL,
    KATEGORI_SES_OLAYI, KATEGORI_UYGULAMA_YOLU, KATEGORI_YARDIM, KATEGORI_YAZI_TIPI,
};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use winreg::enums::{
    HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY,
    KEY_WOW64_64KEY, KEY_WRITE, REG_SZ,
};
use winreg::{RegKey, RegValue};

const UNINSTALL64: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
const UNINSTALL32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";
const UNINSTALL_KULLANICI: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";
const APP_PATHS64: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths";
const APP_PATHS32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths";
const APP_PATHS_KULLANICI: &str = r"Software\Microsoft\Windows\CurrentVersion\App Paths";
const SHARED_DLL64: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\SharedDLLs";
const SHARED_DLL32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\SharedDLLs";
const YAZI_TIPI_YOLU: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
const SES_OLAYI_YOLU: &str = r"AppEvents\Schemes\Apps";
const MUI_YOLU: &str = r"Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache";
const YARDIM_YOLU: &str = r"SOFTWARE\Microsoft\Windows\Help";
const BLOK_YOLU: &str = r"Software\Microsoft\Windows\CurrentVersion\Shell Extensions\Blocked";
const KLASIK_YOLU: &str = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";
const KLASIK_INPROC: &str =
    r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";
const KOMUT_ZAMAN_ASIMI: Duration = Duration::from_secs(60);
const KISAYOL_ZAMAN_ASIMI: Duration = Duration::from_secs(180);
const AZAMI_KISAYOL: usize = 2000;
const AZAMI_DEGER_ADI: usize = 256;
const AZAMI_MUI_DEGER_ADI: usize = 1024;
const KISAYOL_BETIGI: &str = r#"
$ErrorActionPreference='SilentlyContinue'
$ws = New-Object -ComObject WScript.Shell
$kokler = @(
  @{Yol=[Environment]::GetFolderPath('Desktop'); Ad='Masaüstü'},
  @{Yol=[Environment]::GetFolderPath('CommonDesktopDirectory'); Ad='Ortak masaüstü'},
  @{Yol=[Environment]::GetFolderPath('StartMenu'); Ad='Başlat menüsü'},
  @{Yol=[Environment]::GetFolderPath('CommonStartMenu'); Ad='Ortak başlat menüsü'},
  @{Yol=[Environment]::GetFolderPath('Startup'); Ad='Başlangıç'},
  @{Yol=[Environment]::GetFolderPath('CommonStartup'); Ad='Ortak başlangıç'}
)
$sonuc = @()
foreach($kok in $kokler) {
  if([string]::IsNullOrWhiteSpace($kok.Yol)) { continue }
  Get-ChildItem -LiteralPath $kok.Yol -Filter *.lnk -Recurse -Force | ForEach-Object {
    $k = $ws.CreateShortcut($_.FullName)
    $sonuc += [pscustomobject]@{ Yol=$_.FullName; Ad=$_.BaseName; Hedef=$k.TargetPath; Konum=$kok.Ad }
  }
}
@($sonuc) | ConvertTo-Json -Compress
"#;

#[derive(Debug, Clone, Copy)]
struct KayitKoku {
    hklm: bool,
    yol: &'static str,
    gorunum: u32,
}

const UNINSTALL_KOKLERI: [KayitKoku; 3] = [
    KayitKoku {
        hklm: true,
        yol: UNINSTALL64,
        gorunum: 64,
    },
    KayitKoku {
        hklm: true,
        yol: UNINSTALL32,
        gorunum: 32,
    },
    KayitKoku {
        hklm: false,
        yol: UNINSTALL_KULLANICI,
        gorunum: 64,
    },
];

const APP_PATH_KOKLERI: [KayitKoku; 3] = [
    KayitKoku {
        hklm: true,
        yol: APP_PATHS64,
        gorunum: 64,
    },
    KayitKoku {
        hklm: true,
        yol: APP_PATHS32,
        gorunum: 32,
    },
    KayitKoku {
        hklm: false,
        yol: APP_PATHS_KULLANICI,
        gorunum: 64,
    },
];

const SHARED_DLL_KOKLERI: [KayitKoku; 2] = [
    KayitKoku {
        hklm: true,
        yol: SHARED_DLL64,
        gorunum: 64,
    },
    KayitKoku {
        hklm: true,
        yol: SHARED_DLL32,
        gorunum: 32,
    },
];

const YAZI_TIPI_KOKLERI: [KayitKoku; 1] = [KayitKoku {
    hklm: true,
    yol: YAZI_TIPI_YOLU,
    gorunum: 64,
}];

const SES_OLAYI_KOKLERI: [KayitKoku; 1] = [KayitKoku {
    hklm: false,
    yol: SES_OLAYI_YOLU,
    gorunum: 64,
}];

const MUI_KOKLERI: [KayitKoku; 1] = [KayitKoku {
    hklm: false,
    yol: MUI_YOLU,
    gorunum: 64,
}];

const YARDIM_KOKLERI: [KayitKoku; 1] = [KayitKoku {
    hklm: true,
    yol: YARDIM_YOLU,
    gorunum: 64,
}];

const BAGLAM_KAPSAMLARI: [(&str, &str); 6] = [
    ("*", "Tüm dosyalar"),
    ("AllFilesystemObjects", "Tüm dosya sistemi nesneleri"),
    ("Directory", "Klasörler"),
    (r"Directory\Background", "Klasör arka planı"),
    ("Folder", "Klasörler (tüm)"),
    ("Drive", "Sürücüler"),
];

#[derive(Clone)]
pub struct KayitKisayolServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl KayitKisayolServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    pub fn kayit_defteri_tara(&self) -> Sonuc<Vec<KayitDefteriBulgu>> {
        let mut bulgular = Vec::new();
        let mut gorulen = HashSet::new();
        for kok in UNINSTALL_KOKLERI {
            uninstall_tara(&kok, &mut bulgular, &mut gorulen);
        }
        for kok in APP_PATH_KOKLERI {
            app_path_tara(&kok, &mut bulgular, &mut gorulen);
        }
        for kok in SHARED_DLL_KOKLERI {
            shared_dll_tara(&kok, &mut bulgular, &mut gorulen);
        }
        yazi_tipi_tara(&mut bulgular, &mut gorulen);
        ses_olayi_tara(&mut bulgular, &mut gorulen);
        mui_tara(&mut bulgular, &mut gorulen);
        yardim_tara(&mut bulgular, &mut gorulen);
        bulgular.sort_by(|a, b| {
            a.kategori
                .cmp(&b.kategori)
                .then_with(|| a.ad.to_lowercase().cmp(&b.ad.to_lowercase()))
        });
        Ok(bulgular)
    }

    pub fn kayit_defteri_temizle(
        &self,
        bulgular: &[KayitDefteriBulgu],
        karantina: &Path,
    ) -> Sonuc<KayitTemizlikSonucu> {
        fs::create_dir_all(karantina)
            .map_err(|hata| Hata::DisArac(format!("Karantina klasörü oluşturulamadı: {hata}")))?;
        let mut sonuc = KayitTemizlikSonucu::default();
        let mut benzersiz = HashSet::new();
        let mut yedek = YedekDurumu::yeni(karantina);
        for bulgu in bulgular {
            if !benzersiz.insert(bulgu.kimlik.clone()) {
                continue;
            }
            if let Err(hata) = kayit_bulgu_dogrula(bulgu) {
                sonuc.hatalar.push(hata.to_string());
                continue;
            }
            if !kayit_var_mi(bulgu) {
                sonuc
                    .hatalar
                    .push(format!("{}: kayıt artık mevcut değil.", bulgu.ad));
                continue;
            }
            let anahtar = format!("{}\\{}", bulgu.kove, bulgu.yol);
            if let Err(hata) = yedek.hazirla(self.calistirici.as_ref(), &anahtar) {
                sonuc.hatalar.push(format!("{}: {hata}", bulgu.ad));
                continue;
            }
            match self.kayit_sil(bulgu) {
                Ok(()) => sonuc.islenen += 1,
                Err(hata) => sonuc.hatalar.push(format!("{}: {hata}", bulgu.ad)),
            }
        }
        Ok(sonuc)
    }

    fn kayit_sil(&self, bulgu: &KayitDefteriBulgu) -> Result<(), String> {
        let argumanlar = kayit_silme_argumanlari(bulgu);
        match self
            .calistirici
            .calistir("reg.exe", &argumanlar, KOMUT_ZAMAN_ASIMI)
        {
            Ok(komut) if komut.kod == 0 => Ok(()),
            Ok(komut) => Err(format!("silinemedi: {}", hata_ozeti(&komut))),
            Err(hata) => Err(format!("silinemedi: {hata}")),
        }
    }

    pub fn kisayollari_tara(&self) -> Sonuc<Vec<KisayolBulgu>> {
        let argumanlar = vec![
            "-NoProfile".to_string(),
            "-NonInteractive".to_string(),
            "-ExecutionPolicy".to_string(),
            "Bypass".to_string(),
            "-Command".to_string(),
            KISAYOL_BETIGI.to_string(),
        ];
        let sonuc = self
            .calistirici
            .calistir("powershell", &argumanlar, KISAYOL_ZAMAN_ASIMI)?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Kısayol taraması başarısız: {}",
                hata_ozeti(&sonuc)
            )));
        }
        let adaylar = kisayol_json_ayristir(&sonuc.stdout)?;
        Ok(adaylar
            .into_iter()
            .filter(|kisayol| !kisayol.hedef.is_empty())
            .filter(|kisayol| mutlak_hedef_mi(&kisayol.hedef))
            .filter(|kisayol| !Path::new(&kisayol.hedef).exists())
            .take(AZAMI_KISAYOL)
            .collect())
    }

    pub fn kisayollari_temizle(
        &self,
        bulgular: &[KisayolBulgu],
        karantina: &Path,
    ) -> Sonuc<KayitTemizlikSonucu> {
        let hedef_klasor = karantina.join("kisayollar");
        fs::create_dir_all(&hedef_klasor)
            .map_err(|hata| Hata::DisArac(format!("Karantina klasörü oluşturulamadı: {hata}")))?;
        let kokler = kisayol_kokleri();
        let mut sonuc = KayitTemizlikSonucu::default();
        let mut gorulen = HashSet::new();
        let manifest = hedef_klasor.join("manifest.tsv");
        for bulgu in bulgular {
            let Ok(kaynak) = fs::canonicalize(&bulgu.yol) else {
                sonuc
                    .hatalar
                    .push(format!("{}: kısayol bulunamadı.", bulgu.yol));
                continue;
            };
            if !gorulen.insert(kaynak.clone()) {
                continue;
            }
            if !kaynak
                .extension()
                .and_then(|uzanti| uzanti.to_str())
                .is_some_and(|uzanti| uzanti.eq_ignore_ascii_case("lnk"))
            {
                sonuc.hatalar.push(format!("{}: .lnk değil.", bulgu.yol));
                continue;
            }
            if !kokler.iter().any(|kok| kaynak.starts_with(kok)) {
                sonuc
                    .hatalar
                    .push(format!("{}: bilinen kısayol kökünün dışında.", bulgu.yol));
                continue;
            }
            let Some(meta) = fs::symlink_metadata(&kaynak).ok() else {
                sonuc.hatalar.push(format!("{}: okunamadı.", bulgu.yol));
                continue;
            };
            if !meta.is_file() {
                sonuc
                    .hatalar
                    .push(format!("{}: normal dosya değil.", bulgu.yol));
                continue;
            }
            let ad = kaynak
                .file_name()
                .map(|ad| ad.to_string_lossy().into_owned())
                .unwrap_or_else(|| "kisayol.lnk".to_string());
            let hedef = benzersiz_yol(
                &hedef_klasor,
                &format!("{:016x}-{ad}", kayit_ozeti(&bulgu.yol)),
            );
            match tasi(&kaynak, &hedef) {
                Ok(()) => {
                    sonuc.islenen += 1;
                    let _ = manifest_yaz(&manifest, &kaynak, &hedef);
                }
                Err(hata) => sonuc.hatalar.push(format!("{}: {hata}", bulgu.yol)),
            }
        }
        Ok(sonuc)
    }

    pub fn baglam_menusu(&self) -> Sonuc<Vec<BaglamOgesi>> {
        let mut ogeler = Vec::new();
        let mut gorulen = HashSet::new();
        for (kok, kaynak) in [(HKEY_CURRENT_USER, "HKCU"), (HKEY_LOCAL_MACHINE, "HKLM")] {
            for (kapsam_yolu, kapsam) in BAGLAM_KAPSAMLARI {
                let yol = format!(r"Software\Classes\{kapsam_yolu}\shellex\ContextMenuHandlers");
                let Ok(anahtar) = RegKey::predef(kok).open_subkey_with_flags(&yol, KEY_READ) else {
                    continue;
                };
                for alt in anahtar.enum_keys().flatten() {
                    let Ok(alt_anahtar) = anahtar.open_subkey(&alt) else {
                        continue;
                    };
                    let Some(clsid) = handler_clsid(&alt_anahtar, &alt) else {
                        continue;
                    };
                    let anahtar_metni = format!("{}|{}|{}", clsid.to_lowercase(), kapsam, kaynak);
                    if !gorulen.insert(anahtar_metni) {
                        continue;
                    }
                    let ad = clsid_adi(&clsid).unwrap_or_else(|| format!("CLSID {clsid}"));
                    ogeler.push(BaglamOgesi {
                        devre_disi: kayit_blokeli_mi(&clsid),
                        clsid,
                        ad,
                        kapsam: kapsam.to_string(),
                        kaynak: kaynak.to_string(),
                    });
                }
            }
        }
        ogeler.sort_by(|a, b| {
            a.ad.to_lowercase()
                .cmp(&b.ad.to_lowercase())
                .then_with(|| a.kapsam.cmp(&b.kapsam))
        });
        Ok(ogeler)
    }

    pub fn baglam_menu_ayarla(&self, clsid: &str, devre_disi: bool) -> Sonuc<()> {
        let clsid = clsid_normallestir(clsid)?;
        let (anahtar, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(BLOK_YOLU, KEY_READ | KEY_WRITE)
            .map_err(|hata| Hata::Registry(format!("Bağlam menüsü kaydı açılamadı: {hata}")))?;
        if devre_disi {
            let veri = RegValue {
                bytes: vec![0, 0],
                vtype: REG_SZ,
            };
            anahtar.set_raw_value(&clsid, &veri).map_err(|hata| {
                Hata::Registry(format!(
                    "{clsid} bağlam menüsü devre dışı bırakılamadı: {hata}"
                ))
            })
        } else {
            match anahtar.delete_value(&clsid) {
                Ok(()) => Ok(()),
                Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(hata) => Err(Hata::Registry(format!(
                    "{clsid} bağlam menüsü etkinleştirilemedi: {hata}"
                ))),
            }
        }
    }

    pub fn klasik_baglam_menusu(&self) -> Sonuc<bool> {
        Ok(RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(KLASIK_INPROC, KEY_READ)
            .is_ok())
    }

    pub fn klasik_baglam_menu_ayarla(&self, etkin: bool) -> Sonuc<()> {
        if etkin {
            let (anahtar, _) = RegKey::predef(HKEY_CURRENT_USER)
                .create_subkey_with_flags(KLASIK_INPROC, KEY_READ | KEY_WRITE)
                .map_err(|hata| {
                    Hata::Registry(format!("Klasik bağlam menüsü açılamadı: {hata}"))
                })?;
            let veri = RegValue {
                bytes: vec![0, 0],
                vtype: REG_SZ,
            };
            anahtar.set_raw_value("", &veri).map_err(|hata| {
                Hata::Registry(format!("Klasik bağlam menüsü ayarlanamadı: {hata}"))
            })
        } else {
            match RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(KLASIK_YOLU) {
                Ok(()) => Ok(()),
                Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(hata) => Err(Hata::Registry(format!(
                    "Klasik bağlam menüsü kapatılamadı: {hata}"
                ))),
            }
        }
    }
}

fn kayit_yedekle(
    calistirici: &dyn KomutCalistirici,
    anahtar: &str,
    dosya: &Path,
) -> Result<(), String> {
    let argumanlar = vec![
        "export".to_string(),
        anahtar.to_string(),
        dosya.to_string_lossy().into_owned(),
        "/y".to_string(),
    ];
    match calistirici.calistir("reg.exe", &argumanlar, KOMUT_ZAMAN_ASIMI) {
        Ok(komut) if komut.kod == 0 && yedek_gecerli(dosya) => Ok(()),
        Ok(komut) if komut.kod == 0 => Err("yedek dosyası oluşmadı".into()),
        Ok(komut) => Err(hata_ozeti(&komut)),
        Err(hata) => Err(hata.to_string()),
    }
}

fn yedek_gecerli(dosya: &Path) -> bool {
    fs::metadata(dosya)
        .map(|meta| meta.is_file() && meta.len() > 0)
        .unwrap_or(false)
}

fn kayit_silme_argumanlari(bulgu: &KayitDefteriBulgu) -> Vec<String> {
    let mut argumanlar = vec![
        "delete".to_string(),
        format!("{}\\{}", bulgu.kove, bulgu.yol),
    ];
    match bulgu.deger.as_deref() {
        Some("") => argumanlar.push("/ve".to_string()),
        Some(deger) => {
            argumanlar.push("/v".to_string());
            argumanlar.push(deger.to_string());
        }
        None => {}
    }
    argumanlar.push("/f".to_string());
    argumanlar
}

fn hata_ozeti(sonuc: &KomutSonucu) -> String {
    let metin = if sonuc.stderr.trim().is_empty() {
        sonuc.stdout.trim()
    } else {
        sonuc.stderr.trim()
    };
    metin
        .lines()
        .map(str::trim)
        .find(|satir| !satir.is_empty())
        .unwrap_or("bilinmeyen hata")
        .chars()
        .take(300)
        .collect()
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

fn kove_metni(hklm: bool) -> &'static str {
    if hklm {
        "HKLM"
    } else {
        "HKCU"
    }
}

fn kok_ac(kok: &KayitKoku) -> Option<RegKey> {
    let hkey = if kok.hklm {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let bayrak = KEY_READ
        | if kok.gorunum == 32 {
            KEY_WOW64_32KEY
        } else {
            KEY_WOW64_64KEY
        };
    RegKey::predef(hkey)
        .open_subkey_with_flags(kok.yol, bayrak)
        .ok()
}

struct BulguGirdisi {
    hklm: bool,
    kategori: &'static str,
    ad: String,
    ayrinti: String,
    yol: String,
    deger: Option<String>,
}

struct YedekDurumu {
    klasor: PathBuf,
    yedeklenen: HashSet<String>,
    hatali: HashSet<String>,
}

impl YedekDurumu {
    fn yeni(klasor: &Path) -> Self {
        Self {
            klasor: klasor.to_path_buf(),
            yedeklenen: HashSet::new(),
            hatali: HashSet::new(),
        }
    }

    fn hazirla(&mut self, calistirici: &dyn KomutCalistirici, anahtar: &str) -> Result<(), String> {
        if self.hatali.contains(anahtar) {
            return Err("registry yedeği alınamadığı için silinmedi.".into());
        }
        if self.yedeklenen.contains(anahtar) {
            return Ok(());
        }
        let dosya = self
            .klasor
            .join(format!("registry-{:016x}.reg", kayit_ozeti(anahtar)));
        match kayit_yedekle(calistirici, anahtar, &dosya) {
            Ok(()) => {
                self.yedeklenen.insert(anahtar.to_string());
                Ok(())
            }
            Err(hata) => {
                self.hatali.insert(anahtar.to_string());
                Err(format!("registry yedeği alınamadı: {hata}"))
            }
        }
    }
}

fn bulgu_ekle(
    sonuc: &mut Vec<KayitDefteriBulgu>,
    gorulen: &mut HashSet<String>,
    girdi: BulguGirdisi,
) {
    let mut bulgu = KayitDefteriBulgu {
        kimlik: String::new(),
        kategori: girdi.kategori.to_string(),
        ad: girdi.ad,
        ayrinti: girdi.ayrinti,
        kove: kove_metni(girdi.hklm).to_string(),
        yol: girdi.yol,
        deger: girdi.deger,
    };
    bulgu.kimlik = bulgu.kimlik_uret();
    if !gorulen.insert(bulgu.kimlik.clone()) {
        return;
    }
    sonuc.push(bulgu);
}

fn uninstall_tara(
    kok: &KayitKoku,
    sonuc: &mut Vec<KayitDefteriBulgu>,
    gorulen: &mut HashSet<String>,
) {
    let hkey = if kok.hklm {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let bayrak = KEY_READ
        | if kok.gorunum == 32 {
            KEY_WOW64_32KEY
        } else {
            KEY_WOW64_64KEY
        };
    let Ok(anahtar) = RegKey::predef(hkey).open_subkey_with_flags(kok.yol, bayrak) else {
        return;
    };
    for alt in anahtar.enum_keys().flatten() {
        let Ok(alt_anahtar) = anahtar.open_subkey_with_flags(&alt, bayrak) else {
            continue;
        };
        if sayi(&alt_anahtar, "SystemComponent") == Some(1)
            || sayi(&alt_anahtar, "NoRemove") == Some(1)
            || guncelleme_mi(&alt_anahtar)
        {
            continue;
        }
        let Some(ad) = metin(&alt_anahtar, "DisplayName") else {
            continue;
        };
        let Some(komut) = metin(&alt_anahtar, "UninstallString")
            .or_else(|| metin(&alt_anahtar, "QuietUninstallString"))
        else {
            continue;
        };
        let Some(yol) = uninstall_yolu_ayikla(&komut) else {
            continue;
        };
        if Path::new(&yol).exists() {
            continue;
        }
        bulgu_ekle(
            sonuc,
            gorulen,
            BulguGirdisi {
                hklm: kok.hklm,
                kategori: KATEGORI_KALDIRMA,
                ad,
                ayrinti: format!("Kaldırma komutu bulunamadı: {yol}"),
                yol: format!("{}\\{}", kok.yol, alt),
                deger: None,
            },
        );
    }
}

fn genislet(metin: &str) -> Option<String> {
    if !metin.contains('%') {
        return Some(metin.to_string());
    }
    let mut sonuc = String::new();
    let mut kalan = metin;
    while let Some(bas) = kalan.find('%') {
        sonuc.push_str(&kalan[..bas]);
        let kalan_sonra = &kalan[bas + 1..];
        let son = kalan_sonra.find('%')?;
        let degisken = &kalan_sonra[..son];
        if degisken.is_empty() {
            return None;
        }
        sonuc.push_str(&std::env::var(degisken).ok()?);
        kalan = &kalan_sonra[son + 1..];
    }
    sonuc.push_str(kalan);
    Some(sonuc)
}

fn app_path_tara(
    kok: &KayitKoku,
    sonuc: &mut Vec<KayitDefteriBulgu>,
    gorulen: &mut HashSet<String>,
) {
    let Some(anahtar) = kok_ac(kok) else {
        return;
    };
    let bayrak = KEY_READ
        | if kok.gorunum == 32 {
            KEY_WOW64_32KEY
        } else {
            KEY_WOW64_64KEY
        };
    for alt in anahtar.enum_keys().flatten() {
        let Ok(alt_anahtar) = anahtar.open_subkey_with_flags(&alt, bayrak) else {
            continue;
        };
        let Some(ham) = metin(&alt_anahtar, "") else {
            continue;
        };
        let Some(yol) = genislet(&ham) else {
            continue;
        };
        if !mutlak_hedef_mi(&yol) || Path::new(&yol).exists() {
            continue;
        }
        bulgu_ekle(
            sonuc,
            gorulen,
            BulguGirdisi {
                hklm: kok.hklm,
                kategori: KATEGORI_UYGULAMA_YOLU,
                ad: alt.clone(),
                ayrinti: format!("Uygulama yolu bulunamadı: {yol}"),
                yol: format!("{}\\{}", kok.yol, alt),
                deger: None,
            },
        );
    }
}

fn shared_dll_tara(
    kok: &KayitKoku,
    sonuc: &mut Vec<KayitDefteriBulgu>,
    gorulen: &mut HashSet<String>,
) {
    let Some(anahtar) = kok_ac(kok) else {
        return;
    };
    for kayit in anahtar.enum_values().flatten() {
        let (ad, _) = kayit;
        if ad.is_empty() {
            continue;
        }
        let Some(yol) = genislet(&ad) else {
            continue;
        };
        if !mutlak_hedef_mi(&yol) || Path::new(&yol).exists() {
            continue;
        }
        bulgu_ekle(
            sonuc,
            gorulen,
            BulguGirdisi {
                hklm: kok.hklm,
                kategori: KATEGORI_PAYLASILAN_DLL,
                ad: ad.clone(),
                ayrinti: format!("Paylaşılan DLL bulunamadı: {yol}"),
                yol: kok.yol.to_string(),
                deger: Some(ad),
            },
        );
    }
}

fn windows_klasoru() -> Option<PathBuf> {
    std::env::var_os("SystemRoot")
        .or_else(|| std::env::var_os("WINDIR"))
        .map(PathBuf::from)
        .filter(|yol| yol.is_absolute())
}

fn dosya_yok_mu(yol: &str) -> bool {
    matches!(Path::new(yol).try_exists(), Ok(false))
}

fn yazi_tipi_tara(sonuc: &mut Vec<KayitDefteriBulgu>, gorulen: &mut HashSet<String>) {
    let Ok(anahtar) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(YAZI_TIPI_YOLU, KEY_READ | KEY_WOW64_64KEY)
    else {
        return;
    };
    let Some(klasor) = windows_klasoru().map(|yol| yol.join("Fonts")) else {
        return;
    };
    let klasor = klasor.to_string_lossy();
    for (ad, _) in anahtar.enum_values().flatten() {
        if ad.is_empty() {
            continue;
        }
        let Ok(veri) = anahtar.get_value::<String, _>(&ad) else {
            continue;
        };
        let Some(hedef) =
            yazi_tipi_hedef_yolu(&veri, &klasor).or_else(|| yazi_tipi_hedef_yolu(&ad, &klasor))
        else {
            continue;
        };
        if !dosya_yok_mu(&hedef) {
            continue;
        }
        bulgu_ekle(
            sonuc,
            gorulen,
            BulguGirdisi {
                hklm: true,
                kategori: KATEGORI_YAZI_TIPI,
                ad: ad.clone(),
                ayrinti: format!("Yazı tipi dosyası bulunamadı: {hedef}"),
                yol: YAZI_TIPI_YOLU.to_string(),
                deger: Some(ad),
            },
        );
    }
}

fn ses_olayi_tara(sonuc: &mut Vec<KayitDefteriBulgu>, gorulen: &mut HashSet<String>) {
    let Ok(apps) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(SES_OLAYI_YOLU, KEY_READ)
    else {
        return;
    };
    let Some(sistem_kok) = windows_klasoru() else {
        return;
    };
    let sistem_kok = sistem_kok.to_string_lossy();
    for uygulama in apps.enum_keys().flatten() {
        let Ok(uygulama_anahtari) = apps.open_subkey(&uygulama) else {
            continue;
        };
        for olay in uygulama_anahtari.enum_keys().flatten() {
            let Ok(anahtar) = uygulama_anahtari.open_subkey(format!(r"{olay}\.Current")) else {
                continue;
            };
            let Ok(veri) = anahtar.get_value::<String, _>("") else {
                continue;
            };
            let Some(hedef) = ses_olayi_hedef_yolu(&veri, &sistem_kok) else {
                continue;
            };
            if !dosya_yok_mu(&hedef) {
                continue;
            }
            bulgu_ekle(
                sonuc,
                gorulen,
                BulguGirdisi {
                    hklm: false,
                    kategori: KATEGORI_SES_OLAYI,
                    ad: format!("{uygulama}\\{olay}"),
                    ayrinti: format!("Ses dosyası bulunamadı: {hedef}"),
                    yol: format!(r"{SES_OLAYI_YOLU}\{uygulama}\{olay}\.Current"),
                    deger: Some(String::new()),
                },
            );
        }
    }
}

fn mui_tara(sonuc: &mut Vec<KayitDefteriBulgu>, gorulen: &mut HashSet<String>) {
    let Ok(anahtar) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(MUI_YOLU, KEY_READ)
    else {
        return;
    };
    for (ad, _) in anahtar.enum_values().flatten() {
        let Some(exe) = mui_uygulama_yolu_ayikla(&ad) else {
            continue;
        };
        if !dosya_yok_mu(&exe) {
            continue;
        }
        bulgu_ekle(
            sonuc,
            gorulen,
            BulguGirdisi {
                hklm: false,
                kategori: KATEGORI_MUI,
                ad: exe.clone(),
                ayrinti: format!("MUI önbelleğindeki uygulama bulunamadı: {exe}"),
                yol: MUI_YOLU.to_string(),
                deger: Some(ad),
            },
        );
    }
}

fn yardim_tara(sonuc: &mut Vec<KayitDefteriBulgu>, gorulen: &mut HashSet<String>) {
    let Ok(anahtar) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(YARDIM_YOLU, KEY_READ | KEY_WOW64_64KEY)
    else {
        return;
    };
    let Some(klasor) = windows_klasoru().map(|yol| yol.join("Help")) else {
        return;
    };
    let klasor = klasor.to_string_lossy();
    for (ad, _) in anahtar.enum_values().flatten() {
        if ad.is_empty() {
            continue;
        }
        let Ok(veri) = anahtar.get_value::<String, _>(&ad) else {
            continue;
        };
        let Some(hedef) = yardim_dosyasi_hedef_yolu(&veri, &klasor) else {
            continue;
        };
        if !dosya_yok_mu(&hedef) {
            continue;
        }
        bulgu_ekle(
            sonuc,
            gorulen,
            BulguGirdisi {
                hklm: true,
                kategori: KATEGORI_YARDIM,
                ad: ad.clone(),
                ayrinti: format!("Yardım dosyası bulunamadı: {hedef}"),
                yol: YARDIM_YOLU.to_string(),
                deger: Some(ad),
            },
        );
    }
}

fn kayit_bulgu_dogrula(bulgu: &KayitDefteriBulgu) -> Sonuc<()> {
    if bulgu.kove != "HKLM" && bulgu.kove != "HKCU" {
        return Err(Hata::Dogrulama("Geçersiz registry kökü.".into()));
    }
    if !kayit_yolu_guvenli_mi(&bulgu.yol) {
        return Err(Hata::Dogrulama(format!(
            "{}: geçersiz registry yolu.",
            bulgu.ad
        )));
    }
    let Some((kokler, alt_anahtar_serbest)) = kategori_kokleri(&bulgu.kategori) else {
        return Err(Hata::Dogrulama("Bilinmeyen bulgu kategorisi.".into()));
    };
    if !kok_izinli_mi(bulgu, kokler, alt_anahtar_serbest)
        || (bulgu.kategori == KATEGORI_SES_OLAYI && !ses_olay_yolu_uygun_mu(&bulgu.yol))
    {
        return Err(Hata::Dogrulama(format!(
            "{}: izinli registry kökünün dışında.",
            bulgu.ad
        )));
    }
    if yalniz_deger_kategorisi(&bulgu.kategori) && bulgu.deger.is_none() {
        return Err(Hata::Dogrulama(format!(
            "{}: bu kategori yalnızca değer silmeyi destekler.",
            bulgu.ad
        )));
    }
    if !deger_adi_uygun_mu(bulgu) {
        return Err(Hata::Dogrulama(format!(
            "{}: geçersiz registry değer adı.",
            bulgu.ad
        )));
    }
    Ok(())
}

fn kategori_kokleri(kategori: &str) -> Option<(&'static [KayitKoku], bool)> {
    match kategori {
        KATEGORI_KALDIRMA => Some((&UNINSTALL_KOKLERI[..], true)),
        KATEGORI_UYGULAMA_YOLU => Some((&APP_PATH_KOKLERI[..], true)),
        KATEGORI_PAYLASILAN_DLL => Some((&SHARED_DLL_KOKLERI[..], true)),
        KATEGORI_YAZI_TIPI => Some((&YAZI_TIPI_KOKLERI[..], false)),
        KATEGORI_SES_OLAYI => Some((&SES_OLAYI_KOKLERI[..], true)),
        KATEGORI_MUI => Some((&MUI_KOKLERI[..], false)),
        KATEGORI_YARDIM => Some((&YARDIM_KOKLERI[..], false)),
        _ => None,
    }
}

fn kok_izinli_mi(
    bulgu: &KayitDefteriBulgu,
    kokler: &[KayitKoku],
    alt_anahtar_serbest: bool,
) -> bool {
    let hklm = bulgu.kove == "HKLM";
    kokler.iter().any(|kok| {
        if kok.hklm != hklm {
            return false;
        }
        if bulgu.yol.eq_ignore_ascii_case(kok.yol) {
            return true;
        }
        alt_anahtar_serbest
            && bulgu
                .yol
                .to_ascii_lowercase()
                .starts_with(&format!("{}\\", kok.yol.to_ascii_lowercase()))
    })
}

fn ses_olay_yolu_uygun_mu(yol: &str) -> bool {
    let kucuk = yol.to_ascii_lowercase();
    let onek = format!("{}\\", SES_OLAYI_YOLU.to_ascii_lowercase());
    let Some(alt) = kucuk.strip_prefix(&onek) else {
        return false;
    };
    let parcalar = alt.split('\\').collect::<Vec<_>>();
    parcalar.len() >= 3
        && parcalar.iter().all(|parca| !parca.is_empty())
        && parcalar.last().is_some_and(|son| *son == ".current")
}

fn yalniz_deger_kategorisi(kategori: &str) -> bool {
    matches!(
        kategori,
        KATEGORI_YAZI_TIPI | KATEGORI_SES_OLAYI | KATEGORI_MUI | KATEGORI_YARDIM
    )
}

fn deger_adi_uygun_mu(bulgu: &KayitDefteriBulgu) -> bool {
    let Some(deger) = &bulgu.deger else {
        return true;
    };
    if bulgu.kategori == KATEGORI_MUI {
        return deger.len() <= AZAMI_MUI_DEGER_ADI && mui_uygulama_yolu_ayikla(deger).is_some();
    }
    if deger.is_empty() {
        return bulgu.kategori == KATEGORI_SES_OLAYI;
    }
    if bulgu.kategori == KATEGORI_SES_OLAYI {
        return false;
    }
    deger.len() <= AZAMI_DEGER_ADI && kayit_degeri_guvenli_mi(deger)
}

fn kayit_var_mi(bulgu: &KayitDefteriBulgu) -> bool {
    let hkey = if bulgu.kove == "HKLM" {
        HKEY_LOCAL_MACHINE
    } else {
        HKEY_CURRENT_USER
    };
    let Ok(anahtar) = RegKey::predef(hkey).open_subkey_with_flags(&bulgu.yol, KEY_READ) else {
        return false;
    };
    match &bulgu.deger {
        Some(deger) => anahtar.get_raw_value(deger).is_ok(),
        None => true,
    }
}

fn kayit_ozeti(metin: &str) -> u64 {
    let mut ozet = DefaultHasher::new();
    metin.hash(&mut ozet);
    ozet.finish()
}

fn mutlak_hedef_mi(yol: &str) -> bool {
    let baytlar = yol.as_bytes();
    baytlar.len() > 3
        && baytlar[0].is_ascii_alphabetic()
        && baytlar[1] == b':'
        && matches!(baytlar[2], b'\\' | b'/')
}

fn kisayol_kokleri() -> Vec<PathBuf> {
    let mut yollar = Vec::new();
    if let Some(profil) = std::env::var_os("USERPROFILE") {
        yollar.push(PathBuf::from(&profil).join("Desktop"));
        yollar.push(PathBuf::from(&profil).join(r"AppData\Roaming\Microsoft\Windows\Start Menu"));
        yollar.push(
            PathBuf::from(&profil)
                .join(r"AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup"),
        );
    }
    if let Some(ortak) = std::env::var_os("PUBLIC") {
        yollar.push(PathBuf::from(&ortak).join("Desktop"));
    }
    if let Some(programdata) = std::env::var_os("PROGRAMDATA") {
        yollar.push(PathBuf::from(&programdata).join(r"Microsoft\Windows\Start Menu"));
        yollar.push(
            PathBuf::from(&programdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"),
        );
    }
    yollar
        .into_iter()
        .filter_map(|yol| fs::canonicalize(yol).ok())
        .collect()
}

fn benzersiz_yol(klasor: &Path, ad: &str) -> PathBuf {
    let aday = klasor.join(ad);
    if !aday.exists() {
        return aday;
    }
    let yol = Path::new(ad);
    let govde = yol
        .file_stem()
        .map(|govde| govde.to_string_lossy().into_owned())
        .unwrap_or_else(|| "kisayol".to_string());
    let uzanti = yol
        .extension()
        .map(|uzanti| format!(".{}", uzanti.to_string_lossy()))
        .unwrap_or_default();
    for sayac in 1u32.. {
        let aday = klasor.join(format!("{govde}-{sayac}{uzanti}"));
        if !aday.exists() {
            return aday;
        }
    }
    klasor.join(ad)
}

fn tasi(kaynak: &Path, hedef: &Path) -> Result<(), String> {
    if fs::rename(kaynak, hedef).is_ok() {
        return Ok(());
    }
    fs::copy(kaynak, hedef).map_err(|hata| format!("kopyalanamadı: {hata}"))?;
    fs::remove_file(kaynak).map_err(|hata| format!("kopyalandı ancak kaynak silinemedi: {hata}"))
}

fn manifest_yaz(manifest: &Path, kaynak: &Path, hedef: &Path) -> std::io::Result<()> {
    let yeni = !manifest.exists();
    let mut dosya = OpenOptions::new()
        .create(true)
        .append(true)
        .open(manifest)?;
    if yeni {
        writeln!(dosya, "kaynak\thedef")?;
    }
    writeln!(
        dosya,
        "{}\t{}",
        kaynak.to_string_lossy().replace('\t', " "),
        hedef.to_string_lossy().replace('\t', " ")
    )
}

fn handler_clsid(anahtar: &RegKey, alt: &str) -> Option<String> {
    if clsid_gecerli_mi(alt) {
        return Some(alt.to_string());
    }
    if let Some(deger) = metin(anahtar, "") {
        if clsid_gecerli_mi(&deger) {
            return Some(deger);
        }
    }
    for kayit in anahtar.enum_values().flatten() {
        let (_, veri) = kayit;
        let aday = String::from_utf16_lossy(
            &veri
                .bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|parca| u16::from_le_bytes(*parca))
                .collect::<Vec<_>>(),
        )
        .trim_end_matches('\0')
        .to_string();
        if clsid_gecerli_mi(&aday) {
            return Some(aday);
        }
    }
    None
}

fn clsid_adi(clsid: &str) -> Option<String> {
    let yol = format!(r"CLSID\{clsid}");
    let anahtar = RegKey::predef(HKEY_CLASSES_ROOT)
        .open_subkey_with_flags(&yol, KEY_READ)
        .ok()?;
    metin(&anahtar, "")
}

fn kayit_blokeli_mi(clsid: &str) -> bool {
    for kok in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(anahtar) = RegKey::predef(kok).open_subkey_with_flags(BLOK_YOLU, KEY_READ) else {
            continue;
        };
        if anahtar
            .enum_values()
            .flatten()
            .any(|(ad, _)| ad.eq_ignore_ascii_case(clsid))
        {
            return true;
        }
    }
    false
}

fn clsid_normallestir(clsid: &str) -> Sonuc<String> {
    if !clsid_gecerli_mi(clsid) {
        return Err(Hata::Dogrulama("Geçersiz CLSID.".into()));
    }
    let temiz = clsid.trim().trim_matches(['{', '}']);
    Ok(format!("{{{temiz}}}"))
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::sync::Mutex;

    struct SahteCalistirici {
        kod: i32,
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
                stdout: String::new(),
                stderr: "hata".into(),
            })
        }
    }

    fn servis(kod: i32) -> (KayitKisayolServisi, Arc<SahteCalistirici>) {
        let calistirici = Arc::new(SahteCalistirici {
            kod,
            cagrilar: Mutex::new(Vec::new()),
        });
        (KayitKisayolServisi::yeni(calistirici.clone()), calistirici)
    }

    fn bulgu(yol: &str) -> KayitDefteriBulgu {
        KayitDefteriBulgu {
            kimlik: "test".into(),
            kategori: KATEGORI_UYGULAMA_YOLU.into(),
            ad: "Test".into(),
            ayrinti: "test".into(),
            kove: "HKCU".into(),
            yol: yol.into(),
            deger: None,
        }
    }

    #[test]
    fn app_paths_ve_shared_dll_kok_hatasi_duzeltildi() {
        let taban = format!(r"Software\WinNexusTest\Kok{}", kayit_ozeti("kok-testi"));
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (app, _) = hkcu
            .create_subkey_with_flags(format!(r"{taban}\App Paths"), KEY_WRITE)
            .unwrap();
        let (alt, _) = app.create_subkey("WinNexusEksik.exe").unwrap();
        alt.set_value("", &r"C:\WinNexusTest-Yok-olmayan\uygulama.exe")
            .unwrap();
        drop(alt);
        drop(app);
        let (dll, _) = hkcu
            .create_subkey_with_flags(format!(r"{taban}\SharedDLLs"), KEY_WRITE)
            .unwrap();
        dll.set_value(r"C:\WinNexusTest-Yok-olmayan\kutuphane.dll", &1u32)
            .unwrap();
        drop(dll);
        let app_yolu: &'static str = Box::leak(format!(r"{taban}\App Paths").into_boxed_str());
        let dll_yolu: &'static str = Box::leak(format!(r"{taban}\SharedDLLs").into_boxed_str());
        let app_kok = KayitKoku {
            hklm: false,
            yol: app_yolu,
            gorunum: 64,
        };
        let dll_kok = KayitKoku {
            hklm: false,
            yol: dll_yolu,
            gorunum: 64,
        };
        let mut sonuc = Vec::new();
        let mut gorulen = HashSet::new();
        app_path_tara(&app_kok, &mut sonuc, &mut gorulen);
        shared_dll_tara(&dll_kok, &mut sonuc, &mut gorulen);
        assert!(sonuc
            .iter()
            .any(|bulgu| bulgu.kategori == KATEGORI_UYGULAMA_YOLU));
        assert!(sonuc
            .iter()
            .any(|bulgu| bulgu.kategori == KATEGORI_PAYLASILAN_DLL));
        let _ = hkcu.delete_subkey_all(taban);
    }

    #[test]
    fn gecersiz_registry_yolu_reddedilir() {
        let (servis, calistirici) = servis(0);
        let karantina = tempfile::tempdir().unwrap();
        let hatali = bulgu(r"Software\Microsoft\Windows\CurrentVersion\Run");
        let sonuc = servis
            .kayit_defteri_temizle(&[hatali], karantina.path())
            .unwrap();
        assert_eq!(sonuc.islenen, 0);
        assert!(calistirici.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn disa_aktarma_basarisizsa_silmez() {
        let alt = format!("WinNexusTest{}", kayit_ozeti("merkez-test"));
        let kok = format!(r"{APP_PATHS_KULLANICI}\{alt}");
        let (anahtar, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(&kok, KEY_READ | KEY_WRITE)
            .unwrap();
        drop(anahtar);
        let (servis, calistirici) = servis(1);
        let karantina = tempfile::tempdir().unwrap();
        let kayit = bulgu(&kok);
        let sonuc = servis
            .kayit_defteri_temizle(&[kayit], karantina.path())
            .unwrap();
        assert_eq!(sonuc.islenen, 0);
        assert!(RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey::<&str>(&kok)
            .is_ok());
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 1);
        assert_eq!(cagrilar[0].0, "reg.exe");
        assert_eq!(cagrilar[0].1[0], "export");
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&kok);
    }

    #[test]
    fn kaldirma_komutu_olmayan_kayit_bulgu_uretmez() {
        let alt = format!("WinNexusTest{}", kayit_ozeti("bos-kayit"));
        let yol = format!(r"{UNINSTALL_KULLANICI}\{alt}");
        let (anahtar, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(&yol, KEY_READ | KEY_WRITE)
            .unwrap();
        anahtar.set_value("DisplayName", &"WinNexusTest").unwrap();
        drop(anahtar);
        let (servis, _) = servis(0);
        let bulgular = servis.kayit_defteri_tara().unwrap();
        assert!(!bulgular.iter().any(|bulgu| bulgu.ad == "WinNexusTest"));
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&yol);
    }

    #[test]
    fn clsid_normallestirilir() {
        assert_eq!(
            clsid_normallestir("{86CA1AA0-34AA-4E8B-A509-50C905BAE2A2}").unwrap(),
            "{86CA1AA0-34AA-4E8B-A509-50C905BAE2A2}"
        );
        assert!(clsid_normallestir("bozuk").is_err());
    }

    #[test]
    fn mutlak_hedef_denetimi() {
        assert!(mutlak_hedef_mi(r"C:\Uygulama\a.exe"));
        assert!(!mutlak_hedef_mi(r"Uygulama\a.exe"));
        assert!(!mutlak_hedef_mi(r"\\sunucu\paylasim\a.exe"));
    }

    #[test]
    fn manifest_yazilir() {
        let dizin = tempfile::tempdir().unwrap();
        let manifest = dizin.path().join("manifest.tsv");
        manifest_yaz(&manifest, Path::new(r"C:\a.lnk"), Path::new(r"C:\q\a.lnk")).unwrap();
        manifest_yaz(&manifest, Path::new(r"C:\b.lnk"), Path::new(r"C:\q\b.lnk")).unwrap();
        let metin = fs::read_to_string(&manifest).unwrap();
        assert!(metin.starts_with("kaynak\thedef"));
        assert_eq!(metin.lines().count(), 3);
    }

    fn kayit_bulgu(
        kategori: &str,
        kove: &str,
        yol: &str,
        deger: Option<&str>,
    ) -> KayitDefteriBulgu {
        KayitDefteriBulgu {
            kimlik: "test".into(),
            kategori: kategori.into(),
            ad: "Test".into(),
            ayrinti: "test".into(),
            kove: kove.into(),
            yol: yol.into(),
            deger: deger.map(str::to_string),
        }
    }

    const SES_TEST_YOLU: &str = r"AppEvents\Schemes\Apps\Explorer\ActivatingDocument\.Current";
    const MUI_TEST_DEGERI: &str = r"C:\App\a.exe.FriendlyAppName";

    fn dogrula(bulgu: KayitDefteriBulgu, gecerli: bool) {
        let sonuc = kayit_bulgu_dogrula(&bulgu);
        assert_eq!(sonuc.is_ok(), gecerli, "{}", bulgu.kategori);
    }

    #[test]
    fn yeni_kategori_dogrulama_gecerli_bulgulari_kabul_eder() {
        dogrula(
            kayit_bulgu(
                KATEGORI_YAZI_TIPI,
                "HKLM",
                YAZI_TIPI_YOLU,
                Some("Arial (TrueType)"),
            ),
            true,
        );
        dogrula(
            kayit_bulgu(KATEGORI_SES_OLAYI, "HKCU", SES_TEST_YOLU, Some("")),
            true,
        );
        dogrula(
            kayit_bulgu(KATEGORI_MUI, "HKCU", MUI_YOLU, Some(MUI_TEST_DEGERI)),
            true,
        );
        dogrula(
            kayit_bulgu(KATEGORI_YARDIM, "HKLM", YARDIM_YOLU, Some("windows.hlp")),
            true,
        );
    }

    #[test]
    fn yeni_kategori_dogrulama_gecersiz_bulgulari_reddeder() {
        dogrula(
            kayit_bulgu(
                KATEGORI_YAZI_TIPI,
                "HKCU",
                YAZI_TIPI_YOLU,
                Some("arial.ttf"),
            ),
            false,
        );
        dogrula(
            kayit_bulgu(KATEGORI_MUI, "HKLM", MUI_YOLU, Some(MUI_TEST_DEGERI)),
            false,
        );
        dogrula(
            kayit_bulgu(
                KATEGORI_SES_OLAYI,
                "HKCU",
                r"AppEvents\Schemes\Apps\Explorer\ActivatingDocument",
                Some(""),
            ),
            false,
        );
        dogrula(
            kayit_bulgu(
                KATEGORI_YARDIM,
                "HKLM",
                r"SOFTWARE\Microsoft\Windows\Help\..",
                Some("a.hlp"),
            ),
            false,
        );
        dogrula(
            kayit_bulgu(KATEGORI_MUI, "HKCU", MUI_YOLU, Some("bozuk-deger")),
            false,
        );
        dogrula(
            kayit_bulgu(KATEGORI_SES_OLAYI, "HKCU", SES_TEST_YOLU, Some(".Current")),
            false,
        );
        dogrula(
            kayit_bulgu(KATEGORI_YAZI_TIPI, "HKLM", YAZI_TIPI_YOLU, Some("a\"b")),
            false,
        );
    }

    #[test]
    fn yeni_kategorilerde_anahtar_silme_reddedilir() {
        let kategoriler = [
            (KATEGORI_YAZI_TIPI, "HKLM", YAZI_TIPI_YOLU),
            (
                KATEGORI_SES_OLAYI,
                "HKCU",
                r"AppEvents\Schemes\Apps\Explorer\ActivatingDocument\.Current",
            ),
            (KATEGORI_MUI, "HKCU", MUI_YOLU),
            (KATEGORI_YARDIM, "HKLM", YARDIM_YOLU),
        ];
        for (kategori, kove, yol) in kategoriler {
            let bulgu = kayit_bulgu(kategori, kove, yol, None);
            assert!(kayit_bulgu_dogrula(&bulgu).is_err());
        }
    }

    #[test]
    fn kayit_silme_argumanlari_deger_ve_varsayilan_uretir() {
        let deger = r"C:\App\a.exe.FriendlyAppName";
        let argumanlar =
            kayit_silme_argumanlari(&kayit_bulgu(KATEGORI_MUI, "HKCU", MUI_YOLU, Some(deger)));
        assert_eq!(argumanlar[0], "delete");
        assert_eq!(argumanlar[1], format!("HKCU\\{MUI_YOLU}"));
        assert_eq!(argumanlar[2], "/v");
        assert_eq!(argumanlar[3], deger);
        assert_eq!(argumanlar[4], "/f");

        let varsayilan = kayit_silme_argumanlari(&kayit_bulgu(
            KATEGORI_SES_OLAYI,
            "HKCU",
            r"AppEvents\Schemes\Apps\Explorer\ActivatingDocument\.Current",
            Some(""),
        ));
        assert_eq!(varsayilan.len(), 4);
        assert_eq!(varsayilan[2], "/ve");
        assert_eq!(varsayilan[3], "/f");
    }

    #[test]
    fn bos_yedek_dosyasi_gecersiz_sayilir() {
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("yedek.reg");
        assert!(!yedek_gecerli(&dosya));
        fs::write(&dosya, "Windows Registry Editor Version 5.00\n").unwrap();
        assert!(yedek_gecerli(&dosya));
    }

    #[test]
    fn dosya_olusturmayan_export_yedek_sayilmaz() {
        let (_, calistirici) = servis(0);
        let dizin = tempfile::tempdir().unwrap();
        let dosya = dizin.path().join("yedek.reg");
        assert!(kayit_yedekle(calistirici.as_ref(), r"HKCU\Software\Test", &dosya).is_err());
    }

    #[test]
    fn yalnizca_gercekten_olmayan_dosya_bulgu_uretir() {
        let dizin = tempfile::tempdir().unwrap();
        let var_olan = dizin.path().join("a.ttf");
        fs::write(&var_olan, b"x").unwrap();
        assert!(!dosya_yok_mu(&var_olan.to_string_lossy()));
        assert!(dosya_yok_mu(
            &dizin.path().join("yok.ttf").to_string_lossy()
        ));
    }

    #[test]
    fn bulgu_kimligi_kategoriyi_icerir_ve_tekillestirir() {
        let mut sonuc = Vec::new();
        let mut gorulen = HashSet::new();
        let girdi = |kategori| BulguGirdisi {
            hklm: true,
            kategori,
            ad: "Test".into(),
            ayrinti: "test".into(),
            yol: YAZI_TIPI_YOLU.into(),
            deger: Some("arial.ttf".into()),
        };
        bulgu_ekle(&mut sonuc, &mut gorulen, girdi(KATEGORI_YAZI_TIPI));
        bulgu_ekle(&mut sonuc, &mut gorulen, girdi(KATEGORI_YAZI_TIPI));
        bulgu_ekle(&mut sonuc, &mut gorulen, girdi(KATEGORI_YARDIM));
        assert_eq!(sonuc.len(), 2);
        assert!(sonuc[0].kimlik.contains(KATEGORI_YAZI_TIPI));
        assert!(sonuc[1].kimlik.contains(KATEGORI_YARDIM));
    }
}
