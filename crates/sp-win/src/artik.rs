use crate::winget::KomutCalistirici;
use sp_core::artik::{
    kayit_adi_uygun_mu, normalize_adi, paket_tam_adi_gecerli_mi, store_paketlerini_ayristir,
    ArtikOgesi, ArtikTuru, KarantinaSonucu, StorePaketi,
};
use sp_core::error::{Hata, Sonuc};
use sp_core::kaldirici::KuruluUygulama;
use std::collections::HashSet;
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
use winreg::RegKey;

const ARTIK_ORTAMLARI: [&str; 3] = ["APPDATA", "LOCALAPPDATA", "PROGRAMDATA"];
const SISTEM_KLASORLERI: [&str; 4] = ["windows", "microsoft", "common files", "packages"];
const SISTEM_KAYIT_ADLARI: [&str; 8] = [
    "microsoft",
    "windows",
    "classes",
    "policies",
    "clients",
    "registeredapplications",
    "uninstall",
    "wow6432node",
];
const STORE_LISTE_BETIGI: &str = "Get-AppxPackage | Select-Object @{N='Ad';E={$_.Name}},@{N='TamAd';E={$_.PackageFullName}},@{N='Surum';E={$_.Version.ToString()}},@{N='Yayinci';E={$_.Publisher}} | ConvertTo-Json -Compress";
const STORE_KALDIRMA_BETIGI: &str = "$ErrorActionPreference='Stop'; try { Remove-AppxPackage -Package '{0}' -ErrorAction Stop; exit 0 } catch { [Console]::Error.WriteLine($_.Exception.Message); exit 1 }";
const REGISTRY_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const STORE_LISTE_ZAMAN_ASIMI: Duration = Duration::from_secs(180);
const STORE_KALDIRMA_ZAMAN_ASIMI: Duration = Duration::from_secs(1800);
const CIKTI_SINIRI: usize = 2000;
const DOSYA_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

#[derive(Clone)]
pub struct ArtikServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl ArtikServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    pub fn tara(&self, uygulama: &KuruluUygulama) -> Sonuc<Vec<ArtikOgesi>> {
        let hedefler = hedef_adlar(uygulama);
        let mut ogeler = Vec::new();
        let mut gorulen = HashSet::new();
        kurulum_ogesi(uygulama, &mut ogeler, &mut gorulen);
        klasor_ogeleri(&hedefler, &mut ogeler, &mut gorulen);
        registry_ogeleri(&hedefler, &mut ogeler, &mut gorulen);
        ogeler.sort_by_key(|oge| oge.yol.to_lowercase());
        Ok(ogeler)
    }

    pub fn karantinaya_tasi(&self, ogeler: &[ArtikOgesi], hedef: &Path) -> Sonuc<KarantinaSonucu> {
        fs::create_dir_all(hedef).map_err(|hata| {
            Hata::Beklenmeyen(format!("Karantina dizini oluşturulamadı: {hata}"))
        })?;
        let mut sonuc = KarantinaSonucu::default();
        for oge in ogeler {
            let islem = match oge.tur {
                ArtikTuru::Registry => self.registry_karantina(oge, hedef),
                ArtikTuru::Dosya | ArtikTuru::Klasor => dosya_karantina(oge, hedef),
            };
            match islem {
                Ok(()) => sonuc.tasinan += 1,
                Err(hata) => sonuc.hatalar.push(format!("{}: {hata}", oge.yol)),
            }
        }
        Ok(sonuc)
    }

    pub fn store_paketleri(&self) -> Sonuc<Vec<StorePaketi>> {
        let sonuc = self.calistirici.calistir(
            "powershell",
            &powershell_argumanlari(STORE_LISTE_BETIGI),
            STORE_LISTE_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Store paketleri okunamadı: {}",
                sinirla(&sonuc.stderr)
            )));
        }
        store_paketlerini_ayristir(&sonuc.stdout)
    }

    pub fn store_kaldir(&self, tam_ad: &str) -> Sonuc<()> {
        let tam_ad = tam_ad.trim();
        if !paket_tam_adi_gecerli_mi(tam_ad) {
            return Err(Hata::Dogrulama("Store paket adı geçersiz.".into()));
        }
        let betik = STORE_KALDIRMA_BETIGI.replacen("{0}", tam_ad, 1);
        let sonuc = self.calistirici.calistir(
            "powershell",
            &powershell_argumanlari(&betik),
            STORE_KALDIRMA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{tam_ad} kaldırılamadı: {}",
                sinirla(&sonuc.stderr)
            )));
        }
        Ok(())
    }

    fn registry_karantina(&self, oge: &ArtikOgesi, hedef: &Path) -> Sonuc<()> {
        let (kove, alt_yol) = registry_yolu_ayristir(&oge.yol)?;
        let tam_yol = format!("{kove}\\{alt_yol}");
        let dosya_adi = format!("{}.reg", sanitize_ad(&oge.ad));
        let yedek = benzersiz_yol(hedef, &dosya_adi);
        let yedek_metni = yedek.to_string_lossy().to_string();
        let export = self.calistirici.calistir(
            "reg.exe",
            &[
                "export".to_string(),
                tam_yol.clone(),
                yedek_metni,
                "/y".to_string(),
            ],
            REGISTRY_ZAMAN_ASIMI,
        )?;
        if export.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Kayıt defteri yedeklenemedi: {}",
                sinirla(&export.stderr)
            )));
        }
        let silme = self.calistirici.calistir(
            "reg.exe",
            &["delete".to_string(), tam_yol, "/f".to_string()],
            REGISTRY_ZAMAN_ASIMI,
        )?;
        if silme.kod != 0 {
            return Err(Hata::DisArac(format!(
                "Kayıt defteri silinemedi: {}",
                sinirla(&silme.stderr)
            )));
        }
        Ok(())
    }
}

