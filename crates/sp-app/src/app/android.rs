use super::*;

impl WinNexusApp {
    pub(super) fn android_kisa_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        self.bolum_karti(
            "Android Yönetimi".to_string(),
            "ADB, sürücü, marka uyarıları ve bloatware kataloğu tek sayfada.".to_string(),
            Button::new("android-ac")
                .primary()
                .label("Android Yönetimini Aç")
                .on_click(cx.listener(|this, _, window, cx| {
                    this.sayfa = Sayfa::Android;
                    this.android_arama_olustur(window, cx);
                    cx.notify();
                }))
                .into_any_element(),
            cx,
        )
    }

    pub(super) fn android_sayfasi(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let girdi = self.android_arama.clone();
        let uad = self.android_uad.clone();
        let arama = girdi
            .as_ref()
            .map(|girdi| girdi.read(cx).value().trim().to_lowercase())
            .unwrap_or_default();
        let toplam = self.android_paketler.len();
        let filtreli = self
            .android_paketler
            .iter()
            .filter(|paket| {
                if arama.is_empty() {
                    return true;
                }
                let aciklama = uad
                    .as_ref()
                    .and_then(|uad| uad.veritabani.bilgi(&paket.id))
                    .map(|bilgi| bilgi.aciklama.to_lowercase())
                    .unwrap_or_default();
                paket.id.to_lowercase().contains(&arama) || aciklama.contains(&arama)
            })
            .take(40)
            .cloned()
            .collect::<Vec<_>>();
        v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Android Yönetimi"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("ADB, sürücü, marka uyarıları ve açık kaynak paket veritabanıyla güvenli bloatware yönetimi."),
                    ),
            )
            .child(self.android_onay_paneli(cx))
            .child(self.android_durum_kutulari(cx))
            .child(self.android_cihaz_karti(cx))
            .child(self.android_surucu_karti(cx))
            .child(self.android_uad_karti(cx))
            .child(self.android_paket_karti(girdi, arama, toplam, filtreli, uad, cx))
            .child(self.android_gecmis_karti(cx))
            .into_any_element()
    }

    pub(super) fn android_arama_olustur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.android_arama.is_none() {
            let girdi = cx
                .new(|cx| InputState::new(window, cx).placeholder("Paket adı veya açıklamada ara"));
            let abonelik = cx.subscribe(&girdi, |_this: &mut Self, _, olay: &InputEvent, cx| {
                if matches!(olay, InputEvent::Change) {
                    cx.notify();
                }
            });
            self.android_arama = Some(girdi);
            self.android_arama_abonelik = Some(abonelik);
        }
        if !self.android_baslangic_yuklendi {
            self.android_baslangic_yuklendi = true;
            self.android_uad_onbellek(cx);
        }
    }

    fn android_arka_plan<T, F, U>(&mut self, cx: &mut Context<Self>, gorev: F, uygula: U)
    where
        T: Send + 'static,
        F: FnOnce() -> sp_core::Sonuc<T> + Send + 'static,
        U: FnOnce(&mut Self, &mut Context<Self>, sp_core::Sonuc<T>) + 'static,
    {
        if self.android_islemde {
            return;
        }
        self.android_islemde = true;
        self.android_hata = None;
        self.android_mesaj = Some("İşlem sürüyor...".to_string());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let sonuc = cx.background_executor().spawn(async move { gorev() }).await;
            let _ = this.update(cx, |this, cx| {
                this.android_islemde = false;
                uygula(this, cx, sonuc);
                cx.notify();
            });
        })
        .detach();
    }

    fn android_durum_kutulari(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_2()
            .child(if let Some(hata) = &self.android_hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &self.android_mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .into_any_element()
    }

    fn android_onay_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(onay) = &self.android_onay else {
            return div().into_any_element();
        };
        let metin = match onay {
            AndroidOnay::Kaldir {
                paket,
                verileri_koru,
            } => format!(
                "{paket} kullanıcı 0 için kaldırılacak. {} Devam edilsin mi?",
                if *verileri_koru {
                    "Uygulama verileri korunur."
                } else {
                    "Uygulama verileri de silinir."
                }
            ),
            AndroidOnay::DevreDisi(paket) => {
                format!("{paket} devre dışı bırakılacak. Devam edilsin mi?")
            }
            AndroidOnay::Etkinlestir(paket) => {
                format!("{paket} yeniden etkinleştirilecek. Devam edilsin mi?")
            }
            AndroidOnay::GeriYukle(paket) => {
                format!("{paket} install-existing ile geri yüklenecek. Devam edilsin mi?")
            }
            AndroidOnay::TopluKaldir(paketler) => format!(
                "{} güvenli paket kullanıcı 0 için kaldırılacak. Devam edilsin mi?",
                paketler.len()
            ),
            AndroidOnay::PlatformIndir => {
                "Resmî Google Platform Tools arşivi indirilip platform-tools klasörüne açılacak. Devam edilsin mi?"
                    .to_string()
            }
            AndroidOnay::UadIndir => {
                "UAD paket veritabanı resmî kaynaktan indirilecek (GPL-3.0) ve önbelleğe alınacak. Devam edilsin mi?"
                    .to_string()
            }
            AndroidOnay::SurucuInf(yol) => {
                format!("{} imzası doğrulanıp pnputil ile kurulacak. Devam edilsin mi?", yol.display())
            }
            AndroidOnay::SamsungKur(yol) => {
                format!("{} sessiz modda çalıştırılacak. Devam edilsin mi?", yol.display())
            }
            AndroidOnay::GoogleSurucu => {
                "Resmî Google USB sürücüsü indirilip imza doğrulamasından sonra kurulacak. Devam edilsin mi?"
                    .to_string()
            }
            AndroidOnay::Libwdi(yol, _, _) => format!(
                "{} ile WinUSB sürücüsü kurulacak. Bu işlem yönetici izni ve imzalı sürücü gerektirir. Devam edilsin mi?",
                yol.display()
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
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("android-onayla")
                            .primary()
                            .label("Onayla")
                            .on_click(cx.listener(|this, _, _, cx| this.android_onayla(cx))),
                    )
                    .child(
                        Button::new("android-vazgec")
                            .label("Vazgeç")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.android_onay = None;
                                cx.notify();
                            })),
                    ),
            )
            .into_any_element()
    }

    fn android_cihaz_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let cihazlar = self.android_cihazlar.clone();
        let secili = self.android_secili_seri.clone();
        let adb = self.android_adb.clone();
        let islemde = self.android_islemde;
        let cihaz_satirlari = cihazlar
            .iter()
            .take(10)
            .map(|cihaz| {
                let seri = cihaz.seri.clone();
                let aktif = secili.as_deref() == Some(cihaz.seri.as_str());
                let alt = format!(
                    "{} · {} · {}",
                    cihaz.durum.metin(),
                    cihaz
                        .marka
                        .clone()
                        .unwrap_or_else(|| "marka bilinmiyor".into()),
                    cihaz
                        .android_surumu
                        .clone()
                        .map(|surum| format!("Android {surum}"))
                        .unwrap_or_else(|| "Android sürümü bilinmiyor".into())
                );
                self.modul_satiri(
                    cihaz.baslik(),
                    alt,
                    cihaz.seri.clone(),
                    vec![
                        Button::new(format!("android-cihaz-{}", Self::kimlik_ozeti(&cihaz.seri)))
                            .label(if aktif { "Seçili" } else { "Seç" })
                            .disabled(islemde || aktif)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.android_cihaz_sec(seri.clone(), cx);
                            }))
                            .into_any_element(),
                    ],
                    cx,
                )
                .into_any_element()
            })
            .collect::<Vec<_>>();
        let secili_cihaz = cihazlar
            .iter()
            .find(|cihaz| secili.as_deref() == Some(cihaz.seri.as_str()))
            .cloned();
        let uyarilar = secili_cihaz
            .as_ref()
            .map(|cihaz| marka_uyarilari(cihaz.marka.as_deref(), cihaz.android_surumu.as_deref()))
            .unwrap_or_default();
        self.bolum_karti(
            "Cihaz ve ADB".to_string(),
            "ADB yolu, USB hata ayıklama durumu ve marka bazlı bağlantı uyarıları.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!(
                            "ADB: {}",
                            adb.as_ref()
                                .map(|yol| yol.display().to_string())
                                .unwrap_or_else(|| "bulunamadı".to_string())
                        )),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("android-cihaz-yenile")
                                .primary()
                                .label(if islemde { "İşlem sürüyor..." } else { "Cihazları Yenile" })
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_cihazlari_yenile(cx))),
                        )
                        .child(
                            Button::new("android-adb-sec")
                                .label("ADB Seç")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_adb_sec(cx))),
                        )
                        .child(
                            Button::new("android-platform-indir")
                                .label("Platform Tools İndir")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.android_onay = Some(AndroidOnay::PlatformIndir);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("android-saf-rust")
                                .label("Saf Rust Testi")
                                .disabled(islemde || secili.is_none())
                                .on_click(cx.listener(|this, _, _, cx| this.android_saf_rust_testi(cx))),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("İlk ADB çalıştırmasında Windows Güvenlik Duvarı adb.exe için izin isteyebilir; yerel hata ayıklama için izin verin."),
                )
                .children(cihaz_satirlari)
                .child(if let Some(cihaz) = &secili_cihaz {
                    v_flex()
                        .gap_1()
                        .p_3()
                        .rounded_md()
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!(
                                    "{} · {}",
                                    cihaz.model.clone().unwrap_or_else(|| cihaz.seri.clone()),
                                    cihaz.durum.metin()
                                )),
                        )
                        .children(uyarilar.into_iter().map(|uyari| {
                            div()
                                .text_xs()
                                .text_color(cx.theme().warning)
                                .child(uyari)
                        }))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("android-paket-tara")
                                        .primary()
                                        .label("Paketleri Tara")
                                        .disabled(islemde || !cihaz.adb_hazir())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.android_paketleri_tara(cx);
                                        })),
                                ),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Cihaz bulunamadıysa USB hata ayıklamayı ve aşağıdaki sürücü durumunu kontrol edin.")
                        .into_any_element()
                })
                .into_any_element(),
            cx,
        )
    }

    fn android_surucu_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let rapor = self.android_surucu.clone();
        let islemde = self.android_islemde;
        let cihazlar = rapor
            .as_ref()
            .map(|rapor| {
                rapor
                    .cihazlar
                    .iter()
                    .take(10)
                    .map(|cihaz| {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} · VID {:04X} PID {:04X} · {} · {}",
                                cihaz.ad, cihaz.vid, cihaz.pid, cihaz.durum, cihaz.surucu
                            ))
                            .into_any_element()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.bolum_karti(
            "Sürücü Durumu".to_string(),
            "Android ADB arayüzü için WinUSB/üretici sürücüsü. Yalnızca imzalı ve doğrulanmış paketler kurulur.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("android-surucu-tara")
                                .label("Durumu Tara")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_surucu_tara(cx))),
                        )
                        .child(
                            Button::new("android-google-surucu")
                                .primary()
                                .label("Google USB Sürücüsü")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.android_onay = Some(AndroidOnay::GoogleSurucu);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("android-inf-sec")
                                .label("INF Seç")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_inf_sec(cx))),
                        )
                        .child(
                            Button::new("android-samsung-sec")
                                .label("Samsung Paketi Seç")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_samsung_sec(cx))),
                        )
                        .child(
                            Button::new("android-libwdi-sec")
                                .label("libwdi Seç")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_libwdi_sec(cx))),
                        ),
                )
                .child(if let Some(rapor) = &rapor {
                    div()
                        .text_sm()
                        .text_color(if rapor.adb_hazir {
                            cx.theme().success
                        } else {
                            cx.theme().warning
                        })
                        .child(if rapor.adb_hazir {
                            "ADB arayüzü hazır."
                        } else {
                            "ADB arayüzü bulunamadı; sürücü kurulumu gerekebilir."
                        })
                        .into_any_element()
                } else {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Sürücü durumu henüz taranmadı.")
                        .into_any_element()
                })
                .children(cihazlar)
                .children(
                    rapor
                        .map(|rapor| rapor.oneriler)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|oneri| {
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(oneri)
                        }),
                )
                .into_any_element(),
            cx,
        )
    }

    fn android_uad_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let uad = self.android_uad.clone();
        let islemde = self.android_islemde;
        self.bolum_karti(
            "Paket Veritabanı".to_string(),
            "Açık kaynak UAD verisi GPL-3.0 altında resmî kaynaktan indirilir; uygulamaya gömülmez. Önbellek çevrimdışı kullanılabilir.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("android-uad-yukle")
                                .label("Önbelleği Yükle")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_uad_onbellek(cx))),
                        )
                        .child(
                            Button::new("android-uad-guncelle")
                                .primary()
                                .label("Resmî Kaynaktan Güncelle")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.android_onay = Some(AndroidOnay::UadIndir);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("android-uad-ice-aktar")
                                .label("JSON İçe Aktar")
                                .disabled(islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_uad_ice_aktar(cx))),
                        ),
                )
                .child(if let Some(uad) = &uad {
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{} paket · güncelleme {} · lisans {}",
                                    uad.veritabani.paketler.len(),
                                    uad.guncelleme,
                                    uad.lisans
                                )),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(uad.dosya.display().to_string()),
                        )
                        .into_any_element()
                } else {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Veritabanı yüklü değil. Önbelleği yükleyin veya resmî kaynaktan güncelleyin.")
                        .into_any_element()
                })
                .into_any_element(),
            cx,
        )
    }

    fn android_paket_karti(
        &self,
        girdi: Option<Entity<InputState>>,
        arama: String,
        toplam: usize,
        filtreli: Vec<PaketDurumu>,
        uad: Option<Arc<UadDurumu>>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let islemde = self.android_islemde;
        let secili_paket = self.android_secili_paket.clone();
        let top_lu = self.android_toplu_mod;
        let secili_sayi = self.android_secili.len();
        let satirlar = filtreli
            .iter()
            .map(|paket| self.android_paket_satiri(paket, uad.as_deref(), cx))
            .collect::<Vec<_>>();
        self.bolum_karti(
            "Paket Kataloğu".to_string(),
            "Risk seviyeleri ve bağımlılıklar UAD verisinden gelir; bilinmeyen paketler uzman onayı ister.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(if let Some(girdi) = &girdi {
                            Input::new(girdi).cleanable(true).w_full().into_any_element()
                        } else {
                            Button::new("android-arama-hazirla")
                                .label("Aramayı Hazırla")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.android_arama_olustur(window, cx);
                                    cx.notify();
                                }))
                                .into_any_element()
                        })
                        .child(
                            Button::new("android-toplu-mod")
                                .label(if top_lu { "Toplu Seçim: Açık" } else { "Toplu Seçim" })
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.android_toplu_mod = !this.android_toplu_mod;
                                    this.android_secili.clear();
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "{} paketten {} tanesi gösteriliyor{}",
                                    toplam,
                                    satirlar.len(),
                                    if arama.is_empty() { "" } else { " · filtreli" }
                                )),
                        )
                        .child(if top_lu {
                            Button::new("android-toplu-kaldir")
                                .danger()
                                .label(format!("Seçilenleri Kaldır ({secili_sayi})"))
                                .disabled(secili_sayi == 0 || islemde)
                                .on_click(cx.listener(|this, _, _, cx| this.android_toplu_onay(cx)))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        }),
                )
                .children(satirlar)
                .child(self.android_paket_detay(secili_paket, uad, cx))
                .into_any_element(),
            cx,
        )
    }

    fn android_paket_satiri(
        &self,
        paket: &PaketDurumu,
        uad: Option<&UadDurumu>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let bilgi = uad.and_then(|uad| uad.veritabani.bilgi(&paket.id));
        let risk = bilgi
            .map(|bilgi| bilgi.risk)
            .unwrap_or(AndroidRisk::Bilinmiyor);
        let liste = bilgi
            .map(|bilgi| bilgi.liste.as_str())
            .unwrap_or("Bilinmiyor");
        let aciklama = bilgi
            .map(|bilgi| bilgi.aciklama.clone())
            .unwrap_or_default();
        let durum = if paket.kaldirilmis {
            "Kaldırılmış"
        } else if paket.devre_disi {
            "Devre dışı"
        } else if paket.sistem {
            "Sistem"
        } else {
            "Kullanıcı"
        };
        let alt = format!("{} · {} · {}", durum, liste, risk.metin());
        let detay = Self::android_kisalt(&aciklama, 180);
        if self.android_toplu_mod {
            let guvenli = risk.toplu_islem_uygun() && !paket.kaldirilmis;
            let secili = self.android_secili.contains(&paket.id);
            let eylem = if guvenli {
                let kutu = self.android_secim_kutusu(paket.id.clone(), secili, cx);
                vec![kutu.into_any_element()]
            } else {
                vec![div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Toplu işlem dışı")
                    .into_any_element()]
            };
            return self
                .modul_satiri(paket.id.clone(), alt, detay, eylem, cx)
                .into_any_element();
        }
        let kimlik = paket.id.clone();
        let secili = self.android_secili_paket.as_deref() == Some(paket.id.as_str());
        self.modul_satiri(
            paket.id.clone(),
            alt,
            detay,
            vec![
                Button::new(format!("android-paket-{}", Self::kimlik_ozeti(&kimlik)))
                    .label(if secili { "Seçili" } else { "İncele" })
                    .disabled(secili)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.android_secili_paket = Some(kimlik.clone());
                        cx.notify();
                    }))
                    .into_any_element(),
            ],
            cx,
        )
        .into_any_element()
    }

    fn android_paket_detay(
        &self,
        secili: Option<String>,
        uad: Option<Arc<UadDurumu>>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(id) = secili else {
            return div().into_any_element();
        };
        let Some(paket) = self
            .android_paketler
            .iter()
            .find(|paket| paket.id == id)
            .cloned()
        else {
            return div().into_any_element();
        };
        let bilgi = uad
            .as_ref()
            .and_then(|uad| uad.veritabani.bilgi(&paket.id))
            .cloned();
        let risk = bilgi
            .as_ref()
            .map(|bilgi| bilgi.risk)
            .unwrap_or(AndroidRisk::Bilinmiyor);
        let aciklama = bilgi
            .as_ref()
            .map(|bilgi| bilgi.aciklama.clone())
            .filter(|aciklama| !aciklama.is_empty())
            .unwrap_or_else(|| "Bu paket açık kaynak veritabanında bulunamadı.".to_string());
        let uyarilar = bilgi
            .as_ref()
            .map(|bilgi| {
                sp_core::android::bagimlilik_uyarilari(
                    bilgi,
                    &self
                        .android_paketler
                        .iter()
                        .map(|paket| paket.id.clone())
                        .collect(),
                )
            })
            .unwrap_or_default();
        let islemde = self.android_islemde;
        let uzman = self.android_uzman_onayi;
        let engelli = risk == AndroidRisk::Unsafe || islemde;
        let uzman_gerekli = risk.uzman_onayi_gerekir();
        let paket_id = paket.id.clone();
        let paket_id2 = paket.id.clone();
        let paket_id3 = paket.id.clone();
        let paket_id4 = paket.id.clone();
        self.bolum_karti(
            format!("{} · {}", paket.id, risk.metin()),
            format!(
                "{}{}",
                if paket.sistem {
                    "Sistem paketi. "
                } else {
                    "Kullanıcı paketi. "
                },
                if paket.kaldirilmis {
                    "Kaldırılmış; geri yüklenebilir."
                } else if paket.devre_disi {
                    "Devre dışı."
                } else {
                    "Etkin."
                }
            ),
            v_flex()
                .gap_2()
                .child(div().text_sm().child(aciklama))
                .children(
                    uyarilar
                        .into_iter()
                        .map(|uyari| div().text_xs().text_color(cx.theme().warning).child(uyari)),
                )
                .child(if uzman_gerekli {
                    self.android_uzman_kutusu(cx).into_any_element()
                } else {
                    div().into_any_element()
                })
                .child(if paket.kaldirilmis {
                    Button::new("android-geri-yukle")
                        .primary()
                        .label("Geri Yükle")
                        .disabled(islemde)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.android_onay = Some(AndroidOnay::GeriYukle(paket_id4.clone()));
                            cx.notify();
                        }))
                        .into_any_element()
                } else {
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .child(
                            Button::new("android-kaldir-koru")
                                .danger()
                                .label("Kaldır · Verileri Koru")
                                .disabled(engelli || (uzman_gerekli && !uzman))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.android_onay = Some(AndroidOnay::Kaldir {
                                        paket: paket_id.clone(),
                                        verileri_koru: true,
                                    });
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("android-kaldir-sil")
                                .danger()
                                .label("Kaldır · Verileri Sil")
                                .disabled(engelli || (uzman_gerekli && !uzman))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.android_onay = Some(AndroidOnay::Kaldir {
                                        paket: paket_id2.clone(),
                                        verileri_koru: false,
                                    });
                                    cx.notify();
                                })),
                        )
                        .child(if paket.devre_disi {
                            Button::new("android-etkinlestir")
                                .label("Etkinleştir")
                                .disabled(islemde)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.android_onay =
                                        Some(AndroidOnay::Etkinlestir(paket_id3.clone()));
                                    cx.notify();
                                }))
                                .into_any_element()
                        } else {
                            Button::new("android-devre-disi")
                                .danger()
                                .label("Devre Dışı Bırak")
                                .disabled(engelli || (uzman_gerekli && !uzman))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.android_onay =
                                        Some(AndroidOnay::DevreDisi(paket_id3.clone()));
                                    cx.notify();
                                }))
                                .into_any_element()
                        })
                        .into_any_element()
                })
                .into_any_element(),
            cx,
        )
    }

    fn android_gecmis_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.android_gecmis.is_empty() {
            return div().into_any_element();
        }
        self.bolum_karti(
            "Android İşlem Geçmişi".to_string(),
            "Son işlemler ve geri yükleme ipuçları.".to_string(),
            v_flex()
                .gap_1()
                .children(self.android_gecmis.iter().rev().take(10).map(|kayit| {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(kayit.clone())
                }))
                .into_any_element(),
            cx,
        )
    }

    fn android_uzman_kutusu(&self, cx: &mut Context<Self>) -> Checkbox {
        let uygulama = cx.entity();
        Checkbox::new("android-uzman-onayi")
            .checked(self.android_uzman_onayi)
            .label("Uzman onayı: riskli paket değişikliğini anladım.")
            .on_change(move |deger, _, cx| {
                uygulama.update(cx, |this, cx| {
                    this.android_uzman_onayi = *deger;
                    cx.notify();
                });
            })
    }

    fn android_secim_kutusu(
        &self,
        paket: String,
        secili: bool,
        cx: &mut Context<Self>,
    ) -> Checkbox {
        let uygulama = cx.entity();
        Checkbox::new(format!("android-secim-{}", Self::kimlik_ozeti(&paket)))
            .checked(secili)
            .accessibility_label(format!("{paket} seç"))
            .on_change(move |deger, _, cx| {
                let paket = paket.clone();
                uygulama.update(cx, |this, cx| {
                    if *deger {
                        this.android_secili.insert(paket.clone());
                    } else {
                        this.android_secili.remove(&paket);
                    }
                    cx.notify();
                });
            })
    }

    fn android_kisalt(metin: &str, azami: usize) -> String {
        let temiz = metin.replace(['\r', '\n'], " ");
        if temiz.chars().count() <= azami {
            return temiz;
        }
        format!("{}…", temiz.chars().take(azami).collect::<String>())
    }

    fn android_cihazlari_yenile(&mut self, cx: &mut Context<Self>) {
        let Some(adb) = self
            .android_adb
            .clone()
            .or_else(|| self.android_servisi.adb_yolu())
        else {
            self.android_hata = Some("ADB bulunamadı. ADB Seç veya Platform Tools İndir.".into());
            cx.notify();
            return;
        };
        self.android_adb = Some(adb.clone());
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.cihazlari_tara(&adb),
            |this, _cx, sonuc| match sonuc {
                Ok(cihazlar) => {
                    this.android_mesaj = Some(format!("{} cihaz bulundu.", cihazlar.len()));
                    let secili_kaldi = this
                        .android_secili_seri
                        .as_ref()
                        .is_some_and(|seri| cihazlar.iter().any(|cihaz| &cihaz.seri == seri));
                    if !secili_kaldi {
                        this.android_secili_seri = cihazlar.first().map(|cihaz| cihaz.seri.clone());
                        this.android_paketler.clear();
                        this.android_secili_paket = None;
                    }
                    this.android_cihazlar = cihazlar;
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_cihaz_sec(&mut self, seri: String, cx: &mut Context<Self>) {
        self.android_secili_seri = Some(seri);
        self.android_paketler.clear();
        self.android_secili_paket = None;
        self.android_secili.clear();
        self.android_paketleri_tara(cx);
    }

    fn android_paketleri_tara(&mut self, cx: &mut Context<Self>) {
        let (Some(adb), Some(seri)) = (self.android_adb.clone(), self.android_secili_seri.clone())
        else {
            self.android_hata = Some("Önce ADB ve cihaz seçin.".into());
            cx.notify();
            return;
        };
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.paketleri_tara(&adb, &seri),
            |this, _, sonuc| match sonuc {
                Ok(paketler) => {
                    this.android_mesaj = Some(format!("{} paket bulundu.", paketler.len()));
                    this.android_paketler = paketler;
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_surucu_tara(&mut self, cx: &mut Context<Self>) {
        let servis = self.surucu_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.durumu_tara(),
            |this, _, sonuc| match sonuc {
                Ok(rapor) => {
                    this.android_mesaj = Some(format!(
                        "{} USB cihazı bulundu; ADB {}.",
                        rapor.cihazlar.len(),
                        if rapor.adb_hazir {
                            "hazır"
                        } else {
                            "hazır değil"
                        }
                    ));
                    this.android_surucu = Some(rapor);
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_adb_sec(&mut self, cx: &mut Context<Self>) {
        let Some(adb) = rfd::FileDialog::new()
            .add_filter("adb.exe", &["exe"])
            .set_title("ADB çalıştırılabilir dosyasını seçin")
            .pick_file()
        else {
            return;
        };
        self.android_adb = Some(adb.clone());
        let mut ayarlar = self.ayarlar.clone();
        ayarlar.adb_yolu = Some(adb.display().to_string());
        match ayarlar.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            Ok(()) => {
                self.ayarlar = ayarlar;
                self.android_mesaj = Some("ADB yolu kaydedildi.".into());
                self.android_hata = None;
            }
            Err(hata) => self.android_hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn android_saf_rust_testi(&mut self, cx: &mut Context<Self>) {
        let (Some(adb), Some(seri)) = (self.android_adb.clone(), self.android_secili_seri.clone())
        else {
            self.android_hata = Some("Saf Rust testi için ADB ve cihaz gerekli.".into());
            cx.notify();
            return;
        };
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.saf_rust_kabuk(&adb, &seri, "getprop ro.product.brand"),
            |this, _, sonuc| match sonuc {
                Ok(cikti) => {
                    this.android_mesaj = Some(format!("Saf Rust ADB yanıtı: {}", cikti.trim()));
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_uad_onbellek(&mut self, cx: &mut Context<Self>) {
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.uad_onbellek_yukle(),
            |this, _, sonuc| match sonuc {
                Ok(Some(uad)) => {
                    this.android_mesaj = Some(format!(
                        "Paket veritabanı önbelleği yüklendi: {} paket.",
                        uad.veritabani.paketler.len()
                    ));
                    this.android_uad = Some(Arc::new(uad));
                }
                Ok(None) => {
                    this.android_mesaj = Some("Paket veritabanı önbelleği yok.".into());
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_uad_ice_aktar(&mut self, cx: &mut Context<Self>) {
        let Some(dosya) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("UAD paket veritabanını seçin")
            .pick_file()
        else {
            return;
        };
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.uad_yerel_yukle(&dosya),
            |this, _, sonuc| match sonuc {
                Ok(uad) => {
                    this.android_mesaj = Some(format!(
                        "{} paket içe aktarıldı.",
                        uad.veritabani.paketler.len()
                    ));
                    this.android_uad = Some(Arc::new(uad));
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_inf_sec(&mut self, cx: &mut Context<Self>) {
        let Some(inf) = rfd::FileDialog::new()
            .add_filter("INF", &["inf"])
            .set_title("İmzalı Android sürücü INF dosyasını seçin")
            .pick_file()
        else {
            return;
        };
        self.android_onay = Some(AndroidOnay::SurucuInf(inf));
        cx.notify();
    }

    fn android_samsung_sec(&mut self, cx: &mut Context<Self>) {
        let Some(exe) = rfd::FileDialog::new()
            .add_filter("EXE", &["exe"])
            .set_title("Samsung Android USB Driver kurulumunu seçin")
            .pick_file()
        else {
            return;
        };
        self.android_onay = Some(AndroidOnay::SamsungKur(exe));
        cx.notify();
    }

    fn android_libwdi_sec(&mut self, cx: &mut Context<Self>) {
        let Some(arac) = rfd::FileDialog::new()
            .add_filter("wdi-simple.exe", &["exe"])
            .set_title("libwdi wdi-simple aracını seçin")
            .pick_file()
        else {
            return;
        };
        let cihaz = self
            .android_surucu
            .as_ref()
            .and_then(|rapor| {
                rapor
                    .cihazlar
                    .iter()
                    .find(|cihaz| {
                        cihaz.adb_arayuzu || sp_core::android::vid_marka(cihaz.vid).is_some()
                    })
                    .or_else(|| rapor.cihazlar.first())
            })
            .cloned();
        let Some(cihaz) = cihaz else {
            self.android_hata = Some("libwdi için önce sürücü durumunu tarayın.".into());
            cx.notify();
            return;
        };
        self.android_onay = Some(AndroidOnay::Libwdi(arac, cihaz.vid, cihaz.pid));
        cx.notify();
    }

    fn android_toplu_onay(&mut self, cx: &mut Context<Self>) {
        let uad = self.android_uad.clone();
        let paketler = self
            .android_secili
            .iter()
            .filter(|id| {
                uad.as_ref()
                    .and_then(|uad| uad.veritabani.bilgi(id))
                    .map(|bilgi| bilgi.risk.toplu_islem_uygun())
                    .unwrap_or(false)
            })
            .cloned()
            .collect::<Vec<_>>();
        if paketler.is_empty() {
            self.android_hata = Some("Toplu işlem için güvenli paket seçilmedi.".into());
            cx.notify();
            return;
        }
        self.android_onay = Some(AndroidOnay::TopluKaldir(paketler));
        cx.notify();
    }

    fn android_onayla(&mut self, cx: &mut Context<Self>) {
        let Some(onay) = self.android_onay.take() else {
            return;
        };
        match onay {
            AndroidOnay::Kaldir {
                paket,
                verileri_koru,
            } => self.android_paket_uygula(paket, PaketEylemi::Kaldir { verileri_koru }, cx),
            AndroidOnay::DevreDisi(paket) => {
                self.android_paket_uygula(paket, PaketEylemi::DevreDisi, cx)
            }
            AndroidOnay::Etkinlestir(paket) => {
                self.android_paket_uygula(paket, PaketEylemi::Etkinlestir, cx)
            }
            AndroidOnay::GeriYukle(paket) => {
                self.android_paket_uygula(paket, PaketEylemi::GeriYukle, cx)
            }
            AndroidOnay::TopluKaldir(paketler) => self.android_toplu_kaldir(paketler, cx),
            AndroidOnay::PlatformIndir => self.android_platform_indir(cx),
            AndroidOnay::UadIndir => self.android_uad_indir(cx),
            AndroidOnay::SurucuInf(yol) => self.android_surucu_kur(yol, SurucuKurulum::Inf, cx),
            AndroidOnay::SamsungKur(yol) => {
                self.android_surucu_kur(yol, SurucuKurulum::Samsung, cx)
            }
            AndroidOnay::GoogleSurucu => self.android_google_surucu(cx),
            AndroidOnay::Libwdi(yol, vid, pid) => self.android_libwdi_kur(yol, vid, pid, cx),
        }
    }

    fn android_paket_uygula(&mut self, paket: String, eylem: PaketEylemi, cx: &mut Context<Self>) {
        let (Some(adb), Some(seri)) = (self.android_adb.clone(), self.android_secili_seri.clone())
        else {
            self.android_hata = Some("ADB ve cihaz seçilmedi.".into());
            cx.notify();
            return;
        };
        let risk = self
            .android_uad
            .as_ref()
            .and_then(|uad| uad.veritabani.bilgi(&paket))
            .map(|bilgi| bilgi.risk)
            .unwrap_or(AndroidRisk::Bilinmiyor);
        let uzman = self.android_uzman_onayi;
        let servis = self.android_servisi.clone();
        let etiket = paket.clone();
        self.android_arka_plan(
            cx,
            move || servis.paket_eylemi(&adb, &seri, &paket, eylem, risk, uzman),
            move |this, cx, sonuc| match sonuc {
                Ok(_) => {
                    this.android_mesaj = Some(format!("{etiket}: işlem tamamlandı."));
                    this.android_gecmis
                        .push(format!("{etiket}: {}.", android_eylem_metni(eylem)));
                    this.android_paketleri_tara(cx);
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_toplu_kaldir(&mut self, paketler: Vec<String>, cx: &mut Context<Self>) {
        let (Some(adb), Some(seri)) = (self.android_adb.clone(), self.android_secili_seri.clone())
        else {
            self.android_hata = Some("ADB ve cihaz seçilmedi.".into());
            cx.notify();
            return;
        };
        let servis = self.android_servisi.clone();
        let uad = self.android_uad.clone();
        self.android_arka_plan(
            cx,
            move || {
                let mut basarili = 0usize;
                let mut hatalar = Vec::new();
                for paket in &paketler {
                    let risk = uad
                        .as_ref()
                        .and_then(|uad| uad.veritabani.bilgi(paket))
                        .map(|bilgi| bilgi.risk)
                        .unwrap_or(AndroidRisk::Bilinmiyor);
                    if !risk.toplu_islem_uygun() {
                        hatalar.push(format!("{paket}: güvenli olmayan risk."));
                        continue;
                    }
                    match servis.paket_eylemi(
                        &adb,
                        &seri,
                        paket,
                        PaketEylemi::Kaldir {
                            verileri_koru: true,
                        },
                        risk,
                        false,
                    ) {
                        Ok(_) => basarili += 1,
                        Err(hata) => hatalar.push(format!("{paket}: {hata}")),
                    }
                }
                Ok((basarili, hatalar))
            },
            move |this, cx, sonuc| match sonuc {
                Ok((basarili, hatalar)) => {
                    this.android_mesaj = Some(format!(
                        "{} paket kaldırıldı, {} hata.",
                        basarili,
                        hatalar.len()
                    ));
                    this.android_hata = if hatalar.is_empty() {
                        None
                    } else {
                        Some(hatalar.join("\n"))
                    };
                    this.android_secili.clear();
                    this.android_gecmis
                        .push(format!("Toplu kaldırma: {basarili} paket."));
                    this.android_paketleri_tara(cx);
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_platform_indir(&mut self, cx: &mut Context<Self>) {
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.platform_tools_indir(),
            |this, _, sonuc| match sonuc {
                Ok(adb) => {
                    this.android_adb = Some(adb.clone());
                    this.android_mesaj = Some(format!("Platform Tools hazır: {}", adb.display()));
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_uad_indir(&mut self, cx: &mut Context<Self>) {
        let servis = self.android_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.uad_indir(),
            |this, _, sonuc| match sonuc {
                Ok(uad) => {
                    this.android_mesaj = Some(format!(
                        "Paket veritabanı güncellendi: {} paket.",
                        uad.veritabani.paketler.len()
                    ));
                    this.android_uad = Some(Arc::new(uad));
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_surucu_kur(&mut self, yol: PathBuf, tur: SurucuKurulum, cx: &mut Context<Self>) {
        let servis = self.surucu_servisi.clone();
        self.android_arka_plan(
            cx,
            move || match tur {
                SurucuKurulum::Inf => servis.inf_kur(&yol),
                SurucuKurulum::Samsung => servis.samsung_kur(&yol),
            },
            |this, _, sonuc| match sonuc {
                Ok(sonuc) => {
                    this.android_mesaj = Some(format!("{} · imza: {}", sonuc.mesaj, sonuc.imza));
                    this.android_hata = None;
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_google_surucu(&mut self, cx: &mut Context<Self>) {
        let hedef = self
            .dizinler
            .yerel
            .join("android")
            .join("google-usb-driver.zip");
        let servis = self.surucu_servisi.clone();
        if let Some(anne) = hedef.parent() {
            let _ = std::fs::create_dir_all(anne);
        }
        self.android_arka_plan(
            cx,
            move || servis.google_usb_indir_kur(&hedef),
            |this, _, sonuc| match sonuc {
                Ok(sonuc) => {
                    this.android_mesaj = Some(format!("{} · imza: {}", sonuc.mesaj, sonuc.imza));
                    this.android_hata = None;
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }

    fn android_libwdi_kur(&mut self, arac: PathBuf, vid: u16, pid: u16, cx: &mut Context<Self>) {
        let servis = self.surucu_servisi.clone();
        self.android_arka_plan(
            cx,
            move || servis.libwdi_kur(&arac, vid, pid),
            |this, _, sonuc| match sonuc {
                Ok(sonuc) => {
                    this.android_mesaj = Some(format!("{} · imza: {}", sonuc.mesaj, sonuc.imza));
                    this.android_hata = None;
                }
                Err(hata) => this.android_hata = Some(hata.to_string()),
            },
        );
    }
}

#[derive(Clone, Copy)]
enum SurucuKurulum {
    Inf,
    Samsung,
}

fn android_eylem_metni(eylem: PaketEylemi) -> &'static str {
    match eylem {
        PaketEylemi::Kaldir { .. } => "kaldırıldı",
        PaketEylemi::DevreDisi => "devre dışı bırakıldı",
        PaketEylemi::Etkinlestir => "etkinleştirildi",
        PaketEylemi::GeriYukle => "geri yüklendi",
    }
}
