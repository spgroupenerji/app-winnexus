use crate::adaptor::{AdaptorRef, SistemBilgisi};
use crate::error::{Hata, Sonuc};
use crate::gunluk::{Gunluk, GunlukKaydi};
use crate::model::{Dogrulama, Eylem, Kural, KuralDurumu, RegistryVeri, Risk};
use crate::os::OsKosulu;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct PlanAdimi {
    pub aciklama: String,
    pub eylem: Eylem,
    pub onceki: Option<RegistryVeri>,
}

#[derive(Debug, Clone)]
pub struct IsPlani {
    pub kural_id: String,
    pub baslik: String,
    pub risk: Risk,
    pub adimlar: Vec<PlanAdimi>,
    pub dogrulama: Vec<Dogrulama>,
    pub yeniden_baslatma: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IsSonucu {
    pub uygulanan: usize,
}

pub struct IsMotoru {
    adaptor: AdaptorRef,
    gunluk: Arc<Gunluk>,
    sistem: SistemBilgisi,
}

impl IsMotoru {
    pub fn yeni(adaptor: AdaptorRef, gunluk: Arc<Gunluk>, sistem: SistemBilgisi) -> Self {
        Self {
            adaptor,
            gunluk,
            sistem,
        }
    }

    pub fn sistem(&self) -> &SistemBilgisi {
        &self.sistem
    }

    fn os_uygun(&self, kural: &Kural) -> bool {
        match &kural.os {
            Some(ifade) => OsKosulu::ayristir(ifade)
                .map(|kosul| kosul.uygun(&self.sistem.surum))
                .unwrap_or(false),
            None => true,
        }
    }

    pub fn durum(&self, kural: &Kural) -> KuralDurumu {
        if !self.os_uygun(kural) {
            return KuralDurumu::Desteklenmiyor;
        }
        if kural.dogrula.is_empty() {
            return KuralDurumu::Bilinmiyor;
        }
        let mut uyan = 0usize;
        for dogrulama in &kural.dogrula {
            match self.adaptor.registry_oku(&dogrulama.konum()) {
                Ok(deger) => {
                    if deger.as_ref() == dogrulama.beklenen() {
                        uyan += 1;
                    }
                }
                Err(_) => return KuralDurumu::Bilinmiyor,
            }
        }
        if uyan == kural.dogrula.len() {
            KuralDurumu::Uygulandi
        } else if uyan == 0 {
            KuralDurumu::Uygulanmadi
        } else {
            KuralDurumu::Kismi
        }
    }

    pub fn onizle(&self, kural: &Kural) -> Sonuc<IsPlani> {
        if !self.os_uygun(kural) {
            return Err(Hata::Desteklenmiyor(format!(
                "{} bu Windows sürümünde desteklenmiyor.",
                kural.baslik
            )));
        }
        if kural.yonetici && !self.sistem.yonetici {
            return Err(Hata::Yetki(format!(
                "{} için yönetici izni gerekiyor.",
                kural.baslik
            )));
        }
        let mut adimlar = Vec::with_capacity(kural.uygula.len());
        for eylem in &kural.uygula {
            let onceki = self.adaptor.registry_oku(&eylem.konum())?;
            adimlar.push(PlanAdimi {
                aciklama: eylem.aciklama(),
                eylem: eylem.clone(),
                onceki,
            });
        }
        Ok(IsPlani {
            kural_id: kural.id.clone(),
            baslik: kural.baslik.clone(),
            risk: kural.risk,
            adimlar,
            dogrulama: kural.dogrula.clone(),
            yeniden_baslatma: kural.yeniden_baslatma,
        })
    }

