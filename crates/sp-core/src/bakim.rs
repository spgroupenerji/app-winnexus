use crate::error::{Hata, Sonuc};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppxPaketi {
    pub ad: String,
    pub tam_ad: String,
    pub surum: String,
    pub yayinci: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OzellikOgesi {
    pub ad: String,
    pub durum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurucuOgesi {
    pub ad: String,
    pub yayinci: String,
    pub sinif: String,
    pub surum: String,
    pub tarih: String,
    pub inf: String,
}

pub fn appxleri_ayristir(json: &str) -> Sonuc<Vec<AppxPaketi>> {
    Ok(ogeler(json, "Appx")?
        .iter()
        .map(|oge| AppxPaketi {
            ad: alan(oge, &["Ad", "Name"]),
            tam_ad: alan(oge, &["TamAd", "PackageFullName"]),
            surum: alan(oge, &["Surum", "Version"]),
            yayinci: alan(oge, &["Yayinci", "Publisher"]),
        })
        .collect())
}

pub fn ozellikleri_ayristir(json: &str) -> Sonuc<Vec<OzellikOgesi>> {
    Ok(ogeler(json, "Özellik")?
        .iter()
        .map(|oge| OzellikOgesi {
            ad: alan(oge, &["Ad", "FeatureName"]),
            durum: alan(oge, &["Durum", "State"]),
        })
        .collect())
}

pub fn suruculeri_ayristir(json: &str) -> Sonuc<Vec<SurucuOgesi>> {
    Ok(ogeler(json, "Sürücü")?
        .iter()
        .map(|oge| SurucuOgesi {
            ad: alan(oge, &["Ad", "Driver"]),
            yayinci: alan(oge, &["Yayinci", "ProviderName"]),
            sinif: alan(oge, &["Sinif", "ClassName"]),
            surum: alan(oge, &["Surum", "Version"]),
            tarih: alan(oge, &["Tarih", "Date"]),
            inf: alan(oge, &["Inf", "Driver"]),
        })
        .collect())
}

fn ogeler(json: &str, etiket: &str) -> Sonuc<Vec<Value>> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    let deger: Value = serde_json::from_str(json)
        .map_err(|hata| Hata::Beklenmeyen(format!("{etiket} JSON çıktısı okunamadı: {hata}")))?;
    if deger.is_null() {
        return Ok(Vec::new());
    }
    if let Value::Array(liste) = deger {
        return Ok(liste);
    }
    if deger.is_object() {
        return Ok(vec![deger]);
    }
    Err(Hata::Beklenmeyen(format!(
        "{etiket} JSON çıktısı beklenen biçimde değil."
    )))
}

fn alan(oge: &Value, adlar: &[&str]) -> String {
    adlar
        .iter()
        .find_map(|ad| oge.get(*ad))
        .map(deger_metni)
        .unwrap_or_default()
}

fn deger_metni(deger: &Value) -> String {
    match deger {
        Value::String(metin) => metin.trim().to_string(),
        Value::Number(sayi) => sayi.to_string(),
        Value::Bool(dogru) => dogru.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn tek_appx_nesnesi_ayristirilir() {
        let json = r#"{"Ad":"Hesap Makinesi","TamAd":"Microsoft.WindowsCalculator_11.0_x64__8wekyb3d8bbwe","Surum":"11.0.0.0","Yayinci":"CN=Microsoft"}"#;
        let paketler = appxleri_ayristir(json).unwrap();
        assert_eq!(
            paketler,
            vec![AppxPaketi {
                ad: "Hesap Makinesi".into(),
                tam_ad: "Microsoft.WindowsCalculator_11.0_x64__8wekyb3d8bbwe".into(),
                surum: "11.0.0.0".into(),
                yayinci: "CN=Microsoft".into(),
            }]
        );
    }

    #[test]
    fn appx_dizisi_ayristirilir_ve_eksik_alan_bos_olur() {
        let json = r#"[{"Ad":"A","TamAd":"A_1.0_x64__8wekyb3d8bbwe"},{"Ad":"B","TamAd":"B_1.0_x64__8wekyb3d8bbwe","Surum":"1.0","Yayinci":"X"}]"#;
        let paketler = appxleri_ayristir(json).unwrap();
        assert_eq!(paketler.len(), 2);
        assert_eq!(paketler[0].surum, "");
        assert_eq!(paketler[1].surum, "1.0");
    }

    #[test]
    fn tek_ozellik_nesnesi_ayristirilir() {
        let json = r#"{"Ad":"Microsoft-Windows-Subsystem-Linux","Durum":"Enabled"}"#;
        let ozellikler = ozellikleri_ayristir(json).unwrap();
        assert_eq!(
            ozellikler,
            vec![OzellikOgesi {
                ad: "Microsoft-Windows-Subsystem-Linux".into(),
                durum: "Enabled".into(),
            }]
        );
    }

    #[test]
    fn ozellik_dizisi_durum_metniyle_ayristirilir() {
        let json = r#"[{"Ad":"A","Durum":"Disabled"},{"Ad":"B","Durum":"Enabled"}]"#;
        let ozellikler = ozellikleri_ayristir(json).unwrap();
        assert_eq!(ozellikler.len(), 2);
        assert_eq!(ozellikler[0].durum, "Disabled");
        assert_eq!(ozellikler[1].ad, "B");
    }

    #[test]
    fn surucu_dizisi_ayristirilir() {
        let json = r#"[{"Ad":"oem12.inf","Yayinci":"Contoso","Sinif":"Net","Surum":"1.2.3.4","Tarih":"2026-01-05","Inf":"oem12.inf"},{"Ad":"oem13.inf","Yayinci":"Fabrikam","Sinif":"Display","Surum":"2.0.0.0","Tarih":"2025-12-31","Inf":"oem13.inf"}]"#;
        let suruculer = suruculeri_ayristir(json).unwrap();
        assert_eq!(suruculer.len(), 2);
        assert_eq!(
            suruculer[0],
            SurucuOgesi {
                ad: "oem12.inf".into(),
                yayinci: "Contoso".into(),
                sinif: "Net".into(),
                surum: "1.2.3.4".into(),
                tarih: "2026-01-05".into(),
                inf: "oem12.inf".into(),
            }
        );
    }

    #[test]
    fn tek_surucu_nesnesi_ayristirilir() {
        let json = r#"{"Ad":"oem7.inf","Yayinci":"Contoso","Sinif":"USB","Surum":"3.1.0.0","Tarih":"2026-02-10","Inf":"oem7.inf"}"#;
        let suruculer = suruculeri_ayristir(json).unwrap();
        assert_eq!(suruculer.len(), 1);
        assert_eq!(suruculer[0].sinif, "USB");
        assert_eq!(suruculer[0].tarih, "2026-02-10");
    }

    #[test]
    fn bos_ve_gecersiz_json_islenir() {
        assert!(appxleri_ayristir("").unwrap().is_empty());
        assert!(appxleri_ayristir("null").unwrap().is_empty());
        assert!(suruculeri_ayristir("{bozuk").is_err());
        assert!(ozellikleri_ayristir("42").is_err());
    }
}
