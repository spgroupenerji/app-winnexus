use crate::error::{Hata, Sonuc};
use regex::Regex;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::OnceLock;
use strsim::jaro_winkler;

const AZAMI_LISTE_PAKETI: usize = 500;
const AZAMI_KAYNAK_ADRESI: usize = 2048;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profil {
    pub id: String,
    pub ad: String,
    pub aciklama: String,
    #[serde(default)]
    pub paket: Vec<ProfilPaketi>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilPaketi {
    pub id: String,
    #[serde(default)]
    pub ad: Option<String>,
    #[serde(default)]
    pub yayinci: Option<String>,
    #[serde(default)]
    pub surum: Option<String>,
    #[serde(default)]
    pub replaces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WingetPaketi {
    pub id: String,
    pub ad: Option<String>,
    pub kaynak: Option<String>,
    pub surum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuncellemeSatiri {
    pub id: String,
    pub ad: String,
    pub kaynak: String,
    pub kurulu_surum: String,
    pub mevcut_surum: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinTuru {
    Pinning,
    Blocking,
    Gating,
    Bilinmiyor,
}

impl PinTuru {
    pub fn metin(self) -> &'static str {
        match self {
            Self::Pinning => "Sabit",
            Self::Blocking => "Asla güncelle",
            Self::Gating => "Sürüm sabit",
            Self::Bilinmiyor => "Bilinmiyor",
        }
    }

    fn ayristir(metin: &str) -> Self {
        match metin {
            "Pinning" => Self::Pinning,
            "Blocking" => Self::Blocking,
            "Gating" => Self::Gating,
            _ => Self::Bilinmiyor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinKaydi {
    pub id: String,
    pub kaynak: Option<String>,
    pub surum: Option<String>,
    pub tur: PinTuru,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AramaSonucu {
    pub id: String,
    pub ad: String,
    pub surum: Option<String>,
    pub kaynak: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kaynak {
    pub ad: String,
    pub adres: String,
    pub acik: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guven {
    Yuksek,
    Orta,
    Dusuk,
}

impl Guven {
    pub fn metin(self) -> &'static str {
        match self {
            Self::Yuksek => "Yüksek",
            Self::Orta => "Orta",
            Self::Dusuk => "Düşük",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaketOnerisi {
    pub id: String,
    pub ad: String,
    pub surum: Option<String>,
    pub kaynak: Option<String>,
    pub guven: Guven,
    pub puan: f64,
}

#[derive(Debug, Clone)]
pub enum ProfilDurum {
    Eksik,
    Oneri(PaketOnerisi),
    KaynaktaYok,
}

#[derive(Debug, Clone)]
pub struct ProfilTaramaSonucu {
    pub profil_id: String,
    pub profil_ad: String,
    pub paket_id: String,
    pub paket_ad: String,
    pub kullanilacak_id: String,
    pub durum: ProfilDurum,
}

fn kimlik_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._+-]{0,127}$").expect("sabit regex"))
}

fn kaynak_adi_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$").expect("sabit regex"))
}

pub fn paket_kimligi_gecerli(id: &str) -> bool {
    kimlik_regex().is_match(id)
}

pub fn kaynak_adi_gecerli_mi(ad: &str) -> bool {
    kaynak_adi_regex().is_match(ad)
}

pub fn kaynak_adresi_gecerli_mi(adres: &str) -> bool {
    if adres.len() > AZAMI_KAYNAK_ADRESI
        || adres
            .chars()
            .any(|karakter| karakter.is_whitespace() || karakter.is_control())
        || adres.contains('\\')
    {
        return false;
    }
    let Some(yol) = adres.strip_prefix("https://") else {
        return false;
    };
    let yetkili = yol.split(['/', '?', '#']).next().unwrap_or_default();
    if yetkili.is_empty() || yetkili.contains('@') {
        return false;
    }
    let sunucu = yetkili.split(':').next().unwrap_or_default();
    !sunucu.is_empty()
        && sunucu
            .chars()
            .any(|karakter| karakter.is_ascii_alphanumeric())
}

pub fn paket_kimligi_dogrula(id: &str) -> Sonuc<()> {
    if paket_kimligi_gecerli(id) {
        Ok(())
    } else {
        Err(Hata::Beklenmeyen(format!(
            "Geçersiz winget paket kimliği: {id}"
        )))
    }
}

#[derive(Debug, Deserialize)]
struct Ihracat {
    #[serde(rename = "Sources", default)]
    sources: Vec<IhracatKaynak>,
}

#[derive(Debug, Deserialize)]
struct IhracatKaynak {
    #[serde(rename = "Packages", default)]
    packages: Vec<IhracatPaket>,
    #[serde(rename = "SourceDetails", default)]
    source_details: Option<IhracatDetay>,
}

#[derive(Debug, Deserialize)]
struct IhracatPaket {
    #[serde(rename = "PackageIdentifier")]
    package_identifier: String,
    #[serde(rename = "Version", default)]
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IhracatDetay {
    #[serde(rename = "Name", default)]
    name: Option<String>,
}

pub fn ihracat_json_ayristir(metin: &str) -> Sonuc<Vec<WingetPaketi>> {
    let ihracat: Ihracat = serde_json::from_str(metin)
        .map_err(|hata| Hata::Beklenmeyen(format!("Winget export JSON okunamadı: {hata}")))?;
    let mut gorulen = HashSet::new();
    let mut paketler = Vec::new();
    for kaynak in ihracat.sources {
        let kaynak_adi = kaynak.source_details.and_then(|detay| detay.name);
        for paket in kaynak.packages {
            paket_kimligi_dogrula(&paket.package_identifier)?;
            if gorulen.insert(paket.package_identifier.clone()) {
                paketler.push(WingetPaketi {
                    id: paket.package_identifier,
                    ad: None,
                    kaynak: kaynak_adi.clone(),
                    surum: paket.version,
                });
            }
        }
    }
    if paketler.len() > 500 {
        return Err(Hata::Beklenmeyen(
            "Winget export dosyası 500 paket sınırını aşıyor.".into(),
        ));
    }
    Ok(paketler)
}

pub fn guncelleme_tablosu_ayristir(metin: &str) -> Sonuc<Vec<GuncellemeSatiri>> {
    let mut sonuc = Vec::new();
    for satir in metin.lines() {
        let satir = satir.trim();
        if satir.is_empty()
            || satir.starts_with("Name ")
            || satir.starts_with("---")
            || satir.contains("upgrades available")
            || satir.contains("güncelleme mevcut")
        {
            continue;
        }
        let parcalar = satir.split_whitespace().collect::<Vec<_>>();
        if parcalar.len() < 5 {
            continue;
        }

        let kaynak = parcalar[parcalar.len() - 1].to_string();
        let son = parcalar.len() - 2;
        let (mevcut_surum, onceki) = alan_ve_onceki(&parcalar, son);
        let Some(son) = onceki else {
            continue;
        };
        let (kurulu_surum, onceki) = alan_ve_onceki(&parcalar, son);
        let Some(id_idx) = onceki else {
            continue;
        };
        let id = parcalar[id_idx];
        if !paket_kimligi_gecerli(id) {
            continue;
        }
        let ad = parcalar[..id_idx].join(" ");
        if ad.is_empty() {
            continue;
        }
        sonuc.push(GuncellemeSatiri {
            id: id.to_string(),
            ad,
            kaynak,
            kurulu_surum,
            mevcut_surum,
        });
    }
    Ok(sonuc)
}

fn alan_ve_onceki(parcalar: &[&str], son: usize) -> (String, Option<usize>) {
    if son > 0 && parcalar[son - 1] == "<" {
        (
            format!("< {}", parcalar[son]),
            if son >= 2 { Some(son - 2) } else { None },
        )
    } else {
        (
            parcalar[son].to_string(),
            if son >= 1 { Some(son - 1) } else { None },
        )
    }
}

pub fn pin_tablosu_ayristir(metin: &str) -> Sonuc<Vec<PinKaydi>> {
    let mut sonuc = Vec::new();
    for satir in metin.lines() {
        let satir = satir.trim();
        if satir.is_empty()
            || satir.starts_with("PackageIdentifier")
            || satir.starts_with("---")
            || satir.to_lowercase().contains("no pins")
        {
            continue;
        }
        let parcalar = satir.split_whitespace().collect::<Vec<_>>();
        if parcalar.len() < 3 {
            continue;
        }
        let id = parcalar[0];
        if !paket_kimligi_gecerli(id) {
            continue;
        }
        let tur = PinTuru::ayristir(parcalar[parcalar.len() - 1]);
        if tur == PinTuru::Bilinmiyor {
            continue;
        }
        let kaynak = if parcalar.len() >= 3 {
            Some(parcalar[1].to_string())
        } else {
            None
        };
        let surum = if parcalar.len() >= 4 {
            Some(parcalar[2].to_string())
        } else {
            None
        };
        sonuc.push(PinKaydi {
            id: id.to_string(),
            kaynak,
            surum,
            tur,
        });
    }
    Ok(sonuc)
}

pub fn arama_tablosu_ayristir(metin: &str) -> Sonuc<Vec<AramaSonucu>> {
    let kaynak_sutunu = metin
        .lines()
        .map(str::trim)
        .find(|satir| !satir.is_empty())
        .map(|baslik| {
            baslik
                .split_whitespace()
                .any(|parca| parca.eq_ignore_ascii_case("source"))
        })
        .unwrap_or(true);
    let mut sonuc = Vec::new();
    for satir in metin.lines() {
        let satir = satir.trim();
        if satir.is_empty()
            || satir.starts_with("Name ")
            || satir.starts_with("---")
            || satir.to_lowercase().contains("no package found")
        {
            continue;
        }
        let parcalar = satir.split_whitespace().collect::<Vec<_>>();
        let asgari = if kaynak_sutunu { 4 } else { 3 };
        if parcalar.len() < asgari {
            continue;
        }
        let (id_idx, surum, kaynak) = if kaynak_sutunu {
            (
                parcalar.len() - 3,
                parcalar[parcalar.len() - 2].to_string(),
                Some(parcalar[parcalar.len() - 1].to_string()),
            )
        } else {
            (
                parcalar.len() - 2,
                parcalar[parcalar.len() - 1].to_string(),
                None,
            )
        };
        let id = parcalar[id_idx];
        if !paket_kimligi_gecerli(id) {
            continue;
        }
        let ad = parcalar[..id_idx].join(" ");
        if ad.is_empty() {
            continue;
        }
        sonuc.push(AramaSonucu {
            id: id.to_string(),
            ad,
            surum: Some(surum),
            kaynak,
        });
    }
    Ok(sonuc)
}

pub fn kaynak_tablosu_ayristir(metin: &str) -> Sonuc<Vec<Kaynak>> {
    let mut sonuc = Vec::new();
    for satir in metin.lines() {
        let satir = satir.trim();
        if satir.is_empty() || satir.starts_with("---") {
            continue;
        }
        let parcalar = satir.split_whitespace().collect::<Vec<_>>();
        if parcalar.len() < 2 || parcalar[0].eq_ignore_ascii_case("name") {
            continue;
        }
        let ad = parcalar[0];
        if !kaynak_adi_gecerli_mi(ad) {
            continue;
        }
        let son = parcalar[parcalar.len() - 1];
        let (adres_parcalari, acik) =
            if son.eq_ignore_ascii_case("true") || son.eq_ignore_ascii_case("false") {
                (
                    &parcalar[1..parcalar.len() - 1],
                    son.eq_ignore_ascii_case("true"),
                )
            } else {
                (&parcalar[1..], false)
            };
        if adres_parcalari.is_empty() {
            continue;
        }
        let adres = adres_parcalari.join(" ");
        if !kaynak_adresi_gecerli_mi(&adres) {
            continue;
        }
        sonuc.push(Kaynak {
            ad: ad.to_string(),
            adres,
            acik,
        });
    }
    Ok(sonuc)
}

pub fn liste_metni_ayristir(metin: &str) -> Sonuc<Vec<String>> {
    let metin = metin.strip_prefix('\u{feff}').unwrap_or(metin);
    let mut gorulen = HashSet::new();
    let mut idler = Vec::new();
    for (sira, satir) in metin.lines().enumerate() {
        let satir = satir.trim();
        if satir.is_empty()
            || satir.starts_with('#')
            || satir.starts_with(';')
            || satir.starts_with("---")
            || satir.starts_with("Name ")
            || satir.starts_with("PackageIdentifier")
        {
            continue;
        }
        if satir.split_whitespace().count() != 1 {
            return Err(Hata::Beklenmeyen(format!(
                "Satır {}: her satırda tek bir paket kimliği olmalı: {satir}",
                sira + 1
            )));
        }
        paket_kimligi_dogrula(satir)?;
        if gorulen.insert(satir.to_string()) {
            idler.push(satir.to_string());
            if idler.len() > AZAMI_LISTE_PAKETI {
                return Err(Hata::Beklenmeyen(format!(
                    "Paket listesi {AZAMI_LISTE_PAKETI} paket sınırını aşıyor."
                )));
            }
        }
    }
    Ok(idler)
}

pub fn liste_json_olustur(idler: &[String]) -> Sonuc<String> {
    let mut gorulen = HashSet::new();
    let mut paketler = Vec::new();
    for id in idler {
        paket_kimligi_dogrula(id)?;
        if gorulen.insert(id.as_str()) {
            paketler.push(serde_json::json!({ "PackageIdentifier": id }));
            if paketler.len() > AZAMI_LISTE_PAKETI {
                return Err(Hata::Beklenmeyen(format!(
                    "Paket listesi {AZAMI_LISTE_PAKETI} paket sınırını aşıyor."
                )));
            }
        }
    }
    if paketler.is_empty() {
        return Err(Hata::Beklenmeyen(
            "Dışa aktarılacak geçerli paket kimliği bulunamadı.".into(),
        ));
    }
    let belge = serde_json::json!({
        "Sources": [{
            "Packages": paketler,
            "SourceDetails": { "Name": "winget" }
        }]
    });
    serde_json::to_string_pretty(&belge)
        .map_err(|hata| Hata::Beklenmeyen(format!("Winget listesi JSON üretilemedi: {hata}")))
}

pub fn onerileri_sirala(eski: &ProfilPaketi, adaylar: &[AramaSonucu]) -> Vec<PaketOnerisi> {
    let eski_ad = normalize(eski.ad.as_deref().unwrap_or(&eski.id));
    let eski_id = normalize(&eski.id);
    let mut sonuc = adaylar
        .iter()
        .filter(|aday| aday.id != eski.id)
        .map(|aday| {
            let ad_puan = jaro_winkler(&eski_ad, &normalize(&aday.ad));
            let id_puan = jaro_winkler(&eski_id, &normalize(&aday.id));
            let puan = ad_puan * 0.7 + id_puan * 0.3;
            let guven = if puan >= 0.90 {
                Guven::Yuksek
            } else if puan >= 0.75 {
                Guven::Orta
            } else {
                Guven::Dusuk
            };
            PaketOnerisi {
                id: aday.id.clone(),
                ad: aday.ad.clone(),
                surum: aday.surum.clone(),
                kaynak: aday.kaynak.clone(),
                guven,
                puan,
            }
        })
        .collect::<Vec<_>>();
    sonuc.sort_by(|a, b| b.puan.partial_cmp(&a.puan).unwrap_or(Ordering::Equal));
    sonuc.truncate(3);
    sonuc
}

fn normalize(metin: &str) -> String {
    metin
        .to_lowercase()
        .chars()
        .map(|karakter| {
            if karakter.is_alphanumeric() {
                karakter
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn profil_dogrula(profil: &Profil) -> Sonuc<()> {
    if profil.id.trim().is_empty() {
        return Err(Hata::Beklenmeyen("Profil kimliği boş olamaz.".into()));
    }
    if profil.ad.trim().is_empty() {
        return Err(Hata::Beklenmeyen(format!(
            "{}: profil adı boş olamaz.",
            profil.id
        )));
    }
    if profil.paket.is_empty() {
        return Err(Hata::Beklenmeyen(format!(
            "{}: profil en az bir paket içermeli.",
            profil.id
        )));
    }
    if profil.paket.len() > 500 {
        return Err(Hata::Beklenmeyen(format!(
            "{}: profil 500 paket sınırını aşıyor.",
            profil.id
        )));
    }
    let mut kimlikler = HashSet::new();
    for paket in &profil.paket {
        paket_kimligi_dogrula(&paket.id)?;
        if !kimlikler.insert(paket.id.as_str()) {
            return Err(Hata::Beklenmeyen(format!(
                "{}: yinelenen paket kimliği: {}",
                profil.id, paket.id
            )));
        }
        for eski in &paket.replaces {
            paket_kimligi_dogrula(eski)?;
        }
    }
    Ok(())
}

#[derive(RustEmbed)]
#[folder = "../../profiles"]
struct ProfilVarliklari;

pub fn gomulu_profilleri_yukle() -> Sonuc<Vec<Profil>> {
    let mut profiller = Vec::new();
    for dosya in ProfilVarliklari::iter() {
        if !dosya.ends_with(".toml") {
            continue;
        }
        let icerik =
            ProfilVarliklari::get(&dosya).ok_or_else(|| Hata::KuralOkunamadi(dosya.to_string()))?;
        let metin = std::str::from_utf8(&icerik.data)
            .map_err(|hata| Hata::KuralOkunamadi(format!("{dosya}: {hata}")))?;
        let profil: Profil = toml::from_str(metin)
            .map_err(|hata| Hata::KuralGecersiz(format!("{dosya}: {hata}")))?;
        profil_dogrula(&profil)?;
        profiller.push(profil);
    }
    profiller.sort_by(|a, b| a.ad.cmp(&b.ad));
    Ok(profiller)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kimlik_arti_isaretini_kabul_eder() {
        assert!(paket_kimligi_gecerli("Microsoft.VCRedist.2015+.x64"));
        assert!(paket_kimligi_gecerli("Git.Git"));
        assert!(!paket_kimligi_gecerli("kötü paket"));
        assert!(!paket_kimligi_gecerli("a;rm"));
    }

    #[test]
    fn ihracat_json_ayristirir() {
        let metin = r#"{"Sources":[{"Packages":[{"PackageIdentifier":"Git.Git","Version":"2.56.0"}],"SourceDetails":{"Name":"winget"}}]}"#;
        let paketler = ihracat_json_ayristir(metin).unwrap();
        assert_eq!(paketler.len(), 1);
        assert_eq!(paketler[0].id, "Git.Git");
        assert_eq!(paketler[0].kaynak.as_deref(), Some("winget"));
    }

    #[test]
    fn guncelleme_tablosunu_ayristirir() {
        let metin = "Name                          Id                     Version     Available   Source\n-------------------------------------------------------------------------------\nBlender                       BlenderFoundation.Blender 5.1.2       5.2.3       winget\nPython Launcher               Python.Launcher         < 3.14.7    3.14.7      winget\n23 upgrades available.\n";
        let satirlar = guncelleme_tablosu_ayristir(metin).unwrap();
        assert_eq!(satirlar.len(), 2);
        assert_eq!(satirlar[0].id, "BlenderFoundation.Blender");
        assert_eq!(satirlar[0].mevcut_surum, "5.2.3");
        assert_eq!(satirlar[1].id, "Python.Launcher");
        assert_eq!(satirlar[1].kurulu_surum, "< 3.14.7");
    }

    #[test]
    fn pin_tablosunu_ayristirir() {
        let metin = "PackageIdentifier       SourceIdentifier       Version        PinningType\n------------------------------------------------------------------------\nGit.Git                 winget                 2.56.0         Blocking\n";
        let pinler = pin_tablosu_ayristir(metin).unwrap();
        assert_eq!(pinler.len(), 1);
        assert_eq!(pinler[0].tur, PinTuru::Blocking);
        assert_eq!(pinler[0].surum.as_deref(), Some("2.56.0"));
    }

    #[test]
    fn arama_tablosunu_ayristirir() {
        let metin = "Name                  Id                     Version   Source\n----------------------------------------------------------------\nGit                   Git.Git                2.56.0    winget\n";
        let sonuclar = arama_tablosu_ayristir(metin).unwrap();
        assert_eq!(sonuclar.len(), 1);
        assert_eq!(sonuclar[0].id, "Git.Git");
    }

    #[test]
    fn kaynak_tablosunu_ayristirir() {
        let metin = "Name        Argument                                      Explicit\n------------------------------------------------------------------\nmsstore     https://storeedgefd.dsx.mp.microsoft.com/v9.0 false\nwinget      https://cdn.winget.microsoft.com/cache        false\nwinget-font https://cdn.winget.microsoft.com/fonts        true\n";
        let kaynaklar = kaynak_tablosu_ayristir(metin).unwrap();
        assert_eq!(kaynaklar.len(), 3);
        assert_eq!(kaynaklar[0].ad, "msstore");
        assert_eq!(
            kaynaklar[0].adres,
            "https://storeedgefd.dsx.mp.microsoft.com/v9.0"
        );
        assert!(!kaynaklar[0].acik);
        assert!(kaynaklar[2].acik);
        assert_eq!(kaynaklar[2].ad, "winget-font");
    }

    #[test]
    fn bozuk_kaynak_satirlarini_atlar() {
        let metin = "Ad Argument Durum\n---\nboş\nhttps://yalnız.adres\nkötü ad https://ornek.test true\ngecerli https://ornek.test/api TRUE\ngecerli4 https://ornek.test ekstra\ngecerli2 ftp://ornek.test false\ngercerli3 https://ornek.test false\n";
        let kaynaklar = kaynak_tablosu_ayristir(metin).unwrap();
        assert_eq!(kaynaklar.len(), 2);
        assert_eq!(kaynaklar[0].ad, "gecerli");
        assert!(kaynaklar[0].acik);
        assert_eq!(kaynaklar[1].ad, "gercerli3");
        assert!(!kaynaklar[1].acik);
    }

    #[test]
    fn kaynak_adi_ve_adresi_dogrulanir() {
        assert!(kaynak_adi_gecerli_mi("winget"));
        assert!(kaynak_adi_gecerli_mi("winget-font"));
        assert!(kaynak_adi_gecerli_mi("a.b_c-1"));
        assert!(!kaynak_adi_gecerli_mi("-winget"));
        assert!(!kaynak_adi_gecerli_mi("kötü ad"));
        assert!(!kaynak_adi_gecerli_mi(""));

        assert!(kaynak_adresi_gecerli_mi(
            "https://cdn.winget.microsoft.com/cache"
        ));
        assert!(kaynak_adresi_gecerli_mi(
            "https://storeedgefd.dsx.mp.microsoft.com/v9.0"
        ));
        assert!(!kaynak_adresi_gecerli_mi("http://ornek.test"));
        assert!(!kaynak_adresi_gecerli_mi("file:///C:/kaynak"));
        assert!(!kaynak_adresi_gecerli_mi("https://"));
        assert!(!kaynak_adresi_gecerli_mi("https://ornek.test/yol x"));
        assert!(!kaynak_adresi_gecerli_mi("https://kullanici@ornek.test"));
        assert!(!kaynak_adresi_gecerli_mi("https://ornek.test\\yol"));
    }

    #[test]
    fn liste_metnini_ayristirir() {
        let metin = "\u{feff}# yorum\n\nName Argument Explicit\n---\nGit.Git\nGit.Git\n; ikinci yorum\nMicrosoft.VCRedist.2015+.x64\n";
        let idler = liste_metni_ayristir(metin).unwrap();
        assert_eq!(idler, vec!["Git.Git", "Microsoft.VCRedist.2015+.x64"]);
    }

    #[test]
    fn cok_alanli_liste_satirini_reddeder() {
        let hata = liste_metni_ayristir("Git.Git\nGit Git.Git 2.56.0 winget\n").unwrap_err();
        assert!(hata.to_string().contains("Satır 2"));
    }

    #[test]
    fn gecersiz_liste_kimligini_reddeder() {
        assert!(liste_metni_ayristir("kötü paket").is_err());
    }

    #[test]
    fn liste_json_olusturur_ve_geri_okur() {
        let idler = vec!["Git.Git".to_string(), "Git.Git".to_string()];
        let json = liste_json_olustur(&idler).unwrap();
        assert!(json.contains("\"PackageIdentifier\": \"Git.Git\""));
        assert!(json.contains("\"Name\": \"winget\""));
        let paketler = ihracat_json_ayristir(&json).unwrap();
        assert_eq!(paketler.len(), 1);
        assert_eq!(paketler[0].id, "Git.Git");
        assert_eq!(paketler[0].kaynak.as_deref(), Some("winget"));
    }

    #[test]
    fn bos_liste_json_uretmez() {
        assert!(liste_json_olustur(&[]).is_err());
    }

    #[test]
    fn kaynaksiz_arama_tablosunu_ayristirir() {
        let metin = "Name      Id             Version\n-----------------------------------\nGit       Git.Git        2.56.0.3\ngit-crypt AGWA.git-crypt 0.7.0\n";
        let sonuclar = arama_tablosu_ayristir(metin).unwrap();
        assert_eq!(sonuclar.len(), 2);
        assert_eq!(sonuclar[0].id, "Git.Git");
        assert_eq!(sonuclar[0].surum.as_deref(), Some("2.56.0.3"));
        assert_eq!(sonuclar[0].kaynak, None);
    }

    #[test]
    fn benzer_paket_onerir() {
        let eski = ProfilPaketi {
            id: "Git.Git".into(),
            ad: Some("Git".into()),
            yayinci: None,
            surum: None,
            replaces: vec![],
        };
        let adaylar = vec![
            AramaSonucu {
                id: "Git.Git".into(),
                ad: "Git".into(),
                surum: Some("2.56".into()),
                kaynak: Some("winget".into()),
            },
            AramaSonucu {
                id: "GitHub.cli".into(),
                ad: "GitHub CLI".into(),
                surum: Some("2.0".into()),
                kaynak: Some("winget".into()),
            },
        ];
        let oneriler = onerileri_sirala(&eski, &adaylar);
        assert_eq!(oneriler.len(), 1);
        assert_eq!(oneriler[0].id, "GitHub.cli");
    }
}