fn hedef_adlar(uygulama: &KuruluUygulama) -> Vec<String> {
    [
        uygulama.ad.as_str(),
        uygulama.yayinci.as_deref().unwrap_or_default(),
    ]
    .into_iter()
    .filter_map(normalize_adi)
    .collect::<HashSet<_>>()
    .into_iter()
    .collect()
}

fn kurulum_ogesi(
    uygulama: &KuruluUygulama,
    ogeler: &mut Vec<ArtikOgesi>,
    gorulen: &mut HashSet<String>,
) {
    let Some(konum) = uygulama
        .kurulum_konumu
        .as_deref()
        .map(str::trim)
        .map(|konum| konum.trim_matches('"'))
        .filter(|konum| !konum.is_empty())
    else {
        return;
    };
    let yol = Path::new(konum);
    let tur = if yol.is_dir() {
        ArtikTuru::Klasor
    } else if yol.is_file() {
        ArtikTuru::Dosya
    } else {
        return;
    };
    if !yol_guvenli_mi(yol) {
        return;
    }
    let oge = ArtikOgesi {
        kimlik: ArtikOgesi::kimlik_olustur(tur, konum),
        tur,
        ad: "Kurulum konumu".to_string(),
        yol: konum.to_string(),
        boyut_bayt: boyut_bayt(yol, tur),
    };
    oge_ekle(oge, ogeler, gorulen);
}

fn klasor_ogeleri(
    hedefler: &[String],
    ogeler: &mut Vec<ArtikOgesi>,
    gorulen: &mut HashSet<String>,
) {
    for degisken in ARTIK_ORTAMLARI {
        let Some(kok) = std::env::var_os(degisken).map(PathBuf::from) else {
            continue;
        };
        let Ok(girisler) = fs::read_dir(&kok) else {
            continue;
        };
        for giris in girisler.flatten() {
            let Ok(meta) = giris.metadata() else {
                continue;
            };
            if baglanti_mi(&meta) || !meta.is_dir() {
                continue;
            }
            let ad = giris.file_name().to_string_lossy().to_string();
            let Some(normal) = normalize_adi(&ad) else {
                continue;
            };
            if SISTEM_KLASORLERI.contains(&normal.as_str()) || !hedefler.contains(&normal) {
                continue;
            }
            let yol = giris.path();
            let oge = ArtikOgesi {
                kimlik: ArtikOgesi::kimlik_olustur(ArtikTuru::Klasor, &yol.to_string_lossy()),
                tur: ArtikTuru::Klasor,
                ad,
                yol: yol.to_string_lossy().to_string(),
                boyut_bayt: boyut_bayt(&yol, ArtikTuru::Klasor),
            };
            oge_ekle(oge, ogeler, gorulen);
        }
    }
}

