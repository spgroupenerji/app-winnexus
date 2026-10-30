use sp_core::bulucu::{
    disk_ogelerini_sirala, joker_eslesir, kayit_verisi_metni, zaman_metni, DiskAnalizi, DiskOgesi,
    DosyaBulgu, KayitBulgu,
};
use sp_core::error::{Hata, Sonuc};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::fs::{self, Metadata};
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, REG_DWORD, REG_EXPAND_SZ,
    REG_MULTI_SZ, REG_QWORD, REG_SZ,
};
use winreg::{RegKey, RegValue, HKEY};

const REPARSE_NOKTASI: u32 = 0x0000_0400;
const TUR_DOSYA: &str = "Dosya";
const TUR_KLASOR: &str = "Klasör";
const AZAMI_DOSYA_DERINLIGI: usize = 24;
const AZAMI_DOSYA_SONUCU: usize = 2000;
const DOSYA_BUTCESI: Duration = Duration::from_secs(60);
const KAYIT_KOKU: &str = "SOFTWARE";
const KAYIT_DERINLIGI: usize = 6;
const KAYIT_ANAHTAR_SINIRI: usize = 60_000;
const KAYIT_BUTCESI: Duration = Duration::from_secs(45);
const AZAMI_KAYIT_SONUCU: usize = 1000;
const AZAMI_DISK_SONUCU: usize = 1000;
const AZAMI_DISK_COCUGU: usize = 500;
const DISK_BUTCESI: Duration = Duration::from_secs(120);
const ONCELIKLI_KAYITLAR: [&str; 4] = [
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths",
    r"SOFTWARE\Classes\Applications",
    r"SOFTWARE\Classes\CLSID",
];

pub struct BulucuServisi;

impl BulucuServisi {
    pub fn yeni() -> Self {
        Self
    }

    pub fn dosya_ara(&self, kok: &Path, desen: &str, azami: usize) -> Sonuc<Vec<DosyaBulgu>> {
        let desen = desen.trim();
        if desen.is_empty() {
            return Err(Hata::Dogrulama(
                "Arama deseni boş olamaz; bir ad veya joker desen girin.".to_string(),
            ));
        }
        let kok = kok_dogrula(kok)?;
        let azami = azami.clamp(1, AZAMI_DOSYA_SONUCU);
        Ok(DosyaArama::yeni(desen, azami, kok).calistir())
    }

    pub fn kayit_defteri_ara(&self, sorgu: &str, azami: usize) -> Sonuc<Vec<KayitBulgu>> {
        let sorgu = sorgu.trim();
        if sorgu.chars().count() < 2 {
            return Err(Hata::Dogrulama(
                "Kayıt defteri arama sorgusu en az 2 karakter olmalı.".to_string(),
            ));
        }
        let mut tarama =
            KayitTarama::yeni(&sorgu.to_lowercase(), azami.clamp(1, AZAMI_KAYIT_SONUCU));
        let koveler = [(HKEY_LOCAL_MACHINE, "HKLM"), (HKEY_CURRENT_USER, "HKCU")];
        for (kok, kove) in koveler {
            tarama.kok = kok;
            tarama.kove = kove;
            tarama.atla = false;
            for yol in ONCELIKLI_KAYITLAR {
                tarama.tara(yol, 0);
            }
            tarama.atla = true;
            tarama.tara(KAYIT_KOKU, 0);
            if !tarama.devam_mi() {
                break;
            }
        }
        let mut bulgular = tarama.bulgular;
        bulgular.sort_by(|a, b| {
            a.kove
                .cmp(&b.kove)
                .then_with(|| a.yol.cmp(&b.yol))
                .then_with(|| a.deger.cmp(&b.deger))
                .then_with(|| a.veri.cmp(&b.veri))
        });
        Ok(bulgular)
    }

    pub fn disk_analizi(&self, kok: &Path, azami: usize) -> Sonuc<DiskAnalizi> {
        let kok = kok_dogrula(kok)?;
        let azami = azami.clamp(1, AZAMI_DISK_SONUCU);
        Ok(disk_taramasi(&kok, azami).sonuc(&kok))
    }
}

