use sp_core::error::{Hata, Sonuc};
use sp_core::kurulum_izleme::{
    anlik_dosya_haritasi_suzgecli, dosya_farki, dosya_haritasi_ayristir, dosya_haritasi_json,
    kayit_farki, kayit_listesi_ayristir, kayit_listesi_json, kimlik_gecerli_mi, kimlik_uret,
    oturum_ayristir, oturum_json, rapor_json, zaman_damgasi, IzlemeOturumu, IzlemeRaporu,
};
use std::collections::hash_map::RandomState;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::hash::{BuildHasher, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY};
use winreg::RegKey;

type DosyaHaritasi = HashMap<String, (u64, i64)>;

const OTURUM_DOSYASI: &str = "oturum.json";
const DOSYALAR_DOSYASI: &str = "dosyalar.json";
const KAYITLAR_DOSYASI: &str = "kayitlar.json";
const RAPOR_DOSYASI: &str = "rapor.json";
const AZAMI_DOSYA: usize = 200_000;
const AZAMI_DOSYA_DEGISIKLIGI: usize = 5_000;
const AZAMI_KAYIT_DEGISIKLIGI: usize = 1_000;
const KIMLIK_DENEME_SAYISI: u64 = 16;
const GURULTU_ADLARI: [&str; 3] = ["temp", "cache", "crashpad"];
const KOK_ORTAMLARI: [&str; 5] = [
    "ProgramFiles",
    "ProgramFiles(x86)",
    "ProgramData",
    "LOCALAPPDATA",
    "APPDATA",
];
const KAYIT_YOLLARI: [&str; 2] = [
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
    r"Software\Microsoft\Windows\CurrentVersion\App Paths",
];
const KAYIT_KOVELERI: [(winreg::HKEY, &str); 2] =
    [(HKEY_LOCAL_MACHINE, "HKLM"), (HKEY_CURRENT_USER, "HKCU")];

pub struct KurulumIzlemeServisi {
    kok: PathBuf,
}

impl KurulumIzlemeServisi {
    pub fn yeni(kok: PathBuf) -> Self {
        Self { kok }
    }

    pub fn durum(&self) -> Sonuc<Option<IzlemeOturumu>> {
        let metin = match fs::read_to_string(self.kok.join(OTURUM_DOSYASI)) {
            Ok(metin) => metin,
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(hata) => {
                return Err(Hata::Beklenmeyen(format!(
                    "İzleme oturumu okunamadı: {hata}"
                )))
            }
        };
        let (oturum, _) = oturum_ayristir(&metin)?;
        kimlik_dogrula(&oturum.kimlik)?;
        Ok(Some(oturum))
    }

    pub fn baslat(&self) -> Sonuc<IzlemeOturumu> {
        self.baslat_koklerle(&varsayilan_kokler())
    }

    pub fn baslat_koklerle(&self, kokler: &[PathBuf]) -> Sonuc<IzlemeOturumu> {
        if self.durum()?.is_some() {
            return Err(Hata::Dogrulama(
                "Zaten etkin bir izleme oturumu var.".into(),
            ));
        }
        if kokler.is_empty() {
            return Err(Hata::Dogrulama("İzlenecek klasör bulunamadı.".into()));
        }
        let baslangic = SystemTime::now();
        let (dosyalar, kesildi) = self.anlik_dosyalar(kokler);
        if kesildi {
            tracing::warn!("İzleme görüntüsü {AZAMI_DOSYA} dosya sınırına ulaştı.");
        }
        let kayitlar = kayit_adlari();
        let kimlik = kimlik_uretir(&self.kok)?;
        let oturum = IzlemeOturumu {
            kimlik,
            baslangic: zaman_damgasi(baslangic),
            dosya_sayisi: dosyalar.len() as u64,
            kayit_sayisi: kayitlar.len() as u64,
        };
        let dizin = self.oturum_dizini(&oturum.kimlik);
        if let Err(hata) = self.temeli_yaz(&dizin, &oturum, kokler, &dosyalar, &kayitlar) {
            let _ = fs::remove_dir_all(&dizin);
            return Err(hata);
        }
        Ok(oturum)
    }

    pub fn durdur(&self) -> Sonuc<IzlemeRaporu> {
        let oturum_yolu = self.kok.join(OTURUM_DOSYASI);
        let (oturum, kokler) = self.aktif_oturum_oku(&oturum_yolu)?;
        let dizin = self.oturum_dizini(&oturum.kimlik);
        let (once, once_kayitlar) = self.temeli_oku(&dizin)?;
        let (sonra, kesildi) = self.anlik_dosyalar(&kokler);
        let rapor = rapor_olustur(oturum, once, once_kayitlar, sonra, kayit_adlari(), kesildi);
        json_yaz(
            &dizin.join(RAPOR_DOSYASI),
            &rapor_json(&rapor)?,
            "İzleme raporu",
        )?;
        fs::remove_file(&oturum_yolu)
            .map_err(|hata| Hata::Beklenmeyen(format!("Oturum bilgisi silinemedi: {hata}")))?;
        Ok(rapor)
    }

