use crate::error::{Hata, Sonuc};
use crate::model::{Kural, KuralDosyasi};
use crate::validation::kurallari_dogrula;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../rules/tweaks"]
struct KuralVarliklari;

pub fn gomulu_kurallari_yukle() -> Sonuc<Vec<Kural>> {
    let mut kurallar = Vec::new();
    for dosya in KuralVarliklari::iter() {
        if !dosya.ends_with(".toml") {
            continue;
        }
        let icerik =
            KuralVarliklari::get(&dosya).ok_or_else(|| Hata::KuralOkunamadi(dosya.to_string()))?;
        let metin = std::str::from_utf8(&icerik.data)
            .map_err(|e| Hata::KuralOkunamadi(format!("{dosya}: {e}")))?;
        let paket: KuralDosyasi =
            toml::from_str(metin).map_err(|e| Hata::KuralGecersiz(format!("{dosya}: {e}")))?;
        kurallar.extend(paket.rule);
    }
    kurallari_dogrula(&kurallar)?;
    Ok(kurallar)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn gomulu_kurallar_yuklenir() {
        let kurallar = gomulu_kurallari_yukle().unwrap();
        assert!(kurallar.len() >= 40);
        assert!(kurallar.len() <= 60);
        let mut kimlikler = std::collections::HashSet::new();
        for kural in &kurallar {
            assert!(kimlikler.insert(kural.id.clone()));
        }
    }
}