fn kok_dogrula(kok: &Path) -> Sonuc<PathBuf> {
    let kanonik = fs::canonicalize(kok)
        .map_err(|hata| Hata::Dogrulama(format!("{} bulunamadı: {hata}", kok.display())))?;
    if !kanonik.is_dir() {
        return Err(Hata::Dogrulama(format!(
            "{} bir klasör değil.",
            kok.display()
        )));
    }
    Ok(kanonik)
}

fn dosya_bulgusu(yol: &Path, ad: &str, meta: &Metadata) -> DosyaBulgu {
    let klasor = meta.is_dir();
    DosyaBulgu {
        yol: yol_metni(yol),
        ad: ad.to_string(),
        boyut: if klasor { 0 } else { meta.len() },
        degisme: meta.modified().ok().map(zaman_metni),
        tur: if klasor { TUR_KLASOR } else { TUR_DOSYA }.to_string(),
    }
}

fn yeniden_ayristirma_mi(meta: &Metadata) -> bool {
    meta.file_attributes() & REPARSE_NOKTASI != 0
}

fn yol_metni(yol: &Path) -> String {
    let metin = yol.to_string_lossy();
    if !metin.starts_with(r"\\?\") {
        return metin.into_owned();
    }
    let kalan = &metin[4..];
    match kalan.strip_prefix("UNC\\") {
        Some(unc) => format!(r"\\{unc}"),
        None => kalan.to_string(),
    }
}

struct DosyaArama<'a> {
    desen: &'a str,
    azami: usize,
    baslangic: Instant,
    bulgular: Vec<DosyaBulgu>,
    yigin: Vec<(PathBuf, usize)>,
}

impl<'a> DosyaArama<'a> {
    fn yeni(desen: &'a str, azami: usize, kok: PathBuf) -> Self {
        Self {
            desen,
            azami,
            baslangic: Instant::now(),
            bulgular: Vec::new(),
            yigin: vec![(kok, 1)],
        }
    }

    fn calistir(mut self) -> Vec<DosyaBulgu> {
        while let Some((dizin, derinlik)) = self.yigin.pop() {
            if self.bitti_mi() {
                break;
            }
            self.dizini_gez(&dizin, derinlik);
        }
        self.bulgular.sort_by(|a, b| a.yol.cmp(&b.yol));
        self.bulgular
    }

    fn bitti_mi(&self) -> bool {
        self.bulgular.len() >= self.azami || self.baslangic.elapsed() >= DOSYA_BUTCESI
    }

    fn dizini_gez(&mut self, dizin: &Path, derinlik: usize) {
        let Ok(kayitlar) = fs::read_dir(dizin) else {
            return;
        };
        for kayit in kayitlar.flatten() {
            if self.bitti_mi() {
                break;
            }
            let yol = kayit.path();
            let Ok(meta) = fs::symlink_metadata(&yol) else {
                continue;
            };
            if yeniden_ayristirma_mi(&meta) {
                continue;
            }
            let klasor = meta.is_dir();
            if !klasor && !meta.is_file() {
                continue;
            }
            let ad = kayit.file_name().to_string_lossy().into_owned();
            if joker_eslesir(self.desen, &ad) {
                self.bulgular.push(dosya_bulgusu(&yol, &ad, &meta));
            }
            if klasor && derinlik < AZAMI_DOSYA_DERINLIGI {
                self.yigin.push((yol, derinlik + 1));
            }
        }
    }
}

struct KayitTarama {
    sorgu: String,
    azami: usize,
    baslangic: Instant,
    bulgular: Vec<KayitBulgu>,
    gezilen: usize,
    atlanan: Vec<String>,
    kok: HKEY,
    kove: &'static str,
    atla: bool,
}