    pub fn uygula(&self, plan: &IsPlani) -> Sonuc<IsSonucu> {
        let is_id = format!(
            "{}-{}",
            plan.kural_id,
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        );
        let mut uygulanan = Vec::new();
        for (adim_no, adim) in plan.adimlar.iter().enumerate() {
            self.gunluk.yaz(&GunlukKaydi::niyet(
                &is_id,
                &plan.kural_id,
                adim_no,
                &adim.aciklama,
                adim.onceki.as_ref(),
            ))?;
            let konum = adim.eylem.konum();
            let sonuc = match &adim.eylem {
                Eylem::RegistryYaz { veri, .. } => self.adaptor.registry_yaz(&konum, veri),
                Eylem::RegistrySil { .. } => self.adaptor.registry_sil(&konum),
            };
            match sonuc {
                Ok(()) => {
                    uygulanan.push(adim.clone());
                    if let Err(hata) = self.gunluk.yaz(&GunlukKaydi::basarili(
                        &is_id,
                        &plan.kural_id,
                        adim_no,
                        &adim.aciklama,
                        adim.onceki.as_ref(),
                    )) {
                        return Err(self.telafi_hatasi(
                            &is_id,
                            &plan.kural_id,
                            &uygulanan,
                            format!(
                                "{} uygulandı ancak günlüğe yazılamadı: {hata}",
                                adim.aciklama
                            ),
                        ));
                    }
                }
                Err(hata) => {
                    let gunluk_hatasi = self
                        .gunluk
                        .yaz(&GunlukKaydi::hatali(
                            &is_id,
                            &plan.kural_id,
                            adim_no,
                            &adim.aciklama,
                            adim.onceki.as_ref(),
                            &hata.to_string(),
                        ))
                        .err();
                    let mut mesaj = format!("{} uygulanamadı: {hata}", adim.aciklama);
                    if let Some(gunluk_hatasi) = &gunluk_hatasi {
                        mesaj.push_str(&format!("; günlüğe yazılamadı: {gunluk_hatasi}"));
                    }
                    return Err(
                        match self.adimlari_geri_al(&is_id, &plan.kural_id, &uygulanan) {
                            Ok(()) => Hata::Registry(mesaj),
                            Err(geri_alma) => {
                                Hata::Registry(format!("{mesaj}; geri alma başarısız: {geri_alma}"))
                            }
                        },
                    );
                }
            }
        }

        if self.dogrula_plan(plan) != KuralDurumu::Uygulandi {
            let mesaj = format!(
                "{} uygulandı ancak doğrulanamadı; değişiklikler geri alındı.",
                plan.baslik
            );
            return Err(
                match self.adimlari_geri_al(&is_id, &plan.kural_id, &uygulanan) {
                    Ok(()) => Hata::Dogrulama(mesaj),
                    Err(geri_alma) => Hata::Dogrulama(format!(
                        "{} uygulandı ancak doğrulanamadı; geri alma başarısız: {geri_alma}",
                        plan.baslik
                    )),
                },
            );
        }

        Ok(IsSonucu {
            uygulanan: uygulanan.len(),
        })
    }

    fn telafi_hatasi(
        &self,
        is_id: &str,
        kural_id: &str,
        uygulanan: &[PlanAdimi],
        mesaj: String,
    ) -> Hata {
        match self.adimlari_geri_al(is_id, kural_id, uygulanan) {
            Ok(()) => Hata::Gunluk(format!("{mesaj}; değişiklikler geri alındı.")),
            Err(geri_alma) => Hata::GeriAlma(format!("{mesaj}; geri alma başarısız: {geri_alma}")),
        }
    }

    pub fn geri_al(&self, plan: &IsPlani) -> Sonuc<()> {
        let is_id = format!(
            "geri-{}-{}",
            plan.kural_id,
            OffsetDateTime::now_utc().unix_timestamp_nanos()
        );
        self.adimlari_geri_al(&is_id, &plan.kural_id, &plan.adimlar)
    }

    fn dogrula_plan(&self, plan: &IsPlani) -> KuralDurumu {
        if plan.dogrulama.is_empty() {
            return KuralDurumu::Uygulandi;
        }
        let mut uyan = 0usize;
        for dogrulama in &plan.dogrulama {
            match self.adaptor.registry_oku(&dogrulama.konum()) {
                Ok(deger) => {
                    if deger.as_ref() == dogrulama.beklenen() {
                        uyan += 1;
                    }
                }
                Err(_) => return KuralDurumu::Bilinmiyor,
            }
        }
        if uyan == plan.dogrulama.len() {
            KuralDurumu::Uygulandi
        } else if uyan == 0 {
            KuralDurumu::Uygulanmadi
        } else {
            KuralDurumu::Kismi
        }
    }