    fn aktif_oturum_oku(&self, yol: &Path) -> Sonuc<(IzlemeOturumu, Vec<PathBuf>)> {
        let metin = match fs::read_to_string(yol) {
            Ok(metin) => metin,
            Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => {
                return Err(Hata::Dogrulama("Etkin bir izleme oturumu yok.".into()))
            }
            Err(hata) => {
                return Err(Hata::Beklenmeyen(format!(
                    "İzleme oturumu okunamadı: {hata}"
                )))
            }
        };
        let (oturum, kokler) = oturum_ayristir(&metin)?;
        kimlik_dogrula(&oturum.kimlik)?;
        if kokler.is_empty() {
            return Err(Hata::Beklenmeyen("İzleme oturumu kök listesi boş.".into()));
        }
        Ok((
            oturum,
            kokler.into_iter().map(PathBuf::from).collect::<Vec<_>>(),
        ))
    }

    fn temeli_oku(&self, dizin: &Path) -> Sonuc<(DosyaHaritasi, Vec<String>)> {
        let dosyalar = dosya_haritasi_ayristir(&json_oku(
            &dizin.join(DOSYALAR_DOSYASI),
            "Dosya temel görüntüsü",
        )?)?;
        let kayitlar = kayit_listesi_ayristir(&json_oku(
            &dizin.join(KAYITLAR_DOSYASI),
            "Kayıt defteri temel görüntüsü",
        )?)?;
        Ok((dosyalar, kayitlar))
    }

    fn anlik_dosyalar(&self, kokler: &[PathBuf]) -> (HashMap<String, (u64, i64)>, bool) {
        let (mut harita, kesildi) =
            anlik_dosya_haritasi_suzgecli(kokler, AZAMI_DOSYA, &|yol| !gurultu_mu(yol));
        kok_haric_tut(&mut harita, &self.kok);
        (harita, kesildi)
    }

    fn temeli_yaz(
        &self,
        dizin: &Path,
        oturum: &IzlemeOturumu,
        kokler: &[PathBuf],
        dosyalar: &HashMap<String, (u64, i64)>,
        kayitlar: &[String],
    ) -> Sonuc<()> {
        fs::create_dir_all(dizin)
            .map_err(|hata| Hata::Beklenmeyen(format!("İzleme dizini oluşturulamadı: {hata}")))?;
        json_yaz(
            &dizin.join(DOSYALAR_DOSYASI),
            &dosya_haritasi_json(dosyalar)?,
            "Dosya temel görüntüsü",
        )?;
        json_yaz(
            &dizin.join(KAYITLAR_DOSYASI),
            &kayit_listesi_json(kayitlar)?,
            "Kayıt defteri temel görüntüsü",
        )?;
        json_yaz(
            &self.kok.join(OTURUM_DOSYASI),
            &oturum_json(oturum, kokler)?,
            "Oturum bilgisi",
        )
    }

    fn oturum_dizini(&self, kimlik: &str) -> PathBuf {
        self.kok.join(kimlik)
    }
}

fn rapor_olustur(
    oturum: IzlemeOturumu,
    once: HashMap<String, (u64, i64)>,
    once_kayitlar: Vec<String>,
    sonra: HashMap<String, (u64, i64)>,
    sonra_kayitlar: Vec<String>,
    anlik_kesildi: bool,
) -> IzlemeRaporu {
    let mut dosyalar = dosya_farki(&once, &sonra, usize::MAX);
    let mut kayitlar = kayit_farki(&once_kayitlar, &sonra_kayitlar, usize::MAX);
    let toplam_dosya = dosyalar.len();
    let toplam_kayit = kayitlar.len();
    dosyalar.truncate(AZAMI_DOSYA_DEGISIKLIGI);
    kayitlar.truncate(AZAMI_KAYIT_DEGISIKLIGI);
    let kesildi = anlik_kesildi || toplam_dosya > dosyalar.len() || toplam_kayit > kayitlar.len();
    IzlemeRaporu {
        oturum,
        bitis: zaman_damgasi(SystemTime::now()),
        dosyalar,
        kayitlar,
        toplam_dosya,
        toplam_kayit,
        kesildi,
    }
}