impl KayitTarama {
    fn yeni(sorgu: &str, azami: usize) -> Self {
        Self {
            sorgu: sorgu.to_string(),
            azami,
            baslangic: Instant::now(),
            bulgular: Vec::new(),
            gezilen: 0,
            atlanan: ONCELIKLI_KAYITLAR
                .iter()
                .map(|yol| yol.to_lowercase())
                .collect(),
            kok: HKEY_LOCAL_MACHINE,
            kove: "HKLM",
            atla: false,
        }
    }

    fn devam_mi(&self) -> bool {
        self.bulgular.len() < self.azami
            && self.gezilen < KAYIT_ANAHTAR_SINIRI
            && self.baslangic.elapsed() < KAYIT_BUTCESI
    }

    fn tara(&mut self, yol: &str, derinlik: usize) {
        if !self.devam_mi() {
            return;
        }
        let Ok(anahtar) =
            RegKey::predef(self.kok).open_subkey_with_flags(yol, KEY_READ | KEY_WOW64_64KEY)
        else {
            return;
        };
        self.gezilen += 1;
        self.anahtari_incele(&anahtar, yol);
        if derinlik >= KAYIT_DERINLIGI {
            return;
        }
        for alt in anahtar.enum_keys().flatten() {
            if !self.devam_mi() {
                return;
            }
            let alt_yol = format!("{yol}\\{alt}");
            if self.atla && self.atlanmali_mi(&alt_yol) {
                continue;
            }
            self.tara(&alt_yol, derinlik + 1);
        }
    }

    fn anahtari_incele(&mut self, anahtar: &RegKey, yol: &str) {
        let ad = yol.rsplit('\\').next().unwrap_or(yol);
        if ad.to_lowercase().contains(&self.sorgu) {
            self.ekle(KayitBulgu {
                kove: self.kove.to_string(),
                yol: yol.to_string(),
                deger: None,
                veri: String::new(),
            });
        }
        for (deger_adi, deger) in anahtar.enum_values().flatten() {
            if !self.devam_mi() {
                return;
            }
            let Some(veri) = kayit_degeri_metni(&deger) else {
                continue;
            };
            let eslesti = deger_adi.to_lowercase().contains(&self.sorgu)
                || veri.to_lowercase().contains(&self.sorgu);
            if eslesti {
                self.ekle(KayitBulgu {
                    kove: self.kove.to_string(),
                    yol: yol.to_string(),
                    deger: Some(deger_adi),
                    veri,
                });
            }
        }
    }

    fn ekle(&mut self, bulgu: KayitBulgu) {
        if self.bulgular.len() < self.azami {
            self.bulgular.push(bulgu);
        }
    }

    fn atlanmali_mi(&self, yol: &str) -> bool {
        let yol = yol.to_lowercase();
        self.atlanan.iter().any(|kok| {
            yol.starts_with(kok.as_str())
                && (yol.len() == kok.len() || yol.as_bytes()[kok.len()] == b'\\')
        })
    }
}

fn kayit_degeri_metni(deger: &RegValue) -> Option<String> {
    match deger.vtype {
        REG_SZ | REG_EXPAND_SZ | REG_MULTI_SZ => Some(kayit_verisi_metni(&deger.bytes)),
        REG_DWORD => sayi_metni(&deger.bytes),
        REG_QWORD => buyuk_sayi_metni(&deger.bytes),
        _ => None,
    }
}

fn sayi_metni(veri: &[u8]) -> Option<String> {
    let baytlar: [u8; 4] = veri.try_into().ok()?;
    Some(u32::from_le_bytes(baytlar).to_string())
}

fn buyuk_sayi_metni(veri: &[u8]) -> Option<String> {
    let baytlar: [u8; 8] = veri.try_into().ok()?;
    Some(u64::from_le_bytes(baytlar).to_string())
}

struct DiskTarama {
    toplam: u64,
    ogeler: Vec<DiskOgesi>,
    dosyalar: BinaryHeap<Reverse<(u64, String, u32)>>,
    azami: usize,
    kesildi: bool,
}

