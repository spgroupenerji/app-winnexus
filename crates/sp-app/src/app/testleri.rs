use super::{surum_etiketi, Bolum, Sayfa};

#[test]
fn bolum_kapsami_tam() {
    let mut gorulen = Vec::new();
    for bolum in Bolum::HEPSI {
        let sayfalar = bolum.sayfalar();
        assert!(
            !sayfalar.is_empty(),
            "{} bölümünde sayfa yok",
            bolum.baslik()
        );
        for sayfa in sayfalar {
            assert_eq!(
                sayfa.bolum(),
                Some(bolum),
                "{} yanlış bölümde",
                sayfa.baslik()
            );
            assert!(
                !gorulen.contains(sayfa),
                "{} birden fazla bölümde",
                sayfa.baslik()
            );
            gorulen.push(*sayfa);
        }
    }
    for sayfa in Sayfa::HEPSI {
        if sayfa == Sayfa::Ayarlar {
            continue;
        }
        assert!(
            gorulen.contains(&sayfa),
            "{} hiçbir bölümde yok",
            sayfa.baslik()
        );
    }
    assert_eq!(gorulen.len(), Sayfa::HEPSI.len() - 1);
}

#[test]
fn sayfa_kimlikleri_benzersiz_ve_ascii() {
    let mut kimlikler = Vec::new();
    for sayfa in Sayfa::HEPSI {
        assert!(
            !kimlikler.contains(&sayfa.kimlik()),
            "yinelenen kimlik: {}",
            sayfa.kimlik()
        );
        kimlikler.push(sayfa.kimlik());
        assert!(
            sayfa.kimlik().is_ascii(),
            "ASCII olmayan kimlik: {}",
            sayfa.kimlik()
        );
        assert!(!sayfa.baslik().trim().is_empty());
        assert!(!sayfa.aciklama().trim().is_empty());
    }
}

#[test]
fn surum_beta_etiketi() {
    assert!(surum_etiketi().starts_with("beta v"));
}

#[test]
fn arama_gerektiren_arac_sayfalari() {
    for sayfa in [
        Sayfa::WindowsGuncelleme,
        Sayfa::TarayiciUzantilari,
        Sayfa::DosyaKayitBulucu,
        Sayfa::ZorlaKaldir,
        Sayfa::BaglantiYoneticisi,
    ] {
        assert!(
            sayfa.arac_aramasi_gerekli(),
            "{} arama girdisi istemeli",
            sayfa.baslik()
        );
    }
    for sayfa in [
        Sayfa::KurulumIzleme,
        Sayfa::DiskAnalisti,
        Sayfa::Temizlik,
        Sayfa::Bakim,
    ] {
        assert!(
            !sayfa.arac_aramasi_gerekli(),
            "{} arama girdisi istememeli",
            sayfa.baslik()
        );
    }
}
