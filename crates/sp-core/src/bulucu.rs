use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const BIRIMLER: [&str; 4] = ["B", "KB", "MB", "GB"];
const BAYT_TABANI: u128 = 1024;
const IKILI_HEX_SINIRI: usize = 64;
const KLASOR_TURU: &str = "Klasör";
const UZANTISIZ_TUR: &str = "Uzantısız";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DosyaBulgu {
    pub yol: String,
    pub ad: String,
    pub boyut: u64,
    pub degisme: Option<String>,
    pub tur: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KayitBulgu {
    pub kove: String,
    pub yol: String,
    pub deger: Option<String>,
    pub veri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskOgesi {
    pub yol: String,
    pub ad: String,
    pub boyut: u64,
    pub dosya_mi: bool,
    pub derinlik: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskAnalizi {
    pub kok: String,
    pub toplam_bayt: u64,
    pub ogeler: Vec<DiskOgesi>,
    pub kesildi: bool,
}

pub fn joker_eslesir(desen: &str, ad: &str) -> bool {
    if desen.is_empty() {
        return false;
    }
    if !desen.contains(['*', '?']) {
        return ad.to_lowercase().contains(&desen.to_lowercase());
    }
    let desen: Vec<char> = desen.to_lowercase().chars().collect();
    let ad: Vec<char> = ad.to_lowercase().chars().collect();
    joker_karakter_eslesir(&desen, &ad)
}

fn joker_karakter_eslesir(desen: &[char], ad: &[char]) -> bool {
    let (mut desen_konumu, mut ad_konumu) = (0usize, 0usize);
    let (mut yildiz, mut yedek) = (None, 0usize);
    while ad_konumu < ad.len() {
        let joker_mi = desen_konumu < desen.len() && desen[desen_konumu] == '*';
        let eslesiyor_mu = desen_konumu < desen.len()
            && (desen[desen_konumu] == '?' || desen[desen_konumu] == ad[ad_konumu]);
        if eslesiyor_mu {
            desen_konumu += 1;
            ad_konumu += 1;
        } else if joker_mi {
            yildiz = Some(desen_konumu);
            desen_konumu += 1;
            yedek = ad_konumu;
        } else if let Some(yildiz_konumu) = yildiz {
            yedek += 1;
            ad_konumu = yedek;
            desen_konumu = yildiz_konumu + 1;
        } else {
            return false;
        }
    }
    desen[desen_konumu..]
        .iter()
        .all(|karakter| *karakter == '*')
}

pub fn boyut_metni(bayt: u64) -> String {
    let mut birim = 0usize;
    while birim + 1 < BIRIMLER.len() && u128::from(bayt) >= BAYT_TABANI.pow(birim as u32 + 1) {
        birim += 1;
    }
    if birim == 0 {
        return format!("{bayt} B");
    }
    let bolen = BAYT_TABANI.pow(birim as u32);
    let ondalik = (u128::from(bayt) * 10 + bolen / 2) / bolen;
    let (tam, kesir) = (ondalik / 10, ondalik % 10);
    if kesir == 0 {
        format!("{tam} {}", BIRIMLER[birim])
    } else {
        format!("{tam},{kesir} {}", BIRIMLER[birim])
    }
}

pub fn dosya_turu(yol: &Path) -> String {
    if yol.is_dir() {
        return KLASOR_TURU.to_string();
    }
    let Some(uzanti) = yol.extension() else {
        return UZANTISIZ_TUR.to_string();
    };
    let uzanti = uzanti.to_string_lossy();
    let uzanti = uzanti.trim();
    if uzanti.is_empty() {
        return UZANTISIZ_TUR.to_string();
    }
    uzanti.to_uppercase()
}

pub fn kayit_verisi_metni(veri: &[u8]) -> String {
    if veri.is_empty() {
        return String::new();
    }
    if let Some(metin) = utf16_metni(veri) {
        return metin;
    }
    let sinir = veri.len().min(IKILI_HEX_SINIRI);
    let mut metin = hex_metni(&veri[..sinir]);
    if veri.len() > IKILI_HEX_SINIRI {
        metin.push_str(" ...");
    }
    metin
}

fn utf16_metni(veri: &[u8]) -> Option<String> {
    if !veri.len().is_multiple_of(2) {
        return None;
    }
    let (ikililer, _) = veri.as_chunks::<2>();
    let birimler: Vec<u16> = ikililer
        .iter()
        .map(|ikili| u16::from_le_bytes(*ikili))
        .collect();
    if !birimler.iter().all(|birim| metin_birimi_mi(*birim)) {
        return None;
    }
    let metin: String = birimler
        .iter()
        .filter_map(|birim| char::from_u32(u32::from(*birim)))
        .collect();
    let parcalar: Vec<&str> = metin
        .split('\0')
        .map(str::trim)
        .filter(|parca| !parca.is_empty())
        .collect();
    Some(parcalar.join(" | "))
}

fn metin_birimi_mi(birim: u16) -> bool {
    if birim == 0 {
        return true;
    }
    if (0xFDD0..=0xFDEF).contains(&birim) || (birim & 0xFFFE) == 0xFFFE {
        return false;
    }
    char::from_u32(u32::from(birim))
        .is_some_and(|karakter| !karakter.is_control() || matches!(karakter, '\t' | '\r' | '\n'))
}

fn hex_metni(veri: &[u8]) -> String {
    veri.iter()
        .map(|bayt| format!("{bayt:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(clippy::ptr_arg)]
pub fn disk_ogelerini_sirala(ogeler: &mut Vec<DiskOgesi>) {
    ogeler.sort_by(|a, b| b.boyut.cmp(&a.boyut).then_with(|| a.yol.cmp(&b.yol)));
}

pub fn zaman_metni(zaman: SystemTime) -> String {
    let Ok(gecen) = zaman.duration_since(UNIX_EPOCH) else {
        return String::new();
    };
    let Ok(saniye) = i64::try_from(gecen.as_secs()) else {
        return String::new();
    };
    let Ok(an) = time::OffsetDateTime::from_unix_timestamp(saniye) else {
        return String::new();
    };
    an.format(time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]"
    ))
    .unwrap_or_default()
}

#[cfg(test)]
mod testler {
    use super::*;

    fn utf16_bayt(metin: &str) -> Vec<u8> {
        metin.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn disk_ogesi(yol: &str, boyut: u64) -> DiskOgesi {
        DiskOgesi {
            yol: yol.to_string(),
            ad: yol.to_string(),
            boyut,
            dosya_mi: true,
            derinlik: 1,
        }
    }

    #[test]
    fn joker_yildiz_ve_soru_isaretini_destekler() {
        assert!(joker_eslesir("*.TXT", "rapor.txt"));
        assert!(joker_eslesir("rapor?txt", "rapor1txt"));
        assert!(joker_eslesir("*rapor*", "yillik-rapor-2026.pdf"));
        assert!(joker_eslesir("?", "a"));
        assert!(joker_eslesir("*", ""));
        assert!(!joker_eslesir("rapor?.txt", "rapor.txt"));
        assert!(!joker_eslesir("*.doc", "rapor.txt"));
        assert!(!joker_eslesir("?", ""));
    }

    #[test]
    fn joker_olmayan_desen_alt_dizi_arar() {
        assert!(joker_eslesir("RAP", "yillik-rapor"));
        assert!(joker_eslesir("rapor", "RAPOR"));
        assert!(!joker_eslesir("xyz", "rapor"));
        assert!(!joker_eslesir("", "rapor"));
    }

    #[test]
    fn joker_geri_izlemeli_eslesme_yapar() {
        assert!(joker_eslesir("a*b*c", "axxbyyc"));
        assert!(joker_eslesir("*.*", "a.b.c"));
        assert!(!joker_eslesir("a*b*c", "axxbyy"));
    }

    #[test]
    fn boyut_metni_birimleri_ve_turkce_ondaligi_yazar() {
        assert_eq!(boyut_metni(0), "0 B");
        assert_eq!(boyut_metni(1023), "1023 B");
        assert_eq!(boyut_metni(1024), "1 KB");
        assert_eq!(boyut_metni(1536), "1,5 KB");
        assert_eq!(boyut_metni(1024 * 1024), "1 MB");
        assert_eq!(boyut_metni(5 * 1024 * 1024 / 2), "2,5 MB");
        assert_eq!(boyut_metni(1024 * 1024 * 1024), "1 GB");
        assert!(boyut_metni(u64::MAX).ends_with(" GB"));
    }

    #[test]
    fn dosya_turu_uzantiyi_buyuk_harfe_cevirir() {
        assert_eq!(dosya_turu(Path::new(r"C:\Kurulum\WinNexus.exe")), "EXE");
        assert_eq!(dosya_turu(Path::new("arsiv.tar.gz")), "GZ");
        assert_eq!(dosya_turu(Path::new("README")), "Uzantısız");
    }

    #[test]
    fn dosya_turu_klasoru_ayirt_eder() {
        let kok = tempfile::tempdir().unwrap();
        let dosya = kok.path().join("notlar.txt");
        std::fs::write(&dosya, b"veri").unwrap();
        assert_eq!(dosya_turu(kok.path()), "Klasör");
        assert_eq!(dosya_turu(&dosya), "TXT");
    }

    #[test]
    fn kayit_verisi_duz_metni_cozer() {
        assert_eq!(
            kayit_verisi_metni(&utf16_bayt("Merhaba Dünya\0")),
            "Merhaba Dünya"
        );
        assert_eq!(kayit_verisi_metni(&[]), "");
        assert_eq!(kayit_verisi_metni(&utf16_bayt("\0\0")), "");
    }

    #[test]
    fn kayit_verisi_coklu_degerleri_birlestirir() {
        assert_eq!(kayit_verisi_metni(&utf16_bayt("bir\0iki\0\0")), "bir | iki");
    }

    #[test]
    fn kayit_verisi_ikili_veriyi_hex_olarak_sinirlar() {
        assert_eq!(kayit_verisi_metni(&[0x01, 0x02, 0xFF]), "01 02 FF");
        assert_eq!(kayit_verisi_metni(&[0x01, 0x00, 0x00, 0x00]), "01 00 00 00");
        let uzun = kayit_verisi_metni(&[0x01; 65]);
        assert!(uzun.ends_with(" ..."));
        assert_eq!(uzun.split(" ...").next().unwrap().split(' ').count(), 64);
    }

    #[test]
    fn disk_ogeleri_boyuta_gore_azalan_siralanir() {
        let mut ogeler = vec![
            disk_ogesi("a", 5),
            disk_ogesi("b", 100),
            disk_ogesi("c", 100),
        ];
        disk_ogelerini_sirala(&mut ogeler);
        assert_eq!(ogeler[0].yol, "b");
        assert_eq!(ogeler[1].yol, "c");
        assert_eq!(ogeler[2].yol, "a");
    }

    #[test]
    fn zaman_metni_epoch_degerini_bicimlendirir() {
        assert_eq!(zaman_metni(UNIX_EPOCH), "1970-01-01 00:00");
    }
}