fn varsayilan_kokler() -> Vec<PathBuf> {
    let mut kokler = Vec::new();
    for degisken in KOK_ORTAMLARI {
        let Some(kok) = std::env::var_os(degisken).map(PathBuf::from) else {
            continue;
        };
        if kok.is_dir() && !kokler.contains(&kok) {
            kokler.push(kok);
        }
    }
    kokler
}

fn kayit_adlari() -> Vec<String> {
    let mut adlar = BTreeSet::new();
    for (kok, kove) in KAYIT_KOVELERI {
        for yol in KAYIT_YOLLARI {
            anahtarlari_topla(kok, kove, yol, &mut adlar);
            anahtarlari_topla(kok, kove, &wow_yolu(yol), &mut adlar);
        }
    }
    adlar.into_iter().collect()
}

fn anahtarlari_topla(kok: winreg::HKEY, kove: &str, yol: &str, adlar: &mut BTreeSet<String>) {
    let Ok(anahtar) = RegKey::predef(kok).open_subkey_with_flags(yol, KEY_READ | KEY_WOW64_64KEY)
    else {
        return;
    };
    for ad in anahtar.enum_keys().flatten() {
        adlar.insert(format!("{kove}\\{yol}\\{ad}"));
    }
}

fn wow_yolu(yol: &str) -> String {
    format!(
        r"Software\WOW6432Node\{}",
        yol.trim_start_matches(r"Software\")
    )
}

fn gurultu_mu(yol: &Path) -> bool {
    yol.file_name()
        .and_then(|ad| ad.to_str())
        .is_some_and(|ad| {
            GURULTU_ADLARI
                .iter()
                .any(|gurultu| ad.eq_ignore_ascii_case(gurultu))
        })
}

fn kok_haric_tut(harita: &mut HashMap<String, (u64, i64)>, kok: &Path) {
    let onek = format!(
        "{}\\",
        kok.to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase()
    );
    harita.retain(|yol, _| !yol.starts_with(&onek));
}

fn kimlik_uretir(kok: &Path) -> Sonuc<String> {
    let zaman = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    for sira in 0..KIMLIK_DENEME_SAYISI {
        let kimlik = kimlik_uret(zaman, rastgele_sayi().wrapping_add(sira));
        if !kok.join(&kimlik).exists() {
            return Ok(kimlik);
        }
    }
    Err(Hata::Beklenmeyen(
        "İzleme oturumu kimliği üretilemedi.".into(),
    ))
}

fn rastgele_sayi() -> u64 {
    let mut ozet = RandomState::new().build_hasher();
    ozet.write_u64(u64::from(std::process::id()));
    ozet.write_u128(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    );
    ozet.finish()
}

fn kimlik_dogrula(kimlik: &str) -> Sonuc<()> {
    if kimlik_gecerli_mi(kimlik) {
        Ok(())
    } else {
        Err(Hata::Dogrulama("İzleme oturumu kimliği geçersiz.".into()))
    }
}

fn json_oku(yol: &Path, ad: &str) -> Sonuc<String> {
    fs::read_to_string(yol).map_err(|hata| Hata::Beklenmeyen(format!("{ad} okunamadı: {hata}")))
}

fn json_yaz(yol: &Path, icerik: &str, ad: &str) -> Sonuc<()> {
    fs::write(yol, icerik).map_err(|hata| Hata::Beklenmeyen(format!("{ad} yazılamadı: {hata}")))
}

#[cfg(test)]
mod testler {
    use super::*;

    fn servis(gecici: &tempfile::TempDir) -> (KurulumIzlemeServisi, PathBuf) {
        let kok = gecici.path().join("veri");
        let izlenen = gecici.path().join("izlenen");
        fs::create_dir_all(&izlenen).unwrap();
        (KurulumIzlemeServisi::yeni(kok), izlenen)
    }

    #[test]
    fn baslat_durdur_dosya_degisikliklerini_raporlar() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, izlenen) = servis(&gecici);
        fs::write(izlenen.join("kalan.txt"), b"eski").unwrap();
        fs::write(izlenen.join("silinecek.txt"), b"sil").unwrap();
        let oturum = servis
            .baslat_koklerle(std::slice::from_ref(&izlenen))
            .unwrap();
        assert_eq!(oturum.dosya_sayisi, 2);
        assert!(servis.durum().unwrap().is_some());
        assert!(gecici.path().join("veri").join(OTURUM_DOSYASI).exists());

        fs::remove_file(izlenen.join("silinecek.txt")).unwrap();
        fs::write(izlenen.join("kalan.txt"), b"yeni icerik").unwrap();
        fs::write(izlenen.join("eklenen.txt"), b"ek").unwrap();
        let rapor = servis.durdur().unwrap();
        assert_eq!(rapor.toplam_dosya, 3);
        assert!(!rapor.kesildi);
        let tur = |ad: &str| {
            rapor
                .dosyalar
                .iter()
                .find(|degisiklik| degisiklik.yol.ends_with(ad))
                .map(|degisiklik| degisiklik.tur.as_str())
        };
        assert_eq!(tur("eklenen.txt"), Some("Eklendi"));
        assert_eq!(tur("silinecek.txt"), Some("Silindi"));
        assert_eq!(tur("kalan.txt"), Some("Değişti"));

        let oturum_dizini = gecici.path().join("veri").join(&oturum.kimlik);
        assert!(oturum_dizini.join(RAPOR_DOSYASI).exists());
        assert!(oturum_dizini.join(DOSYALAR_DOSYASI).exists());
        assert!(!gecici.path().join("veri").join(OTURUM_DOSYASI).exists());
        assert!(servis.durum().unwrap().is_none());
        assert!(servis.durdur().is_err());
    }

    #[test]
    fn ikinci_baslatma_reddedilir() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, izlenen) = servis(&gecici);
        assert!(servis.durdur().is_err());
        servis
            .baslat_koklerle(std::slice::from_ref(&izlenen))
            .unwrap();
        assert!(servis
            .baslat_koklerle(std::slice::from_ref(&izlenen))
            .is_err());
        assert!(servis.baslat_koklerle(&[]).is_err());
    }

    #[test]
    fn gurultu_klasorleri_izlenmez() {
        let gecici = tempfile::tempdir().unwrap();
        let (servis, izlenen) = servis(&gecici);
        for (klasor, dosya) in [
            ("Temp", "gecici.txt"),
            ("Cache", "onbellek.bin"),
            ("Crashpad", "cokme.dmp"),
        ] {
            fs::create_dir_all(izlenen.join(klasor)).unwrap();
            fs::write(izlenen.join(klasor).join(dosya), b"x").unwrap();
        }
        fs::write(izlenen.join("uygulama.exe"), b"x").unwrap();
        let oturum = servis
            .baslat_koklerle(std::slice::from_ref(&izlenen))
            .unwrap();
        assert_eq!(oturum.dosya_sayisi, 1);
        let rapor = servis.durdur().unwrap();
        assert_eq!(rapor.toplam_dosya, 0);
    }

    #[test]
    fn servis_klasoru_izleme_disarida_tutulur() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("veri");
        let servis = KurulumIzlemeServisi::yeni(kok);
        fs::write(gecici.path().join("uygulama.exe"), b"x").unwrap();
        let oturum = servis
            .baslat_koklerle(&[gecici.path().to_path_buf()])
            .unwrap();
        assert_eq!(oturum.dosya_sayisi, 1);
        let rapor = servis.durdur().unwrap();
        assert_eq!(rapor.toplam_dosya, 0);
    }

    #[test]
    fn gecersiz_kimlik_yol_kacisina_izin_vermez() {
        let gecici = tempfile::tempdir().unwrap();
        let servis = KurulumIzlemeServisi::yeni(gecici.path().join("veri"));
        fs::create_dir_all(gecici.path().join("veri")).unwrap();
        let oturum = r#"{"kimlik":"../../kacis","baslangic":"2026-10-30T12:00:00Z","dosya_sayisi":0,"kayit_sayisi":0,"kokler":[]}"#;
        fs::write(
            gecici.path().join("veri").join(OTURUM_DOSYASI),
            oturum.as_bytes(),
        )
        .unwrap();
        assert!(servis.durum().is_err());
        assert!(servis.durdur().is_err());
        assert!(!gecici.path().join("kacis").exists());
    }

    #[test]
    fn bozuk_oturum_dosyasi_hata_doner() {
        let gecici = tempfile::tempdir().unwrap();
        let servis = KurulumIzlemeServisi::yeni(gecici.path().join("veri"));
        fs::create_dir_all(gecici.path().join("veri")).unwrap();
        fs::write(gecici.path().join("veri").join(OTURUM_DOSYASI), b"{").unwrap();
        assert!(servis.durum().is_err());
        assert!(servis
            .baslat_koklerle(&[gecici.path().to_path_buf()])
            .is_err());
    }

    #[test]
    fn varsayilan_kokler_ortamlardan_toplanir() {
        let kokler = varsayilan_kokler();
        for kok in &kokler {
            assert!(kok.is_absolute());
            assert!(kok.is_dir());
        }
    }

    #[test]
    fn kayit_defteri_anahtar_adlari_okunur() {
        let adlar = kayit_adlari();
        assert!(adlar.iter().all(|ad| ad.contains('\\')));
        assert!(adlar.windows(2).all(|cift| cift[0] <= cift[1]));
    }
}
