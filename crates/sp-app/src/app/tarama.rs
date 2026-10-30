use super::*;

use std::collections::VecDeque;
use std::time::Instant;

const YOKLAMA_ARALIGI: Duration = Duration::from_millis(350);
const VARSAYILAN_ZAMAN_ASIMI_MS: u64 = 180_000;

#[derive(Debug, Clone, Copy)]
enum TaramaAdimi {
    Winget,
    WingetKurulu,
    Kaldirici,
    SistemYonetimi,
    Donanim,
    Temizlik,
    OnarimKategorileri,
    ChromeEski,
    SorunGiderme,
    KayitTemizlik,
    Kisayol,
    BaglamMenu,
    GuncellemeGecmisi,
    GeriNoktalari,
    BakimAppx,
    BakimOzellik,
    BakimSurucu,
    MerkezGuncelleme,
    MerkezUzanti,
    MerkezBaglanti,
}

impl TaramaAdimi {
    const SIRA: [TaramaAdimi; 20] = [
        Self::Winget,
        Self::WingetKurulu,
        Self::Kaldirici,
        Self::SistemYonetimi,
        Self::Donanim,
        Self::Temizlik,
        Self::OnarimKategorileri,
        Self::ChromeEski,
        Self::SorunGiderme,
        Self::KayitTemizlik,
        Self::Kisayol,
        Self::BaglamMenu,
        Self::GuncellemeGecmisi,
        Self::GeriNoktalari,
        Self::BakimAppx,
        Self::BakimOzellik,
        Self::BakimSurucu,
        Self::MerkezGuncelleme,
        Self::MerkezUzanti,
        Self::MerkezBaglanti,
    ];