impl DiskTarama {
    fn yeni(azami: usize) -> Self {
        Self {
            toplam: 0,
            ogeler: Vec::new(),
            dosyalar: BinaryHeap::new(),
            azami,
            kesildi: false,
        }
    }

    fn cocugu_isle(&mut self, kayit: &fs::DirEntry, baslangic: &Instant) {
        let yol = kayit.path();
        let Ok(meta) = fs::symlink_metadata(&yol) else {
            return;
        };
        if yeniden_ayristirma_mi(&meta) {
            return;
        }
        let ad = kayit.file_name().to_string_lossy().into_owned();
        if meta.is_dir() {
            let boyut = self.alt_agaci_isle(&yol, baslangic);
            self.ogeler.push(DiskOgesi {
                yol: yol_metni(&yol),
                ad,
                boyut,
                dosya_mi: false,
                derinlik: 1,
            });
        } else if meta.is_file() {
            let boyut = meta.len();
            self.toplam = self.toplam.saturating_add(boyut);
            self.dosya_ekle(&yol, boyut, 1);
            self.ogeler.push(DiskOgesi {
                yol: yol_metni(&yol),
                ad,
                boyut,
                dosya_mi: true,
                derinlik: 1,
            });
        }
    }

    fn alt_agaci_isle(&mut self, kok: &Path, baslangic: &Instant) -> u64 {
        let mut toplam = 0u64;
        let mut yigin = vec![(kok.to_path_buf(), 2u32)];
        while let Some((dizin, derinlik)) = yigin.pop() {
            if baslangic.elapsed() >= DISK_BUTCESI {
                self.kesildi = true;
                break;
            }
            let Ok(kayitlar) = fs::read_dir(&dizin) else {
                continue;
            };
            for kayit in kayitlar.flatten() {
                if baslangic.elapsed() >= DISK_BUTCESI {
                    self.kesildi = true;
                    break;
                }
                let yol = kayit.path();
                let Ok(meta) = fs::symlink_metadata(&yol) else {
                    continue;
                };
                if yeniden_ayristirma_mi(&meta) {
                    continue;
                }
                if meta.is_dir() {
                    yigin.push((yol, derinlik + 1));
                } else if meta.is_file() {
                    let boyut = meta.len();
                    toplam = toplam.saturating_add(boyut);
                    self.dosya_ekle(&yol, boyut, derinlik);
                }
            }
        }
        self.toplam = self.toplam.saturating_add(toplam);
        toplam
    }

    fn dosya_ekle(&mut self, yol: &Path, boyut: u64, derinlik: u32) {
        self.dosyalar
            .push(Reverse((boyut, yol_metni(yol), derinlik)));
        if self.dosyalar.len() > self.azami {
            self.dosyalar.pop();
        }
    }

    fn sonuc(self, kok: &Path) -> DiskAnalizi {
        let mut ogeler = self.ogeler;
        let cocuklar: HashSet<String> = ogeler.iter().map(|oge| oge.yol.clone()).collect();
        for Reverse((boyut, yol, derinlik)) in self.dosyalar {
            if cocuklar.contains(&yol) {
                continue;
            }
            let ad = Path::new(yol.as_str())
                .file_name()
                .map(|ad| ad.to_string_lossy().into_owned())
                .unwrap_or_default();
            ogeler.push(DiskOgesi {
                yol,
                ad,
                boyut,
                dosya_mi: true,
                derinlik,
            });
        }
        disk_ogelerini_sirala(&mut ogeler);
        DiskAnalizi {
            kok: yol_metni(kok),
            toplam_bayt: self.toplam,
            ogeler,
            kesildi: self.kesildi,
        }
    }
}

fn disk_taramasi(kok: &Path, azami: usize) -> DiskTarama {
    let baslangic = Instant::now();
    let mut tarama = DiskTarama::yeni(azami);
    let Ok(kayitlar) = fs::read_dir(kok) else {
        return tarama;
    };
    for kayit in kayitlar.flatten().take(AZAMI_DISK_COCUGU) {
        if baslangic.elapsed() >= DISK_BUTCESI {
            tarama.kesildi = true;
            break;
        }
        tarama.cocugu_isle(&kayit, &baslangic);
    }
    tarama
}

