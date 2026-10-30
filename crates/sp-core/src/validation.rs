use crate::error::{Hata, Sonuc};
use crate::model::{Eylem, Kove, Kural, RegistryVeri};
use crate::os::OsKosulu;
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

const KATEGORILER: &[&str] = &[
    "gizlilik",
    "gezgin",
    "gorev_cubugu",
    "performans",
    "ag",
    "guncelleme",
    "sistem",
    "oyun",
    "depolama",
    "baglam_menusu",
];

fn kimlik_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9_]+(\.[a-z0-9_]+)+$").expect("sabit kimlik regex'i"))
}

pub fn kural_dogrula(kural: &Kural) -> Sonuc<()> {
    let hata = |mesaj: String| Hata::KuralGecersiz(format!("{}: {mesaj}", kural.id));

    if !kimlik_regex().is_match(&kural.id) {
        return Err(hata("kimlik biçimi geçersiz".into()));
    }
    if kural.baslik.trim().is_empty() {
        return Err(hata("başlık boş olamaz".into()));
    }
    if kural.ozet.trim().is_empty() {
        return Err(hata("özet boş olamaz".into()));
    }
    if !KATEGORILER.contains(&kural.kategori.as_str()) {
        return Err(hata(format!("bilinmeyen kategori: {}", kural.kategori)));
    }
    if !kural.kanit.starts_with("docs/kanit/") {
        return Err(hata("kanıt yolu docs/kanit/ ile başlamalı".into()));
    }
    if kural.algila.is_empty() {
        return Err(hata("en az bir algılama adımı gerekli".into()));
    }
    if kural.uygula.is_empty() {
        return Err(hata("en az bir uygulama adımı gerekli".into()));
    }
    if kural.dogrula.is_empty() {
        return Err(hata("en az bir doğrulama adımı gerekli".into()));
    }
    if let Some(os) = &kural.os {
        OsKosulu::ayristir(os).map_err(|e| hata(e.to_string()))?;
    }

    for algilama in &kural.algila {
        let konum = algilama.konum();
        registry_konum_dogrula(konum.kove, &konum.yol, &konum.deger, konum.gorunum)
            .map_err(|e| hata(e.to_string()))?;
    }
    for eylem in &kural.uygula {
        let konum = eylem.konum();
        registry_konum_dogrula(konum.kove, &konum.yol, &konum.deger, konum.gorunum)
            .map_err(|e| hata(e.to_string()))?;
        if let Eylem::RegistryYaz { veri, .. } = eylem {
            registry_veri_dogrula(veri).map_err(|e| hata(e.to_string()))?;
        }
    }
    for dogrulama in &kural.dogrula {
        let konum = dogrulama.konum();
        registry_konum_dogrula(konum.kove, &konum.yol, &konum.deger, konum.gorunum)
            .map_err(|e| hata(e.to_string()))?;
        if let Some(veri) = dogrulama.beklenen() {
            registry_veri_dogrula(veri).map_err(|e| hata(e.to_string()))?;
        }
    }

    Ok(())
}

pub fn kurallari_dogrula(kurallar: &[Kural]) -> Sonuc<()> {
    let mut kimlikler = HashSet::new();
    for kural in kurallar {
        if !kimlikler.insert(kural.id.as_str()) {
            return Err(Hata::KuralGecersiz(format!(
                "yinelenen kural kimliği: {}",
                kural.id
            )));
        }
        kural_dogrula(kural)?;
    }
    Ok(())
}

fn registry_konum_dogrula(_kove: Kove, yol: &str, deger: &str, gorunum: Option<u32>) -> Sonuc<()> {
    if yol.trim().is_empty() {
        return Err(Hata::KuralGecersiz("registry yolu boş olamaz".into()));
    }
    if deger.trim().is_empty() {
        return Err(Hata::KuralGecersiz("registry değer adı boş olamaz".into()));
    }
    if yol.len() > 512 {
        return Err(Hata::KuralGecersiz("registry yolu çok uzun".into()));
    }
    if yol.contains("..") || yol.contains('\0') || deger.contains('\0') {
        return Err(Hata::KuralGecersiz(
            "registry yolunda geçersiz bileşen var".into(),
        ));
    }
    if yol.starts_with('\\') || yol.ends_with('\\') {
        return Err(Hata::KuralGecersiz(
            "registry yolu başında veya sonunda ters eğik çizgi olamaz".into(),
        ));
    }
    if !matches!(gorunum, None | Some(32) | Some(64)) {
        return Err(Hata::KuralGecersiz(
            "gorunum yalnız 32 veya 64 olabilir".into(),
        ));
    }
    Ok(())
}

fn registry_veri_dogrula(veri: &RegistryVeri) -> Sonuc<()> {
    match veri {
        RegistryVeri::Sz(deger) | RegistryVeri::GenisSz(deger) if deger.is_empty() => Err(
            Hata::KuralGecersiz("metin registry değeri boş olamaz".into()),
        ),
        RegistryVeri::Binary(deger) if deger.is_empty() => Err(Hata::KuralGecersiz(
            "ikili registry değeri boş olamaz".into(),
        )),
        _ => Ok(()),
    }
}