    fn adimlari_geri_al(&self, is_id: &str, kural_id: &str, adimlar: &[PlanAdimi]) -> Sonuc<()> {
        let mut hatalar = Vec::new();
        for (adim_no, adim) in adimlar.iter().enumerate().rev() {
            let konum = adim.eylem.konum();
            let sonuc = match (&adim.eylem, &adim.onceki) {
                (Eylem::RegistryYaz { .. }, Some(onceki))
                | (Eylem::RegistrySil { .. }, Some(onceki)) => {
                    self.adaptor.registry_yaz(&konum, onceki)
                }
                (Eylem::RegistryYaz { .. }, None) | (Eylem::RegistrySil { .. }, None) => {
                    self.adaptor.registry_sil(&konum)
                }
            };
            match sonuc {
                Ok(()) => {
                    if let Err(hata) = self.gunluk.yaz(&GunlukKaydi::basarili(
                        is_id,
                        kural_id,
                        adim_no,
                        "geri alma",
                        adim.onceki.as_ref(),
                    )) {
                        hatalar.push(format!(
                            "{}: geri alma günlüğü yazılamadı: {hata}",
                            adim.aciklama
                        ));
                    }
                }
                Err(hata) => {
                    if let Err(gunluk_hatasi) = self.gunluk.yaz(&GunlukKaydi::hatali(
                        is_id,
                        kural_id,
                        adim_no,
                        "geri alma",
                        adim.onceki.as_ref(),
                        &hata.to_string(),
                    )) {
                        hatalar.push(format!(
                            "{}: geri alma hatası günlüğe yazılamadı: {gunluk_hatasi}",
                            adim.aciklama
                        ));
                    }
                    hatalar.push(format!("{} geri alınamadı: {hata}", adim.aciklama));
                }
            }
        }
        if hatalar.is_empty() {
            Ok(())
        } else {
            Err(Hata::GeriAlma(hatalar.join(" | ")))
        }
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::adaptor::SistemAdaptoru;
    use crate::model::{Algilama, Kove, RegistryKonum};
    use crate::os::Surum;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct BellekAdaptoru {
        veri: Mutex<HashMap<(Kove, String, String), RegistryVeri>>,
        yonetici: bool,
        hata_verisi: Option<RegistryVeri>,
    }

    impl BellekAdaptoru {
        fn yeni(yonetici: bool, hata_verisi: Option<RegistryVeri>) -> Self {
            Self {
                veri: Mutex::new(HashMap::new()),
                yonetici,
                hata_verisi,
            }
        }
    }

    impl SistemAdaptoru for BellekAdaptoru {
        fn registry_oku(&self, konum: &RegistryKonum) -> Sonuc<Option<RegistryVeri>> {
            Ok(self
                .veri
                .lock()
                .unwrap()
                .get(&(konum.kove, konum.yol.clone(), konum.deger.clone()))
                .cloned())
        }

        fn registry_yaz(&self, konum: &RegistryKonum, veri: &RegistryVeri) -> Sonuc<()> {
            if self.hata_verisi.as_ref() == Some(veri) {
                return Err(Hata::Registry("test hatası".into()));
            }
            self.veri.lock().unwrap().insert(
                (konum.kove, konum.yol.clone(), konum.deger.clone()),
                veri.clone(),
            );
            Ok(())
        }

        fn registry_sil(&self, konum: &RegistryKonum) -> Sonuc<()> {
            self.veri
                .lock()
                .unwrap()
                .remove(&(konum.kove, konum.yol.clone(), konum.deger.clone()));
            Ok(())
        }

        fn yonetici_mi(&self) -> bool {
            self.yonetici
        }
    }

    fn kural() -> Kural {
        Kural {
            id: "test.kural".into(),
            baslik: "Test kuralı".into(),
            ozet: "Test".into(),
            kategori: "gezgin".into(),
            risk: Risk::Dusuk,
            yonetici: false,
            os: Some(">=10.0".into()),
            yeniden_baslatma: false,
            kanit: "docs/kanit/test.md#x".into(),
            algila: vec![Algilama::Registry {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "X".into(),
                gorunum: Some(64),
            }],
            uygula: vec![Eylem::RegistryYaz {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "X".into(),
                gorunum: Some(64),
                veri: RegistryVeri::Dword(1),
            }],
            dogrula: vec![Dogrulama::Registry {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "X".into(),
                gorunum: Some(64),
                beklenen: Some(RegistryVeri::Dword(1)),
            }],
        }
    }

    fn motor(
        yonetici: bool,
        hata_verisi: Option<RegistryVeri>,
    ) -> (IsMotoru, Arc<BellekAdaptoru>, tempfile::TempDir) {
        let dizin = tempfile::tempdir().unwrap();
        let adaptor = Arc::new(BellekAdaptoru::yeni(yonetici, hata_verisi));
        let gunluk = Arc::new(Gunluk::yeni(dizin.path().join("journal.jsonl")));
        let sistem = SistemBilgisi {
            surum: Surum {
                major: 10,
                minor: 0,
                build: 26200,
            },
            yonetici,
        };
        let motor = IsMotoru::yeni(adaptor.clone(), gunluk, sistem);
        (motor, adaptor, dizin)
    }

    #[test]
    fn uygula_ve_geri_al() {
        let (motor, adaptor, _dizin) = motor(true, None);
        let kural = kural();
        assert_eq!(motor.durum(&kural), KuralDurumu::Uygulanmadi);

        let plan = motor.onizle(&kural).unwrap();
        motor.uygula(&plan).unwrap();
        assert_eq!(motor.durum(&kural), KuralDurumu::Uygulandi);

        motor.geri_al(&plan).unwrap();
        assert_eq!(motor.durum(&kural), KuralDurumu::Uygulanmadi);
        assert!(adaptor.veri.lock().unwrap().is_empty());
    }

    #[test]
    fn ikinci_adim_hatasinda_birinci_geri_alinir() {
        let (motor, adaptor, _dizin) = motor(true, Some(RegistryVeri::Dword(2)));
        let mut kural = kural();
        kural.uygula = vec![
            Eylem::RegistryYaz {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "X".into(),
                gorunum: Some(64),
                veri: RegistryVeri::Dword(1),
            },
            Eylem::RegistryYaz {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "Y".into(),
                gorunum: Some(64),
                veri: RegistryVeri::Dword(2),
            },
        ];
        kural.dogrula = vec![];
        let plan = motor.onizle(&kural).unwrap();
        assert!(motor.uygula(&plan).is_err());
        assert!(adaptor.veri.lock().unwrap().is_empty());
    }

    #[test]
    fn dogrulama_basarisizsa_geri_alinir() {
        let (motor, adaptor, _dizin) = motor(true, None);
        let mut kural = kural();
        kural.dogrula = vec![Dogrulama::Registry {
            kove: Kove::Hkcu,
            yol: "Software\\Test".into(),
            deger: "X".into(),
            gorunum: Some(64),
            beklenen: Some(RegistryVeri::Dword(2)),
        }];
        let plan = motor.onizle(&kural).unwrap();
        assert!(motor.uygula(&plan).is_err());
        assert!(adaptor.veri.lock().unwrap().is_empty());
    }

    #[test]
    fn yonetici_yoksa_onizle_yetki_hatasi() {
        let (motor, _adaptor, _dizin) = motor(false, None);
        let mut kural = kural();
        kural.yonetici = true;
        assert!(matches!(motor.onizle(&kural), Err(Hata::Yetki(_))));
    }

    #[test]
    fn os_desteklenmiyorsa_durum_desteklenmiyor() {
        let (motor, _adaptor, _dizin) = motor(true, None);
        let mut kural = kural();
        kural.os = Some(">=11.0".into());
        assert_eq!(motor.durum(&kural), KuralDurumu::Desteklenmiyor);
    }

    #[test]
    fn geri_alma_basarisizsa_diger_adimlar_denenir() {
        let (motor, adaptor, _dizin) = motor(true, Some(RegistryVeri::Dword(9)));
        adaptor.veri.lock().unwrap().insert(
            (Kove::Hkcu, "Software\\Test".to_string(), "Y".to_string()),
            RegistryVeri::Dword(9),
        );
        let mut kural = kural();
        kural.uygula = vec![
            Eylem::RegistryYaz {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "X".into(),
                gorunum: Some(64),
                veri: RegistryVeri::Dword(1),
            },
            Eylem::RegistryYaz {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "Y".into(),
                gorunum: Some(64),
                veri: RegistryVeri::Dword(2),
            },
            Eylem::RegistryYaz {
                kove: Kove::Hkcu,
                yol: "Software\\Test".into(),
                deger: "Z".into(),
                gorunum: Some(64),
                veri: RegistryVeri::Dword(9),
            },
        ];
        kural.dogrula = vec![];
        let plan = motor.onizle(&kural).unwrap();
        let hata = motor.uygula(&plan).unwrap_err();
        assert!(hata.to_string().contains("geri alma başarısız"));
        let veri = adaptor.veri.lock().unwrap();
        assert!(!veri.contains_key(&(Kove::Hkcu, "Software\\Test".to_string(), "X".to_string())));
        assert_eq!(
            veri.get(&(Kove::Hkcu, "Software\\Test".to_string(), "Y".to_string())),
            Some(&RegistryVeri::Dword(2))
        );
    }

    #[test]
    fn gunluk_yazilamazsa_uygulama_durur() {
        let dizin = tempfile::tempdir().unwrap();
        std::fs::create_dir(dizin.path().join("journal.jsonl")).unwrap();
        let adaptor = Arc::new(BellekAdaptoru::yeni(true, None));
        let gunluk = Arc::new(Gunluk::yeni(dizin.path().join("journal.jsonl")));
        let sistem = SistemBilgisi {
            surum: Surum {
                major: 10,
                minor: 0,
                build: 26200,
            },
            yonetici: true,
        };
        let motor = IsMotoru::yeni(adaptor.clone(), gunluk, sistem);
        let plan = motor.onizle(&kural()).unwrap();
        assert!(matches!(motor.uygula(&plan), Err(Hata::Gunluk(_))));
        assert!(adaptor.veri.lock().unwrap().is_empty());
    }
}