#[cfg(test)]
mod testler {
    use super::*;
    use winreg::enums::REG_BINARY;

    fn servis() -> BulucuServisi {
        BulucuServisi::yeni()
    }

    #[test]
    fn dosya_arama_deseni_ve_turleri_bulur() {
        let kok = tempfile::tempdir().unwrap();
        fs::write(kok.path().join("rapor.txt"), b"abc").unwrap();
        fs::write(kok.path().join("veri.log"), b"ab").unwrap();
        fs::create_dir(kok.path().join("yillik-rapor-2026")).unwrap();
        let sonuc = servis().dosya_ara(kok.path(), "*.txt", 10).unwrap();
        assert_eq!(sonuc.len(), 1);
        assert_eq!(sonuc[0].ad, "rapor.txt");
        assert_eq!(sonuc[0].boyut, 3);
        assert_eq!(sonuc[0].tur, "Dosya");
        assert!(sonuc[0].degisme.is_some());
        assert!(!sonuc[0].yol.starts_with(r"\\?\"));
        let bulunan = servis().dosya_ara(kok.path(), "rapor", 10).unwrap();
        assert_eq!(bulunan.len(), 2);
        assert!(bulunan.iter().any(|bulgu| bulgu.tur == "Klasör"));
        assert!(bulunan.iter().all(|bulgu| bulgu.ad.contains("rapor")));
    }

    #[test]
    fn dosya_arama_azami_sinirini_uygular() {
        let kok = tempfile::tempdir().unwrap();
        for sira in 0..5 {
            fs::write(kok.path().join(format!("d{sira}.txt")), b"x").unwrap();
        }
        assert_eq!(
            servis().dosya_ara(kok.path(), "d?.txt", 3).unwrap().len(),
            3
        );
        assert_eq!(servis().dosya_ara(kok.path(), "*.txt", 0).unwrap().len(), 1);
        assert_eq!(
            servis().dosya_ara(kok.path(), "*.txt", 5000).unwrap().len(),
            5
        );
    }

    #[test]
    fn dosya_arama_gecersiz_girdiyi_reddeder() {
        let kok = tempfile::tempdir().unwrap();
        assert!(matches!(
            servis().dosya_ara(kok.path(), "  ", 10),
            Err(Hata::Dogrulama(_))
        ));
        assert!(servis()
            .dosya_ara(&kok.path().join("yok"), "*", 10)
            .is_err());
        let dosya = kok.path().join("tek.txt");
        fs::write(&dosya, b"x").unwrap();
        assert!(servis().dosya_ara(&dosya, "*", 10).is_err());
    }

    #[test]
    fn disk_analizi_toplam_ve_en_buyuk_dosyalari_toplars() {
        let kok = tempfile::tempdir().unwrap();
        fs::write(kok.path().join("kucuk.txt"), vec![0u8; 10]).unwrap();
        let alt = kok.path().join("buyuk");
        fs::create_dir(&alt).unwrap();
        fs::write(alt.join("dosya.bin"), vec![0u8; 200]).unwrap();
        let analiz = servis().disk_analizi(kok.path(), 5).unwrap();
        assert!(!analiz.kesildi);
        assert_eq!(analiz.toplam_bayt, 210);
        assert!(!analiz.kok.starts_with(r"\\?\"));
        assert_eq!(analiz.ogeler.len(), 3);
        let klasor = analiz.ogeler.iter().find(|oge| oge.ad == "buyuk").unwrap();
        assert!(!klasor.dosya_mi);
        assert_eq!(klasor.boyut, 200);
        assert_eq!(klasor.derinlik, 1);
        let dosya = analiz
            .ogeler
            .iter()
            .find(|oge| oge.ad == "dosya.bin")
            .unwrap();
        assert!(dosya.dosya_mi);
        assert_eq!(dosya.boyut, 200);
        assert_eq!(dosya.derinlik, 2);
        assert_eq!(analiz.ogeler[0].boyut, 200);
    }

    #[test]
    fn disk_analizi_dosya_sayisini_azamiyla_sinirlar() {
        let kok = tempfile::tempdir().unwrap();
        let alt = kok.path().join("alt");
        fs::create_dir(&alt).unwrap();
        for sira in 0..5u64 {
            fs::write(
                alt.join(format!("f{sira}.bin")),
                vec![0u8; (100 + sira) as usize],
            )
            .unwrap();
        }
        let analiz = servis().disk_analizi(kok.path(), 2).unwrap();
        assert_eq!(analiz.toplam_bayt, 510);
        assert_eq!(analiz.ogeler.len(), 3);
        let dosyalar: Vec<&DiskOgesi> = analiz.ogeler.iter().filter(|oge| oge.dosya_mi).collect();
        assert_eq!(dosyalar.len(), 2);
        assert_eq!(dosyalar[0].boyut, 104);
        assert_eq!(dosyalar[1].boyut, 103);
    }

    #[test]
    fn reparse_noktalari_gezilmez() {
        let kok = tempfile::tempdir().unwrap();
        let disari = tempfile::tempdir().unwrap();
        fs::write(disari.path().join("gizli.txt"), b"gizli").unwrap();
        let baglanti = kok.path().join("baglanti");
        if !baglanti_olustur(&baglanti, disari.path()) {
            return;
        }
        let bulgular = servis().dosya_ara(kok.path(), "*", 50).unwrap();
        assert!(bulgular.is_empty());
        let analiz = servis().disk_analizi(kok.path(), 10).unwrap();
        assert_eq!(analiz.toplam_bayt, 0);
        assert!(analiz.ogeler.is_empty());
    }

    fn baglanti_olustur(baglanti: &Path, hedef: &Path) -> bool {
        if std::os::windows::fs::symlink_dir(hedef, baglanti).is_ok() {
            return true;
        }
        std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(baglanti)
            .arg(hedef)
            .status()
            .map(|durum| durum.success())
            .unwrap_or(false)
    }

    #[test]
    fn kayit_degeri_turleri_metne_cevrilir() {
        let metin = RegValue {
            bytes: "Test\0".encode_utf16().flat_map(u16::to_le_bytes).collect(),
            vtype: REG_SZ,
        };
        assert_eq!(kayit_degeri_metni(&metin).as_deref(), Some("Test"));
        let sayi = RegValue {
            bytes: 42u32.to_le_bytes().to_vec(),
            vtype: REG_DWORD,
        };
        assert_eq!(kayit_degeri_metni(&sayi).as_deref(), Some("42"));
        let ikili = RegValue {
            bytes: vec![0x01, 0x02, 0x03],
            vtype: REG_BINARY,
        };
        assert!(kayit_degeri_metni(&ikili).is_none());
    }

    #[test]
    fn kisa_kayit_sorgusu_reddedilir() {
        assert!(matches!(
            servis().kayit_defteri_ara("a", 10),
            Err(Hata::Dogrulama(_))
        ));
        assert!(matches!(
            servis().kayit_defteri_ara("   ", 10),
            Err(Hata::Dogrulama(_))
        ));
        assert!(matches!(
            servis().kayit_defteri_ara("İ", 10),
            Err(Hata::Dogrulama(_))
        ));
    }

    #[test]
    fn verbatim_yol_onekini_kaldirir() {
        assert_eq!(
            yol_metni(Path::new(r"\\?\C:\Klasor\dosya.txt")),
            r"C:\Klasor\dosya.txt"
        );
        assert_eq!(
            yol_metni(Path::new(r"\\?\UNC\sunucu\pay\dosya.txt")),
            r"\\sunucu\pay\dosya.txt"
        );
        assert_eq!(yol_metni(Path::new(r"C:\Klasor")), r"C:\Klasor");
    }
}