    fn metin(self) -> &'static str {
        match self {
            Self::Winget => "Winget güncellemeleri",
            Self::WingetKurulu => "Kurulu winget paketleri",
            Self::Kaldirici => "Kurulu uygulamalar",
            Self::SistemYonetimi => "Sistem yönetimi",
            Self::Donanim => "Donanım bilgileri",
            Self::Temizlik => "Geçici dosyalar",
            Self::OnarimKategorileri => "Onarım kategorileri",
            Self::ChromeEski => "Eski Chrome sürümleri",
            Self::SorunGiderme => "Sorun denetimi",
            Self::KayitTemizlik => "Kayıt defteri artıkları",
            Self::Kisayol => "Bozuk kısayollar",
            Self::BaglamMenu => "Bağlam menüsü kayıtları",
            Self::GuncellemeGecmisi => "Windows güncelleme geçmişi",
            Self::GeriNoktalari => "Geri yükleme noktaları",
            Self::BakimAppx => "Appx paketleri",
            Self::BakimOzellik => "Windows özellikleri",
            Self::BakimSurucu => "Sürücüler",
            Self::MerkezGuncelleme => "Windows güncellemeleri",
            Self::MerkezUzanti => "Tarayıcı uzantıları",
            Self::MerkezBaglanti => "Etkin bağlantılar",
        }
    }

    fn calistir(self, app: &mut WinNexusApp, cx: &mut Context<WinNexusApp>) {
        match self {
            Self::Winget => app.winget_tara(cx),
            Self::WingetKurulu => app.winget_kurulu_tara(cx),
            Self::Kaldirici => app.kaldirici_tara(cx),
            Self::SistemYonetimi => app.sistem_yonetimi_tara(cx),
            Self::Donanim => app.donanim_tara(cx),
            Self::Temizlik => app.temizlik_tara(cx),
            Self::OnarimKategorileri => app.onarim_kategorileri_tara(cx),
            Self::ChromeEski => app.chrome_eski_tara(cx),
            Self::SorunGiderme => app.sorun_denetle(cx),
            Self::KayitTemizlik => app.kayit_temizlik_tara(cx),
            Self::Kisayol => app.kisayol_tara(cx),
            Self::BaglamMenu => app.baglam_menu_tara(cx),
            Self::GuncellemeGecmisi => app.guncelleme_gecmisi_yukle(cx),
            Self::GeriNoktalari => app.geri_noktalari_yukle(cx),
            Self::BakimAppx => app.bakim_appx_listele(cx),
            Self::BakimOzellik => app.bakim_ozellik_listele(cx),
            Self::BakimSurucu => app.bakim_surucu_listele(cx),
            Self::MerkezGuncelleme => app.merkez_guncelleme_tara(cx),
            Self::MerkezUzanti => app.merkez_uzanti_tara(cx),
            Self::MerkezBaglanti => app.merkez_baglanti_tara(cx),
        }
    }

    fn suruyor_mu(self, app: &WinNexusApp) -> bool {
        match self {
            Self::Winget | Self::WingetKurulu => app.winget_taraniyor,
            Self::Kaldirici => app.kaldirici_taraniyor || app.kaldirici_islemde,
            Self::SistemYonetimi => app.sistem_taraniyor,
            Self::Donanim => app.donanim_taraniyor,
            Self::Temizlik | Self::OnarimKategorileri | Self::ChromeEski => app.temizlik_taraniyor,
            Self::SorunGiderme => app.sorun_denetleniyor,
            Self::KayitTemizlik | Self::Kisayol => app.kayit_taraniyor,
            Self::BaglamMenu => app.baglam_taraniyor,
            Self::GuncellemeGecmisi
            | Self::GeriNoktalari
            | Self::BakimAppx
            | Self::BakimOzellik
            | Self::BakimSurucu => app.bakim_taraniyor,
            Self::MerkezGuncelleme | Self::MerkezUzanti | Self::MerkezBaglanti => {
                app.merkez_islemde
            }
        }
    }

    fn hata_metni(self, app: &WinNexusApp) -> Option<&str> {
        let hata = match self {
            Self::Winget | Self::WingetKurulu => app.winget_hata.as_deref(),
            Self::Kaldirici => app.kaldirici_hata.as_deref(),
            Self::SistemYonetimi => app.sistem_hata.as_deref(),
            Self::Donanim => app.donanim_hata.as_deref(),
            Self::Temizlik | Self::OnarimKategorileri | Self::ChromeEski => {
                app.temizlik_hata.as_deref()
            }
            Self::SorunGiderme => app.sorun_hata.as_deref(),
            Self::KayitTemizlik | Self::Kisayol => app.kayit_hata.as_deref(),
            Self::BaglamMenu => app.baglam_hata.as_deref(),
            Self::GuncellemeGecmisi
            | Self::GeriNoktalari
            | Self::BakimAppx
            | Self::BakimOzellik
            | Self::BakimSurucu => app.bakim_hata.as_deref(),
            Self::MerkezGuncelleme | Self::MerkezUzanti | Self::MerkezBaglanti => {
                app.merkez_hata.as_deref()
            }
        };
        hata.filter(|hata| !hata.trim().is_empty())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TumTaramaDurumu {
    pub etkin: bool,
    pub mesaj: Option<String>,
    pub hata: Option<String>,
    kuyruk: VecDeque<TaramaAdimi>,
    aktif: Option<TaramaAdimi>,
    tamamlanan: usize,
    toplam: usize,
    zaman_asimi_ms: u64,
    adim_baslangici: Option<Instant>,
    hatalar: Vec<String>,
}

impl WinNexusApp {
    pub(crate) fn tum_tarama_baslat(&mut self, cx: &mut Context<Self>) {
        if self.tarama_suruyor() {
            self.tarama.mesaj = Some("Tarama zaten sürüyor.".to_string());
            cx.notify();
            return;
        }
        self.tarama.etkin = true;
        self.tarama.hata = None;
        self.tarama.hatalar.clear();
        self.tarama.kuyruk = VecDeque::from(TaramaAdimi::SIRA);
        let toplam = self.tarama.kuyruk.len();
        self.tarama.toplam = toplam;
        self.tarama.tamamlanan = 0;
        self.tarama.aktif = None;
        self.tarama.adim_baslangici = None;
        self.tarama.zaman_asimi_ms = VARSAYILAN_ZAMAN_ASIMI_MS;
        self.tarama.mesaj = Some("Tüm sistem taranıyor...".to_string());
        self.durumlari_yenile();
        self.sonraki_adimi_baslat(cx);
        Self::izleyici_baslat(cx);
        cx.notify();
    }

    pub(crate) fn tarama_suruyor(&self) -> bool {
        self.tarama.aktif.is_some()
            || self.merkez_islemde
            || self.winget_taraniyor
            || self.kaldirici_taraniyor
            || self.kaldirici_islemde
            || self.sistem_taraniyor
            || self.temizlik_taraniyor
            || self.kayit_taraniyor
            || self.baglam_taraniyor
            || self.bakim_taraniyor
            || self.donanim_taraniyor
            || self.sorun_denetleniyor
    }

    pub(crate) fn tarama_ozeti(&self) -> String {
        let uygulanan = self
            .durumlar
            .values()
            .filter(|durum| **durum == KuralDurumu::Uygulandi)
            .count();
        let bekleyen = self
            .durumlar
            .values()
            .filter(|durum| **durum == KuralDurumu::Uygulanmadi)
            .count();
        let sayilar = [
            ("kural", self.kurallar.len()),
            ("uygulanan", uygulanan),
            ("bekleyen", bekleyen),
            ("güncelleme", self.winget_guncellemeler.len()),
            ("uygulama", self.kaldirici_uygulamalar.len()),
            ("servis", self.servisler.len()),
            ("süreç", self.surecler.len()),
            ("disk", self.diskler.len()),
            ("temizlik hedefi", self.temizlik_hedefleri.len()),
            ("temizlik taraması", self.temizlik_taramalari.len()),
            ("Appx paketi", self.appx_paketleri.len()),
            ("sürücü", self.suruculer.len()),
        ];
        let parcalar = sayilar
            .into_iter()
            .filter(|(_, sayi)| *sayi > 0)
            .map(|(etiket, sayi)| format!("{sayi} {etiket}"))
            .collect::<Vec<_>>();
        if parcalar.is_empty() {
            return "Henüz tarama sonucu yok.".to_string();
        }
        parcalar.join(" · ")
    }

    fn sonraki_adimi_baslat(&mut self, cx: &mut Context<Self>) {
        let Some(ad) = self.tarama.kuyruk.pop_front() else {
            self.tarama_bitir(cx);
            return;
        };
        let sira = self.tarama.tamamlanan + 1;
        let toplam = self.tarama.toplam;
        self.tarama.mesaj = Some(format!("Taranıyor: {sira}/{toplam} · {}", ad.metin()));
        self.tarama.aktif = Some(ad);
        self.tarama.adim_baslangici = Some(Instant::now());
        ad.calistir(self, cx);
        cx.notify();
    }

    fn tarama_bitir(&mut self, cx: &mut Context<Self>) {
        self.tarama.aktif = None;
        self.tarama.adim_baslangici = None;
        if self.tarama.hatalar.is_empty() {
            let toplam = self.tarama.toplam;
            self.tarama.hata = None;
            self.tarama.mesaj = Some(format!("Tüm sistem taraması tamamlandı ({toplam} adım)."));
        } else {
            let hata_sayisi = self.tarama.hatalar.len();
            let birlesik = self.tarama.hatalar.join("\n");
            self.tarama.hata = Some(birlesik);
            self.tarama.mesaj = Some(format!(
                "Tarama tamamlandı; {hata_sayisi} adımda hata oluştu."
            ));
        }
        cx.notify();
    }

    fn izleyici_baslat(cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(YOKLAMA_ARALIGI).await;
            let devam = this
                .update(cx, |this, cx| this.tarama_ilerlemesini_isle(cx))
                .unwrap_or(false);
            if !devam {
                break;
            }
        })
        .detach();
    }

    fn tarama_ilerlemesini_isle(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(ad) = self.tarama.aktif else {
            return false;
        };
        if ad.suruyor_mu(self) {
            if !self.zaman_asimi_gecti() {
                return true;
            }
            self.tarama
                .hatalar
                .push(format!("{}: zaman aşımı", ad.metin()));
        } else {
            self.adim_hatasini_kaydet();
        }
        self.tarama.tamamlanan += 1;
        self.tarama.aktif = None;
        self.tarama.adim_baslangici = None;
        self.sonraki_adimi_baslat(cx);
        cx.notify();
        self.tarama.aktif.is_some()
    }

    fn zaman_asimi_gecti(&self) -> bool {
        let Some(baslangic) = self.tarama.adim_baslangici else {
            return false;
        };
        baslangic.elapsed().as_millis() >= u128::from(self.tarama.zaman_asimi_ms)
    }

    fn adim_hatasini_kaydet(&mut self) {
        let Some(ad) = self.tarama.aktif else {
            return;
        };
        let hata = match ad.hata_metni(self) {
            Some(hata) => hata.to_string(),
            None => return,
        };
        let ilk_satir = hata
            .lines()
            .find(|satir| !satir.trim().is_empty())
            .unwrap_or(hata.as_str())
            .trim()
            .to_string();
        self.tarama
            .hatalar
            .push(format!("{}: {ilk_satir}", ad.metin()));
    }
}
