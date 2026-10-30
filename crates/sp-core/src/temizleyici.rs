use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemizlikHedefi {
    pub kimlik: String,
    pub ad: String,
    pub yol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemizlikTarama {
    pub dosya_sayisi: u64,
    pub bayt: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TemizlikSonucu {
    pub silinen_dosya: u64,
    pub silinen_bayt: u64,
    pub hatalar: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YinelenenGrup {
    pub boyut: u64,
    pub dosyalar: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KarantinaSonucu {
    pub tasinan: u64,
    pub hatalar: Vec<String>,
}

pub fn yol_kok_altinda_mi(kok: &Path, yol: &Path) -> bool {
    let kok = bilesenler(kok);
    let yol = bilesenler(yol);
    !kok.is_empty()
        && kok.len() <= yol.len()
        && kok
            .iter()
            .zip(yol.iter())
            .all(|(kok_bileseni, yol_bileseni)| kok_bileseni == yol_bileseni)
}

fn bilesenler(yol: &Path) -> Vec<String> {
    yol.components()
        .map(|bilesen| bilesen.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn kok_altindaki_yol_dogru_bulunur() {
        let kok = Path::new("veri/gecici");
        assert!(yol_kok_altinda_mi(kok, Path::new("veri/gecici")));
        assert!(yol_kok_altinda_mi(kok, Path::new("veri/gecici/dosya.txt")));
        assert!(yol_kok_altinda_mi(
            kok,
            Path::new("VERI/Gecici/alt/dosya.txt")
        ));
        assert!(!yol_kok_altinda_mi(
            kok,
            Path::new("veri/gecici2/dosya.txt")
        ));
        assert!(!yol_kok_altinda_mi(kok, Path::new("veri")));
        assert!(!yol_kok_altinda_mi(Path::new(""), Path::new("veri")));
    }
}
