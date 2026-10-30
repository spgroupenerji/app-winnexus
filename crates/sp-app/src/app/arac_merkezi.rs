use super::*;

#[derive(Clone, Copy)]
enum AracHedef {
    Sayfa(Sayfa),
    Panel(MerkezAraci),
}

#[derive(Clone, Copy)]
struct AracKaydi {
    ad: &'static str,
    aciklama: &'static str,
    hedef: AracHedef,
}

const KALDIRMA_ARACLARI: [AracKaydi; 6] = [
    AracKaydi {
        ad: "Zorla kaldır",
        aciklama: "Kaldırma komutu çalışmayan uygulamaların klasörünü karantinaya alır ve kayıt anahtarını yedekleyerek siler.",
        hedef: AracHedef::Panel(MerkezAraci::ZorlaKaldir),
    },
    AracKaydi {
        ad: "Kurulum İzleme",
        aciklama: "Kurulum öncesi ve sonrası dosya/kayıt anlık görüntüsünü karşılaştırarak değişiklikleri raporlar.",
        hedef: AracHedef::Panel(MerkezAraci::KurulumIzleme),
    },
    AracKaydi {
        ad: "Windows Mağazası Uygulama Yöneticisi",
        aciklama: "Store uygulamalarını listeler ve seçilenleri kaldırır.",
        hedef: AracHedef::Sayfa(Sayfa::Kaldirici),
    },
    AracKaydi {
        ad: "Program Bileşen Yöneticisi",
        aciklama: "Windows isteğe bağlı bileşenlerini listeler, etkinleştirir veya kaldırır.",
        hedef: AracHedef::Sayfa(Sayfa::Bakim),
    },
    AracKaydi {
        ad: "Windows Güncelleme Yöneticisi",
        aciklama: "Kurulu Windows güncellemelerini listeler ve kaldırılabilir olanları kaldırır.",
        hedef: AracHedef::Panel(MerkezAraci::WindowsGuncelleme),
    },
    AracKaydi {
        ad: "Tarayıcı Uzantısı Yöneticisi",
        aciklama: "Chrome, Edge, Brave, Vivaldi, Yandex, Opera ve Firefox uzantılarını listeler ve karantinaya alır.",
        hedef: AracHedef::Panel(MerkezAraci::TarayiciUzantilari),
    },
];

const TEMIZLIK_ARACLARI: [AracKaydi; 4] = [
    AracKaydi {
        ad: "Kayıt Defteri Temizleyici",
        aciklama: "Doğrulanabilir kayıt defteri artıklarını .reg yedeğiyle karantinaya taşır.",
        hedef: AracHedef::Sayfa(Sayfa::Temizlik),
    },
    AracKaydi {
        ad: "Gereksiz Dosya Temizleyici",
        aciklama: "Geçici dosyaları ve tarayıcı önbelleklerini boyutlarıyla tarar ve temizler.",
        hedef: AracHedef::Sayfa(Sayfa::Temizlik),
    },
    AracKaydi {
        ad: "Boş Klasör Temizleyici",
        aciklama: "Seçilen kök altındaki boş klasörleri bulur ve siler.",
        hedef: AracHedef::Sayfa(Sayfa::Temizlik),
    },
    AracKaydi {
        ad: "Kısayolları Düzelt",
        aciklama: "Hedefi bulunmayan kısayolları tespit eder ve karantinaya taşır.",
        hedef: AracHedef::Sayfa(Sayfa::Temizlik),
    },
];

const ARAMA_ARACLARI: [AracKaydi; 4] = [
    AracKaydi {
        ad: "Dosya ve Kayıt Bulucu",
        aciklama: "Dosya adlarında ve kayıt defterinde joker destekli arama yapar.",
        hedef: AracHedef::Panel(MerkezAraci::DosyaKayitBulucu),
    },
    AracKaydi {
        ad: "Kopya Dosya Bulucu",
        aciklama: "Aynı boyut ve içerik özetine sahip dosyaları gruplar ve karantinaya taşır.",
        hedef: AracHedef::Sayfa(Sayfa::Temizlik),
    },
    AracKaydi {
        ad: "Disk Analisti",
        aciklama: "Klasör ve dosya boyutlarını en büyükten küçüğe analiz eder.",
        hedef: AracHedef::Panel(MerkezAraci::DiskAnalisti),
    },
    AracKaydi {
        ad: "Dosyaları kalıcı sil",
        aciklama: "Dosyayı çok geçişli rastgele veriyle üzerine yazarak parçalar.",
        hedef: AracHedef::Sayfa(Sayfa::Temizlik),
    },
];

const SISTEM_ARACLARI: [AracKaydi; 7] = [
    AracKaydi {
        ad: "İşlem Yöneticisi",
        aciklama: "Çalışan süreçleri listeler ve seçilen süreci sonlandırır.",
        hedef: AracHedef::Sayfa(Sayfa::SistemYonetimi),
    },
    AracKaydi {
        ad: "Bağlantı Yöneticisi",
        aciklama: "Etkin TCP/UDP bağlantılarını süreç bilgisiyle listeler.",
        hedef: AracHedef::Panel(MerkezAraci::BaglantiYoneticisi),
    },
    AracKaydi {
        ad: "Başlangıç Yöneticisi",
        aciklama: "Başlangıç programlarını listeler, etkinleştirir veya devre dışı bırakır.",
        hedef: AracHedef::Sayfa(Sayfa::SistemYonetimi),
    },
    AracKaydi {
        ad: "Hizmet Yöneticisi",
        aciklama: "Windows hizmetlerini listeler, başlatır, durdurur ve başlangıç türünü değiştirir.",
        hedef: AracHedef::Sayfa(Sayfa::SistemYonetimi),
    },
    AracKaydi {
        ad: "Zamanlanmış Görev Yöneticisi",
        aciklama: "Zamanlanmış görevleri listeler, çalıştırır ve etkinlik durumunu değiştirir.",
        hedef: AracHedef::Sayfa(Sayfa::SistemYonetimi),
    },
    AracKaydi {
        ad: "Sağtık Menü Yöneticisi",
        aciklama: "Dosya sağ tık menüsü bileşenlerini listeler, devre dışı bırakır ve klasik menüyü yönetir.",
        hedef: AracHedef::Sayfa(Sayfa::Araclar),
    },
    AracKaydi {
        ad: "Sistem Geri Yükleme Yöneticisi",
        aciklama: "Geri yükleme noktalarını listeler, oluşturur, geri döner ve temizler.",
        hedef: AracHedef::Sayfa(Sayfa::Bakim),
    },
];