fn registry_ogeleri(
    hedefler: &[String],
    ogeler: &mut Vec<ArtikOgesi>,
    gorulen: &mut HashSet<String>,
) {
    for (kove, kok) in [("HKCU", HKEY_CURRENT_USER), ("HKLM", HKEY_LOCAL_MACHINE)] {
        for aday in hedefler {
            if SISTEM_KAYIT_ADLARI.contains(&aday.as_str()) || !kayit_adi_uygun_mu(aday) {
                continue;
            }
            for alt in [
                format!("Software\\{aday}"),
                format!("Software\\WOW6432Node\\{aday}"),
            ] {
                if !kayit_var_mi(kok, &alt) {
                    continue;
                }
                let yol = format!("{kove}\\{alt}");
                let oge = ArtikOgesi {
                    kimlik: ArtikOgesi::kimlik_olustur(ArtikTuru::Registry, &yol),
                    tur: ArtikTuru::Registry,
                    ad: aday.clone(),
                    yol,
                    boyut_bayt: None,
                };
                oge_ekle(oge, ogeler, gorulen);
            }
        }
    }
}

fn kayit_var_mi(kok: winreg::HKEY, yol: &str) -> bool {
    RegKey::predef(kok)
        .open_subkey_with_flags(yol, KEY_READ | KEY_WOW64_64KEY)
        .is_ok()
}

fn oge_ekle(oge: ArtikOgesi, ogeler: &mut Vec<ArtikOgesi>, gorulen: &mut HashSet<String>) {
    if gorulen.insert(oge.kimlik.clone()) {
        ogeler.push(oge);
    }
}

fn boyut_bayt(yol: &Path, tur: ArtikTuru) -> Option<u64> {
    match tur {
        ArtikTuru::Dosya => fs::symlink_metadata(yol)
            .ok()
            .filter(|meta| meta.file_type().is_file() && !baglanti_mi(meta))
            .map(|meta| meta.len()),
        ArtikTuru::Klasor => fs::symlink_metadata(yol)
            .ok()
            .filter(|meta| meta.is_dir() && !baglanti_mi(meta))
            .map(|_| klasor_boyutu(yol)),
        ArtikTuru::Registry => None,
    }
}

fn baglanti_mi(meta: &fs::Metadata) -> bool {
    meta.file_attributes() & DOSYA_ATTRIBUTE_REPARSE_POINT != 0
}

fn klasor_boyutu(kok: &Path) -> u64 {
    let mut toplam = 0u64;
    let mut yigin = vec![kok.to_path_buf()];
    while let Some(dizin) = yigin.pop() {
        let Ok(girisler) = fs::read_dir(&dizin) else {
            continue;
        };
        for giris in girisler.flatten() {
            let Ok(meta) = giris.metadata() else {
                continue;
            };
            if baglanti_mi(&meta) {
                continue;
            }
            if meta.is_dir() {
                yigin.push(giris.path());
            } else {
                toplam = toplam.saturating_add(meta.len());
            }
        }
    }
    toplam
}

fn dosya_karantina(oge: &ArtikOgesi, hedef: &Path) -> Sonuc<()> {
    let kaynak = PathBuf::from(oge.yol.trim());
    if !yol_guvenli_mi(&kaynak) {
        return Err(Hata::Yetki("Yol karantinaya alınamaz.".into()));
    }
    if hedef.starts_with(&kaynak) {
        return Err(Hata::Dogrulama(
            "Karantina dizini kaynağın içinde olamaz.".into(),
        ));
    }
    if fs::symlink_metadata(&kaynak).is_err() {
        return Err(Hata::Beklenmeyen("Artık bulunamadı.".into()));
    }
    let hedef_yol = benzersiz_yol(hedef, &karantina_adi(oge));
    tasi(&kaynak, &hedef_yol).map_err(|hata| Hata::Beklenmeyen(format!("Taşınamadı: {hata}")))
}

