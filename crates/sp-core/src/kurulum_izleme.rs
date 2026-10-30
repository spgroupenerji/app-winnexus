use crate::error::{Hata, Sonuc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DERINLIK_SINIRI: usize = 24;
const DOSYA_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const AZAMI_KIMLIK_UZUNLUGU: usize = 64;

pub const TUR_EKLENDI: &str = "Eklendi";
pub const TUR_SILINDI: &str = "Silindi";
pub const TUR_DEGISTI: &str = "Değişti";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IzlemeOturumu {
    pub kimlik: String,
    pub baslangic: String,
    pub dosya_sayisi: u64,
    pub kayit_sayisi: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IzlemeDegisikligi {
    pub tur: String,
    pub yol: String,
    pub ayrinti: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IzlemeRaporu {
    pub oturum: IzlemeOturumu,
    pub bitis: String,
    pub dosyalar: Vec<IzlemeDegisikligi>,
    pub kayitlar: Vec<IzlemeDegisikligi>,
    pub toplam_dosya: usize,
    pub toplam_kayit: usize,
    pub kesildi: bool,
}

#[derive(Serialize, Deserialize)]
struct OturumDosyasi {
    #[serde(flatten)]
    oturum: IzlemeOturumu,
    kokler: Vec<String>,
}

pub fn anlik_dosya_haritasi(
    kokler: &[PathBuf],
    azami: usize,
) -> (HashMap<String, (u64, i64)>, bool) {
    anlik_dosya_haritasi_suzgecli(kokler, azami, &|_: &Path| true)
}

pub fn anlik_dosya_haritasi_suzgecli(
    kokler: &[PathBuf],
    azami: usize,
    kabul: &dyn Fn(&Path) -> bool,
) -> (HashMap<String, (u64, i64)>, bool) {
    let mut harita = HashMap::new();
    let mut kesildi = false;
    'kokler: for kok in kokler {
        let mut yigin = vec![(kok.clone(), 0usize)];
        while let Some((dizin, derinlik)) = yigin.pop() {
            if dizini_tara(dizin, derinlik, kabul, &mut yigin, &mut harita, azami) {
                kesildi = true;
                break 'kokler;
            }
        }
    }
    (harita, kesildi)
}

fn dizini_tara(
    dizin: PathBuf,
    derinlik: usize,
    kabul: &dyn Fn(&Path) -> bool,
    yigin: &mut Vec<(PathBuf, usize)>,
    harita: &mut HashMap<String, (u64, i64)>,
    azami: usize,
) -> bool {
    let Ok(girisler) = fs::read_dir(&dizin) else {
        return false;
    };
    for giris in girisler.flatten() {
        let yol = giris.path();
        if !kabul(&yol) {
            continue;
        }
        let Ok(meta) = giris.metadata() else {
            continue;
        };
        if baglanti_mi(&meta) {
            continue;
        }
        if meta.is_dir() {
            if derinlik < DERINLIK_SINIRI {
                yigin.push((yol, derinlik + 1));
            }
            continue;
        }
        if !meta.is_file() {
            continue;
        }
        if harita.len() >= azami {
            return true;
        }
        harita.insert(dosya_anahtari(&yol), (meta.len(), degisme_zamani(&meta)));
    }
    false
}

pub fn dosya_farki(
    once: &HashMap<String, (u64, i64)>,
    sonra: &HashMap<String, (u64, i64)>,
    azami: usize,
) -> Vec<IzlemeDegisikligi> {
    let mut farklar = Vec::new();
    for (yol, yeni) in sonra {
        match once.get(yol) {
            None => farklar.push(degisiklik(
                TUR_EKLENDI,
                yol,
                format!("Yeni dosya ({} bayt)", yeni.0),
            )),
            Some(eski) if eski != yeni => farklar.push(degisiklik(
                TUR_DEGISTI,
                yol,
                degisim_ayrintisi(*eski, *yeni),
            )),
            Some(_) => {}
        }
    }
    for (yol, eski) in once {
        if !sonra.contains_key(yol) {
            farklar.push(degisiklik(
                TUR_SILINDI,
                yol,
                format!("Dosya kaldırıldı ({} bayt)", eski.0),
            ));
        }
    }
    sirala_ve_sinirla(farklar, azami)
}

pub fn kayit_farki(once: &[String], sonra: &[String], azami: usize) -> Vec<IzlemeDegisikligi> {
    let eski = once.iter().map(String::as_str).collect::<HashSet<_>>();
    let yeni = sonra.iter().map(String::as_str).collect::<HashSet<_>>();
    let mut farklar = Vec::new();
    for yol in &yeni {
        if !eski.contains(yol) {
            farklar.push(degisiklik(
                TUR_EKLENDI,
                yol,
                "Yeni kayıt defteri anahtarı".to_string(),
            ));
        }
    }
    for yol in &eski {
        if !yeni.contains(yol) {
            farklar.push(degisiklik(
                TUR_SILINDI,
                yol,
                "Kayıt defteri anahtarı kaldırıldı".to_string(),
            ));
        }
    }
    sirala_ve_sinirla(farklar, azami)
}

pub fn kimlik_uret(zaman: u64, rastgele: u64) -> String {
    format!("{zaman:x}-{rastgele:016x}")
}

pub fn kimlik_gecerli_mi(kimlik: &str) -> bool {
    !kimlik.is_empty()
        && kimlik.len() <= AZAMI_KIMLIK_UZUNLUGU
        && kimlik
            .chars()
            .all(|karakter| karakter.is_ascii_alphanumeric() || karakter == '-')
}

pub fn zaman_damgasi(zaman: SystemTime) -> String {
    let an = time::OffsetDateTime::from(zaman);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        an.year(),
        u8::from(an.month()),
        an.day(),
        an.hour(),
        an.minute(),
        an.second()
    )
}

pub fn oturum_json(oturum: &IzlemeOturumu, kokler: &[PathBuf]) -> Sonuc<String> {
    let dosya = OturumDosyasi {
        oturum: oturum.clone(),
        kokler: kokler
            .iter()
            .map(|kok| kok.to_string_lossy().to_string())
            .collect(),
    };
    serde_json::to_string_pretty(&dosya)
        .map_err(|hata| Hata::Beklenmeyen(format!("İzleme oturumu yazılamadı: {hata}")))
}

pub fn oturum_ayristir(json: &str) -> Sonuc<(IzlemeOturumu, Vec<String>)> {
    let dosya: OturumDosyasi = serde_json::from_str(json.trim_start_matches('\u{feff}'))
        .map_err(|hata| Hata::Beklenmeyen(format!("İzleme oturumu ayrıştırılamadı: {hata}")))?;
    Ok((dosya.oturum, dosya.kokler))
}

pub fn dosya_haritasi_json(harita: &HashMap<String, (u64, i64)>) -> Sonuc<String> {
    let sirali = harita
        .iter()
        .map(|(yol, deger)| (yol.as_str(), *deger))
        .collect::<BTreeMap<_, _>>();
    serde_json::to_string_pretty(&sirali)
        .map_err(|hata| Hata::Beklenmeyen(format!("Dosya görüntüsü yazılamadı: {hata}")))
}

pub fn dosya_haritasi_ayristir(json: &str) -> Sonuc<HashMap<String, (u64, i64)>> {
    serde_json::from_str(json.trim_start_matches('\u{feff}'))
        .map_err(|hata| Hata::Beklenmeyen(format!("Dosya görüntüsü ayrıştırılamadı: {hata}")))
}

pub fn kayit_listesi_json(kayitlar: &[String]) -> Sonuc<String> {
    serde_json::to_string_pretty(kayitlar)
        .map_err(|hata| Hata::Beklenmeyen(format!("Kayıt defteri listesi yazılamadı: {hata}")))
}

pub fn kayit_listesi_ayristir(json: &str) -> Sonuc<Vec<String>> {
    serde_json::from_str(json.trim_start_matches('\u{feff}'))
        .map_err(|hata| Hata::Beklenmeyen(format!("Kayıt defteri listesi ayrıştırılamadı: {hata}")))
}

pub fn rapor_json(rapor: &IzlemeRaporu) -> Sonuc<String> {
    serde_json::to_string_pretty(rapor)
        .map_err(|hata| Hata::Beklenmeyen(format!("İzleme raporu yazılamadı: {hata}")))
}

fn dosya_anahtari(yol: &Path) -> String {
    yol.to_string_lossy().to_lowercase()
}

fn degisme_zamani(meta: &fs::Metadata) -> i64 {
    meta.modified()
        .ok()
        .and_then(|zaman| zaman.duration_since(UNIX_EPOCH).ok())
        .map(|sure| sure.as_nanos().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

fn baglanti_mi(meta: &fs::Metadata) -> bool {
    meta.file_attributes() & DOSYA_ATTRIBUTE_REPARSE_POINT != 0
}

fn degisiklik(tur: &str, yol: &str, ayrinti: String) -> IzlemeDegisikligi {
    IzlemeDegisikligi {
        tur: tur.to_string(),
        yol: yol.to_string(),
        ayrinti,
    }
}

fn degisim_ayrintisi(eski: (u64, i64), yeni: (u64, i64)) -> String {
    if eski.0 != yeni.0 {
        format!("Boyut {} → {} bayt", eski.0, yeni.0)
    } else {
        "Değişiklik zamanı güncellendi".to_string()
    }
}

fn sirala_ve_sinirla(mut farklar: Vec<IzlemeDegisikligi>, azami: usize) -> Vec<IzlemeDegisikligi> {
    farklar.sort_by(|a, b| a.yol.cmp(&b.yol).then_with(|| a.tur.cmp(&b.tur)));
    farklar.truncate(azami);
    farklar
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::time::Duration;

    fn harita(girdiler: &[(&str, u64, i64)]) -> HashMap<String, (u64, i64)> {
        girdiler
            .iter()
            .map(|(yol, boyut, zaman)| (yol.to_string(), (*boyut, *zaman)))
            .collect()
    }

    #[test]
    fn dosya_farki_ekleme_silme_degisim_uretir() {
        let once = harita(&[
            (r"c:\a.txt", 10, 1),
            (r"c:\b.txt", 20, 2),
            (r"c:\c.txt", 30, 3),
        ]);
        let sonra = harita(&[
            (r"c:\a.txt", 10, 9),
            (r"c:\b.txt", 25, 2),
            (r"c:\d.txt", 40, 4),
        ]);
        let fark = dosya_farki(&once, &sonra, 10);
        assert_eq!(fark.len(), 4);
        assert_eq!(fark[0].yol, r"c:\a.txt");
        assert_eq!(fark[0].tur, TUR_DEGISTI);
        assert!(fark[0].ayrinti.contains("zaman"));
        assert_eq!(fark[1].tur, TUR_DEGISTI);
        assert!(fark[1].ayrinti.contains("Boyut"));
        assert_eq!(fark[2].tur, TUR_SILINDI);
        assert_eq!(fark[3].tur, TUR_EKLENDI);
    }

    #[test]
    fn dosya_farki_azami_siniri_uygular() {
        let once = harita(&[]);
        let sonra = harita(&[
            (r"c:\a.txt", 1, 1),
            (r"c:\b.txt", 1, 1),
            (r"c:\c.txt", 1, 1),
        ]);
        let fark = dosya_farki(&once, &sonra, 2);
        assert_eq!(fark.len(), 2);
        assert_eq!(fark[0].yol, r"c:\a.txt");
        assert_eq!(fark[1].yol, r"c:\b.txt");
        assert!(dosya_farki(&once, &sonra, 0).is_empty());
    }

    #[test]
    fn kayit_farki_ekleme_silme_uretir() {
        let once = vec!["HKLM\\A".to_string(), "HKLM\\B".to_string()];
        let sonra = vec!["HKLM\\B".to_string(), "HKLM\\C".to_string()];
        let fark = kayit_farki(&once, &sonra, 10);
        assert_eq!(fark.len(), 2);
        assert_eq!(fark[0].yol, "HKLM\\A");
        assert_eq!(fark[0].tur, TUR_SILINDI);
        assert_eq!(fark[1].yol, "HKLM\\C");
        assert_eq!(fark[1].tur, TUR_EKLENDI);
        assert_eq!(kayit_farki(&once, &sonra, 1).len(), 1);
    }

    #[test]
    fn kimlik_bicimi_dosya_adina_uygundur() {
        let kimlik = kimlik_uret(1_700_000_000, 42);
        assert!(kimlik_gecerli_mi(&kimlik));
        assert!(kimlik
            .chars()
            .all(|karakter| karakter.is_ascii_alphanumeric() || karakter == '-'));
        assert_eq!(kimlik, kimlik_uret(1_700_000_000, 42));
        assert_ne!(kimlik, kimlik_uret(1_700_000_000, 43));
        assert!(!kimlik_gecerli_mi(""));
        assert!(!kimlik_gecerli_mi("../../kacis"));
        assert!(!kimlik_gecerli_mi("a b"));
        assert!(!kimlik_gecerli_mi("a/b"));
    }

    #[test]
    fn zaman_damgasi_utc_bicimindedir() {
        assert_eq!(zaman_damgasi(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let sonra = zaman_damgasi(UNIX_EPOCH + Duration::from_secs(3_661));
        assert_eq!(sonra, "1970-01-01T01:01:01Z");
        assert_eq!(sonra.len(), 20);
    }

    #[test]
    fn anlik_goruntu_dosyalari_ve_boyutlari_tplar() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("kok");
        fs::create_dir_all(kok.join("alt")).unwrap();
        fs::write(kok.join("a.txt"), b"abc").unwrap();
        fs::write(kok.join("alt").join("b.txt"), b"12345").unwrap();
        let (harita, kesildi) = anlik_dosya_haritasi(std::slice::from_ref(&kok), 100);
        assert!(!kesildi);
        assert_eq!(harita.len(), 2);
        assert_eq!(harita.values().map(|(boyut, _)| *boyut).sum::<u64>(), 8);
    }

    #[test]
    fn anlik_goruntu_azami_sinirinda_kesilir() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("kok");
        fs::create_dir_all(&kok).unwrap();
        fs::write(kok.join("a.txt"), b"a").unwrap();
        fs::write(kok.join("b.txt"), b"b").unwrap();
        let (harita, kesildi) = anlik_dosya_haritasi(std::slice::from_ref(&kok), 1);
        assert_eq!(harita.len(), 1);
        assert!(kesildi);
        let (bos, kesildi) = anlik_dosya_haritasi(&[gecici.path().join("yok")], 10);
        assert!(bos.is_empty());
        assert!(!kesildi);
    }

    #[test]
    fn anlik_goruntu_derinlik_sinirini_asla_gecmez() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("derin");
        fs::create_dir_all(&kok).unwrap();
        let mut dizin = kok.clone();
        for sira in 0..25 {
            dizin = dizin.join(format!("d{sira}"));
            fs::create_dir_all(&dizin).unwrap();
        }
        fs::write(dizin.join("derin.txt"), b"x").unwrap();
        let (harita, _) = anlik_dosya_haritasi(&[kok], 100);
        assert!(harita.is_empty());
    }

    #[test]
    fn suzgec_reddedilen_dizinleri_taramaz() {
        let gecici = tempfile::tempdir().unwrap();
        let kok = gecici.path().join("kok");
        fs::create_dir_all(kok.join("Temp")).unwrap();
        fs::write(kok.join("uygulama.exe"), b"x").unwrap();
        fs::write(kok.join("Temp").join("gurultu.txt"), b"x").unwrap();
        let kabul = |yol: &Path| {
            yol.file_name()
                .and_then(|ad| ad.to_str())
                .map(|ad| !ad.eq_ignore_ascii_case("temp"))
                .unwrap_or(true)
        };
        let (harita, _) = anlik_dosya_haritasi_suzgecli(&[kok], 100, &kabul);
        assert_eq!(harita.len(), 1);
        assert!(harita.keys().next().unwrap().ends_with("uygulama.exe"));
    }

    #[test]
    fn json_gidis_donusleri_korunur() {
        let oturum = IzlemeOturumu {
            kimlik: "1a2b-000000000000002a".to_string(),
            baslangic: "2026-10-30T12:00:00Z".to_string(),
            dosya_sayisi: 3,
            kayit_sayisi: 4,
        };
        let kokler = vec![PathBuf::from(r"C:\Program Files")];
        let json = oturum_json(&oturum, &kokler).unwrap();
        let (geri, geri_kokler) = oturum_ayristir(&json).unwrap();
        assert_eq!(geri, oturum);
        assert_eq!(geri_kokler, vec![r"C:\Program Files".to_string()]);

        let harita = harita(&[(r"c:\a.txt", 10, 11)]);
        let json = dosya_haritasi_json(&harita).unwrap();
        assert_eq!(dosya_haritasi_ayristir(&json).unwrap(), harita);

        let kayitlar = vec!["HKLM\\A".to_string()];
        let json = kayit_listesi_json(&kayitlar).unwrap();
        assert_eq!(kayit_listesi_ayristir(&json).unwrap(), kayitlar);

        let rapor = IzlemeRaporu {
            oturum: oturum.clone(),
            bitis: "2026-10-30T12:05:00Z".to_string(),
            dosyalar: vec![IzlemeDegisikligi {
                tur: TUR_EKLENDI.to_string(),
                yol: r"c:\a.txt".to_string(),
                ayrinti: "Yeni dosya (10 bayt)".to_string(),
            }],
            kayitlar: Vec::new(),
            toplam_dosya: 1,
            toplam_kayit: 0,
            kesildi: false,
        };
        let json = rapor_json(&rapor).unwrap();
        let geri: IzlemeRaporu = serde_json::from_str(&json).unwrap();
        assert_eq!(geri, rapor);
    }

    #[test]
    fn bozuk_json_hata_doner() {
        assert!(oturum_ayristir("{").is_err());
        assert!(dosya_haritasi_ayristir("bozuk").is_err());
        assert!(kayit_listesi_ayristir("{}").is_err());
    }
}