impl WinNexusApp {
    pub(super) fn arac_merkezi_sayfasi(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(arac) = self.merkez_araci {
            return self.arac_merkezi_detayi(arac, window, cx);
        }
        let gruplar = [
            (
                "Kaldırma ve Uygulama Yönetimi",
                KALDIRMA_ARACLARI.as_slice(),
            ),
            ("Temizlik", TEMIZLIK_ARACLARI.as_slice()),
            ("Arama ve Analiz", ARAMA_ARACLARI.as_slice()),
            ("Sistem Yönetimi", SISTEM_ARACLARI.as_slice()),
        ];
        let mut sira = 0usize;
        let paneller = gruplar
            .into_iter()
            .map(|(baslik, araclar)| {
                let satirlar = araclar
                    .iter()
                    .map(|arac| {
                        let kimlik = sira;
                        sira += 1;
                        self.arac_merkezi_satiri(kimlik, *arac, cx)
                    })
                    .collect::<Vec<_>>();
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().muted_foreground)
                            .child(baslik),
                    )
                    .children(satirlar)
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        v_flex().gap_4().children(paneller).into_any_element()
    }

    fn arac_merkezi_satiri(
        &self,
        sira: usize,
        kayit: AracKaydi,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let hedef = kayit.hedef;
        h_flex()
            .justify_between()
            .items_center()
            .gap_4()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(kayit.ad))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(kayit.aciklama),
                    ),
            )
            .child(
                Button::new(format!("merkez-arac-{sira}"))
                    .label(format!("Aç · {}", kayit.ad))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        match hedef {
                            AracHedef::Sayfa(sayfa) => {
                                this.sayfa = sayfa;
                                this.merkez_araci = None;
                            }
                            AracHedef::Panel(arac) => {
                                this.merkez_araci = Some(arac);
                                this.sayfa = Sayfa::AracMerkezi;
                                this.merkez_mesaj = None;
                                this.merkez_hata = None;
                                this.merkez_onay = None;
                                if matches!(
                                    arac,
                                    MerkezAraci::DosyaKayitBulucu
                                        | MerkezAraci::ZorlaKaldir
                                        | MerkezAraci::WindowsGuncelleme
                                        | MerkezAraci::TarayiciUzantilari
                                        | MerkezAraci::BaglantiYoneticisi
                                ) {
                                    this.merkez_sorgu = None;
                                    this.merkez_sorgu_abonelik = None;
                                    this.merkez_sorgu_olustur(window, cx);
                                }
                            }
                        }
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    fn arac_merkezi_detayi(
        &mut self,
        arac: MerkezAraci,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (ad, aciklama) = match arac {
            MerkezAraci::ZorlaKaldir => (
                "Zorla kaldır",
                "Kaldırma komutu çalışmayan uygulamaların klasörünü karantinaya alır ve kayıt anahtarını yedekleyerek siler.",
            ),
            MerkezAraci::KurulumIzleme => (
                "Kurulum İzleme",
                "Kurulum öncesi ve sonrası dosya/kayıt anlık görüntüsünü karşılaştırarak değişiklikleri raporlar.",
            ),
            MerkezAraci::WindowsGuncelleme => (
                "Windows Güncelleme Yöneticisi",
                "Kurulu Windows güncellemelerini listeler ve kaldırılabilir olanları kaldırır.",
            ),
            MerkezAraci::TarayiciUzantilari => (
                "Tarayıcı Uzantısı Yöneticisi",
                "Chrome, Edge, Brave, Vivaldi, Yandex, Opera ve Firefox uzantılarını listeler ve karantinaya alır.",
            ),
            MerkezAraci::DosyaKayitBulucu => (
                "Dosya ve Kayıt Bulucu",
                "Dosya adlarında ve kayıt defterinde joker destekli arama yapar.",
            ),
            MerkezAraci::DiskAnalisti => (
                "Disk Analisti",
                "Klasör ve dosya boyutlarını en büyükten küçüğe analiz eder.",
            ),
            MerkezAraci::BaglantiYoneticisi => (
                "Bağlantı Yöneticisi",
                "Etkin TCP/UDP bağlantılarını süreç bilgisiyle listeler.",
            ),
        };
        let icerik = match arac {
            MerkezAraci::WindowsGuncelleme => self.merkez_guncelleme_paneli(cx),
            MerkezAraci::TarayiciUzantilari => self.merkez_uzanti_paneli(cx),
            MerkezAraci::DosyaKayitBulucu => self.merkez_bulucu_paneli(window, cx),
            MerkezAraci::DiskAnalisti => self.merkez_disk_paneli(cx),
            MerkezAraci::BaglantiYoneticisi => self.merkez_baglanti_paneli(cx),
            MerkezAraci::ZorlaKaldir => self.merkez_zorla_paneli(cx),
            MerkezAraci::KurulumIzleme => self.merkez_izleme_paneli(cx),
        };
        v_flex()
            .gap_4()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        Button::new("merkez-arac-geri")
                            .label("Geri")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.merkez_araci = None;
                                this.merkez_onay = None;
                                this.merkez_mesaj = None;
                                this.merkez_hata = None;
                                cx.notify();
                            })),
                    )
                    .child(div().text_lg().font_weight(FontWeight::SEMIBOLD).child(ad)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(aciklama),
            )
            .child(self.merkez_onay_paneli(cx))
            .child(self.merkez_durum_kutulari(cx))
            .child(icerik)
            .into_any_element()
    }

    fn merkez_arka_plan<T, F, U>(&mut self, cx: &mut Context<Self>, gorev: F, uygula: U)
    where
        T: Send + 'static,
        F: FnOnce() -> sp_core::Sonuc<T> + Send + 'static,
        U: FnOnce(&mut Self, &mut Context<Self>, sp_core::Sonuc<T>) + 'static,
    {
        if self.merkez_islemde {
            return;
        }
        self.merkez_islemde = true;
        self.merkez_hata = None;
        self.merkez_mesaj = Some("İşlem sürüyor...".to_string());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let sonuc = cx.background_executor().spawn(async move { gorev() }).await;
            let _ = this.update(cx, |this, cx| {
                this.merkez_islemde = false;
                uygula(this, cx, sonuc);
                cx.notify();
            });
        })
        .detach();
    }

    fn merkez_sorgu_metni(&self, cx: &mut Context<Self>) -> String {
        self.merkez_sorgu
            .as_ref()
            .map(|girdi| girdi.read(cx).value().to_string())
            .unwrap_or_default()
    }

    fn merkez_filtre_satiri(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(girdi) = &self.merkez_sorgu else {
            return div().into_any_element();
        };
        v_flex()
            .gap_1()
            .child(Input::new(girdi).cleanable(true).w_full())
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Listeyi filtreleyin; performans için en fazla 40 satır gösterilir."),
            )
            .into_any_element()
    }

    fn merkez_durum_kutulari(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_2()
            .child(if let Some(hata) = &self.merkez_hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &self.merkez_mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .into_any_element()
    }

    fn merkez_onay_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(onay) = &self.merkez_onay else {
            return div().into_any_element();
        };
        let metin = match onay {
            MerkezOnay::ZorlaKaldir(kimlik) => {
                let ad = self
                    .zorla_uygulamalar
                    .iter()
                    .find(|uygulama| &uygulama.kimlik == kimlik)
                    .map(|uygulama| uygulama.ad.clone())
                    .unwrap_or_else(|| "Uygulama".to_string());
                format!(
                    "{ad}: kurulum klasörü karantinaya taşınacak ve kaldırma kaydı .reg yedeği alındıktan sonra silinecek. Devam edilsin mi?"
                )
            }
            MerkezOnay::GuncellemeKaldir(kb) => format!(
                "{kb}: wusa /uninstall çalıştırılacak; bazı güncellemeler kaldırılamaz ve yeniden başlatma gerekebilir. Devam edilsin mi?"
            ),
            MerkezOnay::UzantiKaldir(kimlik) => {
                let (ad, tarayici) = self
                    .tarayici_uzantilari
                    .iter()
                    .find(|uzanti| &uzanti.kimlik == kimlik)
                    .map(|uzanti| (uzanti.ad.clone(), uzanti.tarayici.clone()))
                    .unwrap_or_else(|| ("Uzantı".to_string(), "tarayıcı".to_string()));
                format!(
                    "{ad} ({tarayici}): uzantı klasörü karantinaya taşınacak. Tarayıcı kapalı olmalıdır. Devam edilsin mi?"
                )
            }
            MerkezOnay::BaglantiSonlandir(pid, ad) => format!(
                "{ad} (PID {pid}) süreci sonlandırılacak. Kaydedilmemiş çalışmalar kaybolabilir. Devam edilsin mi?"
            ),
        };
        v_flex()
            .p_3()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().warning)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Onay gerekiyor"),
            )
            .child(div().text_sm().child(metin))
            .child(self.merkez_onay_dugmeleri(cx))
            .into_any_element()
    }

    fn merkez_onay_dugmeleri(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .gap_2()
            .child(
                Button::new("merkez-onayla")
                    .primary()
                    .label("Onayla")
                    .on_click(cx.listener(|this, _, _, cx| this.merkez_onayla(cx))),
            )
            .child(
                Button::new("merkez-vazgec")
                    .label("Vazgeç")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.merkez_onay = None;
                        cx.notify();
                    })),
            )
            .into_any_element()
    }

    fn merkez_onayla(&mut self, cx: &mut Context<Self>) {
        let Some(onay) = self.merkez_onay.take() else {
            return;
        };
        match onay {
            MerkezOnay::GuncellemeKaldir(kb) => {
                let servis = self.gelismis_servis.clone();
                let etiket = kb.clone();
                self.merkez_arka_plan(
                    cx,
                    move || servis.guncelleme_kaldir(&kb),
                    move |this, cx, sonuc| match sonuc {
                        Ok(sonuc) => {
                            this.merkez_mesaj = Some(format!(
                                "{etiket} kaldırma komutu tamamlandı (kod {}).",
                                sonuc.kod
                            ));
                            this.merkez_guncelleme_tara(cx);
                        }
                        Err(hata) => this.merkez_hata = Some(hata.to_string()),
                    },
                );
            }
            MerkezOnay::UzantiKaldir(kimlik) => {
                let Some(uzanti) = self
                    .tarayici_uzantilari
                    .iter()
                    .find(|uzanti| uzanti.kimlik == kimlik)
                    .cloned()
                else {
                    self.merkez_hata = Some("Uzantı listede bulunamadı.".to_string());
                    cx.notify();
                    return;
                };
                let servis = self.gelismis_servis.clone();
                let karantina = self.dizinler.karantina_dizini().join("tarayici-uzanti");
                let ad = uzanti.ad.clone();
                self.merkez_arka_plan(
                    cx,
                    move || servis.uzanti_kaldir(&uzanti, &karantina),
                    move |this, cx, sonuc| match sonuc {
                        Ok(()) => {
                            this.merkez_mesaj = Some(format!("{ad} uzantısı karantinaya taşındı."));
                            this.merkez_uzanti_tara(cx);
                        }
                        Err(hata) => this.merkez_hata = Some(hata.to_string()),
                    },
                );
            }
            MerkezOnay::BaglantiSonlandir(pid, ad) => {
                let servis = self.sistem_yonetimi.clone();
                self.merkez_arka_plan(
                    cx,
                    move || {
                        servis
                            .surec_sonlandir(pid)
                            .map(|()| format!("{ad} süreci sonlandırıldı."))
                    },
                    |this, cx, sonuc| match sonuc {
                        Ok(mesaj) => {
                            this.merkez_mesaj = Some(mesaj);
                            this.merkez_baglanti_tara(cx);
                        }
                        Err(hata) => this.merkez_hata = Some(hata.to_string()),
                    },
                );
            }
            MerkezOnay::ZorlaKaldir(kimlik) => self.merkez_zorla_uygula(kimlik, cx),
        }
    }

    pub(super) fn merkez_guncelleme_tara(&mut self, cx: &mut Context<Self>) {
        let servis = self.gelismis_servis.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.guncellemeleri_listele(),
            |this, _cx, sonuc| match sonuc {
                Ok(guncellemeler) => {
                    this.merkez_mesaj = Some(format!(
                        "{} Windows güncellemesi bulundu.",
                        guncellemeler.len()
                    ));
                    this.guncellemeler = guncellemeler;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    pub(super) fn merkez_uzanti_tara(&mut self, cx: &mut Context<Self>) {
        let servis = self.gelismis_servis.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.tarayici_uzantilari(),
            |this, _cx, sonuc| match sonuc {
                Ok(uzantilar) => {
                    this.merkez_mesaj =
                        Some(format!("{} tarayıcı uzantısı bulundu.", uzantilar.len()));
                    this.tarayici_uzantilari = uzantilar;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    pub(super) fn merkez_baglanti_tara(&mut self, cx: &mut Context<Self>) {
        let servis = self.gelismis_servis.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.baglantilari_listele(),
            |this, _cx, sonuc| match sonuc {
                Ok(baglantilar) => {
                    this.merkez_mesaj =
                        Some(format!("{} etkin bağlantı bulundu.", baglantilar.len()));
                    this.baglantilar = baglantilar;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_dosya_ara(&mut self, kok: PathBuf, desen: String, cx: &mut Context<Self>) {
        let servis = self.bulucu_servisi.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.dosya_ara(&kok, &desen, 500),
            |this, _cx, sonuc| match sonuc {
                Ok(bulgular) => {
                    this.merkez_mesaj = Some(format!("{} dosya/klasör bulundu.", bulgular.len()));
                    this.dosya_bulgulari = bulgular;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_kayit_ara(&mut self, sorgu: String, cx: &mut Context<Self>) {
        let servis = self.bulucu_servisi.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.kayit_defteri_ara(&sorgu, 300),
            |this, _cx, sonuc| match sonuc {
                Ok(bulgular) => {
                    this.merkez_mesaj = Some(format!("{} kayıt bulundu.", bulgular.len()));
                    this.kayit_bulgulari = bulgular;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_disk_tara(&mut self, kok: PathBuf, cx: &mut Context<Self>) {
        let servis = self.bulucu_servisi.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.disk_analizi(&kok, 200),
            |this, _cx, sonuc| match sonuc {
                Ok(analiz) => {
                    this.merkez_mesaj = Some(format!(
                        "{} için toplam {}.",
                        analiz.kok,
                        sp_core::bulucu::boyut_metni(analiz.toplam_bayt)
                    ));
                    this.disk_analizi = Some(analiz);
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_sorgu_olustur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.merkez_sorgu.is_some() {
            return;
        }
        let girdi = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Aranacak ad veya desen (örn. *.log)")
        });
        let abonelik = cx.subscribe(&girdi, |_this: &mut Self, _, olay: &InputEvent, cx| {
            if matches!(olay, InputEvent::Change) {
                cx.notify();
            }
        });
        self.merkez_sorgu = Some(girdi);
        self.merkez_sorgu_abonelik = Some(abonelik);
    }

    fn merkez_guncelleme_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let filtre = self.merkez_sorgu_metni(cx).trim().to_lowercase();
        let toplam = self.guncellemeler.len();
        let guncellemeler = self
            .guncellemeler
            .iter()
            .filter(|guncelleme| {
                filtre.is_empty()
                    || guncelleme.baslik.to_lowercase().contains(&filtre)
                    || guncelleme
                        .kb
                        .as_deref()
                        .is_some_and(|kb| kb.to_lowercase().contains(&filtre))
            })
            .take(40)
            .cloned()
            .collect::<Vec<_>>();
        let islemde = self.merkez_islemde;
        self.bolum_karti(
            "Kurulu Windows Güncellemeleri".to_string(),
            "Get-HotFix listesi; KB numarası doğrulanan güncellemeler wusa ile kaldırılabilir."
                .to_string(),
            v_flex()
                .gap_3()
                .child(
                    Button::new("merkez-guncelleme-tara")
                        .primary()
                        .label(if islemde {
                            "Taranıyor..."
                        } else {
                            "Güncellemeleri Tara"
                        })
                        .disabled(islemde)
                        .on_click(cx.listener(|this, _, _, cx| this.merkez_guncelleme_tara(cx))),
                )
                .child(self.merkez_filtre_satiri(cx))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{toplam} güncellemeden {} tanesi gösteriliyor.",
                            guncellemeler.len()
                        )),
                )
                .children(guncellemeler.iter().map(|guncelleme| {
                    let kb = guncelleme
                        .kb
                        .clone()
                        .unwrap_or_else(|| "KB yok".to_string());
                    let aksiyon = match (&guncelleme.kb, guncelleme.kaldirilabilir) {
                        (Some(kb), true) => {
                            let kb = kb.clone();
                            vec![Button::new(format!("merkez-guncelleme-kaldir-{kb}"))
                                .danger()
                                .label(format!("Kaldır · {kb}"))
                                .disabled(islemde)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.merkez_onay =
                                        Some(MerkezOnay::GuncellemeKaldir(kb.clone()));
                                    cx.notify();
                                }))
                                .into_any_element()]
                        }
                        _ => Vec::new(),
                    };
                    let alt = [Some(guncelleme.tur.clone()), guncelleme.tarih.clone()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" · ");
                    self.modul_satiri(
                        format!("{kb} · {}", guncelleme.baslik),
                        alt,
                        String::new(),
                        aksiyon,
                        cx,
                    )
                    .into_any_element()
                }))
                .into_any_element(),
            cx,
        )
    }

    fn merkez_uzanti_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let filtre = self.merkez_sorgu_metni(cx).trim().to_lowercase();
        let toplam = self.tarayici_uzantilari.len();
        let uzantilar = self
            .tarayici_uzantilari
            .iter()
            .filter(|uzanti| {
                filtre.is_empty()
                    || uzanti.ad.to_lowercase().contains(&filtre)
                    || uzanti.tarayici.to_lowercase().contains(&filtre)
                    || uzanti.profil.to_lowercase().contains(&filtre)
            })
            .take(40)
            .cloned()
            .collect::<Vec<_>>();
        let islemde = self.merkez_islemde;
        self.bolum_karti(
            "Tarayıcı Uzantıları".to_string(),
            "Chromium profilleri ve Firefox uzantıları; kaldırma işlemi klasörü karantinaya taşır."
                .to_string(),
            v_flex()
                .gap_3()
                .child(
                    Button::new("merkez-uzanti-tara")
                        .primary()
                        .label(if islemde {
                            "Taranıyor..."
                        } else {
                            "Uzantıları Tara"
                        })
                        .disabled(islemde)
                        .on_click(cx.listener(|this, _, _, cx| this.merkez_uzanti_tara(cx))),
                )
                .child(self.merkez_filtre_satiri(cx))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{toplam} uzantıdan {} tanesi gösteriliyor.",
                            uzantilar.len()
                        )),
                )
                .children(uzantilar.iter().map(|uzanti| {
                    let kimlik = uzanti.kimlik.clone();
                    let durum = if uzanti.etkin {
                        "Etkin"
                    } else {
                        "Devre dışı"
                    };
                    self.modul_satiri(
                        format!("{} · {}", uzanti.ad, uzanti.surum),
                        format!("{} · {} · {}", uzanti.tarayici, uzanti.profil, durum),
                        uzanti.yol.clone(),
                        vec![Button::new(format!(
                            "merkez-uzanti-kaldir-{}",
                            Self::kimlik_ozeti(&kimlik)
                        ))
                        .danger()
                        .label(format!("Kaldır · {}", uzanti.ad))
                        .disabled(islemde)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.merkez_onay = Some(MerkezOnay::UzantiKaldir(kimlik.clone()));
                            cx.notify();
                        }))
                        .into_any_element()],
                        cx,
                    )
                    .into_any_element()
                }))
                .into_any_element(),
            cx,
        )
    }

    fn merkez_bulucu_paneli(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(girdi) = self.merkez_sorgu.clone() else {
            return self.bolum_karti(
                "Dosya ve Kayıt Bulucu".to_string(),
                "Arama alanı hazır değil.".to_string(),
                v_flex()
                    .gap_2()
                    .child(
                        Button::new("merkez-arama-hazirla")
                            .primary()
                            .label("Aramayı Hazırla")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.merkez_sorgu_olustur(window, cx);
                                cx.notify();
                            })),
                    )
                    .into_any_element(),
                cx,
            );
        };
        let dosya_modu = self.merkez_arama_modu == AramaModu::Dosya;
        let kok = self.merkez_kok.clone();
        let dosya_bulgulari = self.dosya_bulgulari.clone();
        let kayit_bulgulari = self.kayit_bulgulari.clone();
        let islemde = self.merkez_islemde;
        let sonuclar = if dosya_modu {
            dosya_bulgulari
                .iter()
                .take(300)
                .map(|bulgu| {
                    self.modul_satiri(
                        bulgu.ad.clone(),
                        bulgu.yol.clone(),
                        format!(
                            "{} · {}{}",
                            bulgu.tur,
                            sp_core::bulucu::boyut_metni(bulgu.boyut),
                            bulgu
                                .degisme
                                .as_ref()
                                .map(|tarih| format!(" · {tarih}"))
                                .unwrap_or_default()
                        ),
                        Vec::new(),
                        cx,
                    )
                    .into_any_element()
                })
                .collect::<Vec<_>>()
        } else {
            kayit_bulgulari
                .iter()
                .take(300)
                .map(|bulgu| {
                    self.modul_satiri(
                        format!("{} \\ {}", bulgu.kove, bulgu.yol),
                        bulgu
                            .deger
                            .clone()
                            .unwrap_or_else(|| "(anahtar)".to_string()),
                        bulgu.veri.clone(),
                        Vec::new(),
                        cx,
                    )
                    .into_any_element()
                })
                .collect::<Vec<_>>()
        };
        self.bolum_karti(
            "Dosya ve Kayıt Bulucu".to_string(),
            "Dosya araması joker (*, ?) destekler; kayıt araması HKLM/HKCU Software altında sınırlı süre ve sonuçla çalışır.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .child(if dosya_modu {
                            Button::new("merkez-mod-dosya")
                                .label("Dosya")
                                .primary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.merkez_arama_modu = AramaModu::Dosya;
                                    cx.notify();
                                }))
                        } else {
                            Button::new("merkez-mod-dosya").label("Dosya").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.merkez_arama_modu = AramaModu::Dosya;
                                    cx.notify();
                                }),
                            )
                        })
                        .child(if dosya_modu {
                            Button::new("merkez-mod-kayit").label("Kayıt Defteri").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.merkez_arama_modu = AramaModu::Kayit;
                                    cx.notify();
                                }),
                            )
                        } else {
                            Button::new("merkez-mod-kayit")
                                .label("Kayıt Defteri")
                                .primary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.merkez_arama_modu = AramaModu::Kayit;
                                    cx.notify();
                                }))
                        }),
                )
                .child(if dosya_modu {
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("merkez-kok-sec")
                                .label("Kök Klasör Seç")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if let Some(kok) = rfd::FileDialog::new()
                                        .set_title("Arama kökü seçin")
                                        .pick_folder()
                                    {
                                        this.merkez_kok = Some(kok);
                                        cx.notify();
                                    }
                                })),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    kok.as_ref()
                                        .map(|kok| kok.display().to_string())
                                        .unwrap_or_else(|| "Kök klasör seçilmedi".to_string()),
                                ),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Hedef: HKLM ve HKCU \\ Software")
                        .into_any_element()
                })
                .child(
                    h_flex()
                        .gap_2()
                        .child(Input::new(&girdi).cleanable(true).w_full())
                        .child(
                            Button::new("merkez-ara")
                                .primary()
                                .label("Ara")
                                .disabled(islemde)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let sorgu = girdi.read(cx).value().trim().to_string();
                                    if sorgu.is_empty() {
                                        this.merkez_hata =
                                            Some("Arama metni boş olamaz.".to_string());
                                        cx.notify();
                                        return;
                                    }
                                    if this.merkez_arama_modu == AramaModu::Dosya {
                                        let Some(kok) = this.merkez_kok.clone() else {
                                            this.merkez_hata =
                                                Some("Önce kök klasör seçin.".to_string());
                                            cx.notify();
                                            return;
                                        };
                                        this.merkez_dosya_ara(kok, sorgu, cx);
                                    } else {
                                        this.merkez_kayit_ara(sorgu, cx);
                                    }
                                })),
                        ),
                )
                .children(sonuclar)
                .into_any_element(),
            cx,
        )
    }

    fn merkez_disk_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let kok = self.merkez_kok.clone();
        let analiz = self.disk_analizi.clone();
        let islemde = self.merkez_islemde;
        let satirlar = analiz
            .as_ref()
            .map(|analiz| {
                analiz
                    .ogeler
                    .iter()
                    .take(300)
                    .map(|oge| {
                        self.modul_satiri(
                            oge.ad.clone(),
                            oge.yol.clone(),
                            format!(
                                "{} · {}",
                                if oge.dosya_mi { "Dosya" } else { "Klasör" },
                                sp_core::bulucu::boyut_metni(oge.boyut)
                            ),
                            Vec::new(),
                            cx,
                        )
                        .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.bolum_karti(
            "Disk Analisti".to_string(),
            "Seçilen kökün toplam boyutunu, alt öğelerini ve en büyük dosyalarını hesaplar."
                .to_string(),
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Button::new("merkez-disk-kok").label("Klasör Seç").on_click(
                            cx.listener(|this, _, _, cx| {
                                if let Some(kok) = rfd::FileDialog::new()
                                    .set_title("Analiz edilecek klasörü seçin")
                                    .pick_folder()
                                {
                                    this.merkez_kok = Some(kok);
                                    cx.notify();
                                }
                            }),
                        ))
                        .child(
                            Button::new("merkez-disk-tara")
                                .primary()
                                .label(if islemde {
                                    "Analiz ediliyor..."
                                } else {
                                    "Analiz Et"
                                })
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    let Some(kok) = this.merkez_kok.clone() else {
                                        this.merkez_hata =
                                            Some("Önce bir klasör seçin.".to_string());
                                        cx.notify();
                                        return;
                                    };
                                    this.merkez_disk_tara(kok, cx);
                                })),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    kok.as_ref()
                                        .map(|kok| kok.display().to_string())
                                        .unwrap_or_else(|| "Klasör seçilmedi".to_string()),
                                ),
                        ),
                )
                .child(if let Some(analiz) = &analiz {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "Toplam {} · {} öğe{}",
                            sp_core::bulucu::boyut_metni(analiz.toplam_bayt),
                            analiz.ogeler.len(),
                            if analiz.kesildi {
                                " · süre sınırı nedeniyle kısmi"
                            } else {
                                ""
                            }
                        ))
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .children(satirlar)
                .into_any_element(),
            cx,
        )
    }

    fn merkez_baglanti_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let filtre = self.merkez_sorgu_metni(cx).trim().to_lowercase();
        let toplam = self.baglantilar.len();
        let baglantilar = self
            .baglantilar
            .iter()
            .filter(|baglanti| {
                filtre.is_empty()
                    || baglanti.surec.to_lowercase().contains(&filtre)
                    || baglanti.protokol.to_lowercase().contains(&filtre)
                    || baglanti.yerel_adres.to_lowercase().contains(&filtre)
                    || baglanti.uzak_adres.to_lowercase().contains(&filtre)
                    || baglanti.durum.to_lowercase().contains(&filtre)
            })
            .take(40)
            .cloned()
            .collect::<Vec<_>>();
        let islemde = self.merkez_islemde;
        self.bolum_karti(
            "Etkin Bağlantılar".to_string(),
            "TCP/UDP uç noktaları ve sahip süreçler; süreç sonlandırma onay ister.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    Button::new("merkez-baglanti-tara")
                        .primary()
                        .label(if islemde {
                            "Taranıyor..."
                        } else {
                            "Bağlantıları Yenile"
                        })
                        .disabled(islemde)
                        .on_click(cx.listener(|this, _, _, cx| this.merkez_baglanti_tara(cx))),
                )
                .child(self.merkez_filtre_satiri(cx))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{toplam} bağlantıdan {} tanesi gösteriliyor.",
                            baglantilar.len()
                        )),
                )
                .children(baglantilar.iter().map(|baglanti| {
                    let aksiyon = if baglanti.pid > 0 {
                        let pid = baglanti.pid;
                        let surec = baglanti.surec.clone();
                        let anahtar = format!(
                            "{}|{}|{}|{}|{}",
                            pid,
                            baglanti.protokol,
                            baglanti.yerel_adres,
                            baglanti.yerel_port,
                            baglanti.uzak_port
                        );
                        vec![Button::new(format!(
                            "merkez-baglanti-kes-{}",
                            Self::kimlik_ozeti(&anahtar)
                        ))
                        .danger()
                        .label(format!(
                            "Sonlandır · {} [{}]",
                            surec,
                            sp_core::gelismis_arac::uc_nokta(
                                &baglanti.yerel_adres,
                                baglanti.yerel_port
                            )
                        ))
                        .disabled(islemde)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.merkez_onay =
                                Some(MerkezOnay::BaglantiSonlandir(pid, surec.clone()));
                            cx.notify();
                        }))
                        .into_any_element()]
                    } else {
                        Vec::new()
                    };
                    self.modul_satiri(
                        format!(
                            "{} {} → {}",
                            baglanti.protokol,
                            sp_core::gelismis_arac::uc_nokta(
                                &baglanti.yerel_adres,
                                baglanti.yerel_port
                            ),
                            sp_core::gelismis_arac::uc_nokta(
                                &baglanti.uzak_adres,
                                baglanti.uzak_port
                            )
                        ),
                        format!(
                            "{} · {} (PID {})",
                            if baglanti.durum.is_empty() {
                                "Dinleme"
                            } else {
                                baglanti.durum.as_str()
                            },
                            if baglanti.surec.is_empty() {
                                "-"
                            } else {
                                baglanti.surec.as_str()
                            },
                            baglanti.pid
                        ),
                        String::new(),
                        aksiyon,
                        cx,
                    )
                    .into_any_element()
                }))
                .into_any_element(),
            cx,
        )
    }

    fn merkez_zorla_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let filtre = self.merkez_sorgu_metni(cx).trim().to_lowercase();
        let toplam = self.zorla_uygulamalar.len();
        let uygulamalar = self
            .zorla_uygulamalar
            .iter()
            .filter(|uygulama| {
                filtre.is_empty()
                    || uygulama.ad.to_lowercase().contains(&filtre)
                    || uygulama
                        .yayinci
                        .as_deref()
                        .is_some_and(|yayinci| yayinci.to_lowercase().contains(&filtre))
            })
            .take(40)
            .cloned()
            .collect::<Vec<_>>();
        let sonuc = self.zorla_sonucu.clone();
        let islemde = self.merkez_islemde;
        self.bolum_karti(
            "Zorla Kaldır".to_string(),
            "Kaldırma komutu çalışmayan uygulamalar için kurulum klasörü karantinaya taşınır; Uninstall kaydı .reg yedeğiyle silinir.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    Button::new("merkez-zorla-tara")
                        .primary()
                        .label(if islemde {
                            "Taranıyor..."
                        } else {
                            "Uygulamaları Tara"
                        })
                        .disabled(islemde)
                        .on_click(cx.listener(|this, _, _, cx| this.merkez_zorla_tara(cx))),
                )
                .child(self.merkez_filtre_satiri(cx))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "{toplam} uygulamadan {} tanesi gösteriliyor.",
                            uygulamalar.len()
                        )),
                )
                .child(if let Some(sonuc) = &sonuc {
                    v_flex()
                        .gap_1()
                        .p_3()
                        .rounded_md()
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("{} · sonuç", sonuc.ad)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "Klasör: {} · Kayıt: {}",
                                    sonuc
                                        .klasor_tasindi
                                        .clone()
                                        .unwrap_or_else(|| "taşınmadı".to_string()),
                                    if sonuc.kayit_silindi {
                                        "silindi"
                                    } else {
                                        "silinmedi"
                                    }
                                )),
                        )
                        .child(if let Some(yedek) = &sonuc.yedek {
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("Registry yedeği: {yedek}"))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        })
                        .children(sonuc.hatalar.iter().map(|hata| {
                            div()
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .child(hata.clone())
                        }))
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .children(uygulamalar.iter().map(|uygulama| {                    let kimlik = uygulama.kimlik.clone();
                    let alt = [
                        uygulama.surum.clone(),
                        uygulama.yayinci.clone(),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ");
                    self.modul_satiri(
                        uygulama.ad.clone(),
                        if alt.is_empty() {
                            "Sürüm/yayıncı bilinmiyor".to_string()
                        } else {
                            alt
                        },
                        uygulama
                            .kurulum_konumu
                            .clone()
                            .unwrap_or_else(|| "Kurulum konumu kayıtlı değil".to_string()),
                        vec![Button::new(format!(
                            "merkez-zorla-{}",
                            Self::kimlik_ozeti(&kimlik)
                        ))
                        .danger()
                        .label(format!("Zorla Kaldır · {}", uygulama.ad))
                        .disabled(islemde)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.merkez_onay =
                                Some(MerkezOnay::ZorlaKaldir(kimlik.clone()));
                            cx.notify();
                        }))
                        .into_any_element()],
                        cx,
                    )
                    .into_any_element()
                }))
                .into_any_element(),
            cx,
        )
    }

    fn merkez_zorla_tara(&mut self, cx: &mut Context<Self>) {
        let servis = self.kaldirici.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.uygulamalari_tara(),
            |this, _cx, sonuc| match sonuc {
                Ok(uygulamalar) => {
                    this.merkez_mesaj =
                        Some(format!("{} kurulu uygulama bulundu.", uygulamalar.len()));
                    this.zorla_uygulamalar = uygulamalar;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_zorla_uygula(&mut self, kimlik: String, cx: &mut Context<Self>) {
        let Some(uygulama) = self
            .zorla_uygulamalar
            .iter()
            .find(|uygulama| uygulama.kimlik == kimlik)
            .cloned()
        else {
            self.merkez_hata = Some("Uygulama listede bulunamadı.".to_string());
            cx.notify();
            return;
        };
        let servis = self.zorla_servisi.clone();
        let ad = uygulama.ad.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.zorla_kaldir(&uygulama),
            move |this, cx, sonuc| match sonuc {
                Ok(sonuc) => {
                    this.merkez_mesaj = Some(format!(
                        "{ad} zorla kaldırıldı; {} hata.",
                        sonuc.hatalar.len()
                    ));
                    this.zorla_sonucu = Some(sonuc);
                    this.merkez_zorla_tara(cx);
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_izleme_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let oturum = self.izleme_oturumu.clone();
        let rapor = self.izleme_raporu.clone();
        let islemde = self.merkez_islemde;
        let mut satirlar = Vec::new();
        if let Some(rapor) = &rapor {
            for degisiklik in rapor.dosyalar.iter().take(200) {
                satirlar.push(
                    self.modul_satiri(
                        format!("{} · {}", degisiklik.tur, degisiklik.yol),
                        degisiklik.ayrinti.clone(),
                        String::new(),
                        Vec::new(),
                        cx,
                    )
                    .into_any_element(),
                );
            }
            for degisiklik in rapor.kayitlar.iter().take(100) {
                satirlar.push(
                    self.modul_satiri(
                        format!("Kayıt {} · {}", degisiklik.tur, degisiklik.yol),
                        degisiklik.ayrinti.clone(),
                        String::new(),
                        Vec::new(),
                        cx,
                    )
                    .into_any_element(),
                );
            }
        }
        self.bolum_karti(
            "Kurulum İzleme".to_string(),
            "Sürücü kurmaz: seçili dosya kökleri ve Uninstall/App Paths kayıtlarının öncesi/sonrası anlık görüntüsünü karşılaştırır.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("merkez-izleme-durum")
                                .label("Durumu Yenile")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.merkez_izleme_durum(cx);
                                })),
                        )
                        .child(
                            Button::new("merkez-izleme-baslat")
                                .primary()
                                .label("İzlemeyi Başlat")
                                .disabled(islemde || oturum.is_some())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.merkez_izleme_baslat(cx);
                                })),
                        )
                        .child(
                            Button::new("merkez-izleme-durdur")
                                .danger()
                                .label("İzlemeyi Durdur ve Raporla")
                                .disabled(islemde || oturum.is_none())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.merkez_izleme_durdur(cx);
                                })),
                        ),
                )
                .child(if let Some(oturum) = &oturum {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "Aktif oturum {} · başlangıç {} · {} dosya, {} kayıt",
                            oturum.kimlik,
                            oturum.baslangic,
                            oturum.dosya_sayisi,
                            oturum.kayit_sayisi
                        ))
                        .into_any_element()
                } else {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Aktif izleme oturumu yok. Kurulumdan önce başlatın, sonra durdurun.")
                        .into_any_element()
                })
                .child(if let Some(rapor) = &rapor {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "Son rapor {} · {} dosya, {} kayıt değişikliği{}",
                            rapor.bitis,
                            rapor.toplam_dosya,
                            rapor.toplam_kayit,
                            if rapor.kesildi {
                                " · kısmi"
                            } else {
                                ""
                            }
                        ))
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .children(satirlar)
                .into_any_element(),
            cx,
        )
    }

    fn merkez_izleme_durum(&mut self, cx: &mut Context<Self>) {
        let servis = self.izleme_servisi.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.durum(),
            |this, _cx, sonuc| match sonuc {
                Ok(durum) => {
                    this.merkez_mesaj = Some(if durum.is_some() {
                        "Aktif izleme oturumu bulundu.".to_string()
                    } else {
                        "Aktif izleme oturumu yok.".to_string()
                    });
                    this.izleme_oturumu = durum;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_izleme_baslat(&mut self, cx: &mut Context<Self>) {
        let servis = self.izleme_servisi.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.baslat(),
            |this, _cx, sonuc| match sonuc {
                Ok(oturum) => {
                    this.merkez_mesaj = Some(format!(
                        "İzleme başladı: {} dosya, {} kayıt.",
                        oturum.dosya_sayisi, oturum.kayit_sayisi
                    ));
                    this.izleme_oturumu = Some(oturum);
                    this.izleme_raporu = None;
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }

    fn merkez_izleme_durdur(&mut self, cx: &mut Context<Self>) {
        let servis = self.izleme_servisi.clone();
        self.merkez_arka_plan(
            cx,
            move || servis.durdur(),
            |this, _cx, sonuc| match sonuc {
                Ok(rapor) => {
                    this.merkez_mesaj = Some(format!(
                        "İzleme durduruldu: {} dosya, {} kayıt değişikliği.",
                        rapor.toplam_dosya, rapor.toplam_kayit
                    ));
                    this.izleme_oturumu = None;
                    this.izleme_raporu = Some(rapor);
                }
                Err(hata) => this.merkez_hata = Some(hata.to_string()),
            },
        );
    }
}