fn karantina_adi(oge: &ArtikOgesi) -> String {
    let temel = sanitize_ad(&oge.ad);
    if oge.tur != ArtikTuru::Dosya {
        return temel;
    }
    match Path::new(&oge.yol)
        .extension()
        .and_then(|uzanti| uzanti.to_str())
    {
        Some(uzanti) if !uzanti.is_empty() => format!("{temel}.{}", sanitize_ad(uzanti)),
        _ => temel,
    }
}

fn benzersiz_yol(hedef: &Path, dosya_adi: &str) -> PathBuf {
    let aday = hedef.join(dosya_adi);
    if !aday.exists() {
        return aday;
    }
    let yol = Path::new(dosya_adi);
    let govde = yol
        .file_stem()
        .map(|govde| govde.to_string_lossy().to_string())
        .unwrap_or_else(|| dosya_adi.to_string());
    let uzanti = yol
        .extension()
        .map(|uzanti| format!(".{}", uzanti.to_string_lossy()))
        .unwrap_or_default();
    (1u32..)
        .map(|sira| hedef.join(format!("{govde}-{sira}{uzanti}")))
        .find(|aday| !aday.exists())
        .unwrap_or_else(|| hedef.join(dosya_adi))
}

fn sanitize_ad(metin: &str) -> String {
    let temiz = metin
        .chars()
        .map(|karakter| {
            if karakter.is_control()
                || matches!(
                    karakter,
                    '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
                )
            {
                '_'
            } else {
                karakter
            }
        })
        .take(80)
        .collect::<String>();
    let temiz = temiz.trim().trim_end_matches(['.', ' ']).trim().to_string();
    if temiz.is_empty() {
        "artik".to_string()
    } else {
        temiz
    }
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

fn yol_guvenli_mi(yol: &Path) -> bool {
    if !yol.is_absolute() {
        return false;
    }
    let metin = yol.to_string_lossy().replace('/', "\\").to_lowercase();
    let parcalar = metin
        .split('\\')
        .filter(|parca| !parca.is_empty())
        .collect::<Vec<_>>();
    if parcalar.len() < 3 || parcalar.get(1) == Some(&"windows") {
        return false;
    }
    for degisken in [
        "SystemRoot",
        "windir",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
        "APPDATA",
        "LOCALAPPDATA",
        "USERPROFILE",
    ] {
        let Some(kok) = std::env::var_os(degisken) else {
            continue;
        };
        let kok = PathBuf::from(kok)
            .to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase();
        if !kok.is_empty() && metin.trim_end_matches('\\') == kok.as_str() {
            return false;
        }
    }
    true
}

fn registry_yolu_ayristir(yol: &str) -> Sonuc<(&'static str, String)> {
    let yol = yol.trim();
    if yol.is_empty()
        || yol.len() > 512
        || yol
            .chars()
            .any(|karakter| matches!(karakter, '\0' | '\r' | '\n' | '"' | '<' | '>'))
    {
        return Err(Hata::Dogrulama("Kayıt defteri yolu geçersiz.".into()));
    }
    let (kove, kalan) = if let Some(kalan) = yol.strip_prefix("HKCU\\") {
        ("HKCU", kalan)
    } else if let Some(kalan) = yol.strip_prefix("HKLM\\") {
        ("HKLM", kalan)
    } else {
        return Err(Hata::Dogrulama(
            "Kayıt defteri kökü HKCU veya HKLM olmalı.".into(),
        ));
    };
    let alt = kalan.replace('/', "\\").trim_matches('\\').to_string();
    let kucuk = alt.to_lowercase();
    let parcalar = alt.split('\\').collect::<Vec<_>>();
    let son = parcalar
        .last()
        .map(|parca| parca.to_lowercase())
        .unwrap_or_default();
    let derinlik_uygun = parcalar.len() == 2
        || (parcalar.len() == 3 && parcalar[1].eq_ignore_ascii_case("wow6432node"));
    if !kucuk.starts_with("software\\")
        || !derinlik_uygun
        || parcalar
            .iter()
            .any(|parca| parca.is_empty() || *parca == "." || *parca == "..")
        || kucuk.contains("\\uninstall")
        || kucuk.contains("\\microsoft\\windows")
        || SISTEM_KAYIT_ADLARI.contains(&son.as_str())
    {
        return Err(Hata::Yetki(
            "Kayıt defteri anahtarı karantinaya alınamaz.".into(),
        ));
    }
    Ok((kove, alt))
}

fn powershell_argumanlari(betik: &str) -> Vec<String> {
    [
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        betik,
    ]
    .iter()
    .map(|parca| parca.to_string())
    .collect()
}

fn sinirla(metin: &str) -> String {
    metin.trim().chars().take(CIKTI_SINIRI).collect()
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::winget::KomutSonucu;
    use std::sync::Mutex;

    struct SahteCalistirici {
        cagrilar: Mutex<Vec<(String, Vec<String>)>>,
        stdout: String,
        kod: i32,
    }

    impl SahteCalistirici {
        fn yeni(stdout: &str, kod: i32) -> Self {
            Self {
                cagrilar: Mutex::new(Vec::new()),
                stdout: stdout.to_string(),
                kod,
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
            Ok(KomutSonucu {
                komut: program.to_string(),
                kod: self.kod,
                stdout: self.stdout.clone(),
                stderr: String::new(),
            })
        }
    }

    fn servis(stdout: &str) -> (ArtikServisi, Arc<SahteCalistirici>) {
        let sahte = Arc::new(SahteCalistirici::yeni(stdout, 0));
        (ArtikServisi::yeni(sahte.clone()), sahte)
    }

    fn oge(tur: ArtikTuru, ad: &str, yol: &Path) -> ArtikOgesi {
        ArtikOgesi {
            kimlik: ArtikOgesi::kimlik_olustur(tur, &yol.to_string_lossy()),
            tur,
            ad: ad.to_string(),
            yol: yol.to_string_lossy().to_string(),
            boyut_bayt: None,
        }
    }

    #[test]
    fn karantina_dosya_tasir() {
        let gecici = tempfile::tempdir().unwrap();
        let kaynak = gecici.path().join("uygulama.dll");
        fs::write(&kaynak, b"veri").unwrap();
        let hedef = gecici.path().join("karantina");
        let (servis, _) = servis("");
        let sonuc = servis
            .karantinaya_tasi(&[oge(ArtikTuru::Dosya, "Uygulama", &kaynak)], &hedef)
            .unwrap();
        assert_eq!(sonuc.tasinan, 1);
        assert!(sonuc.hatalar.is_empty());
        assert!(!kaynak.exists());
        assert!(hedef.join("Uygulama.dll").exists());
    }

    #[test]
    fn karantina_klasor_tasir() {
        let gecici = tempfile::tempdir().unwrap();
        let kaynak = gecici.path().join("Uygulama Verisi");
        fs::create_dir_all(kaynak.join("alt")).unwrap();
        fs::write(kaynak.join("alt").join("ayar.json"), b"{}").unwrap();
        let hedef = gecici.path().join("karantina");
        let (servis, _) = servis("");
        let sonuc = servis
            .karantinaya_tasi(&[oge(ArtikTuru::Klasor, "Uygulama", &kaynak)], &hedef)
            .unwrap();
        assert_eq!(sonuc.tasinan, 1);
        assert!(!kaynak.exists());
        assert!(hedef
            .join("Uygulama")
            .join("alt")
            .join("ayar.json")
            .exists());
    }

    #[test]
    fn tehlikeli_yol_reddedilir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, _) = servis("");
        let windows = PathBuf::from("C:\\Windows");
        let sonuc = servis
            .karantinaya_tasi(
                &[oge(ArtikTuru::Klasor, "Sistem", &windows)],
                &gecici.path().join("karantina"),
            )
            .unwrap();
        assert_eq!(sonuc.tasinan, 0);
        assert_eq!(sonuc.hatalar.len(), 1);
    }

    #[test]
    fn registry_yedeklenir_ve_silinir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, sahte) = servis("");
        let kayit = oge(
            ArtikTuru::Registry,
            "TestUygulama",
            Path::new("HKCU\\Software\\TestUygulama"),
        );
        let sonuc = servis
            .karantinaya_tasi(&[kayit], &gecici.path().join("karantina"))
            .unwrap();
        assert_eq!(sonuc.tasinan, 1);
        let cagrilar = sahte.cagrilar.lock().unwrap();
        assert_eq!(cagrilar.len(), 2);
        assert_eq!(cagrilar[0].0, "reg.exe");
        assert_eq!(cagrilar[0].1[0], "export");
        assert_eq!(
            cagrilar[1].1,
            vec!["delete", "HKCU\\Software\\TestUygulama", "/f"]
        );
    }

    #[test]
    fn uninstall_anahtari_reddedilir() {
        let yol = "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Test";
        assert!(registry_yolu_ayristir(yol).is_err());
        assert!(registry_yolu_ayristir("HKCU\\Software\\Microsoft").is_err());
        assert!(registry_yolu_ayristir("HKCU\\Software\\WOW6432Node\\Test").is_ok());
        assert!(registry_yolu_ayristir("HKCU\\Software\\Test").is_ok());
        assert!(registry_yolu_ayristir("HKLM\\Software\\Test\\Alt").is_err());
        assert!(registry_yolu_ayristir("HKEY_LOCAL_MACHINE\\Software\\Test").is_err());
    }

    #[test]
    fn store_paketleri_okunur() {
        let json =
            r#"[{"Ad":"A.B","TamAd":"A.B_1.0.0.0_x64__abc","Surum":"1.0.0.0","Yayinci":"A"}]"#;
        let (servis, sahte) = servis(json);
        let paketler = servis.store_paketleri().unwrap();
        assert_eq!(paketler.len(), 1);
        let cagrilar = sahte.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "powershell");
        assert!(cagrilar[0].1.contains(&STORE_LISTE_BETIGI.to_string()));
    }

    #[test]
    fn bozuk_store_ciktisi_hata_verir() {
        let (servis, _) = servis("bozuk");
        assert!(servis.store_paketleri().is_err());
    }

    #[test]
    fn gecersiz_store_adi_kaldirilmaz() {
        let (servis, sahte) = servis("");
        assert!(servis.store_kaldir("paket'; Remove-Item C:\\").is_err());
        assert!(sahte.cagrilar.lock().unwrap().is_empty());
    }

    #[test]
    fn gecerli_store_adi_kaldirilir() {
        let (servis, sahte) = servis("");
        servis
            .store_kaldir("Microsoft.WindowsStore_22410.1401.2.0_x64__8wekyb3d8bbwe")
            .unwrap();
        let cagrilar = sahte.cagrilar.lock().unwrap();
        assert!(cagrilar[0]
            .1
            .last()
            .unwrap()
            .contains("Remove-AppxPackage -Package 'Microsoft.WindowsStore_"));
    }

    #[test]
    fn klasor_boyutu_hesaplanir() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("veri");
        fs::create_dir_all(kok.join("alt")).unwrap();
        fs::write(kok.join("a.bin"), vec![0u8; 10]).unwrap();
        fs::write(kok.join("alt").join("b.bin"), vec![0u8; 5]).unwrap();
        assert_eq!(klasor_boyutu(&kok), 15);
    }

    #[test]
    fn ayni_adli_hedef_cakismaz() {
        let gecici = tempfile::tempdir().unwrap();
        fs::write(gecici.path().join("Uygulama.dll"), b"eski").unwrap();
        let yol = benzersiz_yol(gecici.path(), "Uygulama.dll");
        assert_eq!(
            yol.file_name().unwrap().to_string_lossy().as_ref(),
            "Uygulama-1.dll"
        );
    }
}
