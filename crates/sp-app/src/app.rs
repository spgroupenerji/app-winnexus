use crate::SURUM;
use gpui_kit::component::button::*;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState, Textarea, TextareaState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use sp_core::adaptor::AdaptorRef;
use sp_core::android::{
    marka_uyarilari, AndroidCihaz, PaketDurumu, PaketEylemi, Risk as AndroidRisk,
};
use sp_core::artik::{ArtikOgesi, StorePaketi};
use sp_core::ayarlar::{Ayarlar, Tema};
use sp_core::bakim::{AppxPaketi, OzellikOgesi, SurucuOgesi};
use sp_core::bulucu::{DiskAnalizi, DosyaBulgu, KayitBulgu};
use sp_core::dizinler::Dizinler;
use sp_core::donanim::{
    CpuBilgisi, DiskBilgisi, GpuBilgisi, RamBilgisi, RdpDurumu, SistemOzeti, UacDurumu, WlanProfili,
};
use sp_core::gelismis_arac::{Baglanti, Guncelleme, TarayiciUzantisi};
use sp_core::gunluk::{Gunluk, GunlukKaydi};
use sp_core::kaldirici::KuruluUygulama;
use sp_core::kayit_kisayol::{BaglamOgesi, KayitDefteriBulgu, KisayolBulgu};
use sp_core::kural_yukleyici::gomulu_kurallari_yukle;
use sp_core::kurulum_izleme::{IzlemeOturumu, IzlemeRaporu};
use sp_core::model::{Kural, KuralDurumu, Risk};
use sp_core::motor::{IsMotoru, IsPlani};
use sp_core::onarim::{
    GeriYuklemeNoktasi, GuncellemeKaydi, KategoriTarama, SORUN_GIDERICI_KIMLIKLERI,
};
use sp_core::sistem_yonetimi::{BaslangicOgesi, GorevOgesi, ServisOgesi, SurecOgesi};
use sp_core::sorun_giderme::{ExecutionPolicyDurumu, MmAgentDurumu, NpmDurumu, PolicyKapsami};
use sp_core::temizleyici::{TemizlikHedefi, TemizlikTarama, YinelenenGrup};
use sp_core::winget::{
    gomulu_profilleri_yukle, liste_metni_ayristir, onerileri_sirala, paket_kimligi_gecerli,
    AramaSonucu, GuncellemeSatiri, Kaynak, PinKaydi, PinTuru, Profil, ProfilDurum,
    ProfilTaramaSonucu, WingetPaketi,
};
use sp_core::zorla::ZorlaSonucu;
use sp_win::android::{AndroidServisi, UadDurumu};
use sp_win::android_driver::{SurucuRaporu, SurucuServisi};
use sp_win::artik::ArtikServisi;
use sp_win::bakim::BakimServisi;
use sp_win::bulucu::BulucuServisi;
use sp_win::donanim::DonanimServisi;
use sp_win::gelismis_arac::GelismisAracServisi;
use sp_win::kaldirici::KaldiriciServisi;
use sp_win::kayit_kisayol::KayitKisayolServisi;
use sp_win::kurulum_izleme::KurulumIzlemeServisi;
use sp_win::onarim::OnarimServisi;
use sp_win::sistem_yonetimi::SistemYonetimiServisi;
use sp_win::sorun_giderme::SorunGidermeServisi;
use sp_win::temizleyici::TemizleyiciServisi;
use sp_win::winget::{KomutCalistirici, WindowsKomutCalistirici, WingetAyarlari, WingetServisi};
use sp_win::zorla::ZorlaServisi;
use sp_win::{sistem_bilgisi, WindowsAdaptoru};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

mod android;
mod arac_merkezi;
mod gezinme;
mod tarama;

#[cfg(test)]
mod testleri;

const YASAL_UYARI: &str = include_str!("../../../docs/yasal-uyari.txt");

const DOKUNMA_HEDEFI_PX: f32 = 44.;
const UST_CUBUK_YUKSEKLIGI_PX: f32 = 60.;
const BASLIK_CUBUGU_YUKSEKLIGI_PX: f32 = 52.;
const DONANIM_ETIKET_GENISLIGI_PX: f32 = 110.;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sayfa {
    GenelBakis,
    Donanim,
    Gecmis,
    InceAyarlar,
    SistemYonetimi,
    BaglantiYoneticisi,
    Temizlik,
    DiskAnalisti,
    Bakim,
    WindowsGuncelleme,
    SorunGiderme,
    Kaldirici,
    Winget,
    ZorlaKaldir,
    KurulumIzleme,
    TarayiciUzantilari,
    Araclar,
    DosyaKayitBulucu,
    GodMode,
    Android,
    Ayarlar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bolum {
    Panel,
    Sistem,
    Temizlik,
    Bakim,
    Uygulamalar,
    Araclar,
    Android,
}

impl Bolum {
    const HEPSI: [Bolum; 7] = [
        Bolum::Panel,
        Bolum::Sistem,
        Bolum::Temizlik,
        Bolum::Bakim,
        Bolum::Uygulamalar,
        Bolum::Araclar,
        Bolum::Android,
    ];

    fn baslik(self) -> &'static str {
        match self {
            Self::Panel => "Panel",
            Self::Sistem => "Sistem",
            Self::Temizlik => "Temizlik",
            Self::Bakim => "Bakım",
            Self::Uygulamalar => "Uygulamalar",
            Self::Araclar => "Araçlar",
            Self::Android => "Android",
        }
    }

    fn kimlik(self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::Sistem => "sistem",
            Self::Temizlik => "temizlik",
            Self::Bakim => "bakim",
            Self::Uygulamalar => "uygulamalar",
            Self::Araclar => "araclar",
            Self::Android => "android",
        }
    }

    fn ikon(self) -> IconName {
        match self {
            Self::Panel => IconName::LayoutDashboard,
            Self::Sistem => IconName::Settings2,
            Self::Temizlik => IconName::RefreshCw,
            Self::Bakim => IconName::Heart,
            Self::Uygulamalar => IconName::Inbox,
            Self::Araclar => IconName::Palette,
            Self::Android => IconName::Bot,
        }
    }

    fn sayfalar(self) -> &'static [Sayfa] {
        match self {
            Self::Panel => &[Sayfa::GenelBakis, Sayfa::Donanim, Sayfa::Gecmis],
            Self::Sistem => &[
                Sayfa::InceAyarlar,
                Sayfa::SistemYonetimi,
                Sayfa::BaglantiYoneticisi,
            ],
            Self::Temizlik => &[Sayfa::Temizlik, Sayfa::DiskAnalisti],
            Self::Bakim => &[Sayfa::Bakim, Sayfa::WindowsGuncelleme, Sayfa::SorunGiderme],
            Self::Uygulamalar => &[
                Sayfa::Kaldirici,
                Sayfa::Winget,
                Sayfa::ZorlaKaldir,
                Sayfa::KurulumIzleme,
                Sayfa::TarayiciUzantilari,
            ],
            Self::Araclar => &[Sayfa::Araclar, Sayfa::DosyaKayitBulucu, Sayfa::GodMode],
            Self::Android => &[Sayfa::Android],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AramaModu {
    Dosya,
    Kayit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MerkezOnay {
    ZorlaKaldir(String),
    GuncellemeKaldir(String),
    UzantiKaldir(String),
    BaglantiSonlandir(u32, String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum KaldiriciOnay {
    Kaldir {
        kimlik: String,
        sessiz: bool,
    },
    TopluKaldir {
        kimlikler: Vec<String>,
        sessiz: bool,
    },
    GeriNoktasi,
}

#[derive(Debug, Clone, Copy)]
enum IslemAlani {
    Sistem,
    Temizlik,
    Artik,
    Store,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SistemOnay {
    BaslangicEtkinlestir(String),
    BaslangicDevreDisi(String),
    ServisBaslat(String),
    ServisDurdur(String),
    ServisBaslangic(String, String),
    GorevCalistir(String, String),
    GorevDurum(String, String, bool),
    SurecSonlandir(u32, String),
    Temizlik(String),
    BosKlasorSil(Vec<String>),
    YinelenenKarantina(Vec<String>),
    Parcala(String),
    GeriDonusum,
    DismAnaliz,
    DismTemizle,
    DismSaglik,
    DismOnar,
    AppxKaldir(String, String),
    OzellikAyarla(String, bool),
    SurucuDisaAktar(String, String),
    ArtikKarantina(Vec<String>),
    StoreKaldir(String, String),
    GelismisTemizlik(String),
    ChromeEskiSil(Vec<String>),
    Sfc,
    DismResetBase,
    AppxOnar(String, String),
    GuncellemeGecmisiTemizle,
    EskiGeriNoktalariSil,
    GeriNoktasiOlustur,
    GeriDon(u32, String),
    GeriKoruma(bool),
    IkonOnbellekOnar,
    KayitTemizle(Vec<String>),
    KisayolTemizle(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KayitSecim {
    Kayit,
    Kisayol,
}

struct TopluKaldirmaSonucu {
    basarili: Vec<String>,
    hatalar: Vec<String>,
    kayitlar: Vec<String>,
    artik_tasinan: u64,
    yeni_liste: Option<Vec<KuruluUygulama>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WingetOnay {
    Guncelle(String),
    Asla(String),
    SurumSabitle(String, String),
    TumunuGuncelle,
    EksikleriKur,
    KaynakKaldir(String),
    SeciliKur(Vec<String>),
    SeciliKaldir(Vec<String>),
    SeciliGuncelle(Vec<String>),
    ListeKur(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WingetSekme {
    Guncellemeler,
    Magaza,
    Kurulu,
    Kaynaklar,
    Liste,
    Profiller,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WingetSecim {
    Guncelleme,
    Magaza,
    Kurulu,
    Onerilen,
    IceAktarilan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AracOnay {
    ExplorerYenidenBaslat,
    BiosYenidenBaslat,
    SahiplikEkle,
    SahiplikKaldir,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SorunOnay {
    NpmPathEkle,
    NpmPathGeriAl(String),
    PolicyCurrentUser,
    PolicyLocalMachine,
    MmAgentKapat,
    MmAgentAc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AndroidOnay {
    Kaldir { paket: String, verileri_koru: bool },
    DevreDisi(String),
    Etkinlestir(String),
    GeriYukle(String),
    TopluKaldir(Vec<String>),
    PlatformIndir,
    UadIndir,
    SurucuInf(PathBuf),
    SamsungKur(PathBuf),
    GoogleSurucu,
    Libwdi(PathBuf, u16, u16),
}

impl Sayfa {
    const HEPSI: [Sayfa; 21] = [
        Sayfa::GenelBakis,
        Sayfa::Donanim,
        Sayfa::Gecmis,
        Sayfa::InceAyarlar,
        Sayfa::SistemYonetimi,
        Sayfa::BaglantiYoneticisi,
        Sayfa::Temizlik,
        Sayfa::DiskAnalisti,
        Sayfa::Bakim,
        Sayfa::WindowsGuncelleme,
        Sayfa::SorunGiderme,
        Sayfa::Kaldirici,
        Sayfa::Winget,
        Sayfa::ZorlaKaldir,
        Sayfa::KurulumIzleme,
        Sayfa::TarayiciUzantilari,
        Sayfa::Araclar,
        Sayfa::DosyaKayitBulucu,
        Sayfa::GodMode,
        Sayfa::Android,
        Sayfa::Ayarlar,
    ];

    fn baslik(self) -> &'static str {
        match self {
            Self::GenelBakis => "Genel Bakış",
            Self::Donanim => "Donanım",
            Self::Gecmis => "Geçmiş",
            Self::InceAyarlar => "İnce Ayarlar",
            Self::SistemYonetimi => "Sistem Yönetimi",
            Self::BaglantiYoneticisi => "Bağlantı Yöneticisi",
            Self::Temizlik => "Temizlik",
            Self::DiskAnalisti => "Disk Analisti",
            Self::Bakim => "Bakım",
            Self::WindowsGuncelleme => "Windows Güncelleme Yöneticisi",
            Self::SorunGiderme => "Sorun Giderme",
            Self::Kaldirici => "Kaldırıcı",
            Self::Winget => "Winget Merkezi",
            Self::ZorlaKaldir => "Zorla Kaldır",
            Self::KurulumIzleme => "Kurulum İzleme",
            Self::TarayiciUzantilari => "Tarayıcı Uzantıları",
            Self::Araclar => "Araç Kutusu",
            Self::DosyaKayitBulucu => "Dosya ve Kayıt Bulucu",
            Self::GodMode => "God Mode",
            Self::Android => "Android Yönetimi",
            Self::Ayarlar => "Ayarlar",
        }
    }

    fn kimlik(self) -> &'static str {
        match self {
            Self::GenelBakis => "genel-bakis",
            Self::Donanim => "donanim",
            Self::Gecmis => "gecmis",
            Self::InceAyarlar => "ince-ayarlar",
            Self::SistemYonetimi => "sistem-yonetimi",
            Self::BaglantiYoneticisi => "baglanti-yoneticisi",
            Self::Temizlik => "temizlik",
            Self::DiskAnalisti => "disk-analisti",
            Self::Bakim => "bakim",
            Self::WindowsGuncelleme => "windows-guncelleme",
            Self::SorunGiderme => "sorun-giderme",
            Self::Kaldirici => "kaldirici",
            Self::Winget => "winget",
            Self::ZorlaKaldir => "zorla-kaldir",
            Self::KurulumIzleme => "kurulum-izleme",
            Self::TarayiciUzantilari => "tarayici-uzantilari",
            Self::Araclar => "araclar",
            Self::DosyaKayitBulucu => "dosya-kayit-bulucu",
            Self::GodMode => "god-mode",
            Self::Android => "android",
            Self::Ayarlar => "ayarlar",
        }
    }

    fn aciklama(self) -> &'static str {
        match self {
            Self::GenelBakis => "Sistem durumu, donanım özeti ve tüm modüllerin giriş noktası.",
            Self::Donanim => "İşlemci, ekran kartı, bellek, disk sağlığı ve ağ profilleri.",
            Self::Gecmis => "Uygulanan işlemlerin günlük kayıtları ve sonuçları.",
            Self::InceAyarlar => {
                "Gizlilik, Explorer, telemetri ve görev çubuğu için yerleşik ince ayarlar."
            }
            Self::SistemYonetimi => {
                "Başlangıç öğeleri, servisler, zamanlanmış görevler ve süreçler."
            }
            Self::BaglantiYoneticisi => {
                "Etkin TCP/UDP bağlantılarını süreç bilgisiyle listeler."
            }
            Self::Temizlik => {
                "Geçici dosyalar, tarayıcı önbellekleri, boş klasörler ve yinelenen dosyalar."
            }
            Self::DiskAnalisti => {
                "Klasör ve dosya boyutlarını en büyükten küçüğe analiz eder."
            }
            Self::Bakim => {
                "DISM, SFC, Appx onarımı, sürücü dışa aktarma ve geri yükleme noktaları."
            }
            Self::WindowsGuncelleme => {
                "Kurulu Windows güncellemelerini listeler ve kaldırılabilir olanları kaldırır."
            }
            Self::SorunGiderme => {
                "npm yolu, PowerShell ilkesi ve bellek sıkıştırma ayarlarının denetimi."
            }
            Self::Kaldirici => {
                "Kurulu masaüstü uygulamalarını kaldırın, artıkları karantinaya alın."
            }
            Self::Winget => {
                "Paket araması, güncellemeler, kaynaklar, listeler ve hazır profiller."
            }
            Self::ZorlaKaldir => {
                "Kaldırma komutu çalışmayan uygulamaların klasörünü karantinaya alır ve kayıt anahtarını yedekleyerek siler."
            }
            Self::KurulumIzleme => {
                "Kurulum öncesi ve sonrası dosya/kayıt anlık görüntüsünü karşılaştırarak değişiklikleri raporlar."
            }
            Self::TarayiciUzantilari => {
                "Chrome, Edge, Brave, Vivaldi, Yandex, Opera ve Firefox uzantılarını listeler ve karantinaya alır."
            }
            Self::Araclar => "Explorer, BIOS, sahiplik menüsü ve klasör kısayolları.",
            Self::DosyaKayitBulucu => {
                "Dosya adlarında ve kayıt defterinde joker destekli arama yapar."
            }
            Self::GodMode => {
                "Windows denetim masası görevlerinin tamamını tek pencerede açın."
            }
            Self::Android => "ADB cihazları, sürücü kurulumu ve UAD-ng ile paket yönetimi.",
            Self::Ayarlar => "Sistem taraması, tercihler, veri dizini ve sürüm bilgisi.",
        }
    }

    fn bolum(self) -> Option<Bolum> {
        match self {
            Self::GenelBakis | Self::Donanim | Self::Gecmis => Some(Bolum::Panel),
            Self::InceAyarlar | Self::SistemYonetimi | Self::BaglantiYoneticisi => {
                Some(Bolum::Sistem)
            }
            Self::Temizlik | Self::DiskAnalisti => Some(Bolum::Temizlik),
            Self::Bakim | Self::WindowsGuncelleme | Self::SorunGiderme => Some(Bolum::Bakim),
            Self::Kaldirici
            | Self::Winget
            | Self::ZorlaKaldir
            | Self::KurulumIzleme
            | Self::TarayiciUzantilari => Some(Bolum::Uygulamalar),
            Self::Araclar | Self::DosyaKayitBulucu | Self::GodMode => Some(Bolum::Araclar),
            Self::Android => Some(Bolum::Android),
            Self::Ayarlar => None,
        }
    }

    fn ikon(self) -> IconName {
        match self {
            Self::GenelBakis => IconName::LayoutDashboard,
            Self::Donanim => IconName::Cpu,
            Self::Gecmis => IconName::Undo2,
            Self::InceAyarlar => IconName::Settings2,
            Self::SistemYonetimi => IconName::SquareTerminal,
            Self::BaglantiYoneticisi => IconName::Network,
            Self::Temizlik => IconName::RefreshCw,
            Self::DiskAnalisti => IconName::HardDrive,
            Self::Bakim => IconName::Heart,
            Self::WindowsGuncelleme => IconName::RotateCw,
            Self::SorunGiderme => IconName::CircleAlert,
            Self::Kaldirici => IconName::Delete,
            Self::Winget => IconName::Inbox,
            Self::ZorlaKaldir => IconName::Ban,
            Self::KurulumIzleme => IconName::Inspector,
            Self::TarayiciUzantilari => IconName::Globe,
            Self::Araclar => IconName::Palette,
            Self::DosyaKayitBulucu => IconName::Search,
            Self::GodMode => IconName::Frame,
            Self::Android => IconName::Bot,
            Self::Ayarlar => IconName::Settings,
        }
    }

    fn arac_aramasi_gerekli(self) -> bool {
        matches!(
            self,
            Self::WindowsGuncelleme
                | Self::TarayiciUzantilari
                | Self::DosyaKayitBulucu
                | Self::ZorlaKaldir
                | Self::BaglantiYoneticisi
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KuralKategorisi {
    GorevCubugu,
    Gezgin,
    Gizlilik,
    Oneriler,
    Sistem,
}

impl KuralKategorisi {
    const HEPSI: [KuralKategorisi; 5] = [
        KuralKategorisi::GorevCubugu,
        KuralKategorisi::Gezgin,
        KuralKategorisi::Gizlilik,
        KuralKategorisi::Oneriler,
        KuralKategorisi::Sistem,
    ];

    fn baslik(self) -> &'static str {
        match self {
            Self::GorevCubugu => "Görev çubuğu ve Başlat",
            Self::Gezgin => "Dosya Gezgini",
            Self::Gizlilik => "Gizlilik ve telemetri",
            Self::Oneriler => "Öneriler ve reklamlar",
            Self::Sistem => "Sistem davranışı",
        }
    }

    fn ikon(self) -> IconName {
        match self {
            Self::GorevCubugu => IconName::PanelBottom,
            Self::Gezgin => IconName::Folder,
            Self::Gizlilik => IconName::EyeOff,
            Self::Oneriler => IconName::Star,
            Self::Sistem => IconName::Cpu,
        }
    }
}

fn kural_kategorisi(kural: &Kural) -> KuralKategorisi {
    let id = kural.id.as_str();
    if id.starts_with("gorev_cubugu.") {
        return KuralKategorisi::GorevCubugu;
    }
    if id.starts_with("gezgin.") {
        return KuralKategorisi::Gezgin;
    }
    if id.starts_with("gizlilik.") {
        return KuralKategorisi::Gizlilik;
    }
    if let Some(kalan) = id.strip_prefix("optimizasyon.opt.") {
        if let Ok(sira) = kalan.parse::<u32>() {
            return match sira {
                1..=8 | 23..=24 => KuralKategorisi::GorevCubugu,
                9..=13 => KuralKategorisi::Oneriler,
                15..=22 => KuralKategorisi::Gezgin,
                _ => KuralKategorisi::Sistem,
            };
        }
    }
    KuralKategorisi::Sistem
}

fn surum_etiketi() -> String {
    format!("beta v{SURUM}")
}

pub struct WinNexusApp {
    sayfa: Sayfa,
    merkez_sorgu: Option<Entity<InputState>>,
    merkez_sorgu_abonelik: Option<Subscription>,
    merkez_kok: Option<PathBuf>,
    merkez_islemde: bool,
    merkez_mesaj: Option<String>,
    merkez_hata: Option<String>,
    merkez_arama_modu: AramaModu,
    merkez_onay: Option<MerkezOnay>,
    gelismis_servis: Arc<GelismisAracServisi>,
    bulucu_servisi: Arc<BulucuServisi>,
    guncellemeler: Vec<Guncelleme>,
    tarayici_uzantilari: Vec<TarayiciUzantisi>,
    baglantilar: Vec<Baglanti>,
    dosya_bulgulari: Vec<DosyaBulgu>,
    kayit_bulgulari: Vec<KayitBulgu>,
    disk_analizi: Option<DiskAnalizi>,
    izleme_servisi: Arc<KurulumIzlemeServisi>,
    zorla_servisi: Arc<ZorlaServisi>,
    izleme_oturumu: Option<IzlemeOturumu>,
    izleme_raporu: Option<IzlemeRaporu>,
    zorla_uygulamalar: Vec<KuruluUygulama>,
    zorla_sonucu: Option<ZorlaSonucu>,
    kurallar: Vec<Kural>,
    durumlar: HashMap<String, KuralDurumu>,
    secili: Option<String>,
    onay: Option<IsPlani>,
    son_plan: Option<IsPlani>,
    mesaj: Option<String>,
    hata: Option<String>,
    motor: IsMotoru,
    gunluk: Arc<Gunluk>,
    ayarlar: Ayarlar,
    dizinler: Dizinler,
    winget: WingetServisi,
    profiller: Vec<Profil>,
    winget_guncellemeler: Vec<GuncellemeSatiri>,
    winget_pinler: Vec<PinKaydi>,
    winget_komutlar: Vec<String>,
    winget_mesaj: Option<String>,
    winget_hata: Option<String>,
    winget_taraniyor: bool,
    winget_atla: HashSet<String>,
    winget_profil_sonuclari: Vec<ProfilTaramaSonucu>,
    winget_onay: Option<WingetOnay>,
    winget_sekme: WingetSekme,
    winget_kaynaklar: Vec<Kaynak>,
    winget_kaynak_adi: Option<Entity<InputState>>,
    winget_kaynak_adresi: Option<Entity<InputState>>,
    winget_magaza_arama: Option<Entity<InputState>>,
    winget_magaza_kaynak: String,
    winget_magaza_sonuclari: Vec<AramaSonucu>,
    winget_magaza_secili: HashSet<String>,
    winget_kurulu: Vec<WingetPaketi>,
    winget_kurulu_secili: HashSet<String>,
    winget_guncelleme_secili: HashSet<String>,
    winget_onerilen_secili: HashSet<String>,
    winget_liste_girdi: Option<Entity<TextareaState>>,
    winget_ice_aktarilan: Vec<WingetPaketi>,
    winget_ice_aktarilan_secili: HashSet<String>,
    arac_onay: Option<AracOnay>,
    kaldirici: KaldiriciServisi,
    kaldirici_uygulamalar: Vec<KuruluUygulama>,
    kaldirici_arama: Option<Entity<InputState>>,
    kaldirici_abonelik: Option<Subscription>,
    kaldirici_taraniyor: bool,
    kaldirici_islemde: bool,
    kaldirici_mesaj: Option<String>,
    kaldirici_hata: Option<String>,
    kaldirici_onay: Option<KaldiriciOnay>,
    kaldirici_secili: HashSet<String>,
    kaldirici_toplu_mod: bool,
    kaldirici_gecmis: Vec<String>,
    sistem_yonetimi: Arc<SistemYonetimiServisi>,
    baslangic_ogeleri: Vec<BaslangicOgesi>,
    servisler: Vec<ServisOgesi>,
    gorevler: Vec<GorevOgesi>,
    surecler: Vec<SurecOgesi>,
    sistem_taraniyor: bool,
    sistem_mesaj: Option<String>,
    sistem_hata: Option<String>,
    sistem_onay: Option<SistemOnay>,
    temizleyici: Arc<TemizleyiciServisi>,
    temizlik_hedefleri: Vec<TemizlikHedefi>,
    temizlik_taramalari: HashMap<String, TemizlikTarama>,
    temizlik_mesaj: Option<String>,
    temizlik_hata: Option<String>,
    temizlik_taraniyor: bool,
    bos_klasor_kok: Option<PathBuf>,
    bos_klasorler: Vec<String>,
    yinelenen_kok: Option<PathBuf>,
    yinelenenler: Vec<YinelenenGrup>,
    bakim: Arc<BakimServisi>,
    onarim: Arc<OnarimServisi>,
    onarim_taramalari: Vec<KategoriTarama>,
    chrome_eski: Vec<String>,
    guncelleme_gecmisi: Vec<GuncellemeKaydi>,
    geri_yukleme_noktalari: Vec<GeriYuklemeNoktasi>,
    kayit_kisayol: Arc<KayitKisayolServisi>,
    kayit_temizlik_bulgulari: Vec<KayitDefteriBulgu>,
    kayit_temizlik_secili: HashSet<String>,
    kisayol_bulgulari: Vec<KisayolBulgu>,
    kisayol_secili: HashSet<String>,
    kayit_taraniyor: bool,
    kayit_mesaj: Option<String>,
    kayit_hata: Option<String>,
    baglam_ogeleri: Vec<BaglamOgesi>,
    baglam_klasik: Option<bool>,
    baglam_taraniyor: bool,
    baglam_mesaj: Option<String>,
    baglam_hata: Option<String>,
    dism_ciktisi: Option<String>,
    appx_paketleri: Vec<AppxPaketi>,
    ozellikler: Vec<OzellikOgesi>,
    suruculer: Vec<SurucuOgesi>,
    bakim_taraniyor: bool,
    bakim_mesaj: Option<String>,
    bakim_hata: Option<String>,
    donanim: Arc<DonanimServisi>,
    sistem_ozeti: Option<SistemOzeti>,
    cpu_bilgisi: Vec<CpuBilgisi>,
    gpu_bilgisi: Vec<GpuBilgisi>,
    ram_bilgisi: Option<RamBilgisi>,
    diskler: Vec<DiskBilgisi>,
    wlan_profilleri: Vec<WlanProfili>,
    uac_durumu: Option<UacDurumu>,
    rdp_durumu: Option<RdpDurumu>,
    donanim_taraniyor: bool,
    donanim_mesaj: Option<String>,
    donanim_hata: Option<String>,
    artik: Arc<ArtikServisi>,
    artik_uygulama: Option<KuruluUygulama>,
    artik_ogeler: Vec<ArtikOgesi>,
    artik_taraniyor: bool,
    artik_mesaj: Option<String>,
    artik_hata: Option<String>,
    store_paketleri: Vec<StorePaketi>,
    store_taraniyor: bool,
    store_mesaj: Option<String>,
    store_hata: Option<String>,
    sorun_servisi: SorunGidermeServisi,
    sorun_npm: Option<NpmDurumu>,
    sorun_policy: Option<ExecutionPolicyDurumu>,
    sorun_mmagent: Option<MmAgentDurumu>,
    sorun_mesaj: Option<String>,
    sorun_hata: Option<String>,
    sorun_denetleniyor: bool,
    sorun_onay: Option<SorunOnay>,
    sorun_gecmis: Vec<String>,
    npm_onceki_path: Option<String>,
    android_servisi: Arc<AndroidServisi>,
    surucu_servisi: Arc<SurucuServisi>,
    android_adb: Option<PathBuf>,
    android_cihazlar: Vec<AndroidCihaz>,
    android_secili_seri: Option<String>,
    android_paketler: Vec<PaketDurumu>,
    android_uad: Option<Arc<UadDurumu>>,
    android_surucu: Option<SurucuRaporu>,
    android_secili_paket: Option<String>,
    android_uzman_onayi: bool,
    android_onay: Option<AndroidOnay>,
    android_islemde: bool,
    android_mesaj: Option<String>,
    android_hata: Option<String>,
    android_arama: Option<Entity<InputState>>,
    android_arama_abonelik: Option<Subscription>,
    android_toplu_mod: bool,
    android_secili: HashSet<String>,
    android_gecmis: Vec<String>,
    android_baslangic_yuklendi: bool,
    tarama: tarama::TumTaramaDurumu,
    baslik_cubugu_karanlik: Option<bool>,
    yasal_onay_isaretli: bool,
    yasal_hata: Option<String>,
}

impl WinNexusApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let mut hata = None;
        let dizinler = match Dizinler::varsayilan() {
            Ok(dizinler) => dizinler,
            Err(baslangic_hatasi) => {
                hata = Some(baslangic_hatasi.to_string());
                Dizinler::yeni(PathBuf::from("."), PathBuf::from("."))
            }
        };
        if let Err(olusturma_hatasi) = dizinler.olustur() {
            hata = Some(olusturma_hatasi.to_string());
        }
        let ayar_dosyasi = dizinler.ayarlar_dosyasi();
        if !ayar_dosyasi.exists() {
            if let Some(eski) = Dizinler::eski_ayar_dosyasi().filter(|dosya| dosya.is_file()) {
                let _ = std::fs::copy(eski, &ayar_dosyasi);
            }
        }
        let test_modu = cfg!(debug_assertions) && std::env::args().any(|arg| arg == "--test");
        let mut ayarlar = Ayarlar::yukle(&ayar_dosyasi).unwrap_or_else(|ayar_hatasi| {
            hata = Some(ayar_hatasi.to_string());
            Ayarlar::default()
        });
        if test_modu {
            ayarlar.yasal_uyari_kabul_edildi = true;
        }
        let gunluk = Arc::new(Gunluk::yeni(dizinler.gunluk_dosyasi()));
        let adaptor: AdaptorRef = Arc::new(WindowsAdaptoru);
        let motor = IsMotoru::yeni(adaptor, gunluk.clone(), sistem_bilgisi());
        let kurallar = match gomulu_kurallari_yukle() {
            Ok(kurallar) => kurallar,
            Err(kural_hatasi) => {
                hata = Some(kural_hatasi.to_string());
                Vec::new()
            }
        };
        let mut durumlar = HashMap::new();
        for kural in &kurallar {
            durumlar.insert(kural.id.clone(), motor.durum(kural));
        }
        let winget = WingetServisi::yeni(
            Arc::new(WindowsKomutCalistirici),
            WingetAyarlari {
                lisans_onayi: ayarlar.winget_lisans_onayi,
                kaynak_onayi: ayarlar.winget_kaynak_onayi,
            },
        );
        let profiller = match gomulu_profilleri_yukle() {
            Ok(profiller) => profiller,
            Err(profil_hatasi) => {
                hata = Some(profil_hatasi.to_string());
                Vec::new()
            }
        };
        let sistem_yonetimi = Arc::new(SistemYonetimiServisi::yeni(
            Arc::new(WindowsKomutCalistirici),
            dizinler.yerel.join("sistem_yonetimi"),
        ));
        let temizleyici = Arc::new(TemizleyiciServisi::yeni());
        let temizlik_hedefleri = temizleyici.varsayilan_hedefler();
        let bakim = Arc::new(BakimServisi::yeni(Arc::new(WindowsKomutCalistirici)));
        let donanim = Arc::new(DonanimServisi::yeni(Arc::new(WindowsKomutCalistirici)));
        let artik = Arc::new(ArtikServisi::yeni(Arc::new(WindowsKomutCalistirici)));
        let izleme_koku = dizinler.yerel.join("kurulum-izleme");
        let zorla_karantina = dizinler.karantina_dizini();
        let android_servisi = Arc::new(AndroidServisi::yeni(
            Arc::new(WindowsKomutCalistirici),
            dizinler.yerel.join("android"),
        ));
        let surucu_servisi = Arc::new(SurucuServisi::yeni(Arc::new(WindowsKomutCalistirici)));
        let android_adb = ayarlar
            .adb_yolu
            .as_ref()
            .map(PathBuf::from)
            .filter(|yol| yol.is_file())
            .or_else(|| android_servisi.adb_yolu());

        Self {
            sayfa: Sayfa::GenelBakis,
            merkez_sorgu: None,
            merkez_sorgu_abonelik: None,
            merkez_kok: None,
            merkez_islemde: false,
            merkez_mesaj: None,
            merkez_hata: None,
            merkez_arama_modu: AramaModu::Dosya,
            merkez_onay: None,
            gelismis_servis: Arc::new(GelismisAracServisi::yeni(Arc::new(WindowsKomutCalistirici))),
            bulucu_servisi: Arc::new(BulucuServisi::yeni()),
            guncellemeler: Vec::new(),
            tarayici_uzantilari: Vec::new(),
            baglantilar: Vec::new(),
            dosya_bulgulari: Vec::new(),
            kayit_bulgulari: Vec::new(),
            disk_analizi: None,
            izleme_servisi: Arc::new(KurulumIzlemeServisi::yeni(izleme_koku)),
            zorla_servisi: Arc::new(ZorlaServisi::yeni(
                Arc::new(WindowsKomutCalistirici),
                zorla_karantina,
            )),
            izleme_oturumu: None,
            izleme_raporu: None,
            zorla_uygulamalar: Vec::new(),
            zorla_sonucu: None,
            kurallar,
            durumlar,
            secili: None,
            onay: None,
            son_plan: None,
            mesaj: None,
            hata,
            motor,
            gunluk,
            ayarlar,
            dizinler,
            winget,
            profiller,
            winget_guncellemeler: Vec::new(),
            winget_pinler: Vec::new(),
            winget_komutlar: Vec::new(),
            winget_mesaj: None,
            winget_hata: None,
            winget_taraniyor: false,
            winget_atla: HashSet::new(),
            winget_profil_sonuclari: Vec::new(),
            winget_onay: None,
            winget_sekme: WingetSekme::Guncellemeler,
            winget_kaynaklar: Vec::new(),
            winget_kaynak_adi: None,
            winget_kaynak_adresi: None,
            winget_magaza_arama: None,
            winget_magaza_kaynak: "winget".to_string(),
            winget_magaza_sonuclari: Vec::new(),
            winget_magaza_secili: HashSet::new(),
            winget_kurulu: Vec::new(),
            winget_kurulu_secili: HashSet::new(),
            winget_guncelleme_secili: HashSet::new(),
            winget_onerilen_secili: HashSet::new(),
            winget_liste_girdi: None,
            winget_ice_aktarilan: Vec::new(),
            winget_ice_aktarilan_secili: HashSet::new(),
            arac_onay: None,
            kaldirici: KaldiriciServisi::yeni(Arc::new(WindowsKomutCalistirici)),
            kaldirici_uygulamalar: Vec::new(),
            kaldirici_arama: None,
            kaldirici_abonelik: None,
            kaldirici_taraniyor: false,
            kaldirici_islemde: false,
            kaldirici_mesaj: None,
            kaldirici_hata: None,
            kaldirici_onay: None,
            kaldirici_secili: HashSet::new(),
            kaldirici_toplu_mod: false,
            kaldirici_gecmis: Vec::new(),
            sistem_yonetimi,
            baslangic_ogeleri: Vec::new(),
            servisler: Vec::new(),
            gorevler: Vec::new(),
            surecler: Vec::new(),
            sistem_taraniyor: false,
            sistem_mesaj: None,
            sistem_hata: None,
            sistem_onay: None,
            temizleyici,
            temizlik_hedefleri,
            temizlik_taramalari: HashMap::new(),
            temizlik_mesaj: None,
            temizlik_hata: None,
            temizlik_taraniyor: false,
            bos_klasor_kok: None,
            bos_klasorler: Vec::new(),
            yinelenen_kok: None,
            yinelenenler: Vec::new(),
            bakim,
            onarim: Arc::new(OnarimServisi::yeni(Arc::new(WindowsKomutCalistirici))),
            onarim_taramalari: Vec::new(),
            chrome_eski: Vec::new(),
            guncelleme_gecmisi: Vec::new(),
            geri_yukleme_noktalari: Vec::new(),
            kayit_kisayol: Arc::new(KayitKisayolServisi::yeni(Arc::new(WindowsKomutCalistirici))),
            kayit_temizlik_bulgulari: Vec::new(),
            kayit_temizlik_secili: HashSet::new(),
            kisayol_bulgulari: Vec::new(),
            kisayol_secili: HashSet::new(),
            kayit_taraniyor: false,
            kayit_mesaj: None,
            kayit_hata: None,
            baglam_ogeleri: Vec::new(),
            baglam_klasik: None,
            baglam_taraniyor: false,
            baglam_mesaj: None,
            baglam_hata: None,
            dism_ciktisi: None,
            appx_paketleri: Vec::new(),
            ozellikler: Vec::new(),
            suruculer: Vec::new(),
            bakim_taraniyor: false,
            bakim_mesaj: None,
            bakim_hata: None,
            donanim,
            sistem_ozeti: None,
            cpu_bilgisi: Vec::new(),
            gpu_bilgisi: Vec::new(),
            ram_bilgisi: None,
            diskler: Vec::new(),
            wlan_profilleri: Vec::new(),
            uac_durumu: None,
            rdp_durumu: None,
            donanim_taraniyor: false,
            donanim_mesaj: None,
            donanim_hata: None,
            artik,
            artik_uygulama: None,
            artik_ogeler: Vec::new(),
            artik_taraniyor: false,
            artik_mesaj: None,
            artik_hata: None,
            store_paketleri: Vec::new(),
            store_taraniyor: false,
            store_mesaj: None,
            store_hata: None,
            sorun_servisi: SorunGidermeServisi::yeni(Arc::new(WindowsKomutCalistirici)),
            sorun_npm: None,
            sorun_policy: None,
            sorun_mmagent: None,
            sorun_mesaj: None,
            sorun_hata: None,
            sorun_denetleniyor: false,
            sorun_onay: None,
            sorun_gecmis: Vec::new(),
            npm_onceki_path: None,
            android_servisi,
            surucu_servisi,
            android_adb,
            android_cihazlar: Vec::new(),
            android_secili_seri: None,
            android_paketler: Vec::new(),
            android_uad: None,
            android_surucu: None,
            android_secili_paket: None,
            android_uzman_onayi: false,
            android_onay: None,
            android_islemde: false,
            android_mesaj: None,
            android_hata: None,
            android_arama: None,
            android_arama_abonelik: None,
            android_toplu_mod: false,
            android_secili: HashSet::new(),
            android_gecmis: Vec::new(),
            android_baslangic_yuklendi: false,
            tarama: tarama::TumTaramaDurumu::default(),
            baslik_cubugu_karanlik: None,
            yasal_onay_isaretli: false,
            yasal_hata: None,
        }
    }

    fn durumlari_yenile(&mut self) {
        let kurallar = self.kurallar.clone();
        for kural in &kurallar {
            self.durumlar
                .insert(kural.id.clone(), self.motor.durum(kural));
        }
    }

    fn secili_kural(&self) -> Option<&Kural> {
        self.secili
            .as_ref()
            .and_then(|id| self.kurallar.iter().find(|kural| &kural.id == id))
    }

    fn uygula_secili(&mut self, id: &str, cx: &mut Context<Self>) {
        let kural = self.kurallar.iter().find(|kural| kural.id == id).cloned();
        match kural {
            Some(kural) => match self.motor.onizle(&kural) {
                Ok(plan) => {
                    self.onay = Some(plan);
                    self.hata = None;
                    self.mesaj = None;
                }
                Err(uygulama_hatasi) => self.hata = Some(uygulama_hatasi.to_string()),
            },
            None => self.hata = Some("Kural bulunamadı.".to_string()),
        }
        cx.notify();
    }

    fn onayla(&mut self, cx: &mut Context<Self>) {
        if let Some(plan) = self.onay.take() {
            match self.motor.uygula(&plan) {
                Ok(sonuc) => {
                    self.son_plan = Some(plan);
                    self.mesaj = Some(format!("{} adım uygulandı.", sonuc.uygulanan));
                    self.hata = None;
                    self.durumlari_yenile();
                }
                Err(uygulama_hatasi) => self.hata = Some(uygulama_hatasi.to_string()),
            }
        }
        cx.notify();
    }

    fn son_islemi_geri_al(&mut self, cx: &mut Context<Self>) {
        let Some(plan) = self.son_plan.clone() else {
            self.mesaj = Some("Geri alınacak işlem yok.".to_string());
            cx.notify();
            return;
        };
        match self.motor.geri_al(&plan) {
            Ok(()) => {
                self.son_plan = None;
                self.mesaj = Some("Son işlem geri alındı.".to_string());
                self.hata = None;
                self.durumlari_yenile();
            }
            Err(geri_alma_hatasi) => self.hata = Some(geri_alma_hatasi.to_string()),
        }
        cx.notify();
    }

    fn tema_ayarla(&mut self, tema: Tema, cx: &mut Context<Self>) {
        self.ayarlar.tema = tema;
        if let Err(ayar_hatasi) = self.ayarlar.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            self.hata = Some(ayar_hatasi.to_string());
        }
        crate::tema::tema_ayarla(cx, tema);
        cx.notify();
    }

    fn baslik_cubugu_guncelle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let karanlik = cx.theme().is_dark();
        if self.baslik_cubugu_karanlik == Some(karanlik) {
            return;
        }
        let Ok(tutamak) = raw_window_handle::HasWindowHandle::window_handle(window) else {
            return;
        };
        let raw_window_handle::RawWindowHandle::Win32(pencere) = tutamak.as_raw() else {
            return;
        };
        sp_win::sistem::baslik_cubugu_karanlik_ayarla(pencere.hwnd.get(), karanlik);
        self.baslik_cubugu_karanlik = Some(karanlik);
    }

    fn tercihleri_disa_aktar(&mut self, cx: &mut Context<Self>) {
        let Some(dosya) = rfd::FileDialog::new()
            .set_title("WinNexus tercihlerini kaydet")
            .set_file_name("winnexus-tercihler.json")
            .add_filter("WinNexus tercihleri", &["json"])
            .save_file()
        else {
            return;
        };
        match self.ayarlar.disa_aktar(&dosya) {
            Ok(()) => {
                self.mesaj = Some(format!("Tercihler dışa aktarıldı: {}", dosya.display()));
                self.hata = None;
            }
            Err(hata) => {
                self.hata = Some(hata.to_string());
                self.mesaj = None;
            }
        }
        cx.notify();
    }

    fn tercihleri_ice_aktar(&mut self, cx: &mut Context<Self>) {
        let Some(dosya) = rfd::FileDialog::new()
            .set_title("WinNexus tercih dosyasını seçin")
            .add_filter("WinNexus tercihleri", &["json"])
            .pick_file()
        else {
            return;
        };
        let mut yeni = match Ayarlar::ice_aktar(&dosya) {
            Ok(ayarlar) => ayarlar,
            Err(hata) => {
                self.hata = Some(hata.to_string());
                self.mesaj = None;
                cx.notify();
                return;
            }
        };
        yeni.yasal_uyari_kabul_edildi = self.ayarlar.yasal_uyari_kabul_edildi;
        self.ayarlar = yeni;
        self.winget.ayarlari_guncelle(WingetAyarlari {
            lisans_onayi: self.ayarlar.winget_lisans_onayi,
            kaynak_onayi: self.ayarlar.winget_kaynak_onayi,
        });
        if let Err(hata) = self.ayarlar.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            self.hata = Some(hata.to_string());
            self.mesaj = None;
            cx.notify();
            return;
        }
        crate::tema::tema_ayarla(cx, self.ayarlar.tema);
        self.mesaj = Some(format!("Tercihler içe aktarıldı: {}", dosya.display()));
        self.hata = None;
        cx.notify();
    }

    fn winget_tara(&mut self, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Winget kaynakları taranıyor...".to_string());
        cx.notify();

        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    (
                        servis.guncellemeler(),
                        servis.pinler(),
                        servis.komut_gecmisi(),
                    )
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (guncellemeler, pinler, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match (guncellemeler, pinler) {
                    (Ok(guncellemeler), Ok(pinler)) => {
                        this.winget_guncellemeler = guncellemeler;
                        this.winget_pinler = pinler;
                        this.winget_mesaj = Some(format!(
                            "{} güncelleme bulundu.",
                            this.winget_guncellemeler.len()
                        ));
                        this.winget_hata = None;
                    }
                    (Err(hata), _) | (_, Err(hata)) => {
                        this.winget_hata = Some(hata.to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kaynak_guncelle(&mut self, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Winget kaynakları güncelleniyor...".to_string());
        cx.notify();

        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { (servis.kaynak_guncelle(), servis.komut_gecmisi()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(_) => {
                        this.winget_mesaj = Some("Winget kaynakları güncellendi.".to_string());
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_guncelle(&mut self, id: String, cx: &mut Context<Self>) {
        if !self.ayarlar.winget_lisans_onayi {
            self.winget_hata =
                Some("Paket lisans onayı kapalı. Güncelleme için onayı açın.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{id} güncelleniyor..."));
        cx.notify();

        let servis = self.winget.clone();
        let paket = id.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.guncelle(&paket);
                    (sonuc, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(_) => {
                        this.winget_guncellemeler
                            .retain(|guncelleme| guncelleme.id != id);
                        this.winget_mesaj = Some(format!("{id} güncellendi."));
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_asla(&mut self, id: String, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{id} için blocking pin ekleniyor..."));
        cx.notify();

        let servis = self.winget.clone();
        let paket = id.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.asla_guncelle(&paket);
                    (sonuc, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(_) => {
                        this.winget_guncellemeler
                            .retain(|guncelleme| guncelleme.id != id);
                        this.winget_pinler.push(PinKaydi {
                            id: id.clone(),
                            kaynak: None,
                            surum: None,
                            tur: PinTuru::Blocking,
                        });
                        this.winget_mesaj =
                            Some(format!("{id} artık güncellenmeyecek şekilde sabitlendi."));
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_surum_sabitle(&mut self, id: String, surum: String, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{id} {surum} sürümüne sabitleniyor..."));
        cx.notify();

        let servis = self.winget.clone();
        let paket = id.clone();
        let hedef = surum.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.surum_sabitle(&paket, &hedef);
                    (sonuc, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(_) => {
                        this.winget_guncellemeler
                            .retain(|guncelleme| guncelleme.id != id);
                        this.winget_pinler.push(PinKaydi {
                            id: id.clone(),
                            kaynak: None,
                            surum: Some(surum.clone()),
                            tur: PinTuru::Gating,
                        });
                        this.winget_mesaj = Some(format!("{id} {surum} sürümüne sabitlendi."));
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_pin_kaldir(&mut self, id: String, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{id} pin kaydı kaldırılıyor..."));
        cx.notify();

        let servis = self.winget.clone();
        let paket = id.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.pin_kaldir(&paket);
                    (sonuc, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(_) => {
                        this.winget_pinler.retain(|pin| pin.id != id);
                        this.winget_mesaj = Some(format!("{id} pin kaydı kaldırıldı."));
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_bu_sefer(&mut self, id: String, cx: &mut Context<Self>) {
        self.winget_atla.insert(id.clone());
        self.winget_mesaj = Some(format!("{id} bu güncelleme turunda atlandı."));
        self.winget_hata = None;
        cx.notify();
    }

    fn winget_onay_iste(&mut self, onay: WingetOnay, cx: &mut Context<Self>) {
        self.winget_onay = Some(onay);
        self.winget_hata = None;
        cx.notify();
    }

    fn winget_onayla(&mut self, cx: &mut Context<Self>) {
        let Some(onay) = self.winget_onay.take() else {
            return;
        };
        match onay {
            WingetOnay::Guncelle(id) => self.winget_guncelle(id, cx),
            WingetOnay::Asla(id) => self.winget_asla(id, cx),
            WingetOnay::SurumSabitle(id, surum) => self.winget_surum_sabitle(id, surum, cx),
            WingetOnay::TumunuGuncelle => self.winget_tumunu_guncelle(cx),
            WingetOnay::EksikleriKur => self.winget_eksikleri_kur(cx),
            WingetOnay::KaynakKaldir(ad) => self.winget_kaynak_kaldir(ad, cx),
            WingetOnay::SeciliKur(idler) | WingetOnay::ListeKur(idler) => {
                self.winget_kur_coklu(idler, cx)
            }
            WingetOnay::SeciliKaldir(idler) => self.winget_kaldir_coklu(idler, cx),
            WingetOnay::SeciliGuncelle(idler) => self.winget_guncelle_coklu(idler, cx),
        }
        cx.notify();
    }

    fn winget_onay_metni(&self) -> String {
        match &self.winget_onay {
            Some(WingetOnay::Guncelle(id)) => format!(
                "{id} güncellenecek. Winget güncellemesi geri alınamaz; devam edilsin mi?"
            ),
            Some(WingetOnay::Asla(id)) => format!(
                "{id} için blocking pin eklenecek. Bu paket WinNexus dışında da güncellenemeyecek. Devam edilsin mi?"
            ),
            Some(WingetOnay::SurumSabitle(id, surum)) => format!(
                "{id}, {surum} sürümüne sabitlenecek. Gating pin eklenecek; daha yeni sürümlere otomatik güncellenmeyecek. Devam edilsin mi?"
            ),
            Some(WingetOnay::TumunuGuncelle) => "Sabitlenmemiş ve atlanmamış tüm paketler sırayla güncellenecek. Devam edilsin mi?".to_string(),
            Some(WingetOnay::EksikleriKur) => "Eksik paketler kurulacak. Paket lisans onayı açık olmalı. Devam edilsin mi?".to_string(),
            Some(WingetOnay::KaynakKaldir(ad)) => format!(
                "{ad} kaynağı kaldırılacak. Bu kaynaktaki paketler artık bu merkezden kurulamaz. Devam edilsin mi?"
            ),
            Some(WingetOnay::SeciliKur(idler)) | Some(WingetOnay::ListeKur(idler)) => format!(
                "{} paket kurulacak. Paket lisans onayı açık olmalı. Devam edilsin mi?",
                idler.len()
            ),
            Some(WingetOnay::SeciliKaldir(idler)) => format!(
                "{} paket kaldırılacak. Kaldırma geri alınamaz; uygulama verileri kalabilir. Devam edilsin mi?",
                idler.len()
            ),
            Some(WingetOnay::SeciliGuncelle(idler)) => format!(
                "{} paket güncellenecek. Güncelleme geri alınamaz. Devam edilsin mi?",
                idler.len()
            ),
            None => String::new(),
        }
    }

    fn winget_lisans_degistir(&mut self, cx: &mut Context<Self>) {
        self.ayarlar.winget_lisans_onayi = !self.ayarlar.winget_lisans_onayi;
        self.winget.ayarlari_guncelle(WingetAyarlari {
            lisans_onayi: self.ayarlar.winget_lisans_onayi,
            kaynak_onayi: self.ayarlar.winget_kaynak_onayi,
        });
        if let Err(hata) = self.ayarlar.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            self.winget_hata = Some(hata.to_string());
        }
        cx.notify();
    }

    fn winget_kaynak_degistir(&mut self, cx: &mut Context<Self>) {
        self.ayarlar.winget_kaynak_onayi = !self.ayarlar.winget_kaynak_onayi;
        self.winget.ayarlari_guncelle(WingetAyarlari {
            lisans_onayi: self.ayarlar.winget_lisans_onayi,
            kaynak_onayi: self.ayarlar.winget_kaynak_onayi,
        });
        if let Err(hata) = self.ayarlar.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            self.winget_hata = Some(hata.to_string());
        }
        cx.notify();
    }

    fn winget_tumunu_guncelle(&mut self, cx: &mut Context<Self>) {
        if !self.ayarlar.winget_lisans_onayi {
            self.winget_hata =
                Some("Paket lisans onayı kapalı. Toplu güncelleme için onayı açın.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        let adaylar = self
            .winget_guncellemeler
            .iter()
            .filter(|guncelleme| {
                !self.winget_atla.contains(&guncelleme.id)
                    && !self.winget_pinler.iter().any(|pin| pin.id == guncelleme.id)
            })
            .map(|guncelleme| guncelleme.id.clone())
            .collect::<Vec<_>>();
        if adaylar.is_empty() {
            self.winget_mesaj =
                Some("Güncellenecek paket yok; tümü sabit veya atlanmış.".to_string());
            cx.notify();
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{} paket güncelleniyor...", adaylar.len()));
        cx.notify();

        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let mut basarili = Vec::new();
                    let mut hatalar = Vec::new();
                    for id in adaylar {
                        match servis.guncelle(&id) {
                            Ok(_) => basarili.push(id),
                            Err(hata) => hatalar.push(format!("{id}: {hata}")),
                        }
                    }
                    (basarili, hatalar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (basarili, hatalar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                this.winget_guncellemeler
                    .retain(|guncelleme| !basarili.contains(&guncelleme.id));
                this.winget_atla.clear();
                this.winget_mesaj = Some(format!(
                    "{} paket güncellendi, {} hata.",
                    basarili.len(),
                    hatalar.len()
                ));
                this.winget_hata = if hatalar.is_empty() {
                    None
                } else {
                    Some(hatalar.join("\n"))
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_profilleri_tara(&mut self, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Profiller taranıyor...".to_string());
        cx.notify();

        let servis = self.winget.clone();
        let profiller = self.profiller.clone();
        let eslesmeler = self.ayarlar.winget_eslesmeler.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let kurulu = match servis.kurulu_paketler() {
                        Ok(paketler) => paketler
                            .into_iter()
                            .map(|paket| paket.id)
                            .collect::<HashSet<_>>(),
                        Err(hata) => return (Err(hata), servis.komut_gecmisi()),
                    };
                    let mut sonuclar = Vec::new();
                    for profil in &profiller {
                        for paket in &profil.paket {
                            if kurulu.contains(&paket.id) {
                                continue;
                            }
                            let aranan_id = eslesmeler
                                .get(&paket.id)
                                .cloned()
                                .unwrap_or_else(|| paket.id.clone());
                            let paket_ad = paket.ad.clone().unwrap_or_else(|| paket.id.clone());
                            if servis.paket_var_mi(&aranan_id).unwrap_or(false) {
                                sonuclar.push(ProfilTaramaSonucu {
                                    profil_id: profil.id.clone(),
                                    profil_ad: profil.ad.clone(),
                                    paket_id: paket.id.clone(),
                                    paket_ad,
                                    kullanilacak_id: aranan_id,
                                    durum: ProfilDurum::Eksik,
                                });
                            } else {
                                let adaylar = servis.ara(&paket_ad).unwrap_or_default();
                                let oneriler = onerileri_sirala(paket, &adaylar);
                                let durum = match oneriler.into_iter().next() {
                                    Some(oneri) => ProfilDurum::Oneri(oneri),
                                    None => ProfilDurum::KaynaktaYok,
                                };
                                sonuclar.push(ProfilTaramaSonucu {
                                    profil_id: profil.id.clone(),
                                    profil_ad: profil.ad.clone(),
                                    paket_id: paket.id.clone(),
                                    paket_ad,
                                    kullanilacak_id: paket.id.clone(),
                                    durum,
                                });
                            }
                        }
                    }
                    (Ok(sonuclar), servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(sonuclar) => {
                        let sayi = sonuclar.len();
                        this.winget_profil_sonuclari = sonuclar;
                        this.winget_mesaj = Some(format!("{sayi} eksik/öneri bulundu."));
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_esleme_kabul(&mut self, eski: String, yeni: String, cx: &mut Context<Self>) {
        self.ayarlar
            .winget_eslesmeler
            .insert(eski.clone(), yeni.clone());
        for sonuc in &mut self.winget_profil_sonuclari {
            if sonuc.paket_id == eski {
                sonuc.kullanilacak_id = yeni.clone();
                sonuc.durum = ProfilDurum::Eksik;
            }
        }
        if let Err(hata) = self.ayarlar.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            self.winget_hata = Some(hata.to_string());
        }
        self.winget_mesaj = Some(format!("{eski} → {yeni} eşlemesi kaydedildi."));
        cx.notify();
    }

    fn winget_eksikleri_kur(&mut self, cx: &mut Context<Self>) {
        if !self.ayarlar.winget_lisans_onayi {
            self.winget_hata =
                Some("Paket lisans onayı kapalı. Kurulum için onayı açın.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        let idler = self
            .winget_profil_sonuclari
            .iter()
            .filter(|sonuc| matches!(sonuc.durum, ProfilDurum::Eksik))
            .map(|sonuc| sonuc.kullanilacak_id.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if idler.is_empty() {
            self.winget_mesaj = Some("Kurulacak eksik paket yok.".to_string());
            cx.notify();
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{} eksik paket kuruluyor...", idler.len()));
        cx.notify();

        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let mut basarili = Vec::new();
                    let mut hatalar = Vec::new();
                    for id in idler {
                        match servis.kur(&id) {
                            Ok(_) => basarili.push(id),
                            Err(hata) => hatalar.push(format!("{id}: {hata}")),
                        }
                    }
                    (basarili, hatalar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (basarili, hatalar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                this.winget_profil_sonuclari
                    .retain(|sonuc| !basarili.contains(&sonuc.kullanilacak_id));
                this.winget_mesaj = Some(format!(
                    "{} paket kuruldu, {} hata.",
                    basarili.len(),
                    hatalar.len()
                ));
                this.winget_hata = if hatalar.is_empty() {
                    None
                } else {
                    Some(hatalar.join("\n"))
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kaynak_girdileri(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> (Entity<InputState>, Entity<InputState>) {
        let ad = match &self.winget_kaynak_adi {
            Some(girdi) => girdi.clone(),
            None => {
                let girdi = cx
                    .new(|cx| InputState::new(window, cx).placeholder("Kaynak adı (örn. sirket)"));
                self.winget_kaynak_adi = Some(girdi.clone());
                girdi
            }
        };
        let adres = match &self.winget_kaynak_adresi {
            Some(girdi) => girdi.clone(),
            None => {
                let girdi = cx.new(|cx| {
                    InputState::new(window, cx).placeholder("https://kaynak.example/api")
                });
                self.winget_kaynak_adresi = Some(girdi.clone());
                girdi
            }
        };
        (ad, adres)
    }

    fn winget_magaza_girdisi(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        match &self.winget_magaza_arama {
            Some(girdi) => girdi.clone(),
            None => {
                let girdi =
                    cx.new(|cx| InputState::new(window, cx).placeholder("Paket adı veya kimliği"));
                self.winget_magaza_arama = Some(girdi.clone());
                girdi
            }
        }
    }

    fn winget_liste_girdisi(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TextareaState> {
        match &self.winget_liste_girdi {
            Some(girdi) => girdi.clone(),
            None => {
                let girdi = cx.new(|cx| {
                    TextareaState::new(window, cx)
                        .placeholder("Her satıra bir paket kimliği yazın; ör. Git.Git")
                        .auto_grow(4, 10)
                });
                self.winget_liste_girdi = Some(girdi.clone());
                girdi
            }
        }
    }

    fn winget_secim_degistir(&mut self, tur: WingetSecim, id: &str, secili: bool) {
        let hedef = match tur {
            WingetSecim::Guncelleme => &mut self.winget_guncelleme_secili,
            WingetSecim::Magaza => &mut self.winget_magaza_secili,
            WingetSecim::Kurulu => &mut self.winget_kurulu_secili,
            WingetSecim::Onerilen => &mut self.winget_onerilen_secili,
            WingetSecim::IceAktarilan => &mut self.winget_ice_aktarilan_secili,
        };
        if secili {
            hedef.insert(id.to_string());
        } else {
            hedef.remove(id);
        }
    }

    fn winget_secim_kutusu(
        &self,
        anahtar: String,
        id: String,
        secili: bool,
        tur: WingetSecim,
        cx: &mut Context<Self>,
    ) -> Checkbox {
        let uygulama = cx.entity();
        let kimlik = id.clone();
        Checkbox::new(format!("winget-secim-{anahtar}"))
            .checked(secili)
            .accessibility_label(format!("{id} seç"))
            .on_change(move |deger, _, cx| {
                let id = kimlik.clone();
                uygulama.update(cx, |this, cx| {
                    this.winget_secim_degistir(tur, &id, *deger);
                    cx.notify();
                });
            })
    }

    fn winget_kaynaklari_listele(&mut self, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Winget kaynakları listeleniyor...".to_string());
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { (servis.kaynaklar(), servis.komut_gecmisi()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(kaynaklar) => {
                        this.winget_mesaj = Some(format!("{} kaynak bulundu.", kaynaklar.len()));
                        if !kaynaklar
                            .iter()
                            .any(|kaynak| kaynak.ad == this.winget_magaza_kaynak)
                        {
                            this.winget_magaza_kaynak = kaynaklar
                                .first()
                                .map(|kaynak| kaynak.ad.clone())
                                .unwrap_or_else(|| "winget".to_string());
                        }
                        this.winget_kaynaklar = kaynaklar;
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kaynak_ekle(&mut self, cx: &mut Context<Self>) {
        let (Some(ad_girdi), Some(adres_girdi)) = (
            self.winget_kaynak_adi.clone(),
            self.winget_kaynak_adresi.clone(),
        ) else {
            return;
        };
        let ad = ad_girdi.read(cx).value().trim().to_string();
        let adres = adres_girdi.read(cx).value().trim().to_string();
        if ad.is_empty() || adres.is_empty() {
            self.winget_hata = Some("Kaynak adı ve adresi zorunlu.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{ad} kaynağı ekleniyor..."));
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let ekleme = servis.kaynak_ekle(&ad, &adres);
                    let kaynaklar = ekleme.as_ref().ok().and_then(|_| servis.kaynaklar().ok());
                    (ekleme, kaynaklar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (ekleme, kaynaklar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match ekleme {
                    Ok(_) => {
                        if let Some(kaynaklar) = kaynaklar {
                            this.winget_kaynaklar = kaynaklar;
                        }
                        this.winget_mesaj = Some("Kaynak eklendi.".to_string());
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kaynak_kaldir(&mut self, ad: String, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{ad} kaynağı kaldırılıyor..."));
        cx.notify();
        let servis = self.winget.clone();
        let kaldirilan = ad.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let kaldirma = servis.kaynak_kaldir(&ad);
                    let kaynaklar = kaldirma.as_ref().ok().and_then(|_| servis.kaynaklar().ok());
                    (kaldirma, kaynaklar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (kaldirma, kaynaklar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match kaldirma {
                    Ok(_) => {
                        if let Some(kaynaklar) = kaynaklar {
                            this.winget_kaynaklar = kaynaklar;
                        }
                        if this.winget_magaza_kaynak == kaldirilan {
                            this.winget_magaza_kaynak = "winget".to_string();
                        }
                        this.winget_mesaj = Some("Kaynak kaldırıldı.".to_string());
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_magaza_ara(&mut self, cx: &mut Context<Self>) {
        let Some(girdi) = self.winget_magaza_arama.clone() else {
            return;
        };
        let arama = girdi.read(cx).value().trim().to_string();
        let kaynak = self.winget_magaza_kaynak.clone();
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{kaynak} kaynağında aranıyor..."));
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.ara_kaynak(&arama, &kaynak, 30);
                    (sonuc, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(sonuclar) => {
                        this.winget_mesaj = Some(format!("{} paket bulundu.", sonuclar.len()));
                        this.winget_magaza_sonuclari = sonuclar;
                        this.winget_magaza_secili.clear();
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kur_coklu(&mut self, idler: Vec<String>, cx: &mut Context<Self>) {
        if !self.ayarlar.winget_lisans_onayi {
            self.winget_hata =
                Some("Paket lisans onayı kapalı. Kurulum için onayı açın.".to_string());
            cx.notify();
            return;
        }
        let idler = idler
            .into_iter()
            .filter(|id| !id.trim().is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if idler.is_empty() {
            self.winget_mesaj = Some("Kurulacak paket seçilmedi.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{} paket kuruluyor...", idler.len()));
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let mut basarili = Vec::new();
                    let mut hatalar = Vec::new();
                    for id in idler {
                        match servis.kur(&id) {
                            Ok(_) => basarili.push(id),
                            Err(hata) => hatalar.push(format!("{id}: {hata}")),
                        }
                    }
                    (basarili, hatalar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (basarili, hatalar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                if !basarili.is_empty() {
                    for id in &basarili {
                        this.winget_ice_aktarilan.retain(|paket| &paket.id != id);
                        this.winget_ice_aktarilan_secili.remove(id);
                        this.winget_magaza_secili.remove(id);
                        this.winget_onerilen_secili.remove(id);
                    }
                }
                this.winget_mesaj = Some(format!(
                    "{} paket kuruldu, {} hata.",
                    basarili.len(),
                    hatalar.len()
                ));
                this.winget_hata = if hatalar.is_empty() {
                    None
                } else {
                    Some(hatalar.join("\n"))
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kurulu_tara(&mut self, cx: &mut Context<Self>) {
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Kurulu winget paketleri taranıyor...".to_string());
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { (servis.kurulu_paketler(), servis.komut_gecmisi()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(paketler) => {
                        this.winget_mesaj =
                            Some(format!("{} kurulu paket bulundu.", paketler.len()));
                        this.winget_kurulu = paketler;
                        this.winget_kurulu_secili.clear();
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_kaldir_coklu(&mut self, idler: Vec<String>, cx: &mut Context<Self>) {
        let idler = idler
            .into_iter()
            .filter(|id| !id.trim().is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if idler.is_empty() {
            self.winget_mesaj = Some("Kaldırılacak paket seçilmedi.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{} paket kaldırılıyor...", idler.len()));
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let mut basarili = Vec::new();
                    let mut hatalar = Vec::new();
                    for id in idler {
                        match servis.kaldir(&id) {
                            Ok(_) => basarili.push(id),
                            Err(hata) => hatalar.push(format!("{id}: {hata}")),
                        }
                    }
                    (basarili, hatalar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (basarili, hatalar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                if !basarili.is_empty() {
                    this.winget_kurulu
                        .retain(|paket| !basarili.contains(&paket.id));
                    this.winget_guncellemeler
                        .retain(|guncelleme| !basarili.contains(&guncelleme.id));
                    for id in &basarili {
                        this.winget_kurulu_secili.remove(id);
                        this.winget_guncelleme_secili.remove(id);
                    }
                }
                this.winget_mesaj = Some(format!(
                    "{} paket kaldırıldı, {} hata.",
                    basarili.len(),
                    hatalar.len()
                ));
                this.winget_hata = if hatalar.is_empty() {
                    None
                } else {
                    Some(hatalar.join("\n"))
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_guncelle_coklu(&mut self, idler: Vec<String>, cx: &mut Context<Self>) {
        if !self.ayarlar.winget_lisans_onayi {
            self.winget_hata =
                Some("Paket lisans onayı kapalı. Güncelleme için onayı açın.".to_string());
            cx.notify();
            return;
        }
        let idler = idler
            .into_iter()
            .filter(|id| !id.trim().is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        if idler.is_empty() {
            self.winget_mesaj = Some("Güncellenecek paket seçilmedi.".to_string());
            cx.notify();
            return;
        }
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some(format!("{} paket güncelleniyor...", idler.len()));
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let mut basarili = Vec::new();
                    let mut hatalar = Vec::new();
                    for id in idler {
                        match servis.guncelle(&id) {
                            Ok(_) => basarili.push(id),
                            Err(hata) => hatalar.push(format!("{id}: {hata}")),
                        }
                    }
                    (basarili, hatalar, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (basarili, hatalar, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                if !basarili.is_empty() {
                    this.winget_guncellemeler
                        .retain(|guncelleme| !basarili.contains(&guncelleme.id));
                    for id in &basarili {
                        this.winget_guncelleme_secili.remove(id);
                    }
                }
                this.winget_mesaj = Some(format!(
                    "{} paket güncellendi, {} hata.",
                    basarili.len(),
                    hatalar.len()
                ));
                this.winget_hata = if hatalar.is_empty() {
                    None
                } else {
                    Some(hatalar.join("\n"))
                };
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_liste_coz(&mut self, cx: &mut Context<Self>) {
        let Some(girdi) = self.winget_liste_girdi.clone() else {
            return;
        };
        let metin = girdi.read(cx).value().to_string();
        match liste_metni_ayristir(&metin) {
            Ok(idler) => {
                self.winget_ice_aktarilan = idler
                    .iter()
                    .map(|id| WingetPaketi {
                        id: id.clone(),
                        ad: None,
                        kaynak: Some("winget".to_string()),
                        surum: None,
                    })
                    .collect();
                self.winget_ice_aktarilan_secili = idler.into_iter().collect();
                self.winget_mesaj = Some(format!(
                    "{} paket kimliği çözüldü.",
                    self.winget_ice_aktarilan.len()
                ));
                self.winget_hata = None;
            }
            Err(hata) => self.winget_hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn winget_liste_ice_aktar(&mut self, cx: &mut Context<Self>) {
        let Some(dosya) = rfd::FileDialog::new()
            .set_title("Winget paket listesini seçin")
            .add_filter("Winget listesi", &["json", "txt"])
            .pick_file()
        else {
            return;
        };
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Liste içe aktarılıyor...".to_string());
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { (servis.ice_aktar(&dosya), servis.komut_gecmisi()) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(paketler) => {
                        this.winget_ice_aktarilan_secili =
                            paketler.iter().map(|paket| paket.id.clone()).collect();
                        this.winget_mesaj =
                            Some(format!("{} paket içe aktarıldı.", paketler.len()));
                        this.winget_ice_aktarilan = paketler;
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn winget_liste_disa_aktar(&mut self, cx: &mut Context<Self>) {
        let idler: Vec<String> = if !self.winget_ice_aktarilan_secili.is_empty() {
            self.winget_ice_aktarilan_secili.iter().cloned().collect()
        } else if !self.winget_ice_aktarilan.is_empty() {
            self.winget_ice_aktarilan
                .iter()
                .map(|paket| paket.id.clone())
                .collect()
        } else if !self.winget_kurulu_secili.is_empty() {
            self.winget_kurulu_secili.iter().cloned().collect()
        } else {
            self.winget_kurulu
                .iter()
                .map(|paket| paket.id.clone())
                .collect()
        };
        if idler.is_empty() {
            self.winget_hata = Some(
                "Dışa aktarılacak paket yok. Önce kurulu paketleri tarayın veya liste çözün."
                    .to_string(),
            );
            cx.notify();
            return;
        }
        let Some(dosya) = rfd::FileDialog::new()
            .set_title("Winget listesini kaydet")
            .set_file_name("winget-liste.json")
            .add_filter("Winget listesi", &["json"])
            .save_file()
        else {
            return;
        };
        if self.winget_taraniyor {
            return;
        }
        self.winget_taraniyor = true;
        self.winget_hata = None;
        self.winget_mesaj = Some("Liste dışa aktarılıyor...".to_string());
        cx.notify();
        let servis = self.winget.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.disa_aktar(&idler, &dosya);
                    (sonuc, servis.komut_gecmisi())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.winget_taraniyor = false;
                let (sonuc, komutlar) = sonuc;
                this.winget_komutlar = komutlar;
                match sonuc {
                    Ok(()) => {
                        this.winget_mesaj = Some("Liste dışa aktarıldı.".to_string());
                        this.winget_hata = None;
                    }
                    Err(hata) => this.winget_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn profil_sonucu_satiri(sonuc: ProfilTaramaSonucu, cx: &mut Context<Self>) -> AnyElement {
        let (etiket, renk) = match &sonuc.durum {
            ProfilDurum::Eksik => ("Eksik", cx.theme().info),
            ProfilDurum::Oneri(_) => ("Yeniden adlandırılmış olabilir", cx.theme().warning),
            ProfilDurum::KaynaktaYok => ("Kaynakta yok", cx.theme().muted_foreground),
        };
        let mut detay = v_flex()
            .min_w_0()
            .gap_1()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("{} · {}", sonuc.profil_ad, sonuc.paket_ad)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Kimlik: {}", sonuc.paket_id)),
            );
        if sonuc.kullanilacak_id != sonuc.paket_id {
            detay = detay.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Kullanılacak kimlik: {}", sonuc.kullanilacak_id)),
            );
        }
        let mut aksiyon = h_flex().gap_2().items_center().child(
            h_flex()
                .gap_1()
                .items_center()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(cx.theme().muted)
                .text_color(cx.theme().foreground)
                .border_1()
                .border_color(renk)
                .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(renk))
                .child(div().text_sm().child(etiket)),
        );
        if let ProfilDurum::Oneri(oneri) = &sonuc.durum {
            detay = detay.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "Öneri: {} · Güven: {}",
                        oneri.id,
                        oneri.guven.metin()
                    )),
            );
            let eski = sonuc.paket_id.clone();
            let yeni = oneri.id.clone();
            aksiyon = aksiyon.child(
                Button::new(format!("we-{}", sonuc.paket_id))
                    .primary()
                    .label("Yeni kimliği kullan")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.winget_esleme_kabul(eski.clone(), yeni.clone(), cx);
                    })),
            );
        }
        h_flex()
            .justify_between()
            .items_center()
            .p_3()
            .gap_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(detay)
            .child(aksiyon)
            .into_any_element()
    }

    fn sorun_denetle(&mut self, cx: &mut Context<Self>) {
        if self.sorun_denetleniyor {
            return;
        }
        self.sorun_denetleniyor = true;
        self.sorun_hata = None;
        self.sorun_mesaj = Some("Sorunlar denetleniyor...".to_string());
        cx.notify();

        let servis = self.sorun_servisi.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    (
                        servis.npm_durumu(),
                        servis.execution_policy_durumu(),
                        servis.mmagent_durumu(),
                    )
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.sorun_denetleniyor = false;
                let (npm, policy, mmagent) = sonuc;
                let mut hatalar = Vec::new();
                this.sorun_npm = match npm {
                    Ok(durum) => Some(durum),
                    Err(hata) => {
                        hatalar.push(format!("npm: {hata}"));
                        None
                    }
                };
                this.sorun_policy = match policy {
                    Ok(durum) => Some(durum),
                    Err(hata) => {
                        hatalar.push(format!("ExecutionPolicy: {hata}"));
                        None
                    }
                };
                this.sorun_mmagent = match mmagent {
                    Ok(durum) => Some(durum),
                    Err(hata) => {
                        hatalar.push(format!("MMAgent: {hata}"));
                        None
                    }
                };
                this.sorun_hata = if hatalar.is_empty() {
                    None
                } else {
                    Some(hatalar.join("\n"))
                };
                this.sorun_mesaj = Some(if hatalar.is_empty() {
                    "Denetim tamamlandı.".to_string()
                } else {
                    "Denetim tamamlandı; bazı kontroller başarısız.".to_string()
                });
                cx.notify();
            });
        })
        .detach();
    }

    fn sorun_onay_iste(&mut self, onay: SorunOnay, cx: &mut Context<Self>) {
        self.sorun_onay = Some(onay);
        self.sorun_hata = None;
        cx.notify();
    }

    fn sorun_onay_metni(&self) -> String {
        match &self.sorun_onay {
            Some(SorunOnay::NpmPathEkle) => {
                let prefix = self
                    .sorun_npm
                    .as_ref()
                    .map(|durum| durum.prefix.clone())
                    .unwrap_or_else(|| "%APPDATA%\\npm".to_string());
                format!(
                    "{prefix} kullanıcı PATH'ine eklenecek. Değişiklik yeni terminallerde geçerli olur. Devam edilsin mi?"
                )
            }
            Some(SorunOnay::NpmPathGeriAl(_)) => {
                "Kullanıcı PATH'i önceki değerine döndürülecek. Devam edilsin mi?".to_string()
            }
            Some(SorunOnay::PolicyCurrentUser) => {
                "Yalnızca bu kullanıcı için RemoteSigned uygulanacak. Bu, makine genelinden daha güvenli ve önerilen seçenektir; GPO varsa etkisiz kalabilir. Devam edilsin mi?"
                    .to_string()
            }
            Some(SorunOnay::PolicyLocalMachine) => {
                "RemoteSigned tüm kullanıcılar için makine geneline uygulanacak; yönetici izni/UAC gerekir. GPO varsa etkisiz kalabilir. Devam edilsin mi?"
                    .to_string()
            }
            Some(SorunOnay::MmAgentKapat) => {
                "Bellek sıkıştırma kapatılacak. Bellek kullanımı artabilir; değişiklik için yeniden başlatma önerilir. Devam edilsin mi?"
                    .to_string()
            }
            Some(SorunOnay::MmAgentAc) => {
                "Bellek sıkıştırma açılacak; değişiklik için yeniden başlatma önerilir. Devam edilsin mi?"
                    .to_string()
            }
            None => String::new(),
        }
    }

    fn sorun_onayla(&mut self, cx: &mut Context<Self>) {
        let Some(onay) = self.sorun_onay.take() else {
            return;
        };
        let servis = self.sorun_servisi.clone();
        self.sorun_denetleniyor = true;
        self.sorun_hata = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let sonuc =
                cx.background_executor()
                    .spawn(async move {
                        match onay {
                        SorunOnay::NpmPathEkle => servis.path_ekle().map(|sonuc| {
                            (
                                if sonuc.zaten_var {
                                    "npm yolu kullanıcı PATH'inde zaten var.".to_string()
                                } else {
                                    "npm yolu kullanıcı PATH'ine eklendi; yeni terminaller açın."
                                        .to_string()
                                },
                                if sonuc.zaten_var {
                                    None
                                } else {
                                    Some(sonuc.onceki)
                                },
                            )
                        }),
                        SorunOnay::NpmPathGeriAl(onceki) => {
                            servis.path_geri_al(&onceki).map(|()| {
                                ("Kullanıcı PATH'i önceki değerine döndürüldü.".to_string(), None)
                            })
                        }
                        SorunOnay::PolicyCurrentUser => servis
                            .execution_policy_ayarla(PolicyKapsami::CurrentUser)
                            .map(|_| {
                                (
                                    "Execution policy (CurrentUser): RemoteSigned.".to_string(),
                                    None,
                                )
                            }),
                        SorunOnay::PolicyLocalMachine => servis
                            .execution_policy_ayarla(PolicyKapsami::LocalMachine)
                            .map(|_| {
                                (
                                    "Execution policy (LocalMachine): RemoteSigned.".to_string(),
                                    None,
                                )
                            }),
                        SorunOnay::MmAgentKapat => servis.mmagent_ayarla(false).map(|_| {
                            (
                                "Bellek sıkıştırma kapatıldı; yeniden başlatma önerilir."
                                    .to_string(),
                                None,
                            )
                        }),
                        SorunOnay::MmAgentAc => servis.mmagent_ayarla(true).map(|_| {
                            (
                                "Bellek sıkıştırma açıldı; yeniden başlatma önerilir."
                                    .to_string(),
                                None,
                            )
                        }),
                    }
                    })
                    .await;
            let _ = this.update(cx, |this, cx| {
                this.sorun_denetleniyor = false;
                match sonuc {
                    Ok((mesaj, onceki)) => {
                        if onceki.is_some() {
                            this.npm_onceki_path = onceki;
                        }
                        this.sorun_mesaj = Some(mesaj.clone());
                        this.sorun_gecmis.push(mesaj);
                        this.sorun_gecmis = this
                            .sorun_gecmis
                            .iter()
                            .rev()
                            .take(10)
                            .cloned()
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect();
                        this.sorun_denetle(cx);
                    }
                    Err(hata) => this.sorun_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn sorun_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut sayfa = v_flex().gap_4().child(
            h_flex().justify_end().items_center().child(
                Button::new("sorun-denetle")
                    .primary()
                    .label(if self.sorun_denetleniyor {
                        "Denetleniyor..."
                    } else {
                        "Tümünü Denetle"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sorun_denetle(cx);
                    })),
            ),
        );

        if let Some(hata) = &self.sorun_hata {
            sayfa = sayfa.child(self.hata_kutusu(hata, cx));
        }
        if let Some(mesaj) = &self.sorun_mesaj {
            sayfa = sayfa.child(self.mesaj_kutusu(mesaj, cx));
        }
        if self.sorun_onay.is_some() {
            sayfa = sayfa.child(
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
                    .child(div().text_sm().child(self.sorun_onay_metni()))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("sorun-onayla")
                                    .primary()
                                    .label("Onayla")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sorun_onayla(cx);
                                    })),
                            )
                            .child(Button::new("sorun-vazgec").label("Vazgeç").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.sorun_onay = None;
                                    cx.notify();
                                }),
                            )),
                    ),
            );
        }

        let npm_kart = v_flex()
            .p_4()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("npm Global PATH"),
            )
            .child(match &self.sorun_npm {
                Some(durum) => v_flex()
                    .gap_2()
                    .child(div().text_sm().child(format!("npm: {}", durum.npm_yolu)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("Global prefix: {}", durum.prefix)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(if durum.path_kaynagi.is_some() {
                                cx.theme().success
                            } else {
                                cx.theme().danger
                            })
                            .child(match &durum.path_kaynagi {
                                Some(kaynak) => format!("PATH: ekli ({kaynak})"),
                                None => "PATH: eksik".to_string(),
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "Global paketler ({}): {}",
                                durum.global_paketler.len(),
                                if durum.global_paketler.is_empty() {
                                    "yok".to_string()
                                } else {
                                    durum.global_paketler.join(", ")
                                }
                            )),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(if durum.path_kaynagi.is_none() {
                                Button::new("npm-path-ekle")
                                    .primary()
                                    .label("PATH'e Ekle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sorun_onay_iste(SorunOnay::NpmPathEkle, cx);
                                    }))
                                    .into_any_element()
                            } else if let Some(onceki) = &self.npm_onceki_path {
                                Button::new("npm-path-geri")
                                    .label("PATH'i Geri Al")
                                    .on_click(cx.listener({
                                        let onceki = onceki.clone();
                                        move |this, _, _, cx| {
                                            this.sorun_onay_iste(
                                                SorunOnay::NpmPathGeriAl(onceki.clone()),
                                                cx,
                                            );
                                        }
                                    }))
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            })
                            .child(Button::new("npm-yenile").label("Yeniden Denetle").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.sorun_denetle(cx);
                                }),
                            )),
                    )
                    .into_any_element(),
                None => div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz denetlenmedi veya npm bulunamadı.")
                    .into_any_element(),
            });

        let policy_kart = v_flex()
            .p_4()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("PowerShell Execution Policy"),
            )
            .child(match &self.sorun_policy {
                Some(durum) => {
                    let mut icerik = v_flex().gap_2();
                    for kapsam in &durum.scopes {
                        icerik = icerik.child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("{}: {}", kapsam.scope, kapsam.policy)),
                        );
                    }
                    if durum.gpo {
                        icerik = icerik.child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().warning)
                                .child("GPO tarafından yönetiliyor; politikayı GPO belirler."),
                        );
                    }
                    icerik
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("policy-current")
                                        .primary()
                                        .label("CurrentUser RemoteSigned")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.sorun_onay_iste(SorunOnay::PolicyCurrentUser, cx);
                                        })),
                                )
                                .child(
                                    Button::new("policy-machine")
                                        .label("LocalMachine RemoteSigned")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.sorun_onay_iste(SorunOnay::PolicyLocalMachine, cx);
                                        })),
                                ),
                        )
                        .into_any_element()
                }
                None => div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz denetlenmedi.")
                    .into_any_element(),
            });

        let mm_kart = v_flex()
            .p_4()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Bellek Sıkıştırma (MMAgent)"),
            )
            .child(match &self.sorun_mmagent {
                Some(durum) => v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "MemoryCompression: {} · PageCombining: {}",
                                if durum.memory_compression {
                                    "Açık"
                                } else {
                                    "Kapalı"
                                },
                                if durum.page_combining {
                                    "Açık"
                                } else {
                                    "Kapalı"
                                }
                            )),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(
                            "Değişiklikler yönetici izni gerektirir; yeniden başlatma önerilir.",
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("mm-kapat").danger().label("Kapat").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.sorun_onay_iste(SorunOnay::MmAgentKapat, cx);
                                }),
                            ))
                            .child(Button::new("mm-ac").primary().label("Aç").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.sorun_onay_iste(SorunOnay::MmAgentAc, cx);
                                }),
                            )),
                    )
                    .into_any_element(),
                None => div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz denetlenmedi veya MMAgent kullanılamıyor.")
                    .into_any_element(),
            });

        sayfa = sayfa.child(npm_kart).child(policy_kart).child(mm_kart);

        if !self.sorun_gecmis.is_empty() {
            sayfa = sayfa.child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Son İşlemler"),
                    )
                    .children(self.sorun_gecmis.iter().rev().map(|satir| {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(satir.clone())
                    })),
            );
        }

        sayfa.into_any_element()
    }

    fn arac_onay_iste(&mut self, onay: AracOnay, cx: &mut Context<Self>) {
        self.arac_onay = Some(onay);
        self.hata = None;
        cx.notify();
    }

    fn arac_onayla(&mut self, cx: &mut Context<Self>) {
        let Some(onay) = self.arac_onay.take() else {
            return;
        };
        match onay {
            AracOnay::ExplorerYenidenBaslat => self.explorer_yeniden_baslat(cx),
            AracOnay::BiosYenidenBaslat => self.bios_yeniden_baslat(cx),
            AracOnay::SahiplikEkle => self.sahiplik_menu_ekle(cx),
            AracOnay::SahiplikKaldir => self.sahiplik_menu_kaldir(cx),
        }
        cx.notify();
    }

    fn arac_onay_metni(&self) -> String {
        match self.arac_onay {
            Some(AracOnay::ExplorerYenidenBaslat) => {
                "Görev çubuğu ve masaüstü kısa süreliğine kapanıp yeniden açılacak. Devam edilsin mi?"
                    .to_string()
            }
            Some(AracOnay::BiosYenidenBaslat) => {
                "Bilgisayar yeniden başlatılacak ve UEFI/BIOS ekranı açılacak. Kaydedilmemiş çalışmalarınızı kaydedin. Devam edilsin mi?"
                    .to_string()
            }
            Some(AracOnay::SahiplikEkle) => {
                "Dosya ve klasörlerin sağ tık menüsüne 'Sahipliği Al' eklenecek. Bu işlem yönetici izni (UAC) gerektirebilir. Devam edilsin mi?"
                    .to_string()
            }
            Some(AracOnay::SahiplikKaldir) => {
                "Sahipliği Al sağ tık menüsü kaldırılacak. Devam edilsin mi?".to_string()
            }
            None => String::new(),
        }
    }

    fn explorer_yeniden_baslat(&mut self, cx: &mut Context<Self>) {
        let calistirici = WindowsKomutCalistirici;
        let _ = calistirici.calistir(
            "taskkill",
            &[
                "/f".to_string(),
                "/im".to_string(),
                "explorer.exe".to_string(),
            ],
            Duration::from_secs(30),
        );
        match calistirici.calistir("explorer.exe", &[], Duration::from_secs(30)) {
            Ok(_) => {
                self.mesaj = Some("Explorer yeniden başlatıldı.".to_string());
                self.hata = None;
            }
            Err(hata) => self.hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn bios_yeniden_baslat(&mut self, cx: &mut Context<Self>) {
        let calistirici = WindowsKomutCalistirici;
        let sonuc = calistirici.calistir(
            "shutdown",
            &[
                "/r".to_string(),
                "/fw".to_string(),
                "/t".to_string(),
                "0".to_string(),
            ],
            Duration::from_secs(30),
        );
        match sonuc {
            Ok(sonuc) if sonuc.kod == 0 => {
                self.mesaj = Some("BIOS'a yeniden başlatma başlatıldı.".to_string());
                self.hata = None;
            }
            Ok(sonuc) => {
                self.hata = Some(format!(
                    "BIOS'a yeniden başlatma başarısız (kod {}): {}",
                    sonuc.kod,
                    sonuc.stderr.trim()
                ));
            }
            Err(hata) => self.hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn sahiplik_menu_ekle(&mut self, cx: &mut Context<Self>) {
        let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("WinNexus.exe"));
        match sp_win::sahiplik::menu_ekle(&exe) {
            Ok(()) => {
                self.mesaj = Some("Sahipliği Al sağ tık menüsü eklendi.".to_string());
                self.hata = None;
            }
            Err(hata) => self.hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn sahiplik_menu_kaldir(&mut self, cx: &mut Context<Self>) {
        match sp_win::sahiplik::menu_kaldir() {
            Ok(()) => {
                self.mesaj = Some("Sahipliği Al sağ tık menüsü kaldırıldı.".to_string());
                self.hata = None;
            }
            Err(hata) => self.hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn baglam_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let ogeler = self.baglam_ogeleri.clone();
        let klasik = self.baglam_klasik;
        let taraniyor = self.baglam_taraniyor;
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Bağlam Menüsü Yöneticisi"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Kabuk uzantılarını kullanıcı kapsamında devre dışı bırakın; kayıt silinmez, Blocked listesine eklenir. Aynı CLSID tüm kapsamlarda etkilenir."),
                            ),
                    )
                    .child(
                        Button::new("baglam-tara")
                            .primary()
                            .label(if taraniyor { "Taranıyor..." } else { "Bağlam Menüsünü Tara" })
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| this.baglam_menu_tara(cx))),
                    ),
            )
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("Klasik sağ tık menüsü"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(match klasik {
                                        Some(true) => "Klasik menü açık.",
                                        Some(false) => "Windows 11 menüsü etkin.",
                                        None => "Durum için tarama yapın.",
                                    }),
                            ),
                    )
                    .child(
                        Button::new("baglam-klasik")
                            .label(if klasik == Some(true) {
                                "Windows 11 Menüsüne Dön"
                            } else {
                                "Klasik Menüyü Aç"
                            })
                            .disabled(klasik.is_none())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.klasik_baglam_ayarla(this.baglam_klasik != Some(true), cx);
                            })),
                    ),
            )
            .child(if let Some(hata) = &self.baglam_hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &self.baglam_mesaj {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(mesaj.clone())
                    .into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if ogeler.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz tarama yapılmadı veya kabuk uzantısı bulunamadı.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .children(ogeler.iter().take(300).enumerate().map(|(sira, oge)| {
                        let clsid = oge.clsid.clone();
                        let devre_disi = oge.devre_disi;
                        h_flex()
                            .justify_between()
                            .items_center()
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(oge.ad.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("{} · {}", oge.kapsam, oge.kaynak)),
                                            )
                                            .child(if devre_disi {
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().warning)
                                                    .child("Devre dışı")
                                                    .into_any_element()
                                            } else {
                                                div().into_any_element()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(oge.clsid.clone()),
                                    ),
                            )
                            .child(
                                Button::new(format!("baglam-oge-{sira}-{}", oge.clsid))
                                    .label(if devre_disi {
                                        "Etkinleştir"
                                    } else {
                                        "Devre Dışı Bırak"
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.baglam_menu_ayarla(
                                            clsid.clone(),
                                            !devre_disi,
                                            cx,
                                        );
                                    })),
                            )
                    }))
                    .into_any_element()
            })
            .into_any_element()
    }

    fn arac_kutusu(&self, cx: &mut Context<Self>) -> AnyElement {
        let kurallar = self
            .kurallar
            .iter()
            .filter(|kural| {
                kural.id.starts_with("sistem.") || kural.id.starts_with("baglam_menusu.")
            })
            .cloned()
            .collect::<Vec<_>>();

        let komut_paneli = v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Komutlar"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("arac-explorer")
                            .label("Explorer'ı yeniden başlat")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.arac_onay_iste(AracOnay::ExplorerYenidenBaslat, cx);
                            })),
                    )
                    .child(
                        Button::new("arac-bios")
                            .danger()
                            .label("BIOS'a yeniden başlat")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.arac_onay_iste(AracOnay::BiosYenidenBaslat, cx);
                            })),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("arac-sahiplik-ekle")
                            .label("Sahipliği Al menüsü ekle")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.arac_onay_iste(AracOnay::SahiplikEkle, cx);
                            })),
                    )
                    .child(
                        Button::new("arac-sahiplik-kaldir")
                            .label("Sahipliği Al menüsünü kaldır")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.arac_onay_iste(AracOnay::SahiplikKaldir, cx);
                            })),
                    ),
            )
            .child(if self.arac_onay.is_some() {
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
                    .child(div().text_sm().child(self.arac_onay_metni()))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("arac-onayla")
                                    .primary()
                                    .label("Onayla")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.arac_onayla(cx);
                                    })),
                            )
                            .child(Button::new("arac-vazgec").label("Vazgeç").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.arac_onay = None;
                                    cx.notify();
                                }),
                            )),
                    )
                    .into_any_element()
            } else {
                div().into_any_element()
            });

        v_flex()
            .gap_4()
            .child(komut_paneli)
            .child(self.baglam_paneli(cx))
            .child(self.android_kisa_karti(cx))
            .child(
                h_flex()
                    .gap_6()
                    .items_start()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_3()
                            .children(kurallar.iter().map(|kural| self.kural_satiri(kural, cx))),
                    )
                    .child(self.detay_paneli(cx)),
            )
            .into_any_element()
    }

    fn yasal_kabul(&mut self, cx: &mut Context<Self>) {
        let mut yeni = self.ayarlar.clone();
        yeni.yasal_uyari_kabul_edildi = true;
        match yeni.kaydet(&self.dizinler.ayarlar_dosyasi()) {
            Ok(()) => {
                self.ayarlar = yeni;
                self.yasal_hata = None;
            }
            Err(hata) => {
                self.yasal_hata = Some(format!(
                    "Kabul kaydedilemedi. Uygulama klasörünün yazılabilir olduğundan emin olun: {hata}"
                ));
            }
        }
        cx.notify();
    }

    fn yasal_ekrani(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let uygulama = cx.entity();
        let onay = Checkbox::new("yasal-onay")
            .checked(self.yasal_onay_isaretli)
            .label("Yasal Uyarı ve Sorumluluk Reddi metnini okudum, anladım ve kabul ediyorum.");
        let onay = onay.on_change(move |deger, _, cx| {
            uygulama.update(cx, |this, cx| {
                this.yasal_onay_isaretli = *deger;
                cx.notify();
            });
        });
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .p_8()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(1000.))
                    .h(relative(0.9))
                    .gap_3()
                    .p_6()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Yasal Uyarı ve Sorumluluk Reddi"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        "Uygulamayı kullanmadan önce bu metni okumanız gerekir.",
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scrollbar()
                            .p_3()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                v_flex().gap_2().children(
                                    YASAL_UYARI
                                        .lines()
                                        .filter(|satir| !satir.trim().is_empty())
                                        .map(|satir| {
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(satir.to_string())
                                        }),
                                ),
                            ),
                    )
                    .child(if let Some(hata) = &self.yasal_hata {
                        self.hata_kutusu(hata, cx).into_any_element()
                    } else {
                        div().into_any_element()
                    })
                    .child(onay)
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Button::new("yasal-kabul")
                                    .primary()
                                    .label("Kabul Ediyorum")
                                    .disabled(!self.yasal_onay_isaretli)
                                    .on_click(cx.listener(|this, _, _, cx| this.yasal_kabul(cx))),
                            )
                            .child(
                                Button::new("yasal-red")
                                    .label("Kabul Etmiyorum")
                                    .on_click(|_, _, _| std::process::exit(0)),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn god_mode_ac(&mut self, cx: &mut Context<Self>) {
        match sp_win::god_mode_ac() {
            Ok(()) => {
                self.mesaj = Some(
                    "God Mode Explorer penceresinde açıldı. Masaüstünde klasör oluşturulmadı."
                        .to_string(),
                );
                self.hata = None;
            }
            Err(hata) => self.hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn god_mode_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_4()
            .child(self.bolum_karti(
                "Tüm Görevler".to_string(),
                "God Mode, Windows'un sistem, donanım, ağ, kullanıcı hesapları, görünüm, saat/dil, erişim kolaylığı ve programlar görevlerini tek bir Explorer penceresinde toplar.".to_string(),
                v_flex()
                    .gap_3()
                    .child(
                        Button::new("god-mode-ac")
                            .primary()
                            .label("God Mode'u Aç")
                            .on_click(cx.listener(|this, _, _, cx| this.god_mode_ac(cx))),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Masaüstünde veya başka bir dizinde klasör oluşturulmaz. God Mode, Explorer'ın shell::: protokolüyle ayrı bir pencerede açılır."),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("CLSID: {ED7BA470-8E54-465E-825C-99712043E01C}"),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("İçerdiği başlıca kategoriler"))
                            .children([
                                "Sistem ve Güvenlik",
                                "Donanım ve Ses",
                                "Ağ ve İnternet",
                                "Kullanıcı Hesapları",
                                "Görünüm ve Kişiselleştirme",
                                "Saat, Dil ve Bölge",
                                "Erişim Kolaylığı",
                                "Programlar ve Özellikler",
                            ]
                            .into_iter()
                            .map(|kategori| {
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("• {kategori}"))
                            })),
                    )
                    .into_any_element(),
                cx,
            ))
            .into_any_element()
    }

    fn kaldirici_arama_girdisi(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(girdi) = &self.kaldirici_arama {
            return girdi.clone();
        }
        let girdi = cx.new(|cx| InputState::new(window, cx));
        let abonelik = cx.subscribe(&girdi, |_this: &mut Self, _, olay: &InputEvent, cx| {
            if matches!(olay, InputEvent::Change) {
                cx.notify();
            }
        });
        self.kaldirici_arama = Some(girdi.clone());
        self.kaldirici_abonelik = Some(abonelik);
        girdi
    }

    fn kaldirici_tara(&mut self, cx: &mut Context<Self>) {
        if self.kaldirici_taraniyor || self.kaldirici_islemde {
            return;
        }
        self.kaldirici_taraniyor = true;
        self.kaldirici_hata = None;
        self.kaldirici_mesaj = Some("Kurulu uygulamalar taranıyor...".to_string());
        cx.notify();

        let servis = self.kaldirici.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.uygulamalari_tara() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kaldirici_taraniyor = false;
                match sonuc {
                    Ok(uygulamalar) => {
                        this.kaldirici_mesaj =
                            Some(format!("{} kurulu uygulama bulundu.", uygulamalar.len()));
                        this.kaldirici_uygulamalar = uygulamalar;
                        this.kaldirici_hata = None;
                    }
                    Err(hata) => this.kaldirici_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kaldirici_secim_kutusu(
        &self,
        anahtar: String,
        kimlik: String,
        secili: bool,
        cx: &mut Context<Self>,
    ) -> Checkbox {
        let uygulama = cx.entity();
        let kimlik = kimlik.clone();
        Checkbox::new(format!("kaldirici-secim-{anahtar}-c"))
            .checked(secili)
            .accessibility_label(format!("{kimlik} seç"))
            .on_change(move |deger, _, cx| {
                let kimlik = anahtar.clone();
                uygulama.update(cx, |this, cx| {
                    if *deger {
                        this.kaldirici_secili.insert(kimlik);
                    } else {
                        this.kaldirici_secili.remove(&kimlik);
                    }
                    cx.notify();
                });
            })
    }

    fn kaldirici_toplu_kaldir(
        &mut self,
        kimlikler: Vec<String>,
        sessiz: bool,
        geri_noktasi: bool,
        cx: &mut Context<Self>,
    ) {
        if self.kaldirici_islemde {
            return;
        }
        let uygulamalar = self
            .kaldirici_uygulamalar
            .iter()
            .filter(|uygulama| kimlikler.contains(&uygulama.kimlik))
            .cloned()
            .collect::<Vec<_>>();
        if uygulamalar.is_empty() {
            self.kaldirici_hata = Some("Seçili uygulama bulunamadı.".to_string());
            cx.notify();
            return;
        }
        let servis = self.kaldirici.clone();
        let artik_servisi = self.artik.clone();
        let karantina_kok = self
            .dizinler
            .karantina_dizini()
            .join(format!("toplu-kaldirma-{}", Self::zaman_damgasi()));
        self.kaldirici_islemde = true;
        self.kaldirici_hata = None;
        self.kaldirici_mesaj = Some(format!(
            "{} uygulama sırayla kaldırılıyor...",
            uygulamalar.len()
        ));
        cx.notify();
        cx.spawn(async move |this, cx| {
            let sonuc: sp_core::Sonuc<TopluKaldirmaSonucu> = cx
                .background_executor()
                .spawn(async move {
                    if geri_noktasi {
                        servis.geri_yukleme_noktasi("toplu kaldırma öncesi")?;
                    }
                    let mut basarili = Vec::new();
                    let mut hatalar = Vec::new();
                    let mut kayitlar = Vec::new();
                    let mut artik_tasinan = 0u64;
                    for (sira, uygulama) in uygulamalar.into_iter().enumerate() {
                        match servis.kaldir(&uygulama, sessiz) {
                            Ok(kaldirma) if kaldirma.basarili() => {
                                basarili.push(uygulama.kimlik.clone());
                                let mesaj =
                                    format!("{} kaldırma komutu tamamlandı.", uygulama.ad);
                                kayitlar.push(mesaj);
                                match artik_servisi.tara(&uygulama) {
                                    Ok(ogeler) if !ogeler.is_empty() => {
                                        let hedef = karantina_kok.join(format!("uygulama-{sira}"));
                                        match artik_servisi.karantinaya_tasi(&ogeler, &hedef) {
                                            Ok(sonuc) => {
                                                artik_tasinan += sonuc.tasinan;
                                                if !sonuc.hatalar.is_empty() {
                                                    let mesaj = format!(
                                                        "{} artıkları: {}",
                                                        uygulama.ad,
                                                        sonuc.hatalar.join("; ")
                                                    );
                                                    hatalar.push(mesaj.clone());
                                                    kayitlar.push(mesaj);
                                                }
                                            }
                                            Err(hata) => {
                                                let mesaj = format!(
                                                    "{} artıkları karantinaya taşınamadı: {hata}",
                                                    uygulama.ad
                                                );
                                                hatalar.push(mesaj.clone());
                                                kayitlar.push(mesaj);
                                            }
                                        }
                                    }
                                    Ok(_) => {}
                                    Err(hata) => {
                                        let mesaj = format!(
                                            "{} artıkları taranamadı: {hata}",
                                            uygulama.ad
                                        );
                                        hatalar.push(mesaj.clone());
                                        kayitlar.push(mesaj);
                                    }
                                }
                            }
                            Ok(kaldirma) => {
                                let mesaj = format!(
                                    "{} kaldırma komutu tamamlanmadı: {}",
                                    uygulama.ad,
                                    kaldirma.ozet()
                                );
                                hatalar.push(mesaj.clone());
                                kayitlar.push(mesaj);
                            }
                            Err(hata) => {
                                let mesaj = format!("{}: {hata}", uygulama.ad);
                                hatalar.push(mesaj.clone());
                                kayitlar.push(mesaj);
                            }
                        }
                    }
                    let yeni_liste = servis.uygulamalari_tara().ok();
                    Ok(TopluKaldirmaSonucu {
                        basarili,
                        hatalar,
                        kayitlar,
                        artik_tasinan,
                        yeni_liste,
                    })
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kaldirici_islemde = false;
                match sonuc {
                    Ok(sonuc) => {
                        if let Some(yeni_liste) = sonuc.yeni_liste {
                            this.kaldirici_uygulamalar = yeni_liste;
                        }
                        for kimlik in &sonuc.basarili {
                            this.kaldirici_secili.remove(kimlik);
                        }
                        let mesaj = format!(
                            "{} uygulama kaldırma komutu tamamlandı, {} artık karantinaya alındı, {} hata.",
                            sonuc.basarili.len(),
                            sonuc.artik_tasinan,
                            sonuc.hatalar.len()
                        );
                        this.kaldirici_mesaj = Some(mesaj.clone());
                        this.kaldirici_hata = if sonuc.hatalar.is_empty() {
                            None
                        } else {
                            Some(sonuc.hatalar.join("\n"))
                        };
                        for kayit in sonuc.kayitlar {
                            this.kaldirici_gecmis.push(kayit);
                        }
                        this.kaldirici_gecmis = this
                            .kaldirici_gecmis
                            .iter()
                            .rev()
                            .take(10)
                            .cloned()
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect();
                    }
                    Err(hata) => this.kaldirici_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kaldirici_onay_iste(&mut self, kimlik: String, sessiz: bool, cx: &mut Context<Self>) {
        self.kaldirici_onay = Some(KaldiriciOnay::Kaldir { kimlik, sessiz });
        self.kaldirici_hata = None;
        cx.notify();
    }

    fn kaldirici_onayla(&mut self, geri_noktasi: bool, cx: &mut Context<Self>) {
        let Some(onay) = self.kaldirici_onay.take() else {
            return;
        };
        if self.kaldirici_islemde {
            return;
        }
        if let KaldiriciOnay::TopluKaldir { kimlikler, sessiz } = &onay {
            self.kaldirici_toplu_kaldir(kimlikler.clone(), *sessiz, geri_noktasi, cx);
            return;
        }
        let servis = self.kaldirici.clone();
        let uygulamalar = self.kaldirici_uygulamalar.clone();
        self.kaldirici_islemde = true;
        self.kaldirici_hata = None;
        self.kaldirici_mesaj = Some("İşlem sürüyor...".to_string());
        cx.notify();

        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    match onay {
                        KaldiriciOnay::TopluKaldir { .. } => Err(sp_core::Hata::Beklenmeyen(
                            "Toplu kaldırma ayrı yürütülür.".to_string(),
                        )),
                        KaldiriciOnay::GeriNoktasi => servis
                            .geri_yukleme_noktasi("manuel nokta")
                            .map(|_| ("Geri yükleme noktası oluşturuldu.".to_string(), None)),
                        KaldiriciOnay::Kaldir { kimlik, sessiz } => {
                            let uygulama = uygulamalar
                                .iter()
                                .find(|uygulama| uygulama.kimlik == kimlik)
                                .cloned()
                                .ok_or_else(|| {
                                    sp_core::Hata::Beklenmeyen(
                                        "Uygulama listede bulunamadı.".to_string(),
                                    )
                                })?;
                            if geri_noktasi {
                                servis.geri_yukleme_noktasi(&format!(
                                    "{} kaldırma öncesi",
                                    uygulama.ad
                                ))?;
                            }
                            let kaldirma = servis.kaldir(&uygulama, sessiz)?;
                            let yeni_liste = servis.uygulamalari_tara().ok();
                            let mesaj = if kaldirma.basarili() {
                                format!("{} kaldırma komutu tamamlandı.", uygulama.ad)
                            } else {
                                format!(
                                    "{} kaldırma komutu tamamlanmadı: {}",
                                    uygulama.ad,
                                    kaldirma.ozet()
                                )
                            };
                            Ok((mesaj, yeni_liste))
                        }
                    }
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kaldirici_islemde = false;
                match sonuc {
                    Ok((mesaj, yeni_liste)) => {
                        if let Some(yeni_liste) = yeni_liste {
                            this.kaldirici_uygulamalar = yeni_liste;
                        }
                        this.kaldirici_mesaj = Some(mesaj.clone());
                        this.kaldirici_hata = None;
                        this.kaldirici_gecmis.push(mesaj);
                        this.kaldirici_gecmis = this
                            .kaldirici_gecmis
                            .iter()
                            .rev()
                            .take(10)
                            .cloned()
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect();
                    }
                    Err(hata) => this.kaldirici_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn artik_tara(&mut self, uygulama: KuruluUygulama, cx: &mut Context<Self>) {
        if self.artik_taraniyor {
            return;
        }
        self.artik_uygulama = Some(uygulama.clone());
        self.artik_ogeler.clear();
        self.artik_taraniyor = true;
        self.artik_hata = None;
        self.artik_mesaj = Some(format!("{} artıkları taranıyor...", uygulama.ad));
        cx.notify();
        let servis = self.artik.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.tara(&uygulama) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.artik_taraniyor = false;
                match sonuc {
                    Ok(ogeler) => {
                        this.artik_mesaj = Some(format!("{} artık bulundu.", ogeler.len()));
                        this.artik_ogeler = ogeler;
                        this.artik_hata = None;
                    }
                    Err(hata) => this.artik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn store_tara(&mut self, cx: &mut Context<Self>) {
        if self.store_taraniyor {
            return;
        }
        self.store_taraniyor = true;
        self.store_hata = None;
        self.store_mesaj = Some("Store uygulamaları taranıyor...".to_string());
        cx.notify();
        let servis = self.artik.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.store_paketleri() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.store_taraniyor = false;
                match sonuc {
                    Ok(paketler) => {
                        this.store_mesaj =
                            Some(format!("{} Store uygulaması bulundu.", paketler.len()));
                        this.store_paketleri = paketler;
                        this.store_hata = None;
                    }
                    Err(hata) => this.store_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kaldirici_satiri(
        &self,
        uygulama: &KuruluUygulama,
        islemde: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let kimlik = uygulama.kimlik.clone();
        let sessiz_kimlik = uygulama.kimlik.clone();
        let uygulama_artik = uygulama.clone();
        let kaldirilabilir = uygulama.kaldirilabilir();
        let sessiz_kaldirilabilir = uygulama.sessiz_kaldirilabilir();
        let surum = uygulama
            .surum
            .clone()
            .unwrap_or_else(|| "sürüm bilinmiyor".to_string());
        let yayinci = uygulama
            .yayinci
            .clone()
            .unwrap_or_else(|| "yayıncı bilinmiyor".to_string());
        let ek_bilgi = [uygulama.boyut_metni(), uygulama.kurulum_tarihi.clone()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" · ");

        let mut aksiyonlar = h_flex().gap_2().items_center().child(
            Button::new(format!("kaldir-{}", uygulama.kimlik))
                .danger()
                .label("Kaldır")
                .disabled(!kaldirilabilir || islemde)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.kaldirici_onay_iste(kimlik.clone(), false, cx);
                })),
        );
        if sessiz_kaldirilabilir {
            aksiyonlar = aksiyonlar.child(
                Button::new(format!("sessiz-{}", uygulama.kimlik))
                    .label("Sessiz Kaldır")
                    .disabled(islemde)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.kaldirici_onay_iste(sessiz_kimlik.clone(), true, cx);
                    })),
            );
        }
        aksiyonlar = aksiyonlar.child(
            Button::new(format!("artik-{}", uygulama.kimlik))
                .ghost()
                .label("Artıklar")
                .disabled(self.artik_taraniyor)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.artik_tara(uygulama_artik.clone(), cx);
                })),
        );

        h_flex()
            .justify_between()
            .items_center()
            .p_3()
            .gap_4()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(uygulama.ad.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(uygulama.kaynak.metin()),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{surum} · {yayinci}")),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(if ek_bilgi.is_empty() {
                                "Boyut ve kurulum tarihi kayıtlı değil.".to_string()
                            } else {
                                ek_bilgi
                            }),
                    )
                    .child(if kaldirilabilir {
                        div().into_any_element()
                    } else {
                        div()
                            .text_xs()
                            .text_color(cx.theme().danger)
                            .child("Kaldırma komutu kayıtlı değil; bu kayıt kaldırılamaz.")
                            .into_any_element()
                    }),
            )
            .child(aksiyonlar)
    }

    fn kaldirici_toplu_satiri(
        &self,
        uygulama: &KuruluUygulama,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let secili = self.kaldirici_secili.contains(&uygulama.kimlik);
        let anahtar = Self::kimlik_ozeti(&uygulama.kimlik);
        let secim = self.kaldirici_secim_kutusu(anahtar, uygulama.kimlik.clone(), secili, cx);
        let surum = uygulama
            .surum
            .clone()
            .unwrap_or_else(|| "sürüm bilinmiyor".to_string());
        let yayinci = uygulama
            .yayinci
            .clone()
            .unwrap_or_else(|| "yayıncı bilinmiyor".to_string());
        h_flex()
            .gap_3()
            .items_center()
            .p_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(secim)
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(uygulama.ad.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(uygulama.kaynak.metin()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{surum} · {yayinci}")),
                    ),
            )
    }

    fn kaldirici_sayfasi(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let girdi = self.kaldirici_arama_girdisi(window, cx);
        let arama = girdi.read(cx).value().to_string().to_lowercase();
        let toplam = self.kaldirici_uygulamalar.len();
        let uygulamalar = self
            .kaldirici_uygulamalar
            .iter()
            .filter(|uygulama| {
                arama.is_empty()
                    || uygulama.ad.to_lowercase().contains(&arama)
                    || uygulama
                        .yayinci
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&arama)
                    || uygulama
                        .surum
                        .as_deref()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&arama)
            })
            .cloned()
            .collect::<Vec<_>>();
        let islemde = self.kaldirici_islemde;
        let taraniyor = self.kaldirici_taraniyor;
        let yonetici = self.motor.sistem().yonetici;
        let mesaj = self.kaldirici_mesaj.clone();
        let hata = self.kaldirici_hata.clone();
        let gecmis = self.kaldirici_gecmis.clone();
        let onay = self.kaldirici_onay.clone();
        let artik_uygulama = self.artik_uygulama.clone();
        let artik_ogeler = self.artik_ogeler.clone();
        let artik_mesaj = self.artik_mesaj.clone();
        let artik_hata = self.artik_hata.clone();
        let artik_taraniyor = self.artik_taraniyor;
        let store_paketleri = self.store_paketleri.clone();
        let store_mesaj = self.store_mesaj.clone();
        let store_hata = self.store_hata.clone();
        let store_taraniyor = self.store_taraniyor;
        let secilebilir = uygulamalar
            .iter()
            .filter(|uygulama| uygulama.kaldirilabilir())
            .map(|uygulama| uygulama.kimlik.clone())
            .collect::<Vec<_>>();
        let secilen = uygulamalar
            .iter()
            .filter(|uygulama| self.kaldirici_secili.contains(&uygulama.kimlik))
            .map(|uygulama| uygulama.kimlik.clone())
            .collect::<Vec<_>>();
        let sessiz_secilebilir = uygulamalar
            .iter()
            .filter(|uygulama| {
                self.kaldirici_secili.contains(&uygulama.kimlik) && uygulama.sessiz_kaldirilabilir()
            })
            .map(|uygulama| uygulama.kimlik.clone())
            .collect::<Vec<_>>();

        let onay_paneli = if let Some(onay) = onay {
            let (ad, komut, kaldirma_mi) = match &onay {
                KaldiriciOnay::Kaldir { kimlik, sessiz } => {
                    let uygulama = self
                        .kaldirici_uygulamalar
                        .iter()
                        .find(|uygulama| &uygulama.kimlik == kimlik);
                    let komut = uygulama.and_then(|uygulama| {
                        if *sessiz {
                            uygulama.sessiz_kaldirma_komutu.clone()
                        } else {
                            uygulama.kaldirma_komutu.clone()
                        }
                    });
                    (
                        uygulama
                            .map(|uygulama| uygulama.ad.clone())
                            .unwrap_or_else(|| "Uygulama".to_string()),
                        komut.unwrap_or_else(|| "Kaldırma komutu bulunamadı.".to_string()),
                        true,
                    )
                }
                KaldiriciOnay::TopluKaldir { kimlikler, sessiz } => (
                    format!("{} uygulama kaldırılacak", kimlikler.len()),
                    if *sessiz {
                        "Seçilen uygulamalar için sessiz kaldırma komutları sırayla çalıştırılacak; ardından artıklar taranıp karantinaya taşınacak. Sessiz komutu olmayanlar atlanır.".to_string()
                    } else {
                        "Seçilen uygulamaların kendi kaldırıcıları sırayla açılacak; ardından artıklar taranıp karantinaya taşınacak. Her uygulama için onay penceresi gelebilir.".to_string()
                    },
                    true,
                ),
                KaldiriciOnay::GeriNoktasi => (
                    "Sistem geri yükleme noktası".to_string(),
                    "Checkpoint-Computer ile geri yükleme noktası oluşturulacak.".to_string(),
                    false,
                ),
            };
            let mut butonlar =
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("kaldirici-vazgec")
                            .label("Vazgeç")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.kaldirici_onay = None;
                                cx.notify();
                            })),
                    );
            if kaldirma_mi {
                butonlar = butonlar
                    .child(
                        Button::new("kaldirici-geri-ile")
                            .primary()
                            .label("Geri Noktası + Kaldır")
                            .disabled(islemde)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.kaldirici_onayla(true, cx);
                            })),
                    )
                    .child(
                        Button::new("kaldirici-geri-olmadan")
                            .danger()
                            .label("Geri Noktası Olmadan")
                            .disabled(islemde)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.kaldirici_onayla(false, cx);
                            })),
                    );
            } else {
                butonlar = butonlar.child(
                    Button::new("kaldirici-nokta-onayla")
                        .primary()
                        .label("Onayla")
                        .disabled(islemde)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.kaldirici_onayla(false, cx);
                        })),
                );
            }
            v_flex()
                .p_4()
                .gap_2()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().warning)
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Onay gerekiyor"),
                )
                .child(div().text_sm().child(ad))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(komut),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(if kaldirma_mi {
                            "Kaldırma komutu çalıştırılacak. Önce geri yükleme noktası oluşturulması önerilir. Uygulamanın kendi kaldırıcısı açılabilir."
                        } else {
                            "Sistem koruması açık değilse veya sık oluşturma sınırına takılırsanız işlem başarısız olabilir."
                        }),
                )
                .child(butonlar)
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let liste = if taraniyor {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Kurulu uygulamalar taranıyor...")
                .into_any_element()
        } else if self.kaldirici_uygulamalar.is_empty() {
            v_flex()
                .gap_2()
                .p_4()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Henüz tarama yapılmadı"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Kurulu masaüstü uygulamalarını listelemek için Tara düğmesini kullanın. Sistem bileşenleri ve güncellemeler gizlenir."),
                )
                .child(
                    Button::new("kaldirici-bos-tara")
                        .primary()
                        .label("Uygulamaları Tara")
                        .on_click(cx.listener(|this, _, _, cx| this.kaldirici_tara(cx))),
                )
                .into_any_element()
        } else if uygulamalar.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Arama ile eşleşen uygulama yok.")
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(if self.kaldirici_toplu_mod {
                    uygulamalar
                        .iter()
                        .map(|uygulama| {
                            self.kaldirici_toplu_satiri(uygulama, cx).into_any_element()
                        })
                        .collect::<Vec<_>>()
                } else {
                    uygulamalar
                        .iter()
                        .map(|uygulama| {
                            self.kaldirici_satiri(uygulama, islemde, cx)
                                .into_any_element()
                        })
                        .collect::<Vec<_>>()
                })
                .into_any_element()
        };

        let gecmis_karti = if gecmis.is_empty() {
            div().into_any_element()
        } else {
            v_flex()
                .p_4()
                .gap_2()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Son işlemler"),
                )
                .children(gecmis.iter().map(|kayit| {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(kayit.clone())
                }))
                .into_any_element()
        };

        let sistem_onay_paneli = if let Some(onay) = &self.sistem_onay {
            self.sistem_onay_paneli(onay.clone(), cx)
        } else {
            div().into_any_element()
        };

        let artik_karti = if let Some(uygulama) = &artik_uygulama {
            let kimlikler = artik_ogeler
                .iter()
                .map(|oge| oge.kimlik.clone())
                .collect::<Vec<_>>();
            v_flex()
                .p_4()
                .gap_2()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("Artıklar: {}", uygulama.ad)),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("artik-yenile")
                                        .label("Yeniden Tara")
                                        .disabled(artik_taraniyor)
                                        .on_click(cx.listener({
                                            let uygulama = uygulama.clone();
                                            move |this, _, _, cx| {
                                                this.artik_tara(uygulama.clone(), cx);
                                            }
                                        })),
                                )
                                .child(
                                    Button::new("artik-karantina")
                                        .danger()
                                        .label("Karantinaya Taşı")
                                        .disabled(kimlikler.is_empty() || artik_taraniyor)
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.sistem_onay_iste(
                                                SistemOnay::ArtikKarantina(kimlikler.clone()),
                                                cx,
                                            );
                                        })),
                                ),
                        ),
                )
                .child(if let Some(hata) = &artik_hata {
                    self.hata_kutusu(hata, cx).into_any_element()
                } else {
                    div().into_any_element()
                })
                .child(if let Some(mesaj) = &artik_mesaj {
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(mesaj.clone())
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .children(artik_ogeler.iter().take(200).map(|oge| {
                    self.modul_satiri(
                        oge.ad.clone(),
                        format!("{} · {}", oge.tur.metin(), oge.yol),
                        oge.boyut_bayt.map(Self::bayt_metni).unwrap_or_default(),
                        Vec::new(),
                        cx,
                    )
                }))
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let store_karti = v_flex()
            .p_4()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Store Uygulamaları"),
                    )
                    .child(
                        Button::new("store-tara")
                            .label(if store_taraniyor {
                                "Taranıyor..."
                            } else {
                                "Tara"
                            })
                            .disabled(store_taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| this.store_tara(cx))),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Microsoft/Windows sistem paketleri korunur; yalnızca üçüncü taraf paketler kaldırılabilir."),
            )
            .child(if let Some(hata) = &store_hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &store_mesaj {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(mesaj.clone())
                    .into_any_element()
            } else {
                div().into_any_element()
            })
            .children(store_paketleri.iter().take(300).map(|paket| {
                let tam_ad = paket.tam_ad.clone();
                let ad = paket.ad.clone();
                let sistem = Self::sistem_paketi_mi(&paket.ad, &paket.yayinci);
                let aksiyon = if sistem {
                    Button::new(format!("store-sistem-{}", paket.tam_ad))
                        .label("Sistem")
                        .disabled(true)
                        .into_any_element()
                } else {
                    Button::new(format!("store-kaldir-{}", paket.tam_ad))
                        .danger()
                        .label("Kaldır")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.sistem_onay_iste(
                                SistemOnay::StoreKaldir(ad.clone(), tam_ad.clone()),
                                cx,
                            );
                        }))
                        .into_any_element()
                };
                self.modul_satiri(
                    paket.ad.clone(),
                    format!("{} · {}", paket.surum, paket.yayinci),
                    paket.tam_ad.clone(),
                    vec![aksiyon],
                    cx,
                )
            }))
            .into_any_element();

        let sayfa = v_flex()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if self.kaldirici_toplu_mod {
                                        "Toplu seçim modu: kutucuklarla uygulama seçin; kaldırma ve artık taraması otomatiktir."
                                    } else {
                                        "Tekil kaldırma modu: her uygulama için Kaldır, Sessiz Kaldır veya Artıklar."
                                    }),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .flex_wrap()
                            .child(
                                Button::new("kaldirici-mod")
                                    .label(if self.kaldirici_toplu_mod {
                                        "Normal Görünüm"
                                    } else {
                                        "Toplu Seçim"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.kaldirici_toplu_mod = !this.kaldirici_toplu_mod;
                                        cx.notify();
                                    })),
                            )
                            .when(self.kaldirici_toplu_mod, |this| {
                                this.child(
                                    Button::new("kaldirici-tumunu-sec")
                                        .label("Tümünü Seç")
                                        .disabled(secilebilir.is_empty() || islemde)
                                        .on_click(cx.listener({
                                            let secilebilir = secilebilir.clone();
                                            move |this, _, _, cx| {
                                                for kimlik in secilebilir.clone() {
                                                    this.kaldirici_secili.insert(kimlik);
                                                }
                                                cx.notify();
                                            }
                                        })),
                                )
                                .child(
                                    Button::new("kaldirici-secimi-temizle")
                                        .label("Seçimi Temizle")
                                        .disabled(self.kaldirici_secili.is_empty())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.kaldirici_secili.clear();
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("kaldirici-secili-kaldir")
                                        .danger()
                                        .label(format!(
                                            "Seçilenleri Kaldır ({})",
                                            secilen.len()
                                        ))
                                        .disabled(secilen.is_empty() || islemde)
                                        .on_click(cx.listener({
                                            let secilen = secilen.clone();
                                            move |this, _, _, cx| {
                                                this.kaldirici_onay = Some(
                                                    KaldiriciOnay::TopluKaldir {
                                                        kimlikler: secilen.clone(),
                                                        sessiz: false,
                                                    },
                                                );
                                                this.kaldirici_hata = None;
                                                cx.notify();
                                            }
                                        })),
                                )
                                .child(
                                    Button::new("kaldirici-secili-sessiz")
                                        .label(format!(
                                            "Sessiz Kaldır ({})",
                                            sessiz_secilebilir.len()
                                        ))
                                        .disabled(sessiz_secilebilir.is_empty() || islemde)
                                        .on_click(cx.listener({
                                            let secilen = sessiz_secilebilir.clone();
                                            move |this, _, _, cx| {
                                                this.kaldirici_onay = Some(
                                                    KaldiriciOnay::TopluKaldir {
                                                        kimlikler: secilen.clone(),
                                                        sessiz: true,
                                                    },
                                                );
                                                this.kaldirici_hata = None;
                                                cx.notify();
                                            }
                                        })),
                                )
                            })
                            .when(!self.kaldirici_toplu_mod, |this| {
                                this.child(
                                    Button::new("kaldirici-nokta")
                                        .label("Geri Noktası Oluştur")
                                        .disabled(islemde)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.kaldirici_onay =
                                                Some(KaldiriciOnay::GeriNoktasi);
                                            this.kaldirici_hata = None;
                                            cx.notify();
                                        })),
                                )
                            })
                            .child(
                                Button::new("kaldirici-tara")
                                    .primary()
                                    .label(if taraniyor { "Taranıyor..." } else { "Tara" })
                                    .disabled(taraniyor || islemde)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.kaldirici_tara(cx);
                                    })),
                            ),
                    ),
            )
            .child(if let Some(hata) = &hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if yonetici {
                div().into_any_element()
            } else {
                self.uyari_kutusu(
                    "Kaldırma işlemleri ve geri yükleme noktası için yönetici izni gerekebilir.",
                    cx,
                )
                .into_any_element()
            })
            .child(onay_paneli)
            .child(sistem_onay_paneli)
            .child(store_karti)
            .child(artik_karti)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(div().text_sm().child("Ara"))
                    .child(Input::new(&girdi).cleanable(true).w_full())
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} / {toplam} uygulama", uygulamalar.len())),
                    ),
            )
            .child(liste)
            .child(gecmis_karti);
        sayfa.into_any_element()
    }

    fn sistem_yonetimi_tara(&mut self, cx: &mut Context<Self>) {
        if self.sistem_taraniyor {
            return;
        }
        self.sistem_taraniyor = true;
        self.sistem_hata = None;
        self.sistem_mesaj = Some("Başlangıç, servis, görev ve süreçler taranıyor...".to_string());
        cx.notify();
        let servis = self.sistem_yonetimi.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    std::thread::scope(|kapsam| {
                        let baslangic = kapsam.spawn(|| servis.baslangic_ogeleri());
                        let servisler = kapsam.spawn(|| servis.servisler());
                        let gorevler = kapsam.spawn(|| servis.gorevler());
                        let surecler = kapsam.spawn(|| servis.surecler());
                        (
                            is_parcasi(baslangic.join(), "Başlangıç"),
                            is_parcasi(servisler.join(), "Servisler"),
                            is_parcasi(gorevler.join(), "Görevler"),
                            is_parcasi(surecler.join(), "Süreçler"),
                        )
                    })
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.sistem_taraniyor = false;
                let (baslangic, servisler, gorevler, surecler) = sonuc;
                let mut hatalar = Vec::new();
                this.baslangic_ogeleri = baslangic.unwrap_or_else(|hata| {
                    hatalar.push(format!("Başlangıç: {hata}"));
                    Vec::new()
                });
                this.servisler = servisler.unwrap_or_else(|hata| {
                    hatalar.push(format!("Servisler: {hata}"));
                    Vec::new()
                });
                this.gorevler = gorevler.unwrap_or_else(|hata| {
                    hatalar.push(format!("Görevler: {hata}"));
                    Vec::new()
                });
                this.surecler = surecler.unwrap_or_else(|hata| {
                    hatalar.push(format!("Süreçler: {hata}"));
                    Vec::new()
                });
                this.sistem_mesaj = Some(format!(
                    "{} başlangıç, {} servis, {} görev, {} süreç bulundu.",
                    this.baslangic_ogeleri.len(),
                    this.servisler.len(),
                    this.gorevler.len(),
                    this.surecler.len()
                ));
                this.sistem_hata = (!hatalar.is_empty()).then(|| hatalar.join("\n"));
                cx.notify();
            });
        })
        .detach();
    }

    fn sistem_onay_iste(&mut self, onay: SistemOnay, cx: &mut Context<Self>) {
        self.sistem_onay = Some(onay);
        self.sistem_hata = None;
        cx.notify();
    }

    fn arka_plan_islem<F>(&mut self, cx: &mut Context<Self>, alan: IslemAlani, islem: F)
    where
        F: FnOnce() -> sp_core::Sonuc<String> + Send + 'static,
    {
        match alan {
            IslemAlani::Sistem => {
                self.sistem_taraniyor = true;
                self.sistem_hata = None;
                self.sistem_mesaj = Some("İşlem sürüyor...".to_string());
            }
            IslemAlani::Temizlik => {
                self.temizlik_taraniyor = true;
                self.temizlik_hata = None;
                self.temizlik_mesaj = Some("İşlem sürüyor...".to_string());
            }
            IslemAlani::Artik => {
                self.artik_taraniyor = true;
                self.artik_hata = None;
                self.artik_mesaj = Some("İşlem sürüyor...".to_string());
            }
            IslemAlani::Store => {
                self.store_taraniyor = true;
                self.store_hata = None;
                self.store_mesaj = Some("İşlem sürüyor...".to_string());
            }
        }
        cx.notify();
        cx.spawn(async move |this, cx| {
            let sonuc = cx.background_executor().spawn(async move { islem() }).await;
            let _ = this.update(cx, |this, cx| {
                match alan {
                    IslemAlani::Sistem => this.sistem_taraniyor = false,
                    IslemAlani::Temizlik => this.temizlik_taraniyor = false,
                    IslemAlani::Artik => this.artik_taraniyor = false,
                    IslemAlani::Store => this.store_taraniyor = false,
                }
                match sonuc {
                    Ok(mesaj) => match alan {
                        IslemAlani::Sistem => {
                            this.sistem_mesaj = Some(mesaj);
                            this.sistem_hata = None;
                            this.sistem_yonetimi_tara(cx);
                        }
                        IslemAlani::Temizlik => {
                            this.temizlik_mesaj = Some(mesaj);
                            this.temizlik_hata = None;
                            this.temizlik_tara(cx);
                        }
                        IslemAlani::Artik => {
                            this.artik_mesaj = Some(mesaj);
                            this.artik_hata = None;
                            if let Some(uygulama) = this.artik_uygulama.clone() {
                                this.artik_tara(uygulama, cx);
                            }
                        }
                        IslemAlani::Store => {
                            this.store_mesaj = Some(mesaj);
                            this.store_hata = None;
                            this.store_tara(cx);
                        }
                    },
                    Err(hata) => match alan {
                        IslemAlani::Sistem => this.sistem_hata = Some(hata.to_string()),
                        IslemAlani::Temizlik => this.temizlik_hata = Some(hata.to_string()),
                        IslemAlani::Artik => this.artik_hata = Some(hata.to_string()),
                        IslemAlani::Store => this.store_hata = Some(hata.to_string()),
                    },
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kimlik_ozeti(kimlik: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut ozet = std::collections::hash_map::DefaultHasher::new();
        kimlik.hash(&mut ozet);
        format!("{:016x}", ozet.finish())
    }

    fn zaman_damgasi() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|sure| sure.as_secs())
            .unwrap_or(0)
    }

    fn bakim_islem<F>(&mut self, cx: &mut Context<Self>, islem: F, dism: bool)
    where
        F: FnOnce() -> sp_core::Sonuc<String> + Send + 'static,
    {
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("İşlem sürüyor...".to_string());
        cx.notify();
        cx.spawn(async move |this, cx| {
            let sonuc = cx.background_executor().spawn(async move { islem() }).await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(cikti) => {
                        if dism {
                            this.dism_ciktisi = Some(cikti);
                            this.bakim_mesaj = Some("İşlem tamamlandı; çıktı aşağıda.".to_string());
                        } else {
                            this.bakim_mesaj = Some(if cikti.is_empty() {
                                "İşlem tamamlandı.".to_string()
                            } else {
                                cikti
                            });
                        }
                        this.bakim_hata = None;
                    }
                    Err(hata) => {
                        this.bakim_hata = Some(hata.to_string());
                        this.bakim_mesaj = Some("İşlem başarısız.".to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn sistem_onay_metni(&self, onay: &SistemOnay) -> (String, String, bool) {
        match onay {
            SistemOnay::BaslangicEtkinlestir(kimlik) => {
                let ad = self
                    .baslangic_ogeleri
                    .iter()
                    .find(|oge| &oge.kimlik == kimlik)
                    .map(|oge| oge.ad.clone())
                    .unwrap_or_else(|| "Başlangıç öğesi".to_string());
                (
                    format!("{ad} etkinleştirilecek"),
                    "Kayıt yeniden başlangıçta çalışacak. Devam edilsin mi?".to_string(),
                    false,
                )
            }
            SistemOnay::BaslangicDevreDisi(kimlik) => {
                let ad = self
                    .baslangic_ogeleri
                    .iter()
                    .find(|oge| &oge.kimlik == kimlik)
                    .map(|oge| oge.ad.clone())
                    .unwrap_or_else(|| "Başlangıç öğesi".to_string());
                (
                    format!("{ad} devre dışı bırakılacak"),
                    "Kayıt yedeklenerek kaldırılacak; daha sonra etkinleştirilebilir.".to_string(),
                    false,
                )
            }
            SistemOnay::ServisBaslat(ad) => (
                format!("{ad} servisi başlatılacak"),
                "Servis başlatma işlemi yönetici izni gerektirebilir.".to_string(),
                false,
            ),
            SistemOnay::ServisDurdur(ad) => (
                format!("{ad} servisi durdurulacak"),
                "Bu servise bağlı uygulamalar etkilenebilir. Devam edilsin mi?".to_string(),
                true,
            ),
            SistemOnay::ServisBaslangic(ad, tur) => {
                let tur = match tur.as_str() {
                    "auto" => "Otomatik",
                    "demand" => "Manuel",
                    "disabled" => "Devre dışı",
                    _ => tur.as_str(),
                };
                (
                    format!("{ad} servisinin başlangıç türü değiştirilecek"),
                    format!("Yeni tür: {tur}. Devam edilsin mi?"),
                    false,
                )
            }
            SistemOnay::GorevCalistir(ad, _) => (
                format!("{ad} görevi çalıştırılacak"),
                "Görev hemen başlatılacak. Devam edilsin mi?".to_string(),
                false,
            ),
            SistemOnay::GorevDurum(ad, _, etkin) => (
                format!("{ad} görevi {}", if *etkin { "etkinleştirilecek" } else { "devre dışı bırakılacak" }),
                "Görev zamanlaması değiştirilecek. Devam edilsin mi?".to_string(),
                false,
            ),
            SistemOnay::SurecSonlandir(pid, ad) => (
                format!("{ad} (PID {pid}) sonlandırılacak"),
                "Kaydedilmemiş veriler kaybolabilir. Zorla sonlandırma yapılacak.".to_string(),
                true,
            ),
            SistemOnay::Temizlik(kimlik) => {
                let hedef = self
                    .temizlik_hedefleri
                    .iter()
                    .find(|hedef| &hedef.kimlik == kimlik);
                (
                    "Geçici dosyalar temizlenecek".to_string(),
                    format!(
                        "{} · {}. Kullanımdaki dosyalar atlanır.",
                        hedef.map(|hedef| hedef.ad.clone()).unwrap_or_default(),
                        hedef.map(|hedef| hedef.yol.clone()).unwrap_or_default()
                    ),
                    true,
                )
            }
            SistemOnay::BosKlasorSil(klasorler) => (
                "Boş klasörler silinecek".to_string(),
                format!("{} boş klasör kaldırılacak. Devam edilsin mi?", klasorler.len()),
                true,
            ),
            SistemOnay::YinelenenKarantina(dosyalar) => (
                "Yinelenen dosyalar karantinaya taşınacak".to_string(),
                format!(
                    "{} kopya taşınacak; ilk dosyalar korunur. Devam edilsin mi?",
                    dosyalar.len()
                ),
                true,
            ),
            SistemOnay::Parcala(dosya) => (
                "Dosya parçalanacak".to_string(),
                format!("{dosya} üzerine 3 geçiş yazılıp silinecek. Geri alınamaz."),
                true,
            ),
            SistemOnay::GeriDonusum => (
                "Geri dönüşüm kutusu temizlenecek".to_string(),
                "Tüm öğeler kalıcı olarak silinecek. Geri alınamaz.".to_string(),
                true,
            ),
            SistemOnay::DismAnaliz => (
                "DISM analizi çalıştırılacak".to_string(),
                "Bileşen deposu analiz edilecek; sistem değişikliği yapılmaz.".to_string(),
                false,
            ),
            SistemOnay::DismTemizle => (
                "DISM bileşen temizliği çalıştırılacak".to_string(),
                "Eski bileşen sürümleri kaldırılır; geri alınamaz ve uzun sürebilir.".to_string(),
                true,
            ),
            SistemOnay::DismSaglik => (
                "DISM sağlık taraması çalıştırılacak".to_string(),
                "Bileşen deposu bozulması taranacak; sistem değişikliği yapılmaz.".to_string(),
                false,
            ),
            SistemOnay::DismOnar => (
                "DISM onarımı çalıştırılacak".to_string(),
                "RestoreHealth uzun sürebilir ve kaynak/internet gerektirebilir.".to_string(),
                true,
            ),
            SistemOnay::AppxKaldir(ad, _) => (
                format!("{ad} Appx paketi kaldırılacak"),
                "Paket kaldırma geri alınamaz; uygulama verisi etkilenebilir.".to_string(),
                true,
            ),
            SistemOnay::OzellikAyarla(ad, etkin) => (
                format!("{ad} özelliği {}", if *etkin { "açılacak" } else { "kapatılacak" }),
                "İşlem yönetici izni gerektirir ve yeniden başlatma isteyebilir.".to_string(),
                true,
            ),
            SistemOnay::SurucuDisaAktar(inf, hedef) => (
                format!("{inf} sürücüsü dışa aktarılacak"),
                format!("Hedef: {hedef}. Devam edilsin mi?"),
                false,
            ),
            SistemOnay::ArtikKarantina(kimlikler) => (
                "Artıklar karantinaya taşınacak".to_string(),
                format!(
                    "{} dosya/klasör ve registry kaydı işlenecek; registry kayıtları .reg olarak yedeklenir. Devam edilsin mi?",
                    kimlikler.len()
                ),
                true,
            ),
            SistemOnay::StoreKaldir(ad, _) => (
                format!("{ad} Store paketi kaldırılacak"),
                "Paket yalnızca geçerli kullanıcıdan kaldırılır. Devam edilsin mi?".to_string(),
                true,
            ),
            SistemOnay::GelismisTemizlik(kimlik) => {
                let kategori = sp_core::onarim::kategoriler()
                    .into_iter()
                    .find(|kategori| &kategori.kimlik == kimlik);
                let (ad, aciklama, riskli) = kategori
                    .map(|kategori| {
                        (kategori.ad, kategori.aciklama, kategori.riskli)
                    })
                    .unwrap_or_else(|| {
                        (
                            "Gelişmiş temizlik".to_string(),
                            "Seçilen kategori temizlenecek.".to_string(),
                            false,
                        )
                    });
                (
                    format!("{ad} temizlenecek"),
                    if riskli {
                        format!(
                            "{aciklama} Bu kategori riskli olarak işaretlendi; onarım/kaldırma işlemleri için yeniden indirme gerekebilir. Devam edilsin mi?"
                        )
                    } else {
                        format!("{aciklama} Devam edilsin mi?")
                    },
                    true,
                )
            }
            SistemOnay::ChromeEskiSil(yollar) => (
                "Eski Chrome sürümleri silinecek".to_string(),
                format!(
                    "{} eski sürüm klasörü kalıcı olarak silinecek. Geçerli Chrome sürümü korunur. Devam edilsin mi?",
                    yollar.len()
                ),
                true,
            ),
            SistemOnay::Sfc => (
                "Sistem Dosyası Denetleyicisi çalıştırılacak".to_string(),
                "sfc /scannow uzun sürebilir ve sistem dosyalarını onarabilir. Devam edilsin mi?"
                    .to_string(),
                true,
            ),
            SistemOnay::DismResetBase => (
                "DISM ResetBase çalıştırılacak".to_string(),
                "Tüm eski bileşen sürümleri kaldırılır; yüklenen güncellemeler geri alınamaz. Devam edilsin mi?".to_string(),
                true,
            ),
            SistemOnay::AppxOnar(ad, _) => (
                format!("{ad} Appx paketi onarılacak"),
                "Paket yeniden kaydedilecek; kullanıcı verileri korunur. Devam edilsin mi?"
                    .to_string(),
                false,
            ),
            SistemOnay::GuncellemeGecmisiTemizle => (
                "Windows Update veri deposu temizlenecek".to_string(),
                "Güncelleme hizmetleri durdurulup SoftwareDistribution veri deposu ve günlükleri silinir. Devam edilsin mi?".to_string(),
                true,
            ),
            SistemOnay::EskiGeriNoktalariSil => (
                "Eski geri yükleme noktaları silinecek".to_string(),
                "En yeni geri yükleme noktası korunur; diğerleri silinir. Devam edilsin mi?"
                    .to_string(),
                true,
            ),
            SistemOnay::GeriNoktasiOlustur => (
                "Geri yükleme noktası oluşturulacak".to_string(),
                "Sistem Koruması açık olmalıdır. Windows 24 saatte bir nokta oluşturmaya izin verir. Devam edilsin mi?".to_string(),
                false,
            ),
            SistemOnay::GeriDon(sira, aciklama) => (
                format!("#{sira} · {aciklama} geri yüklenecek"),
                "Sistem geri yükleme tamamlandığında bilgisayar yeniden başlatılacak. Kaydedilmemiş çalışmalarınızı kaydedin. Devam edilsin mi?".to_string(),
                true,
            ),
            SistemOnay::GeriKoruma(etkin) => (
                if *etkin {
                    "Sistem Koruması açılacak".to_string()
                } else {
                    "Sistem Koruması kapatılacak".to_string()
                },
                if *etkin {
                    "Sistem sürücüsünde geri yükleme noktalarının oluşturulmasına izin verilir. Devam edilsin mi?".to_string()
                } else {
                    "Sistem Koruması kapatıldığında yeni geri yükleme noktası oluşturulamaz. Devam edilsin mi?".to_string()
                },
                true,
            ),
            SistemOnay::IkonOnbellekOnar => (
                "Simge önbelleği onarılacak".to_string(),
                "Gezgin geçici olarak kapatılacak, simge ve küçük resim önbelleği silinip yeniden oluşturulacak. Devam edilsin mi?".to_string(),
                true,
            ),
            SistemOnay::KayitTemizle(kimlikler) => (
                "Registry artıkları karantinaya taşınacak".to_string(),
                format!(
                    "{} kayıt önce .reg olarak yedeklenecek, sonra silinecek. Devam edilsin mi?",
                    kimlikler.len()
                ),
                true,
            ),
            SistemOnay::KisayolTemizle(yollar) => (
                "Bozuk kısayollar karantinaya taşınacak".to_string(),
                format!(
                    "{} .lnk dosyası silinmek yerine karantinaya taşınacak. Devam edilsin mi?",
                    yollar.len()
                ),
                true,
            ),
        }
    }

    fn sistem_onay_paneli(&self, onay: SistemOnay, cx: &mut Context<Self>) -> AnyElement {
        let (baslik, aciklama, tehlikeli) = self.sistem_onay_metni(&onay);
        let mesgul = self.sistem_taraniyor
            || self.temizlik_taraniyor
            || self.bakim_taraniyor
            || self.artik_taraniyor
            || self.store_taraniyor
            || self.kaldirici_islemde;
        let onay_butonu = if tehlikeli {
            Button::new("sistem-onayla").danger().label("Onayla")
        } else {
            Button::new("sistem-onayla").primary().label("Onayla")
        };
        v_flex()
            .p_4()
            .gap_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().warning)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Onay gerekiyor"),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(baslik),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(aciklama),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("sistem-vazgec")
                            .label("Vazgeç")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        onay_butonu
                            .disabled(mesgul)
                            .on_click(cx.listener(|this, _, _, cx| this.sistem_onayla(cx))),
                    ),
            )
            .into_any_element()
    }

    fn sistem_onayla(&mut self, cx: &mut Context<Self>) {
        let Some(onay) = self.sistem_onay.take() else {
            return;
        };
        match onay {
            SistemOnay::BaslangicEtkinlestir(kimlik) => {
                let Some(oge) = self
                    .baslangic_ogeleri
                    .iter()
                    .find(|oge| oge.kimlik == kimlik)
                    .cloned()
                else {
                    self.sistem_hata = Some("Başlangıç öğesi bulunamadı.".to_string());
                    cx.notify();
                    return;
                };
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .baslangic_etkinlestir(&oge)
                        .map(|()| format!("{} etkinleştirildi.", oge.ad))
                });
            }
            SistemOnay::BaslangicDevreDisi(kimlik) => {
                let Some(oge) = self
                    .baslangic_ogeleri
                    .iter()
                    .find(|oge| oge.kimlik == kimlik)
                    .cloned()
                else {
                    self.sistem_hata = Some("Başlangıç öğesi bulunamadı.".to_string());
                    cx.notify();
                    return;
                };
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .baslangic_devre_disi(&oge)
                        .map(|()| format!("{} devre dışı bırakıldı.", oge.ad))
                });
            }
            SistemOnay::ServisBaslat(ad) => {
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .servis_baslat(&ad)
                        .map(|()| format!("{ad} başlatıldı."))
                });
            }
            SistemOnay::ServisDurdur(ad) => {
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .servis_durdur(&ad)
                        .map(|()| format!("{ad} durduruldu."))
                });
            }
            SistemOnay::ServisBaslangic(ad, tur) => {
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .servis_baslangic_ayarla(&ad, &tur)
                        .map(|()| format!("{ad} başlangıç türü değiştirildi."))
                });
            }
            SistemOnay::GorevCalistir(ad, yol) => {
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .gorev_calistir(&ad, &yol)
                        .map(|()| format!("{ad} görevi başlatıldı."))
                });
            }
            SistemOnay::GorevDurum(ad, yol, etkin) => {
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis.gorev_etkinlestir(&ad, &yol, etkin).map(|()| {
                        format!(
                            "{ad} görevi {}.",
                            if etkin {
                                "etkinleştirildi"
                            } else {
                                "devre dışı bırakıldı"
                            }
                        )
                    })
                });
            }
            SistemOnay::SurecSonlandir(pid, ad) => {
                let servis = self.sistem_yonetimi.clone();
                self.arka_plan_islem(cx, IslemAlani::Sistem, move || {
                    servis
                        .surec_sonlandir(pid)
                        .map(|()| format!("{ad} (PID {pid}) sonlandırıldı."))
                });
            }
            SistemOnay::Temizlik(kimlik) => {
                let Some(hedef) = self
                    .temizlik_hedefleri
                    .iter()
                    .find(|hedef| hedef.kimlik == kimlik)
                    .cloned()
                else {
                    self.temizlik_hata = Some("Temizlik hedefi bulunamadı.".to_string());
                    cx.notify();
                    return;
                };
                let servis = self.temizleyici.clone();
                self.arka_plan_islem(cx, IslemAlani::Temizlik, move || {
                    servis.temizle(&hedef).map(|sonuc| {
                        format!(
                            "{} dosya, {} temizlendi. {} hata.",
                            sonuc.silinen_dosya,
                            Self::bayt_metni(sonuc.silinen_bayt),
                            sonuc.hatalar.len()
                        )
                    })
                });
            }
            SistemOnay::BosKlasorSil(klasorler) => {
                let Some(kok) = self.bos_klasor_kok.clone() else {
                    self.temizlik_hata = Some("Önce bir klasör seçin.".to_string());
                    cx.notify();
                    return;
                };
                let servis = self.temizleyici.clone();
                self.arka_plan_islem(cx, IslemAlani::Temizlik, move || {
                    servis.bos_klasorleri_sil(&kok, &klasorler).map(|sonuc| {
                        format!(
                            "{} boş klasör silindi. {} hata.",
                            sonuc.silinen_dosya,
                            sonuc.hatalar.len()
                        )
                    })
                });
            }
            SistemOnay::YinelenenKarantina(dosyalar) => {
                let Some(kok) = self.yinelenen_kok.clone() else {
                    self.temizlik_hata = Some("Önce bir klasör seçin.".to_string());
                    cx.notify();
                    return;
                };
                let karantina = self
                    .dizinler
                    .yerel
                    .join("quarantine")
                    .join(format!("yinelenen-{}", Self::zaman_damgasi()));
                let servis = self.temizleyici.clone();
                self.arka_plan_islem(cx, IslemAlani::Temizlik, move || {
                    servis
                        .yinelenenleri_karantinaya_tasi(&kok, &dosyalar, &karantina)
                        .map(|sonuc| {
                            format!(
                                "{} kopya karantinaya taşındı. {} hata.",
                                sonuc.tasinan,
                                sonuc.hatalar.len()
                            )
                        })
                });
            }
            SistemOnay::Parcala(dosya) => {
                let servis = self.temizleyici.clone();
                self.arka_plan_islem(cx, IslemAlani::Temizlik, move || {
                    servis
                        .parcala(std::path::Path::new(&dosya))
                        .map(|()| format!("{dosya} parçalandı."))
                });
            }
            SistemOnay::GeriDonusum => {
                let servis = self.temizleyici.clone();
                self.arka_plan_islem(cx, IslemAlani::Temizlik, move || {
                    servis
                        .geri_donusum_temizle()
                        .map(|()| "Geri dönüşüm kutusu temizlendi.".to_string())
                });
            }
            SistemOnay::DismAnaliz => {
                let servis = self.bakim.clone();
                self.bakim_islem(cx, move || servis.dism_analiz(), true);
            }
            SistemOnay::DismTemizle => {
                let servis = self.bakim.clone();
                self.bakim_islem(cx, move || servis.dism_temizle(), true);
            }
            SistemOnay::DismSaglik => {
                let servis = self.bakim.clone();
                self.bakim_islem(cx, move || servis.dism_saglik_tara(), true);
            }
            SistemOnay::DismOnar => {
                let servis = self.bakim.clone();
                self.bakim_islem(cx, move || servis.dism_onar(), true);
            }
            SistemOnay::AppxKaldir(ad, tam_ad) => {
                let servis = self.bakim.clone();
                self.bakim_islem(
                    cx,
                    move || {
                        servis
                            .appx_kaldir(&tam_ad)
                            .map(|()| format!("{ad} kaldırıldı."))
                    },
                    false,
                );
            }
            SistemOnay::OzellikAyarla(ad, etkin) => {
                let servis = self.bakim.clone();
                self.bakim_islem(
                    cx,
                    move || {
                        servis.ozellik_ayarla(&ad, etkin).map(|cikti| {
                            if cikti.is_empty() {
                                format!("{ad} {}.", if etkin { "açıldı" } else { "kapatıldı" })
                            } else {
                                cikti
                            }
                        })
                    },
                    false,
                );
            }
            SistemOnay::SurucuDisaAktar(inf, hedef) => {
                let servis = self.bakim.clone();
                self.bakim_islem(
                    cx,
                    move || {
                        servis
                            .surucu_disa_aktar(&inf, std::path::Path::new(&hedef))
                            .map(|()| format!("{inf} → {hedef}"))
                    },
                    false,
                );
            }
            SistemOnay::ArtikKarantina(kimlikler) => {
                let ogeler = self
                    .artik_ogeler
                    .iter()
                    .filter(|oge| kimlikler.contains(&oge.kimlik))
                    .cloned()
                    .collect::<Vec<_>>();
                if ogeler.is_empty() {
                    self.artik_hata = Some("Karantinaya taşınacak artık bulunamadı.".to_string());
                    cx.notify();
                    return;
                }
                let hedef = self
                    .dizinler
                    .yerel
                    .join("quarantine")
                    .join(format!("artik-{}", Self::zaman_damgasi()));
                let servis = self.artik.clone();
                self.arka_plan_islem(cx, IslemAlani::Artik, move || {
                    servis.karantinaya_tasi(&ogeler, &hedef).map(|sonuc| {
                        format!(
                            "{} artık karantinaya taşındı. {} hata.",
                            sonuc.tasinan,
                            sonuc.hatalar.len()
                        )
                    })
                });
            }
            SistemOnay::StoreKaldir(ad, tam_ad) => {
                let servis = self.artik.clone();
                self.arka_plan_islem(cx, IslemAlani::Store, move || {
                    servis
                        .store_kaldir(&tam_ad)
                        .map(|()| format!("{ad} Store paketi kaldırıldı."))
                });
            }
            SistemOnay::GelismisTemizlik(kimlik) => {
                self.onarim_kategori_temizle(kimlik, cx);
            }
            SistemOnay::ChromeEskiSil(yollar) => {
                self.chrome_eski_sil(yollar, cx);
            }
            SistemOnay::Sfc => self.sfc_calistir(cx),
            SistemOnay::DismResetBase => self.dism_reset_base(cx),
            SistemOnay::AppxOnar(ad, tam_ad) => self.appx_onar(ad, tam_ad, cx),
            SistemOnay::GuncellemeGecmisiTemizle => self.guncelleme_gecmisi_temizle(cx),
            SistemOnay::EskiGeriNoktalariSil => self.eski_geri_noktalari_sil(cx),
            SistemOnay::GeriNoktasiOlustur => self.geri_noktasi_olustur(cx),
            SistemOnay::GeriDon(sira, aciklama) => self.geri_yukle(sira, aciklama, cx),
            SistemOnay::GeriKoruma(etkin) => self.geri_koruma_ayarla(etkin, cx),
            SistemOnay::IkonOnbellekOnar => self.ikon_onbellek_onar(cx),
            SistemOnay::KayitTemizle(kimlikler) => {
                self.kayit_temizlik_temizle(kimlikler, cx);
            }
            SistemOnay::KisayolTemizle(yollar) => {
                self.kisayol_temizle(yollar, cx);
            }
        }
    }

    fn sistem_yonetimi_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        let baslangic = self.baslangic_ogeleri.clone();
        let servisler = self.servisler.clone();
        let gorevler = self.gorevler.clone();
        let surecler = self.surecler.clone();
        let taraniyor = self.sistem_taraniyor;
        let mesaj = self.sistem_mesaj.clone();
        let hata = self.sistem_hata.clone();

        let baslangic_icerik = if baslangic.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Başlangıçta çalışan kayıt bulunamadı veya henüz taranmadı.")
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(baslangic.iter().map(|oge| {
                    let kimlik = oge.kimlik.clone();
                    let aksiyon = if oge.etkin {
                        Button::new(format!("bas-kapat-{}", oge.kimlik))
                            .label("Devre Dışı Bırak")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.sistem_onay_iste(
                                    SistemOnay::BaslangicDevreDisi(kimlik.clone()),
                                    cx,
                                );
                            }))
                            .into_any_element()
                    } else {
                        Button::new(format!("bas-ac-{}", oge.kimlik))
                            .primary()
                            .label("Etkinleştir")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.sistem_onay_iste(
                                    SistemOnay::BaslangicEtkinlestir(kimlik.clone()),
                                    cx,
                                );
                            }))
                            .into_any_element()
                    };
                    self.modul_satiri(
                        oge.ad.clone(),
                        oge.konum.metin().to_string(),
                        oge.komut.clone(),
                        vec![
                            div()
                                .text_xs()
                                .text_color(if oge.etkin {
                                    cx.theme().success
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .child(if oge.etkin { "Etkin" } else { "Devre dışı" })
                                .into_any_element(),
                            aksiyon,
                        ],
                        cx,
                    )
                }))
                .into_any_element()
        };

        let servis_icerik = if servisler.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Servis bulunamadı veya henüz taranmadı.")
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(servisler.iter().take(300).map(|servis| {
                    let ad = servis.ad.clone();
                    let durdur = servis.durum.eq_ignore_ascii_case("Running")
                        || servis.durum.eq_ignore_ascii_case("Çalışıyor");
                    let mut aksiyonlar = vec![
                        Button::new(format!("svc-ac-{}", servis.ad))
                            .primary()
                            .label("Başlat")
                            .disabled(durdur)
                            .on_click(cx.listener({
                                let ad = ad.clone();
                                move |this, _, _, cx| {
                                    this.sistem_onay_iste(SistemOnay::ServisBaslat(ad.clone()), cx);
                                }
                            }))
                            .into_any_element(),
                        Button::new(format!("svc-kapat-{}", servis.ad))
                            .label("Durdur")
                            .disabled(!durdur)
                            .on_click(cx.listener({
                                let ad = ad.clone();
                                move |this, _, _, cx| {
                                    this.sistem_onay_iste(SistemOnay::ServisDurdur(ad.clone()), cx);
                                }
                            }))
                            .into_any_element(),
                    ];
                    for (etiket, tur) in
                        [("Oto", "auto"), ("Man", "demand"), ("Kapalı", "disabled")]
                    {
                        aksiyonlar.push(
                            Button::new(format!("svc-{tur}-{}", servis.ad))
                                .ghost()
                                .label(etiket)
                                .on_click(cx.listener({
                                    let ad = ad.clone();
                                    move |this, _, _, cx| {
                                        this.sistem_onay_iste(
                                            SistemOnay::ServisBaslangic(
                                                ad.clone(),
                                                tur.to_string(),
                                            ),
                                            cx,
                                        );
                                    }
                                }))
                                .into_any_element(),
                        );
                    }
                    self.modul_satiri(
                        servis.gorunen_ad.clone(),
                        format!("{} · {}", servis.ad, servis.durum),
                        format!("Başlangıç: {}", servis.baslangic),
                        aksiyonlar,
                        cx,
                    )
                }))
                .into_any_element()
        };

        let gorev_icerik = if gorevler.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Zamanlanmış görev bulunamadı veya henüz taranmadı.")
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(gorevler.iter().take(250).map(|gorev| {
                    let ad = gorev.ad.clone();
                    let yol = gorev.yol.clone();
                    let etkin = !gorev.durum.eq_ignore_ascii_case("Disabled");
                    self.modul_satiri(
                        gorev.ad.clone(),
                        gorev.yol.clone(),
                        format!("Durum: {} · Sonraki: {}", gorev.durum, gorev.sonraki),
                        vec![
                            Button::new(format!("gorev-calistir-{}", gorev.kimlik))
                                .primary()
                                .label("Çalıştır")
                                .on_click(cx.listener({
                                    let ad = ad.clone();
                                    let yol = yol.clone();
                                    move |this, _, _, cx| {
                                        this.sistem_onay_iste(
                                            SistemOnay::GorevCalistir(ad.clone(), yol.clone()),
                                            cx,
                                        );
                                    }
                                }))
                                .into_any_element(),
                            Button::new(format!("gorev-durum-{}", gorev.kimlik))
                                .label(if etkin {
                                    "Devre Dışı"
                                } else {
                                    "Etkinleştir"
                                })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.sistem_onay_iste(
                                        SistemOnay::GorevDurum(ad.clone(), yol.clone(), !etkin),
                                        cx,
                                    );
                                }))
                                .into_any_element(),
                        ],
                        cx,
                    )
                }))
                .into_any_element()
        };

        let surec_icerik = if surecler.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Süreç bulunamadı veya henüz taranmadı.")
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(surecler.iter().take(250).map(|surec| {
                    let pid = surec.pid;
                    let ad = surec.ad.clone();
                    let cpu = surec
                        .cpu_saniye
                        .map(|saniye| format!("{saniye:.1} sn CPU"))
                        .unwrap_or_else(|| "CPU bilinmiyor".to_string());
                    self.modul_satiri(
                        format!("{} (PID {})", surec.ad, surec.pid),
                        format!("{} MB · {cpu}", surec.bellek_mb),
                        surec
                            .kullanici
                            .clone()
                            .or_else(|| surec.baslama.clone())
                            .unwrap_or_default(),
                        vec![Button::new(format!("surec-kapat-{pid}"))
                            .danger()
                            .label("Sonlandır")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.sistem_onay_iste(
                                    SistemOnay::SurecSonlandir(pid, ad.clone()),
                                    cx,
                                );
                            }))
                            .into_any_element()],
                        cx,
                    )
                }))
                .into_any_element()
        };

        let sistem_onay_paneli = if let Some(onay) = &self.sistem_onay {
            self.sistem_onay_paneli(onay.clone(), cx)
        } else {
            div().into_any_element()
        };

        let sayfa = v_flex()
            .gap_4()
            .child(
                h_flex()
                    .justify_end()
                    .items_center()
                    .child(
                        Button::new("sistem-tara")
                            .primary()
                            .label(if taraniyor { "Taranıyor..." } else { "Tara" })
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| this.sistem_yonetimi_tara(cx))),
                    ),
            )
            .child(if let Some(hata) = &hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(sistem_onay_paneli)
            .child(self.bolum_karti(
                "Başlangıç".to_string(),
                "HKCU/HKLM Run, RunOnce ve başlangıç klasörleri. Devre dışı bırakılanlar yedeklenir.".to_string(),
                baslangic_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Servisler".to_string(),
                "Başlat/durdur ve başlangıç türü. İlk 300 servis gösterilir.".to_string(),
                servis_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Zamanlanmış Görevler".to_string(),
                "Çalıştır, etkinleştir veya devre dışı bırak. İlk 250 görev gösterilir.".to_string(),
                gorev_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Süreçler".to_string(),
                "Bellek ve CPU süresi; sonlandırma yönetici izni gerektirebilir. İlk 250 süreç gösterilir.".to_string(),
                surec_icerik,
                cx,
            ));
        sayfa.into_any_element()
    }

    fn onarim_kategorileri_tara(&mut self, cx: &mut Context<Self>) {
        if self.temizlik_taraniyor {
            return;
        }
        self.temizlik_taraniyor = true;
        self.temizlik_hata = None;
        self.temizlik_mesaj = Some("Gelişmiş temizlik kategorileri taranıyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.kategorileri_tara() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(taramalar) => {
                        let toplam = taramalar.iter().map(|tarama| tarama.bayt).sum();
                        this.temizlik_mesaj = Some(format!(
                            "{} kategori tarandı; {}.",
                            taramalar.len(),
                            Self::bayt_metni(toplam)
                        ));
                        this.onarim_taramalari = taramalar;
                        this.temizlik_hata = None;
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn onarim_kategori_temizle(&mut self, kimlik: String, cx: &mut Context<Self>) {
        if self.temizlik_taraniyor {
            return;
        }
        let ad = self
            .onarim_taramalari
            .iter()
            .find(|tarama| tarama.kategori.kimlik == kimlik)
            .map(|tarama| tarama.kategori.ad.clone())
            .unwrap_or_else(|| "Gelişmiş temizlik".to_string());
        self.temizlik_taraniyor = true;
        self.temizlik_hata = None;
        self.temizlik_mesaj = Some(format!("{ad} temizleniyor..."));
        cx.notify();
        let servis = self.onarim.clone();
        let hedef = kimlik.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.kategori_temizle(&hedef) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(sonuc) => {
                        this.onarim_taramalari
                            .retain(|tarama| tarama.kategori.kimlik != kimlik);
                        this.temizlik_mesaj = Some(format!(
                            "{ad}: {} dosya, {} temizlendi. {} hata.",
                            sonuc.silinen_dosya,
                            Self::bayt_metni(sonuc.silinen_bayt),
                            sonuc.hatalar.len()
                        ));
                        this.temizlik_hata = if sonuc.hatalar.is_empty() {
                            None
                        } else {
                            Some(sonuc.hatalar.join("\n"))
                        };
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn chrome_eski_tara(&mut self, cx: &mut Context<Self>) {
        if self.temizlik_taraniyor {
            return;
        }
        self.temizlik_taraniyor = true;
        self.temizlik_hata = None;
        self.temizlik_mesaj = Some("Eski Chrome sürümleri taranıyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.chrome_eski_surum_tara() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(yollar) => {
                        this.temizlik_mesaj = Some(format!(
                            "{} eski Chrome sürüm klasörü bulundu.",
                            yollar.len()
                        ));
                        this.chrome_eski = yollar;
                        this.temizlik_hata = None;
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn chrome_eski_sil(&mut self, yollar: Vec<String>, cx: &mut Context<Self>) {
        if self.temizlik_taraniyor {
            return;
        }
        self.temizlik_taraniyor = true;
        self.temizlik_hata = None;
        self.temizlik_mesaj = Some("Eski Chrome sürümleri siliniyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.chrome_eski_surumleri_sil(&yollar) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(sonuc) => {
                        this.chrome_eski.clear();
                        this.temizlik_mesaj = Some(format!(
                            "{} dosya, {} silindi. {} hata.",
                            sonuc.silinen_dosya,
                            Self::bayt_metni(sonuc.silinen_bayt),
                            sonuc.hatalar.len()
                        ));
                        this.temizlik_hata = if sonuc.hatalar.is_empty() {
                            None
                        } else {
                            Some(sonuc.hatalar.join("\n"))
                        };
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn sfc_calistir(&mut self, cx: &mut Context<Self>) {
        let servis = self.onarim.clone();
        self.bakim_islem(cx, move || servis.sfc_calistir(), true);
    }

    fn dism_reset_base(&mut self, cx: &mut Context<Self>) {
        let servis = self.onarim.clone();
        self.bakim_islem(cx, move || servis.dism_reset_base(), true);
    }

    fn appx_onar(&mut self, ad: String, tam_ad: String, cx: &mut Context<Self>) {
        let servis = self.onarim.clone();
        self.bakim_islem(
            cx,
            move || {
                servis.appx_onar(&tam_ad).map(|cikti| {
                    if cikti.is_empty() {
                        format!("{ad} yeniden kaydedildi.")
                    } else {
                        cikti
                    }
                })
            },
            false,
        );
    }

    fn guncelleme_gecmisi_yukle(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Windows güncelleme geçmişi okunuyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.guncelleme_gecmisi() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(kayitlar) => {
                        this.bakim_mesaj =
                            Some(format!("{} güncelleme kaydı bulundu.", kayitlar.len()));
                        this.guncelleme_gecmisi = kayitlar;
                        this.bakim_hata = None;
                    }
                    Err(hata) => this.bakim_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn guncelleme_gecmisi_temizle(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Windows Update veri deposu temizleniyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.guncelleme_gecmisi_temizle() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(()) => {
                        this.guncelleme_gecmisi.clear();
                        this.bakim_mesaj = Some(
                            "Windows Update veri deposu temizlendi; hizmetler yeniden başlatıldı."
                                .to_string(),
                        );
                        this.bakim_hata = None;
                    }
                    Err(hata) => this.bakim_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn geri_noktalari_yukle(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Geri yükleme noktaları okunuyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.geri_yukleme_noktalari() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(noktalar) => {
                        this.bakim_mesaj =
                            Some(format!("{} geri yükleme noktası bulundu.", noktalar.len()));
                        this.geri_yukleme_noktalari = noktalar;
                        this.bakim_hata = None;
                    }
                    Err(hata) => this.bakim_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn eski_geri_noktalari_sil(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Eski geri yükleme noktaları siliniyor...".to_string());
        cx.notify();
        let servis = self.onarim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.eski_geri_yukleme_noktalarini_sil() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(sayi) => {
                        this.geri_yukleme_noktalari.clear();
                        this.bakim_mesaj = Some(format!(
                            "{sayi} eski geri yükleme noktası silindi; en yeni nokta korundu."
                        ));
                        this.bakim_hata = None;
                    }
                    Err(hata) => this.bakim_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn geri_noktasi_olustur(&mut self, cx: &mut Context<Self>) {
        let servis = self.onarim.clone();
        self.bakim_islem(
            cx,
            move || servis.geri_yukleme_olustur("WinNexus manuel nokta"),
            false,
        );
    }

    fn geri_yukle(&mut self, sira: u32, _aciklama: String, cx: &mut Context<Self>) {
        let servis = self.onarim.clone();
        self.bakim_islem(cx, move || servis.geri_yukle(sira), false);
    }

    fn geri_koruma_ayarla(&mut self, etkin: bool, cx: &mut Context<Self>) {
        let surucu = std::env::var("SystemDrive")
            .map(|surucu| format!("{surucu}\\"))
            .unwrap_or_else(|_| "C:\\".to_string());
        let servis = self.onarim.clone();
        self.bakim_islem(
            cx,
            move || servis.geri_yukleme_etkinlestir(&surucu, etkin),
            false,
        );
    }

    fn ikon_onbellek_onar(&mut self, cx: &mut Context<Self>) {
        let servis = self.onarim.clone();
        self.bakim_islem(
            cx,
            move || {
                servis
                    .ikon_onbellek_onar()
                    .map(|()| "Simge ve küçük resim önbelleği yeniden oluşturuldu.".to_string())
            },
            false,
        );
    }

    fn sorun_giderici_baslat(&mut self, kimlik: String, ad: String, cx: &mut Context<Self>) {
        match self.onarim.sorun_giderici_baslat(&kimlik) {
            Ok(()) => {
                self.bakim_mesaj = Some(format!("{ad} sorun gidericisi açıldı."));
                self.bakim_hata = None;
            }
            Err(hata) => self.bakim_hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn kayit_secim_degistir(&mut self, tur: KayitSecim, id: &str, secili: bool) {
        let hedef = match tur {
            KayitSecim::Kayit => &mut self.kayit_temizlik_secili,
            KayitSecim::Kisayol => &mut self.kisayol_secili,
        };
        if secili {
            hedef.insert(id.to_string());
        } else {
            hedef.remove(id);
        }
    }

    fn kayit_secim_kutusu(
        &self,
        anahtar: String,
        id: String,
        secili: bool,
        tur: KayitSecim,
        cx: &mut Context<Self>,
    ) -> Checkbox {
        let uygulama = cx.entity();
        let kimlik = id.clone();
        Checkbox::new(format!("kayit-secim-{anahtar}"))
            .checked(secili)
            .accessibility_label(format!("{id} seç"))
            .on_change(move |deger, _, cx| {
                let id = kimlik.clone();
                uygulama.update(cx, |this, cx| {
                    this.kayit_secim_degistir(tur, &id, *deger);
                    cx.notify();
                });
            })
    }

    fn kayit_temizlik_tara(&mut self, cx: &mut Context<Self>) {
        if self.kayit_taraniyor {
            return;
        }
        self.kayit_taraniyor = true;
        self.kayit_hata = None;
        self.kayit_mesaj = Some("Kayıt defteri artıkları taranıyor...".to_string());
        cx.notify();
        let servis = self.kayit_kisayol.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.kayit_defteri_tara() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kayit_taraniyor = false;
                match sonuc {
                    Ok(bulgular) => {
                        this.kayit_temizlik_secili =
                            bulgular.iter().map(|bulgu| bulgu.kimlik.clone()).collect();
                        this.kayit_mesaj =
                            Some(format!("{} kayıt defteri bulgusu bulundu.", bulgular.len()));
                        this.kayit_temizlik_bulgulari = bulgular;
                        this.kayit_hata = None;
                    }
                    Err(hata) => this.kayit_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kisayol_tara(&mut self, cx: &mut Context<Self>) {
        if self.kayit_taraniyor {
            return;
        }
        self.kayit_taraniyor = true;
        self.kayit_hata = None;
        self.kayit_mesaj = Some("Bozuk kısayollar taranıyor...".to_string());
        cx.notify();
        let servis = self.kayit_kisayol.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.kisayollari_tara() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kayit_taraniyor = false;
                match sonuc {
                    Ok(bulgular) => {
                        this.kisayol_secili =
                            bulgular.iter().map(|bulgu| bulgu.yol.clone()).collect();
                        this.kayit_mesaj =
                            Some(format!("{} bozuk kısayol bulundu.", bulgular.len()));
                        this.kisayol_bulgulari = bulgular;
                        this.kayit_hata = None;
                    }
                    Err(hata) => this.kayit_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kayit_temizlik_temizle(&mut self, kimlikler: Vec<String>, cx: &mut Context<Self>) {
        if self.kayit_taraniyor || kimlikler.is_empty() {
            return;
        }
        let secili = self
            .kayit_temizlik_bulgulari
            .iter()
            .filter(|bulgu| kimlikler.contains(&bulgu.kimlik))
            .cloned()
            .collect::<Vec<_>>();
        if secili.is_empty() {
            self.kayit_hata = Some("Karantinaya taşınacak bulgu yok.".to_string());
            cx.notify();
            return;
        }
        self.kayit_taraniyor = true;
        self.kayit_hata = None;
        self.kayit_mesaj = Some("Registry bulguları karantinaya taşınıyor...".to_string());
        cx.notify();
        let servis = self.kayit_kisayol.clone();
        let karantina = self
            .dizinler
            .karantina_dizini()
            .join(format!("kayit-temizlik-{}", Self::zaman_damgasi()));
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.kayit_defteri_temizle(&secili, &karantina);
                    let kalan = servis.kayit_defteri_tara().unwrap_or_default();
                    (sonuc, kalan)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kayit_taraniyor = false;
                let (sonuc, kalan) = sonuc;
                match sonuc {
                    Ok(sonuc) => {
                        this.kayit_temizlik_secili.clear();
                        this.kayit_temizlik_bulgulari = kalan;
                        this.kayit_mesaj = Some(format!(
                            "{} registry kaydı karantinaya alındı. {} hata.",
                            sonuc.islenen,
                            sonuc.hatalar.len()
                        ));
                        this.kayit_hata = if sonuc.hatalar.is_empty() {
                            None
                        } else {
                            Some(sonuc.hatalar.join("\n"))
                        };
                    }
                    Err(hata) => this.kayit_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn kisayol_temizle(&mut self, yollar: Vec<String>, cx: &mut Context<Self>) {
        if self.kayit_taraniyor || yollar.is_empty() {
            return;
        }
        let secili = self
            .kisayol_bulgulari
            .iter()
            .filter(|bulgu| yollar.contains(&bulgu.yol))
            .cloned()
            .collect::<Vec<_>>();
        if secili.is_empty() {
            self.kayit_hata = Some("Karantinaya taşınacak kısayol yok.".to_string());
            cx.notify();
            return;
        }
        self.kayit_taraniyor = true;
        self.kayit_hata = None;
        self.kayit_mesaj = Some("Kısayollar karantinaya taşınıyor...".to_string());
        cx.notify();
        let servis = self.kayit_kisayol.clone();
        let karantina = self
            .dizinler
            .karantina_dizini()
            .join(format!("kisayol-temizlik-{}", Self::zaman_damgasi()));
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let sonuc = servis.kisayollari_temizle(&secili, &karantina);
                    let kalan = servis.kisayollari_tara().unwrap_or_default();
                    (sonuc, kalan)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.kayit_taraniyor = false;
                let (sonuc, kalan) = sonuc;
                match sonuc {
                    Ok(sonuc) => {
                        this.kisayol_secili.clear();
                        this.kisayol_bulgulari = kalan;
                        this.kayit_mesaj = Some(format!(
                            "{} kısayol karantinaya taşındı. {} hata.",
                            sonuc.islenen,
                            sonuc.hatalar.len()
                        ));
                        this.kayit_hata = if sonuc.hatalar.is_empty() {
                            None
                        } else {
                            Some(sonuc.hatalar.join("\n"))
                        };
                    }
                    Err(hata) => this.kayit_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn baglam_menu_tara(&mut self, cx: &mut Context<Self>) {
        if self.baglam_taraniyor {
            return;
        }
        self.baglam_taraniyor = true;
        self.baglam_hata = None;
        self.baglam_mesaj = Some("Bağlam menüsü kayıtları taranıyor...".to_string());
        cx.notify();
        let servis = self.kayit_kisayol.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    let ogeler = servis.baglam_menusu();
                    let klasik = servis.klasik_baglam_menusu();
                    (ogeler, klasik)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.baglam_taraniyor = false;
                let (ogeler, klasik) = sonuc;
                match ogeler {
                    Ok(ogeler) => {
                        this.baglam_mesaj =
                            Some(format!("{} bağlam menüsü kaydı bulundu.", ogeler.len()));
                        this.baglam_ogeleri = ogeler;
                        this.baglam_klasik = klasik.ok();
                        this.baglam_hata = None;
                    }
                    Err(hata) => this.baglam_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn baglam_menu_ayarla(&mut self, clsid: String, devre_disi: bool, cx: &mut Context<Self>) {
        match self.kayit_kisayol.baglam_menu_ayarla(&clsid, devre_disi) {
            Ok(()) => {
                for oge in &mut self.baglam_ogeleri {
                    if oge.clsid.eq_ignore_ascii_case(&clsid) {
                        oge.devre_disi = devre_disi;
                    }
                }
                self.baglam_mesaj = Some(
                    "Bağlam menüsü değişti; aynı CLSID tüm kapsamlarda etkilenir ve Explorer yeniden başlatılmalı."
                        .to_string(),
                );
                self.baglam_hata = None;
            }
            Err(hata) => self.baglam_hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn klasik_baglam_ayarla(&mut self, etkin: bool, cx: &mut Context<Self>) {
        match self.kayit_kisayol.klasik_baglam_menu_ayarla(etkin) {
            Ok(()) => {
                self.baglam_klasik = Some(etkin);
                self.baglam_mesaj = Some(if etkin {
                    "Klasik bağlam menüsü açıldı; Explorer yeniden başlatılmalı.".to_string()
                } else {
                    "Windows 11 bağlam menüsü geri yüklendi.".to_string()
                });
                self.baglam_hata = None;
            }
            Err(hata) => self.baglam_hata = Some(hata.to_string()),
        }
        cx.notify();
    }

    fn temizlik_tara(&mut self, cx: &mut Context<Self>) {
        if self.temizlik_taraniyor {
            return;
        }
        self.temizlik_taraniyor = true;
        self.temizlik_hata = None;
        self.temizlik_mesaj = Some("Geçici dosya hedefleri taranıyor...".to_string());
        cx.notify();
        let servis = self.temizleyici.clone();
        let hedefler = self.temizlik_hedefleri.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    hedefler
                        .iter()
                        .map(|hedef| (hedef.kimlik.clone(), servis.tara(hedef)))
                        .collect::<Vec<_>>()
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                let mut hatalar = Vec::new();
                for (kimlik, sonuc) in sonuc {
                    match sonuc {
                        Ok(tarama) => {
                            this.temizlik_taramalari.insert(kimlik, tarama);
                        }
                        Err(hata) => hatalar.push(hata.to_string()),
                    }
                }
                this.temizlik_mesaj = Some("Geçici dosya taraması tamamlandı.".to_string());
                this.temizlik_hata = (!hatalar.is_empty()).then(|| hatalar.join("\n"));
                cx.notify();
            });
        })
        .detach();
    }

    fn temizlik_hedef_tara(&mut self, kimlik: String, cx: &mut Context<Self>) {
        let Some(hedef) = self
            .temizlik_hedefleri
            .iter()
            .find(|hedef| hedef.kimlik == kimlik)
            .cloned()
        else {
            return;
        };
        if self.temizlik_taraniyor {
            return;
        }
        self.temizlik_taraniyor = true;
        self.temizlik_hata = None;
        self.temizlik_mesaj = Some(format!("{} taranıyor...", hedef.ad));
        cx.notify();
        let servis = self.temizleyici.clone();
        let tarama_hedef = hedef.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.tara(&tarama_hedef) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(tarama) => {
                        this.temizlik_mesaj = Some(format!(
                            "{}: {} dosya, {}.",
                            hedef.ad,
                            tarama.dosya_sayisi,
                            Self::bayt_metni(tarama.bayt)
                        ));
                        this.temizlik_taramalari
                            .insert(hedef.kimlik.clone(), tarama);
                        this.temizlik_hata = None;
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn bos_klasor_sec(&mut self, cx: &mut Context<Self>) {
        let Some(kok) = rfd::FileDialog::new()
            .set_title("Boş klasörlerin taranacağı klasörü seçin")
            .pick_folder()
        else {
            return;
        };
        self.bos_klasor_kok = Some(kok.clone());
        self.bos_klasorler.clear();
        self.temizlik_taraniyor = true;
        self.temizlik_mesaj = Some("Boş klasörler taranıyor...".to_string());
        self.temizlik_hata = None;
        cx.notify();
        let servis = self.temizleyici.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.bos_klasorleri_tara(&kok) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(klasorler) => {
                        this.temizlik_mesaj =
                            Some(format!("{} boş klasör bulundu.", klasorler.len()));
                        this.bos_klasorler = klasorler;
                        this.temizlik_hata = None;
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn yinelenen_sec(&mut self, cx: &mut Context<Self>) {
        let Some(kok) = rfd::FileDialog::new()
            .set_title("Yinelenen dosyaların taranacağı klasörü seçin")
            .pick_folder()
        else {
            return;
        };
        self.yinelenen_kok = Some(kok.clone());
        self.yinelenenler.clear();
        self.temizlik_taraniyor = true;
        self.temizlik_mesaj = Some("Yinelenen dosyalar taranıyor...".to_string());
        self.temizlik_hata = None;
        cx.notify();
        let servis = self.temizleyici.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.yinelenenleri_tara(&kok, 1024) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.temizlik_taraniyor = false;
                match sonuc {
                    Ok(gruplar) => {
                        this.temizlik_mesaj =
                            Some(format!("{} yinelenen grup bulundu.", gruplar.len()));
                        this.yinelenenler = gruplar;
                        this.temizlik_hata = None;
                    }
                    Err(hata) => this.temizlik_hata = Some(hata.to_string()),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn parcala_sec(&mut self, cx: &mut Context<Self>) {
        let Some(dosya) = rfd::FileDialog::new()
            .set_title("Parçalanacak dosyayı seçin")
            .pick_file()
        else {
            return;
        };
        self.sistem_onay_iste(SistemOnay::Parcala(dosya.display().to_string()), cx);
    }

    fn temizlik_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        let hedefler = self.temizlik_hedefleri.clone();
        let taramalar = self.temizlik_taramalari.clone();
        let mesaj = self.temizlik_mesaj.clone();
        let hata = self.temizlik_hata.clone();
        let taraniyor = self.temizlik_taraniyor;
        let bos_klasor_kok = self.bos_klasor_kok.clone();
        let bos_klasorler = self.bos_klasorler.clone();
        let yinelenen_kok = self.yinelenen_kok.clone();
        let yinelenenler = self.yinelenenler.clone();
        let onarim_taramalari = self.onarim_taramalari.clone();
        let chrome_eski = self.chrome_eski.clone();
        let merkez_kayit = self.kayit_temizlik_bulgulari.clone();
        let kayit_temizlik_secili = self.kayit_temizlik_secili.clone();
        let merkez_kisayol = self.kisayol_bulgulari.clone();
        let kisayol_secili = self.kisayol_secili.clone();
        let kayit_taraniyor = self.kayit_taraniyor;
        let kayit_mesaj = self.kayit_mesaj.clone();
        let kayit_hata = self.kayit_hata.clone();

        let kayit_temizleyici_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Yalnızca doğrulanabilir kaldırma kayıtları, uygulama yolları ve paylaşılan DLL kayıtları taranır. Silmeden önce .reg yedeği alınır."),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("kayit-temizlik-tara")
                                    .primary()
                                    .label("Kayıt Defterini Tara")
                                    .disabled(kayit_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.kayit_temizlik_tara(cx);
                                    })),
                            )
                            .child(
                                Button::new("kayit-temizlik-tumunu-sec")
                                    .label("Tümünü Seç")
                                    .disabled(merkez_kayit.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.kayit_temizlik_secili = this
                                            .kayit_temizlik_bulgulari
                                            .iter()
                                            .map(|bulgu| bulgu.kimlik.clone())
                                            .collect();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("kayit-temizlik-temizle")
                                    .danger()
                                    .label(format!(
                                        "Karantinaya Taşı ({})",
                                        kayit_temizlik_secili.len()
                                    ))
                                    .disabled(kayit_temizlik_secili.is_empty() || kayit_taraniyor)
                                    .on_click(cx.listener({
                                        let secili: Vec<String> =
                                            kayit_temizlik_secili.iter().cloned().collect();
                                        move |this, _, _, cx| {
                                            this.sistem_onay_iste(
                                                SistemOnay::KayitTemizle(secili.clone()),
                                                cx,
                                            );
                                        }
                                    })),
                            ),
                    ),
            )
            .child(if let Some(hata) = &kayit_hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &kayit_mesaj {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(mesaj.clone())
                    .into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if merkez_kayit.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Kayıt bulunamadı veya henüz tarama yapılmadı.")
                    .into_any_element()
            } else {
                div().into_any_element()
            })
            .children(merkez_kayit.iter().take(200).map(|bulgu| {
                let secili = kayit_temizlik_secili.contains(&bulgu.kimlik);
                h_flex()
                    .gap_2()
                    .items_start()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(self.kayit_secim_kutusu(
                        format!("kayit-{}", bulgu.kimlik),
                        bulgu.kimlik.clone(),
                        secili,
                        KayitSecim::Kayit,
                        cx,
                    ))
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_1()
                            .min_w_0()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(bulgu.ad.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(bulgu.kategori.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(bulgu.ayrinti.clone()),
                            ),
                    )
            }))
            .into_any_element();

        let kisayol_temizleyici_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Masaüstü, başlat menüsü ve başlangıç klasörlerindeki hedefi bulunmayan .lnk dosyaları."),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("kisayol-tara")
                                    .primary()
                                    .label("Kısayolları Tara")
                                    .disabled(kayit_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.kisayol_tara(cx);
                                    })),
                            )
                            .child(
                                Button::new("kisayol-tumunu-sec")
                                    .label("Tümünü Seç")
                                    .disabled(merkez_kisayol.is_empty())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.kisayol_secili = this
                                            .kisayol_bulgulari
                                            .iter()
                                            .map(|bulgu| bulgu.yol.clone())
                                            .collect();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("kisayol-temizle")
                                    .danger()
                                    .label(format!(
                                        "Karantinaya Taşı ({})",
                                        kisayol_secili.len()
                                    ))
                                    .disabled(kisayol_secili.is_empty() || kayit_taraniyor)
                                    .on_click(cx.listener({
                                        let secili: Vec<String> =
                                            kisayol_secili.iter().cloned().collect();
                                        move |this, _, _, cx| {
                                            this.sistem_onay_iste(
                                                SistemOnay::KisayolTemizle(secili.clone()),
                                                cx,
                                            );
                                        }
                                    })),
                            ),
                    ),
            )
            .child(if merkez_kisayol.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Bozuk kısayol bulunamadı veya henüz tarama yapılmadı.")
                    .into_any_element()
            } else {
                div().into_any_element()
            })
            .children(merkez_kisayol.iter().take(200).map(|bulgu| {
                let secili = kisayol_secili.contains(&bulgu.yol);
                h_flex()
                    .gap_2()
                    .items_start()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(self.kayit_secim_kutusu(
                        format!("kisayol-{}", bulgu.yol),
                        bulgu.yol.clone(),
                        secili,
                        KayitSecim::Kisayol,
                        cx,
                    ))
                    .child(
                        v_flex()
                            .flex_1()
                            .gap_1()
                            .min_w_0()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(bulgu.ad.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(bulgu.konum.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("Hedef: {}", bulgu.hedef)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(bulgu.yol.clone()),
                            ),
                    )
            }))
            .into_any_element();

        let gelismis_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Simge önbelleği, hata raporları, güncelleme indirmeleri, Defender geçmişi ve geliştirici önbellekleri."),
                    )
                    .child(
                        Button::new("onarim-kategori-tara")
                            .primary()
                            .label(if taraniyor {
                                "Taranıyor..."
                            } else {
                                "Kategorileri Tara"
                            })
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.onarim_kategorileri_tara(cx);
                            })),
                    ),
            )
            .child(if onarim_taramalari.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz gelişmiş kategori taraması yapılmadı.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .children(onarim_taramalari.iter().map(|tarama| {
                        let kimlik = tarama.kategori.kimlik.clone();
                        h_flex()
                            .justify_between()
                            .items_center()
                            .p_3()
                            .gap_3()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_1()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(tarama.kategori.ad.clone()),
                                            )
                                            .child(if tarama.kategori.yonetici {
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("Yönetici")
                                                    .into_any_element()
                                            } else {
                                                div().into_any_element()
                                            })
                                            .child(if tarama.kategori.riskli {
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().warning)
                                                    .child("Riskli")
                                                    .into_any_element()
                                            } else {
                                                div().into_any_element()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(tarama.kategori.aciklama.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!(
                                                "{} dosya · {}",
                                                tarama.dosya_sayisi,
                                                Self::bayt_metni(tarama.bayt)
                                            )),
                                    ),
                            )
                            .child(
                                Button::new(format!("onarim-temizle-{}", tarama.kategori.kimlik))
                                    .danger()
                                    .label("Temizle")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.sistem_onay_iste(
                                            SistemOnay::GelismisTemizlik(kimlik.clone()),
                                            cx,
                                        );
                                    })),
                            )
                    }))
                    .into_any_element()
            })
            .into_any_element();

        let chrome_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("chrome-eski-tara")
                            .label("Eski Sürümleri Tara")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| this.chrome_eski_tara(cx))),
                    )
                    .child(
                        Button::new("chrome-eski-sil")
                            .danger()
                            .label(format!("{} Klasörü Sil", chrome_eski.len()))
                            .disabled(chrome_eski.is_empty() || taraniyor)
                            .on_click(cx.listener({
                                let chrome_eski = chrome_eski.clone();
                                move |this, _, _, cx| {
                                    this.sistem_onay_iste(
                                        SistemOnay::ChromeEskiSil(chrome_eski.clone()),
                                        cx,
                                    );
                                }
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Geçerli Chrome sürümü kayıt defterinden okunur; yalnızca sürüm biçimindeki eski klasörler listelenir. Geçerli sürüm okunamazsa hiçbir şey silinmez."),
            )
            .children(chrome_eski.iter().map(|yol| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(yol.clone())
            }))
            .into_any_element();

        let hedef_icerik = v_flex()
            .gap_2()
            .children(hedefler.iter().map(|hedef| {
                let kimlik = hedef.kimlik.clone();
                let tarama = taramalar.get(&hedef.kimlik);
                let detay = tarama
                    .map(|tarama| {
                        format!(
                            "{} dosya · {}",
                            tarama.dosya_sayisi,
                            Self::bayt_metni(tarama.bayt)
                        )
                    })
                    .unwrap_or_else(|| "Henüz taranmadı".to_string());
                self.modul_satiri(
                    hedef.ad.clone(),
                    hedef.yol.clone(),
                    detay,
                    vec![
                        Button::new(format!("temizlik-tara-{}", hedef.kimlik))
                            .label("Tara")
                            .on_click(cx.listener({
                                let kimlik = kimlik.clone();
                                move |this, _, _, cx| {
                                    this.temizlik_hedef_tara(kimlik.clone(), cx);
                                }
                            }))
                            .into_any_element(),
                        Button::new(format!("temizlik-yap-{}", hedef.kimlik))
                            .danger()
                            .label("Temizle")
                            .disabled(taraniyor)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::Temizlik(kimlik.clone()), cx);
                            }))
                            .into_any_element(),
                    ],
                    cx,
                )
            }))
            .into_any_element();

        let bos_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("bos-sec")
                            .label("Klasör Seç")
                            .on_click(cx.listener(|this, _, _, cx| this.bos_klasor_sec(cx))),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(
                                bos_klasor_kok
                                    .as_ref()
                                    .map(|kok| kok.display().to_string())
                                    .unwrap_or_else(|| "Klasör seçilmedi".to_string()),
                            ),
                    )
                    .child(
                        Button::new("bos-sil")
                            .danger()
                            .label("Boş Klasörleri Sil")
                            .disabled(bos_klasorler.is_empty() || taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(
                                    SistemOnay::BosKlasorSil(this.bos_klasorler.clone()),
                                    cx,
                                );
                            })),
                    ),
            )
            .children(bos_klasorler.iter().take(200).map(|yol| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(yol.clone())
            }))
            .into_any_element();

        let mut yinelenen_icerik = v_flex().gap_2().child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    Button::new("yinelenen-sec")
                        .label("Klasör Seç")
                        .on_click(cx.listener(|this, _, _, cx| this.yinelenen_sec(cx))),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(
                            yinelenen_kok
                                .as_ref()
                                .map(|kok| kok.display().to_string())
                                .unwrap_or_else(|| "Klasör seçilmedi".to_string()),
                        ),
                ),
        );
        if !yinelenenler.is_empty() {
            let kopyalar = yinelenenler
                .iter()
                .flat_map(|grup| grup.dosyalar.iter().skip(1).cloned())
                .collect::<Vec<_>>();
            yinelenen_icerik = yinelenen_icerik.child(
                h_flex().gap_2().items_center().child(
                    Button::new("yinelenen-karantina")
                        .danger()
                        .label(format!("{} Kopyayı Karantinaya Taşı", kopyalar.len()))
                        .disabled(kopyalar.is_empty() || taraniyor)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.sistem_onay_iste(
                                SistemOnay::YinelenenKarantina(kopyalar.clone()),
                                cx,
                            );
                        })),
                ),
            );
            yinelenen_icerik =
                yinelenen_icerik.children(yinelenenler.iter().take(100).map(|grup| {
                    v_flex()
                        .gap_1()
                        .p_2()
                        .rounded_md()
                        .border_1()
                        .border_color(cx.theme().border)
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!(
                                    "{} · {} dosya",
                                    Self::bayt_metni(grup.boyut),
                                    grup.dosyalar.len()
                                )),
                        )
                        .children(grup.dosyalar.iter().map(|dosya| {
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(dosya.clone())
                        }))
                }));
        }

        let sistem_onay_paneli = if let Some(onay) = &self.sistem_onay {
            self.sistem_onay_paneli(onay.clone(), cx)
        } else {
            div().into_any_element()
        };

        let sayfa = v_flex()
            .gap_4()
            .child(
                h_flex()
                    .justify_end()
                    .items_center()
                    .child(
                        Button::new("temizlik-tara")
                            .primary()
                            .label(if taraniyor { "Taranıyor..." } else { "Tümünü Tara" })
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| this.temizlik_tara(cx))),
                    ),
            )
            .child(if let Some(hata) = &hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(sistem_onay_paneli)
            .child(self.bolum_karti(
                "Gelişmiş Temizlik".to_string(),
                "Simge/thumbnail önbelleği, son kullanılanlar, WER, Windows günlükleri, Windows Update indirmeleri, Defender geçmişi, geliştirici önbellekleri ve Chrome kurulum önbelleği. Riskli kategoriler onay ister.".to_string(),
                gelismis_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Eski Chrome Sürümleri".to_string(),
                "Chrome Application klasöründeki geçerli sürüm dışındaki sürüm klasörlerini temizler.".to_string(),
                chrome_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Kayıt Defteri Temizleyici".to_string(),
                "Eksik kaldırma komutları, uygulama yolları ve paylaşılan DLL kayıtları. Her kayıt .reg olarak yedeklenir; seçim sizde.".to_string(),
                kayit_temizleyici_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Bozuk Kısayollar".to_string(),
                "Hedefi bulunmayan kısayollar silinmez, karantinaya taşınır ve manifest ile izlenir.".to_string(),
                kisayol_temizleyici_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Geçici Dosyalar".to_string(),
                "Yalnızca izinli geçici ve önbellek yolları; kullanımdaki dosyalar atlanır.".to_string(),
                hedef_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Boş Klasörler".to_string(),
                "Seçilen kök altında, sembolik bağlantıları takip etmeden en fazla 12 seviye.".to_string(),
                bos_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Yinelenen Dosyalar".to_string(),
                "Önce boyut, sonra hash ve bayt karşılaştırması; ilk dosya korunur, kopyalar karantinaya taşınır.".to_string(),
                yinelenen_icerik.into_any_element(),
                cx,
            ))
            .child(self.bolum_karti(
                "Dosya Parçalayıcı".to_string(),
                "3 geçişli üzerine yazma ve silme. SSD'lerde fiziksel garantisi yoktur; kritik dosyalarda kullanmayın.".to_string(),
                v_flex()
                    .gap_2()
                    .child(
                        Button::new("parcala-sec")
                            .danger()
                            .label("Dosya Seç ve Parçala")
                            .on_click(cx.listener(|this, _, _, cx| this.parcala_sec(cx))),
                    )
                    .into_any_element(),
                cx,
            ))
            .child(self.bolum_karti(
                "Geri Dönüşüm Kutusu".to_string(),
                "Geri dönüşüm kutusundaki tüm öğeler kalıcı olarak silinir.".to_string(),
                Button::new("geri-donusum")
                    .danger()
                    .label("Geri Dönüşüm Kutusunu Temizle")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sistem_onay_iste(SistemOnay::GeriDonusum, cx);
                    }))
                    .into_any_element(),
                cx,
            ));
        sayfa.into_any_element()
    }

    fn bakim_appx_listele(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Appx paketleri listeleniyor...".to_string());
        cx.notify();
        let servis = self.bakim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.appx_listele() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(paketler) => {
                        this.bakim_mesaj = Some(format!("{} Appx paketi bulundu.", paketler.len()));
                        this.appx_paketleri = paketler;
                        this.bakim_hata = None;
                    }
                    Err(hata) => {
                        this.bakim_hata = Some(hata.to_string());
                        this.bakim_mesaj = Some("Listeleme başarısız.".to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn bakim_ozellik_listele(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Windows özellikleri listeleniyor...".to_string());
        cx.notify();
        let servis = self.bakim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.ozellikler() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(ozellikler) => {
                        this.bakim_mesaj =
                            Some(format!("{} Windows özelliği bulundu.", ozellikler.len()));
                        this.ozellikler = ozellikler;
                        this.bakim_hata = None;
                    }
                    Err(hata) => {
                        this.bakim_hata = Some(hata.to_string());
                        this.bakim_mesaj = Some("Listeleme başarısız.".to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn bakim_surucu_listele(&mut self, cx: &mut Context<Self>) {
        if self.bakim_taraniyor {
            return;
        }
        self.bakim_taraniyor = true;
        self.bakim_hata = None;
        self.bakim_mesaj = Some("Sürücüler listeleniyor...".to_string());
        cx.notify();
        let servis = self.bakim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move { servis.suruculer() })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.bakim_taraniyor = false;
                match sonuc {
                    Ok(suruculer) => {
                        this.bakim_mesaj = Some(format!("{} sürücü bulundu.", suruculer.len()));
                        this.suruculer = suruculer;
                        this.bakim_hata = None;
                    }
                    Err(hata) => {
                        this.bakim_hata = Some(hata.to_string());
                        this.bakim_mesaj = Some("Listeleme başarısız.".to_string());
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn surucu_disa_aktar_sec(&mut self, inf: String, cx: &mut Context<Self>) {
        let Some(hedef) = rfd::FileDialog::new()
            .set_title("Sürücünün dışa aktarılacağı klasörü seçin")
            .pick_folder()
        else {
            return;
        };
        self.sistem_onay_iste(
            SistemOnay::SurucuDisaAktar(inf, hedef.display().to_string()),
            cx,
        );
    }

    fn bakim_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        let appx = self.appx_paketleri.clone();
        let ozellikler = self.ozellikler.clone();
        let suruculer = self.suruculer.clone();
        let dism = self.dism_ciktisi.clone();
        let mesaj = self.bakim_mesaj.clone();
        let hata = self.bakim_hata.clone();
        let taraniyor = self.bakim_taraniyor;
        let guncelleme_gecmisi = self.guncelleme_gecmisi.clone();
        let geri_noktalari = self.geri_yukleme_noktalari.clone();

        let onarim_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        Button::new("sfc-calistir")
                            .primary()
                            .label("SFC /scannow")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::Sfc, cx);
                            })),
                    )
                    .child(
                        Button::new("dism-reset-base")
                            .danger()
                            .label("DISM ResetBase")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::DismResetBase, cx);
                            })),
                    )
                    .child(
                        Button::new("ikon-onbellek-onar")
                            .label("Simge Önbelleğini Onar")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::IkonOnbellekOnar, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Windows Sorun Gidericileri"),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children(SORUN_GIDERICI_KIMLIKLERI.iter().map(|(kimlik, ad)| {
                        Button::new(format!("sorun-giderici-{kimlik}"))
                            .label(*ad)
                            .disabled(taraniyor)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.sorun_giderici_baslat(
                                    kimlik.to_string(),
                                    ad.to_string(),
                                    cx,
                                );
                            }))
                    })),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("SFC ve DISM uzun sürebilir, yönetici izni gerektirir. Sorun gidericiler MSDT varsa doğrudan, yoksa Windows Ayarları üzerinden açılır."),
            )
            .into_any_element();

        let guncelleme_icerik = v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Windows Güncelleme Geçmişi"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("guncelleme-gecmisi-yukle")
                                    .label("Geçmişi Yükle")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.guncelleme_gecmisi_yukle(cx);
                                    })),
                            )
                            .child(
                                Button::new("guncelleme-gecmisi-temizle")
                                    .danger()
                                    .label("Geçmişi Temizle")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sistem_onay_iste(
                                            SistemOnay::GuncellemeGecmisiTemizle,
                                            cx,
                                        );
                                    })),
                            ),
                    ),
            )
            .children(guncelleme_gecmisi.iter().take(100).map(|kayit| {
                h_flex()
                    .justify_between()
                    .gap_2()
                    .text_sm()
                    .child(div().child(format!("{} · {}", kayit.kimlik, kayit.aciklama)))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(kayit.tarih.clone()),
                    )
            }))
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Geri Yükleme Noktaları"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("geri-noktasi-olustur")
                                    .primary()
                                    .label("Nokta Oluştur")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sistem_onay_iste(
                                            SistemOnay::GeriNoktasiOlustur,
                                            cx,
                                        );
                                    })),
                            )
                            .child(
                                Button::new("geri-noktalari-yukle")
                                    .label("Noktaları Listele")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.geri_noktalari_yukle(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .child(
                        Button::new("geri-koruma-ac")
                            .label("Sistem Korumasını Aç")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::GeriKoruma(true), cx);
                            })),
                    )
                    .child(
                        Button::new("geri-koruma-kapat")
                            .label("Sistem Korumasını Kapat")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::GeriKoruma(false), cx);
                            })),
                    )
                    .child(
                        Button::new("eski-geri-noktalari-sil")
                            .danger()
                            .label("Eskileri Sil")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::EskiGeriNoktalariSil, cx);
                            })),
                    ),
            )
            .children(geri_noktalari.iter().take(100).map(|nokta| {
                let sira = nokta.sira;
                let aciklama = nokta.aciklama.clone();
                self.modul_satiri(
                    format!("#{sira} · {}", nokta.aciklama),
                    nokta.tarih.clone(),
                    String::new(),
                    vec![Button::new(format!("geri-don-{sira}"))
                        .danger()
                        .label("Geri Dön")
                        .disabled(taraniyor)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.sistem_onay_iste(
                                SistemOnay::GeriDon(sira, aciklama.clone()),
                                cx,
                            );
                        }))
                        .into_any_element()],
                    cx,
                )
                .into_any_element()
            }))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Güncelleme geçmişini temizlemek SoftwareDistribution veri deposunu sıfırlar. Geri yükleme noktaları için Sistem Koruması açık ve yönetici izni gerekir."),
            )
            .into_any_element();

        let dism_icerik = v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("dism-analiz")
                            .label("Analiz Et")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::DismAnaliz, cx);
                            })),
                    )
                    .child(
                        Button::new("dism-temizle")
                            .danger()
                            .label("Bileşen Temizliği")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::DismTemizle, cx);
                            })),
                    )
                    .child(
                        Button::new("dism-saglik")
                            .label("Sağlık Tara")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::DismSaglik, cx);
                            })),
                    )
                    .child(
                        Button::new("dism-onar")
                            .danger()
                            .label("Onar")
                            .disabled(taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sistem_onay_iste(SistemOnay::DismOnar, cx);
                            })),
                    ),
            )
            .child(if let Some(cikti) = &dism {
                div()
                    .max_h(px(360.))
                    .overflow_y_scrollbar()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .child(cikti.clone())
                    .into_any_element()
            } else {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz DISM işlemi çalıştırılmadı.")
                    .into_any_element()
            })
            .into_any_element();

        let appx_icerik = v_flex()
            .gap_2()
            .child(
                Button::new("appx-listele")
                    .label("Paketleri Listele")
                    .disabled(taraniyor)
                    .on_click(cx.listener(|this, _, _, cx| this.bakim_appx_listele(cx))),
            )
            .children(appx.iter().take(300).map(|paket| {
                let tam_ad = paket.tam_ad.clone();
                let ad = paket.ad.clone();
                let ad_onar = paket.ad.clone();
                let tam_ad_onar = paket.tam_ad.clone();
                let sistem = Self::sistem_paketi_mi(&paket.ad, &paket.yayinci);
                let mut aksiyonlar = vec![Button::new(format!("appx-onar-{}", paket.tam_ad))
                    .label("Onar")
                    .disabled(taraniyor)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.sistem_onay_iste(
                            SistemOnay::AppxOnar(ad_onar.clone(), tam_ad_onar.clone()),
                            cx,
                        );
                    }))
                    .into_any_element()];
                if sistem {
                    aksiyonlar.push(
                        Button::new(format!("appx-sistem-{}", paket.tam_ad))
                            .label("Sistem")
                            .disabled(true)
                            .into_any_element(),
                    );
                } else {
                    aksiyonlar.push(
                        Button::new(format!("appx-kaldir-{}", paket.tam_ad))
                            .danger()
                            .label("Kaldır")
                            .disabled(taraniyor)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.sistem_onay_iste(
                                    SistemOnay::AppxKaldir(ad.clone(), tam_ad.clone()),
                                    cx,
                                );
                            }))
                            .into_any_element(),
                    );
                }
                self.modul_satiri(
                    paket.ad.clone(),
                    format!("{} · {}", paket.surum, paket.yayinci),
                    paket.tam_ad.clone(),
                    aksiyonlar,
                    cx,
                )
            }))
            .into_any_element();

        let ozellik_icerik = v_flex()
            .gap_2()
            .child(
                Button::new("ozellik-listele")
                    .label("Özellikleri Listele")
                    .disabled(taraniyor)
                    .on_click(cx.listener(|this, _, _, cx| this.bakim_ozellik_listele(cx))),
            )
            .children(ozellikler.iter().take(300).map(|ozellik| {
                let ad = ozellik.ad.clone();
                let etkin = ozellik.durum.eq_ignore_ascii_case("Enabled");
                self.modul_satiri(
                    ozellik.ad.clone(),
                    ozellik.durum.clone(),
                    String::new(),
                    vec![Button::new(format!("ozellik-{}", ozellik.ad))
                        .label(if etkin { "Kapat" } else { "Aç" })
                        .disabled(taraniyor)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.sistem_onay_iste(
                                SistemOnay::OzellikAyarla(ad.clone(), !etkin),
                                cx,
                            );
                        }))
                        .into_any_element()],
                    cx,
                )
            }))
            .into_any_element();

        let surucu_icerik = v_flex()
            .gap_2()
            .child(
                Button::new("surucu-listele")
                    .label("Sürücüleri Listele")
                    .disabled(taraniyor)
                    .on_click(cx.listener(|this, _, _, cx| this.bakim_surucu_listele(cx))),
            )
            .children(suruculer.iter().take(250).map(|surucu| {
                let inf = surucu.inf.clone();
                self.modul_satiri(
                    surucu.ad.clone(),
                    format!("{} · {} · {}", surucu.yayinci, surucu.sinif, surucu.surum),
                    format!("{} · {}", surucu.tarih, surucu.inf),
                    vec![Button::new(format!("surucu-{}", surucu.inf))
                        .label("Dışa Aktar")
                        .disabled(taraniyor)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.surucu_disa_aktar_sec(inf.clone(), cx);
                        }))
                        .into_any_element()],
                    cx,
                )
            }))
            .into_any_element();

        let sistem_onay_paneli = if let Some(onay) = &self.sistem_onay {
            self.sistem_onay_paneli(onay.clone(), cx)
        } else {
            div().into_any_element()
        };

        let sayfa = v_flex()
            .gap_4()
            .child(if let Some(hata) = &hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(sistem_onay_paneli)
            .child(self.bolum_karti(
                "Bileşen Deposu (DISM)".to_string(),
                "Analiz ve sağlık taraması okuma; temizlik/onarım sistem değişikliği yapar ve yönetici ister.".to_string(),
                dism_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Sistem Onarımı".to_string(),
                "SFC, DISM ResetBase, simge önbelleği onarımı ve Windows sorun gidericileri. Uzun süren işlemler arka planda çalışır.".to_string(),
                onarim_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Güncelleme ve Geri Yükleme".to_string(),
                "Windows güncelleme geçmişini görüntüleyin/temizleyin; eski geri yükleme noktalarını en yenisini koruyarak silin.".to_string(),
                guncelleme_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Appx Paketleri".to_string(),
                "Geçerli kullanıcı paketleri; Microsoft/Windows sistem paketleri korunur. Üçüncü taraf kaldırma geri alınamaz.".to_string(),
                appx_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Windows Özellikleri".to_string(),
                "Aç/kapat işlemleri yeniden başlatma gerektirebilir; yönetici izni ister.".to_string(),
                ozellik_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Sürücüler".to_string(),
                "Get-WindowsDriver listesi; seçilen INF dosyası pnputil ile dışa aktarılır.".to_string(),
                surucu_icerik,
                cx,
            ));
        sayfa.into_any_element()
    }

    fn donanim_tara(&mut self, cx: &mut Context<Self>) {
        if self.donanim_taraniyor {
            return;
        }
        self.donanim_taraniyor = true;
        self.donanim_hata = None;
        self.donanim_mesaj = Some("Donanım ve sistem bilgileri okunuyor...".to_string());
        cx.notify();
        let servis = self.donanim.clone();
        cx.spawn(async move |this, cx| {
            let sonuc = cx
                .background_executor()
                .spawn(async move {
                    std::thread::scope(|kapsam| {
                        let ozet = kapsam.spawn(|| servis.sistem_ozeti());
                        let cpu = kapsam.spawn(|| servis.cpu());
                        let gpu = kapsam.spawn(|| servis.gpu());
                        let ram = kapsam.spawn(|| servis.ram());
                        let diskler = kapsam.spawn(|| servis.diskler());
                        let wlan = kapsam.spawn(|| servis.wlan_profilleri());
                        let uac = kapsam.spawn(|| servis.uac_durumu());
                        let rdp = kapsam.spawn(|| servis.rdp_durumu());
                        (
                            is_parcasi(ozet.join(), "Sistem"),
                            is_parcasi(cpu.join(), "CPU"),
                            is_parcasi(gpu.join(), "GPU"),
                            is_parcasi(ram.join(), "RAM"),
                            is_parcasi(diskler.join(), "Disk"),
                            is_parcasi(wlan.join(), "WLAN"),
                            is_parcasi(uac.join(), "UAC"),
                            is_parcasi(rdp.join(), "RDP"),
                        )
                    })
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.donanim_taraniyor = false;
                let mut hatalar = Vec::new();
                let (ozet, cpu, gpu, ram, diskler, wlan, uac, rdp) = sonuc;
                this.sistem_ozeti = ozet
                    .map_err(|hata| hatalar.push(format!("Sistem: {hata}")))
                    .ok();
                this.cpu_bilgisi = cpu.unwrap_or_else(|hata| {
                    hatalar.push(format!("CPU: {hata}"));
                    Vec::new()
                });
                this.gpu_bilgisi = gpu.unwrap_or_else(|hata| {
                    hatalar.push(format!("GPU: {hata}"));
                    Vec::new()
                });
                this.ram_bilgisi = ram
                    .map_err(|hata| hatalar.push(format!("RAM: {hata}")))
                    .ok();
                this.diskler = diskler.unwrap_or_else(|hata| {
                    hatalar.push(format!("Disk: {hata}"));
                    Vec::new()
                });
                this.wlan_profilleri = wlan.unwrap_or_else(|hata| {
                    hatalar.push(format!("WLAN: {hata}"));
                    Vec::new()
                });
                this.uac_durumu = uac
                    .map_err(|hata| hatalar.push(format!("UAC: {hata}")))
                    .ok();
                this.rdp_durumu = rdp
                    .map_err(|hata| hatalar.push(format!("RDP: {hata}")))
                    .ok();
                this.donanim_mesaj = Some("Donanım bilgileri yenilendi.".to_string());
                this.donanim_hata = (!hatalar.is_empty()).then(|| hatalar.join("\n"));
                cx.notify();
            });
        })
        .detach();
    }

    fn donanim_ozeti_satiri(etiket: &str, deger: String, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .gap_3()
            .items_start()
            .child(
                div()
                    .w(px(DONANIM_ETIKET_GENISLIGI_PX))
                    .flex_shrink_0()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(etiket.to_string()),
            )
            .child(div().text_sm().child(deger))
            .into_any_element()
    }

    fn donanim_ozeti_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let ozet = self.sistem_ozeti.as_ref();
        let cpu = self.cpu_bilgisi.first();
        let gpu = self.gpu_bilgisi.first();
        let ram = self.ram_bilgisi.as_ref();
        let disk_sayisi = self.diskler.len();
        let disk_gb: f64 = self.diskler.iter().map(|disk| disk.boyut_gb).sum();
        let veri_var =
            ozet.is_some() || cpu.is_some() || gpu.is_some() || ram.is_some() || disk_sayisi > 0;
        let mut icerik = v_flex().gap_2();
        if self.donanim_taraniyor {
            icerik = icerik.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Donanım bilgileri okunuyor..."),
            );
        } else if !veri_var {
            icerik = icerik.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Donanım bilgileri henüz okunmadı."),
            );
        } else {
            if let Some(ozet) = ozet {
                icerik = icerik.child(Self::donanim_ozeti_satiri(
                    "Sistem",
                    format!("{} · {}", ozet.bilgisayar, ozet.isletim_sistemi),
                    cx,
                ));
            }
            if let Some(cpu) = cpu {
                let ek = if self.cpu_bilgisi.len() > 1 {
                    format!(" +{} işlemci", self.cpu_bilgisi.len() - 1)
                } else {
                    String::new()
                };
                icerik = icerik.child(Self::donanim_ozeti_satiri(
                    "İşlemci",
                    format!("{} · {} çekirdek{}", cpu.ad, cpu.cekirdek, ek),
                    cx,
                ));
            }
            if let Some(gpu) = gpu {
                let ek = if self.gpu_bilgisi.len() > 1 {
                    format!(" +{} ekran kartı", self.gpu_bilgisi.len() - 1)
                } else {
                    String::new()
                };
                icerik = icerik.child(Self::donanim_ozeti_satiri(
                    "Ekran Kartı",
                    format!("{}{}", gpu.ad, ek),
                    cx,
                ));
            }
            if let Some(ram) = ram {
                icerik = icerik.child(Self::donanim_ozeti_satiri(
                    "Bellek",
                    format!("{} MB", ram.toplam_mb),
                    cx,
                ));
            }
            if disk_sayisi > 0 {
                icerik = icerik.child(Self::donanim_ozeti_satiri(
                    "Disk",
                    format!("{disk_sayisi} disk · {} GB", disk_gb.round() as u64),
                    cx,
                ));
            }
        }
        if let Some(hata) = &self.donanim_hata {
            icerik = icerik.child(self.hata_kutusu(hata, cx));
        }
        let eylemler = h_flex()
            .gap_2()
            .child(
                Button::new("donanim-ozet-oku")
                    .primary()
                    .label(if self.donanim_taraniyor {
                        "Okunuyor..."
                    } else if veri_var {
                        "Yenile"
                    } else {
                        "Donanım Bilgilerini Oku"
                    })
                    .disabled(self.donanim_taraniyor)
                    .on_click(cx.listener(|this, _, _, cx| this.donanim_tara(cx))),
            )
            .child(
                Button::new("donanim-ozet-ayrinti")
                    .label("Ayrıntılar")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.sayfaya_git(Sayfa::Donanim, window, cx);
                    })),
            );
        self.bolum_karti(
            "Donanım Özeti".to_string(),
            "Sistem, işlemci, ekran kartı, bellek ve disk özeti; ayrıntılar Donanım sayfasında."
                .to_string(),
            v_flex()
                .gap_3()
                .child(icerik)
                .child(eylemler)
                .into_any_element(),
            cx,
        )
    }

    fn donanim_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        let ozet = self.sistem_ozeti.clone();
        let cpu = self.cpu_bilgisi.clone();
        let gpu = self.gpu_bilgisi.clone();
        let ram = self.ram_bilgisi.clone();
        let diskler = self.diskler.clone();
        let wlan = self.wlan_profilleri.clone();
        let uac = self.uac_durumu.clone();
        let rdp = self.rdp_durumu.clone();
        let mesaj = self.donanim_mesaj.clone();
        let hata = self.donanim_hata.clone();
        let taraniyor = self.donanim_taraniyor;

        let ozet_icerik = match &ozet {
            Some(ozet) => v_flex()
                .gap_1()
                .child(format!("{} · {}", ozet.bilgisayar, ozet.isletim_sistemi))
                .child(format!(
                    "Sürüm: {} · Çalışma süresi: {}",
                    ozet.surum, ozet.uptime
                ))
                .child(format!(
                    "{} {} · BIOS: {}",
                    ozet.uretici, ozet.model, ozet.bios
                ))
                .into_any_element(),
            None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Henüz okunmadı.")
                .into_any_element(),
        };

        let cpu_icerik = v_flex()
            .gap_2()
            .children(cpu.iter().map(|cpu| {
                self.modul_satiri(
                    cpu.ad.clone(),
                    format!(
                        "{} çekirdek · {} mantıksal · {} MHz",
                        cpu.cekirdek, cpu.mantiksal, cpu.hiz_mhz
                    ),
                    String::new(),
                    Vec::new(),
                    cx,
                )
            }))
            .into_any_element();

        let gpu_icerik = v_flex()
            .gap_2()
            .children(gpu.iter().map(|gpu| {
                self.modul_satiri(
                    gpu.ad.clone(),
                    format!(
                        "{} · Sürücü: {}",
                        gpu.bellek_mb
                            .map(|mb| format!("{mb} MB"))
                            .unwrap_or_else(|| "Bellek bilinmiyor".to_string()),
                        gpu.surucu
                    ),
                    String::new(),
                    Vec::new(),
                    cx,
                )
            }))
            .into_any_element();

        let ram_icerik = match &ram {
            Some(ram) => v_flex()
                .gap_2()
                .child(format!("Toplam: {} MB", ram.toplam_mb))
                .children(ram.moduller.iter().map(|modul| {
                    self.modul_satiri(
                        modul.ad.clone(),
                        format!(
                            "{} MB · {}",
                            modul.boyut_mb,
                            modul
                                .hiz_mhz
                                .map(|hiz| format!("{hiz} MHz"))
                                .unwrap_or_else(|| "Hız bilinmiyor".to_string())
                        ),
                        String::new(),
                        Vec::new(),
                        cx,
                    )
                }))
                .into_any_element(),
            None => div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Henüz okunmadı.")
                .into_any_element(),
        };

        let disk_icerik = v_flex()
            .gap_2()
            .children(diskler.iter().map(|disk| {
                self.modul_satiri(
                    disk.ad.clone(),
                    format!("{:.1} GB · {}", disk.boyut_gb, disk.medya),
                    format!("Sağlık: {}", disk.saglik),
                    Vec::new(),
                    cx,
                )
            }))
            .into_any_element();

        let wlan_icerik = v_flex()
            .gap_2()
            .children(wlan.iter().map(|profil| {
                self.modul_satiri(
                    profil.ad.clone(),
                    format!("SSID: {}", profil.ssid),
                    String::new(),
                    Vec::new(),
                    cx,
                )
            }))
            .into_any_element();

        let guvenlik_icerik = v_flex()
            .gap_2()
            .child(match &uac {
                Some(uac) => format!(
                    "UAC: {} · İstem: {} · Güvenli masaüstü: {}",
                    if uac.etkin { "açık" } else { "kapalı" },
                    uac.prompt,
                    if uac.secure_desktop {
                        "açık"
                    } else {
                        "kapalı"
                    }
                ),
                None => "UAC: henüz okunmadı.".to_string(),
            })
            .child(match &rdp {
                Some(rdp) => format!(
                    "Uzak Masaüstü: {}",
                    if rdp.etkin { "açık" } else { "kapalı" }
                ),
                None => "Uzak Masaüstü: henüz okunmadı.".to_string(),
            })
            .into_any_element();

        let sayfa = v_flex()
            .gap_4()
            .child(
                h_flex().justify_end().items_center().child(
                    Button::new("donanim-yenile")
                        .primary()
                        .label(if taraniyor { "Okunuyor..." } else { "Yenile" })
                        .disabled(taraniyor)
                        .on_click(cx.listener(|this, _, _, cx| this.donanim_tara(cx))),
                ),
            )
            .child(if let Some(hata) = &hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(self.bolum_karti(
                "Sistem".to_string(),
                "Çekirdek sürücüsü kullanılmadan CIM/registry ile okunur.".to_string(),
                ozet_icerik,
                cx,
            ))
            .child(self.bolum_karti("İşlemci".to_string(), String::new(), cpu_icerik, cx))
            .child(self.bolum_karti("Ekran Kartı".to_string(), String::new(), gpu_icerik, cx))
            .child(self.bolum_karti("Bellek".to_string(), String::new(), ram_icerik, cx))
            .child(self.bolum_karti(
                "Diskler".to_string(),
                "Fiziksel disk sağlığı; SMART üreticiye göre sınırlı olabilir.".to_string(),
                disk_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "WLAN Profilleri".to_string(),
                "Kayıtlı profil adları ve SSID; parola gösterilmez.".to_string(),
                wlan_icerik,
                cx,
            ))
            .child(self.bolum_karti(
                "Güvenlik".to_string(),
                "UAC ve Uzak Masaüstü durumu salt okunur.".to_string(),
                guvenlik_icerik,
                cx,
            ));
        if let Some(onay) = &self.sistem_onay {
            sayfa
                .child(self.sistem_onay_paneli(onay.clone(), cx))
                .into_any_element()
        } else {
            sayfa.into_any_element()
        }
    }

    fn winget_durum_rozeti(
        pin: Option<&PinKaydi>,
        atlandi: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (metin, renk) = if let Some(pin) = pin {
            (pin.tur.metin(), cx.theme().warning)
        } else if atlandi {
            ("Bu sefer atlandı", cx.theme().muted_foreground)
        } else {
            ("Güncelleme var", cx.theme().info)
        };
        h_flex()
            .gap_1()
            .items_center()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(renk)
            .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(renk))
            .child(div().text_sm().child(metin.to_string()))
    }

    fn winget_guncelleme_satiri(
        &self,
        guncelleme: &GuncellemeSatiri,
        pinler: &[PinKaydi],
        atla: &HashSet<String>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pin = pinler.iter().find(|pin| pin.id == guncelleme.id);
        let atlandi = atla.contains(&guncelleme.id);
        let id_guncelle = guncelleme.id.clone();
        let id_asla = guncelleme.id.clone();
        let id_sabitle = guncelleme.id.clone();
        let surum_sabitle = guncelleme.mevcut_surum.clone();
        let id_atla = guncelleme.id.clone();
        let id_kaldir = guncelleme.id.clone();
        let secili = self.winget_guncelleme_secili.contains(&guncelleme.id);
        let secim = self.winget_secim_kutusu(
            format!("guncelleme-{}", guncelleme.id),
            guncelleme.id.clone(),
            secili,
            WingetSecim::Guncelleme,
            cx,
        );
        let mut aksiyonlar = h_flex()
            .gap_2()
            .items_center()
            .child(Self::winget_durum_rozeti(pin, atlandi, cx));
        if pin.is_some() {
            aksiyonlar = aksiyonlar.child(
                Button::new(format!("wpk-{}", guncelleme.id))
                    .label("Pin kaldır")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.winget_pin_kaldir(id_kaldir.clone(), cx);
                    })),
            );
        } else {
            aksiyonlar = aksiyonlar
                .child(
                    Button::new(format!("wg-{}", guncelleme.id))
                        .primary()
                        .label("Güncelle")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.winget_onay_iste(WingetOnay::Guncelle(id_guncelle.clone()), cx);
                        })),
                )
                .child(
                    Button::new(format!("wa-{}", guncelleme.id))
                        .label("Asla")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.winget_onay_iste(WingetOnay::Asla(id_asla.clone()), cx);
                        })),
                )
                .child(
                    Button::new(format!("ws-{}", guncelleme.id))
                        .label("Sürüm Sabitle")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.winget_onay_iste(
                                WingetOnay::SurumSabitle(id_sabitle.clone(), surum_sabitle.clone()),
                                cx,
                            );
                        })),
                )
                .child(
                    Button::new(format!("wt-{}", guncelleme.id))
                        .label("Bu sefer")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.winget_bu_sefer(id_atla.clone(), cx);
                        })),
                );
        }
        h_flex()
            .justify_between()
            .items_center()
            .p_3()
            .gap_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(secim)
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(guncelleme.ad.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!(
                                "{} · {} → {}",
                                guncelleme.id, guncelleme.kurulu_surum, guncelleme.mevcut_surum
                            )),
                    ),
            )
            .child(aksiyonlar)
    }

    fn winget_guncellemeler_bolumu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let guncellemeler = self.winget_guncellemeler.clone();
        let pinler = self.winget_pinler.clone();
        let atla = self.winget_atla.clone();
        let adaylar = guncellemeler
            .iter()
            .filter(|guncelleme| {
                !atla.contains(&guncelleme.id) && !pinler.iter().any(|pin| pin.id == guncelleme.id)
            })
            .map(|guncelleme| guncelleme.id.clone())
            .collect::<Vec<_>>();
        let secilen = guncellemeler
            .iter()
            .filter(|guncelleme| self.winget_guncelleme_secili.contains(&guncelleme.id))
            .map(|guncelleme| guncelleme.id.clone())
            .collect::<Vec<_>>();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Güncellemeler"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Paketleri tek tek veya seçerek güncelleyin. Pinli ve atlanan paketler seçilmez."),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-guncelleme-sec")
                                    .label("Tümünü Seç")
                                    .disabled(adaylar.is_empty())
                                    .on_click(cx.listener({
                                        let adaylar = adaylar.clone();
                                        move |this, _, _, cx| {
                                            for id in adaylar.clone() {
                                                this.winget_guncelleme_secili.insert(id);
                                            }
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("winget-guncelleme-temizle")
                                    .label("Seçimi Temizle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_guncelleme_secili.clear();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("winget-secili-guncelle")
                                    .primary()
                                    .label(format!("Seçilenleri Güncelle ({})", secilen.len()))
                                    .disabled(secilen.is_empty())
                                    .on_click(cx.listener({
                                        let secilen = secilen.clone();
                                        move |this, _, _, cx| {
                                            this.winget_onay_iste(
                                                WingetOnay::SeciliGuncelle(secilen.clone()),
                                                cx,
                                            );
                                        }
                                    })),
                            )
                            .child(
                                Button::new("winget-tumu")
                                    .danger()
                                    .label("Tümünü Güncelle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_onay_iste(WingetOnay::TumunuGuncelle, cx);
                                    })),
                            ),
                    ),
            )
            .child(if guncellemeler.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Güncelleme bulunamadı veya henüz tarama yapılmadı.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .children(
                        guncellemeler
                            .iter()
                            .map(|guncelleme| {
                                self.winget_guncelleme_satiri(guncelleme, &pinler, &atla, cx)
                            }),
                    )
                    .into_any_element()
            })
            .into_any_element()
    }

    fn winget_onerilenler_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let kurulu = &self.winget_kurulu;
        let mut secilebilir = Vec::new();
        for profil in &self.profiller {
            for paket in &profil.paket {
                if !kurulu
                    .iter()
                    .any(|kurulu_paket| kurulu_paket.id == paket.id)
                    && !secilebilir.contains(&paket.id)
                {
                    secilebilir.push(paket.id.clone());
                }
            }
        }
        let secilen = secilebilir
            .iter()
            .filter(|id| self.winget_onerilen_secili.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Önerilenler"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(
                                        "Hazır profillerdeki paketlerden istediklerinizi seçin.",
                                    ),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-oneri-sec")
                                    .label("Tümünü Seç")
                                    .disabled(secilebilir.is_empty())
                                    .on_click(cx.listener({
                                        let secilebilir = secilebilir.clone();
                                        move |this, _, _, cx| {
                                            for id in secilebilir.clone() {
                                                this.winget_onerilen_secili.insert(id);
                                            }
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("winget-oneri-temizle")
                                    .label("Seçimi Temizle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_onerilen_secili.clear();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("winget-oneri-kur")
                                    .primary()
                                    .label(format!("Seçilenleri Kur ({})", secilen.len()))
                                    .disabled(secilen.is_empty())
                                    .on_click(cx.listener({
                                        let secilen = secilen.clone();
                                        move |this, _, _, cx| {
                                            this.winget_onay_iste(
                                                WingetOnay::SeciliKur(secilen.clone()),
                                                cx,
                                            );
                                        }
                                    })),
                            ),
                    ),
            )
            .children(self.profiller.iter().map(|profil| {
                let idler = profil
                    .paket
                    .iter()
                    .filter(|paket| {
                        !kurulu
                            .iter()
                            .any(|kurulu_paket| kurulu_paket.id == paket.id)
                    })
                    .map(|paket| paket.id.clone())
                    .collect::<Vec<_>>();
                v_flex()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                v_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(profil.ad.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(profil.aciklama.clone()),
                                    ),
                            )
                            .child(
                                Button::new(format!("winget-profil-sec-{}", profil.id))
                                    .label("Profili Seç")
                                    .disabled(idler.is_empty())
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        for id in idler.clone() {
                                            this.winget_onerilen_secili.insert(id);
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .children(profil.paket.iter().map(|paket| {
                        let kurulu_mu = kurulu
                            .iter()
                            .any(|kurulu_paket| kurulu_paket.id == paket.id);
                        let secili = self.winget_onerilen_secili.contains(&paket.id);
                        let ad = paket.ad.clone().unwrap_or_else(|| paket.id.clone());
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(if kurulu_mu {
                                div().w(px(22.)).into_any_element()
                            } else {
                                self.winget_secim_kutusu(
                                    format!("oneri-{}-{}", profil.id, paket.id),
                                    paket.id.clone(),
                                    secili,
                                    WingetSecim::Onerilen,
                                    cx,
                                )
                                .into_any_element()
                            })
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(div().text_sm().child(ad))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(paket.id.clone()),
                                    ),
                            )
                            .child(if kurulu_mu {
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Kurulu")
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            })
                    }))
            }))
            .into_any_element()
    }

    fn winget_magaza_sonuc_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let sonuclar = &self.winget_magaza_sonuclari;
        let secilen = sonuclar
            .iter()
            .filter(|sonuc| self.winget_magaza_secili.contains(&sonuc.id))
            .map(|sonuc| sonuc.id.clone())
            .collect::<Vec<_>>();
        if sonuclar.is_empty() {
            return div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Arama sonucu yok. Bir paket adı yazıp arayın.")
                .into_any_element();
        }
        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(format!("{} sonuç", sonuclar.len()))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-magaza-sec")
                                    .label("Tümünü Seç")
                                    .on_click(cx.listener({
                                        let idler = sonuclar
                                            .iter()
                                            .map(|sonuc| sonuc.id.clone())
                                            .collect::<Vec<_>>();
                                        move |this, _, _, cx| {
                                            for id in idler.clone() {
                                                this.winget_magaza_secili.insert(id);
                                            }
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("winget-magaza-temizle")
                                    .label("Seçimi Temizle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_magaza_secili.clear();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("winget-magaza-kur")
                                    .primary()
                                    .label(format!("Seçilenleri Kur ({})", secilen.len()))
                                    .disabled(secilen.is_empty())
                                    .on_click(cx.listener({
                                        let secilen = secilen.clone();
                                        move |this, _, _, cx| {
                                            this.winget_onay_iste(
                                                WingetOnay::SeciliKur(secilen.clone()),
                                                cx,
                                            );
                                        }
                                    })),
                            ),
                    ),
            )
            .children(sonuclar.iter().enumerate().map(|(sira, sonuc)| {
                let secili = self.winget_magaza_secili.contains(&sonuc.id);
                h_flex()
                    .gap_2()
                    .items_center()
                    .p_2()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(self.winget_secim_kutusu(
                        format!("magaza-{sira}-{}", sonuc.id),
                        sonuc.id.clone(),
                        secili,
                        WingetSecim::Magaza,
                        cx,
                    ))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_sm().child(sonuc.ad.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!(
                                        "{} · {} · {}",
                                        sonuc.id,
                                        sonuc.surum.clone().unwrap_or_else(|| "sürüm yok".into()),
                                        sonuc.kaynak.clone().unwrap_or_else(|| "kaynak yok".into())
                                    )),
                            ),
                    )
            }))
            .into_any_element()
    }

    fn winget_magaza_bolumu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let girdi = self.winget_magaza_girdisi(window, cx);
        let arama = girdi.read(cx).value().trim().to_string();
        let kaynaklar = if self.winget_kaynaklar.is_empty() {
            vec![
                Kaynak {
                    ad: "winget".into(),
                    adres: "https://cdn.winget.microsoft.com/cache".into(),
                    acik: false,
                },
                Kaynak {
                    ad: "msstore".into(),
                    adres: "https://storeedgefd.dsx.mp.microsoft.com/v9.0".into(),
                    acik: false,
                },
            ]
        } else {
            self.winget_kaynaklar.clone()
        };
        let kimlik = paket_kimligi_gecerli(&arama).then(|| arama.clone());
        let secili_kaynak = self.winget_magaza_kaynak.clone();
        v_flex()
            .gap_4()
            .child(
                v_flex()
                    .gap_2()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Winget Mağazası"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Kaynak seçin, paket arayın ve kutucuklarla seçtiğiniz paketleri kurun."),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .children(kaynaklar.iter().map(|kaynak| {
                                let ad = kaynak.ad.clone();
                                let aktif = kaynak.ad == secili_kaynak;
                                let buton = Button::new(format!("winget-kaynak-sec-{}", kaynak.ad))
                                    .label(kaynak.ad.clone());
                                let buton = if aktif { buton.primary() } else { buton.ghost() };
                                buton.on_click(cx.listener(move |this, _, _, cx| {
                                    this.winget_magaza_kaynak = ad.clone();
                                    cx.notify();
                                }))
                            })),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Input::new(&girdi).cleanable(true).w_full())
                            .child(
                                Button::new("winget-magaza-ara")
                                    .primary()
                                    .label("Ara")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_magaza_ara(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-kimlikle-kur")
                                    .label("Kimlikle Kur")
                                    .disabled(kimlik.is_none() || self.winget_taraniyor)
                                    .on_click(cx.listener({
                                        let kimlik = kimlik.clone();
                                        move |this, _, _, cx| {
                                            if let Some(kimlik) = kimlik.clone() {
                                                this.winget_onay_iste(
                                                    WingetOnay::SeciliKur(vec![kimlik]),
                                                    cx,
                                                );
                                            }
                                        }
                                    })),
                            ),
                    )
                    .child(self.winget_magaza_sonuc_karti(cx)),
            )
            .child(self.winget_onerilenler_karti(cx))
            .into_any_element()
    }

    fn winget_kurulu_bolumu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let kurulu = self.winget_kurulu.clone();
        let secilen = kurulu
            .iter()
            .filter(|paket| self.winget_kurulu_secili.contains(&paket.id))
            .map(|paket| paket.id.clone())
            .collect::<Vec<_>>();
        let guncellenebilir = secilen
            .iter()
            .filter(|id| {
                self.winget_guncellemeler
                    .iter()
                    .any(|guncelleme| &guncelleme.id == *id)
            })
            .cloned()
            .collect::<Vec<_>>();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Kurulu Winget Paketleri"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Winget tarafından yönetilen kurulu paketleri tarayın; seçtiklerinizi kaldırın veya güncelleyin."),
                            ),
                    )
                    .child(
                        Button::new("winget-kurulu-tara")
                            .primary()
                            .label("Kurulu Paketleri Tara")
                            .disabled(self.winget_taraniyor)
                            .on_click(cx.listener(|this, _, _, cx| this.winget_kurulu_tara(cx))),
                    ),
            )
            .child(if kurulu.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz tarama yapılmadı veya kurulu winget paketi yok.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Button::new("winget-kurulu-sec")
                                    .label("Tümünü Seç")
                                    .on_click(cx.listener({
                                        let idler = kurulu
                                            .iter()
                                            .map(|paket| paket.id.clone())
                                            .collect::<Vec<_>>();
                                        move |this, _, _, cx| {
                                            for id in idler.clone() {
                                                this.winget_kurulu_secili.insert(id);
                                            }
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("winget-kurulu-temizle")
                                    .label("Seçimi Temizle")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_kurulu_secili.clear();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("winget-kurulu-kaldir")
                                    .danger()
                                    .label(format!("Seçilenleri Kaldır ({})", secilen.len()))
                                    .disabled(secilen.is_empty())
                                    .on_click(cx.listener({
                                        let secilen = secilen.clone();
                                        move |this, _, _, cx| {
                                            this.winget_onay_iste(
                                                WingetOnay::SeciliKaldir(secilen.clone()),
                                                cx,
                                            );
                                        }
                                    })),
                            )
                            .child(
                                Button::new("winget-kurulu-guncelle")
                                    .label(format!("Seçilenleri Güncelle ({})", guncellenebilir.len()))
                                    .disabled(guncellenebilir.is_empty())
                                    .on_click(cx.listener({
                                        let guncellenebilir = guncellenebilir.clone();
                                        move |this, _, _, cx| {
                                            this.winget_onay_iste(
                                                WingetOnay::SeciliGuncelle(
                                                    guncellenebilir.clone(),
                                                ),
                                                cx,
                                            );
                                        }
                                    })),
                            ),
                    )
                    .children(kurulu.iter().map(|paket| {
                        let secili = self.winget_kurulu_secili.contains(&paket.id);
                        let guncelleme = self
                            .winget_guncellemeler
                            .iter()
                            .find(|guncelleme| guncelleme.id == paket.id);
                        let ad = paket.ad.clone().unwrap_or_else(|| paket.id.clone());
                        h_flex()
                            .gap_2()
                            .items_center()
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(self.winget_secim_kutusu(
                                format!("kurulu-{}", paket.id),
                                paket.id.clone(),
                                secili,
                                WingetSecim::Kurulu,
                                cx,
                            ))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(div().text_sm().child(ad))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!(
                                                "{} · {}",
                                                paket.id,
                                                paket.surum.clone().unwrap_or_else(|| "sürüm yok".into())
                                            )),
                                    ),
                            )
                            .child(if let Some(guncelleme) = guncelleme {
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().info)
                                    .child(format!(
                                        "{} → {}",
                                        guncelleme.kurulu_surum, guncelleme.mevcut_surum
                                    ))
                                    .into_any_element()
                            } else {
                                div().into_any_element()
                            })
                    }))
                    .into_any_element()
            })
            .into_any_element()
    }

    fn winget_kaynaklar_bolumu(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (ad_girdi, adres_girdi) = self.winget_kaynak_girdileri(window, cx);
        let kaynaklar = self.winget_kaynaklar.clone();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Winget Kaynakları"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Kaynak ekleyin, kaldırın veya güncelleyin. Adres yalnızca https:// olabilir."),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-kaynak-listele")
                                    .label("Kaynakları Listele")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_kaynaklari_listele(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-kaynak-guncelle")
                                    .label("Güncelle")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_kaynak_guncelle(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Yeni Kaynak"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Input::new(&ad_girdi).cleanable(true).w_full())
                            .child(Input::new(&adres_girdi).cleanable(true).w_full())
                            .child(
                                Button::new("winget-kaynak-ekle")
                                    .primary()
                                    .label("Ekle")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_kaynak_ekle(cx);
                                    })),
                            ),
                    ),
            )
            .child(if kaynaklar.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Kaynak listesi henüz yüklenmedi.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .children(kaynaklar.iter().map(|kaynak| {
                        let ad = kaynak.ad.clone();
                        h_flex()
                            .justify_between()
                            .items_center()
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .child(kaynak.ad.clone()),
                                            )
                                            .child(if kaynak.acik {
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().warning)
                                                    .child("Açık")
                                                    .into_any_element()
                                            } else {
                                                div().into_any_element()
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(kaynak.adres.clone()),
                                    ),
                            )
                            .child(
                                Button::new(format!("winget-kaynak-kaldir-{}", kaynak.ad))
                                    .danger()
                                    .label("Kaldır")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.winget_onay_iste(
                                            WingetOnay::KaynakKaldir(ad.clone()),
                                            cx,
                                        );
                                    })),
                            )
                    }))
                    .into_any_element()
            })
            .into_any_element()
    }

    fn winget_liste_bolumu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let girdi = self.winget_liste_girdisi(window, cx);
        let paketler = self.winget_ice_aktarilan.clone();
        let secilen = paketler
            .iter()
            .filter(|paket| self.winget_ice_aktarilan_secili.contains(&paket.id))
            .map(|paket| paket.id.clone())
            .collect::<Vec<_>>();
        v_flex()
            .gap_3()
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Winget Listesi"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Listeyi yapıştırın, dosyadan içe aktarın veya dışa aktarın. Dışa aktarma önce içe aktarılan seçimi, sonra kurulu seçimi, yoksa tüm kurulu listeyi kullanır."),
                    )
                    .child(Textarea::new(&girdi).h(px(120.)).w_full())
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-liste-coz")
                                    .primary()
                                    .label("Yapıştırılanı Çöz")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_liste_coz(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-liste-ice")
                                    .label("Dosyadan İçe Aktar")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_liste_ice_aktar(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-liste-disa")
                                    .label("Dışa Aktar")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_liste_disa_aktar(cx);
                                    })),
                            ),
                    ),
            )
            .child(if paketler.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz içe aktarılan paket listesi yok.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(format!("{} paket", paketler.len()))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("winget-liste-sec")
                                            .label("Tümünü Seç")
                                            .on_click(cx.listener({
                                                let idler = paketler
                                                    .iter()
                                                    .map(|paket| paket.id.clone())
                                                    .collect::<Vec<_>>();
                                                move |this, _, _, cx| {
                                                    for id in idler.clone() {
                                                        this.winget_ice_aktarilan_secili.insert(id);
                                                    }
                                                    cx.notify();
                                                }
                                            })),
                                    )
                                    .child(
                                        Button::new("winget-liste-temizle")
                                            .label("Seçimi Temizle")
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.winget_ice_aktarilan_secili.clear();
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("winget-liste-kur")
                                            .primary()
                                            .label(format!("Seçilenleri Kur ({})", secilen.len()))
                                            .disabled(secilen.is_empty())
                                            .on_click(cx.listener({
                                                let secilen = secilen.clone();
                                                move |this, _, _, cx| {
                                                    this.winget_onay_iste(
                                                        WingetOnay::ListeKur(secilen.clone()),
                                                        cx,
                                                    );
                                                }
                                            })),
                                    ),
                            ),
                    )
                    .children(paketler.iter().map(|paket| {
                        let secili = self.winget_ice_aktarilan_secili.contains(&paket.id);
                        h_flex()
                            .gap_2()
                            .items_center()
                            .p_2()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(self.winget_secim_kutusu(
                                format!("liste-{}", paket.id),
                                paket.id.clone(),
                                secili,
                                WingetSecim::IceAktarilan,
                                cx,
                            ))
                            .child(
                                v_flex()
                                    .flex_1()
                                    .min_w_0()
                                    .child(div().text_sm().child(paket.id.clone()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(
                                                paket
                                                    .surum
                                                    .clone()
                                                    .unwrap_or_else(|| "sürüm belirtilmedi".into()),
                                            ),
                                    ),
                            )
                    }))
                    .into_any_element()
            })
            .into_any_element()
    }

    fn winget_profiller_bolumu(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let profil_sonuclari = self.winget_profil_sonuclari.clone();
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Profiller"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Profil paketlerini kurulu listeyle karşılaştırın; eksikleri ve yeniden adlandırılmış olabilecekleri görün."),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-profil-tara")
                                    .label("Profilleri Tara")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_profilleri_tara(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-eksik-kur")
                                    .primary()
                                    .label("Eksikleri Kur")
                                    .disabled(self.winget_taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_onay_iste(WingetOnay::EksikleriKur, cx);
                                    })),
                            ),
                    ),
            )
            .child(h_flex().gap_3().children(
                self.profiller
                    .iter()
                    .map(|profil| {
                        Self::kart(
                            &profil.ad,
                            profil.paket.len().to_string(),
                            IconName::Star,
                            cx,
                        )
                    }),
            ))
            .child(if profil_sonuclari.is_empty() {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("Henüz profil taraması yapılmadı.")
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .children(
                        profil_sonuclari
                            .into_iter()
                            .map(|sonuc| Self::profil_sonucu_satiri(sonuc, cx)),
                    )
                    .into_any_element()
            })
            .into_any_element()
    }

    fn winget_sayfasi(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let lisans = self.ayarlar.winget_lisans_onayi;
        let kaynak = self.ayarlar.winget_kaynak_onayi;
        let taraniyor = self.winget_taraniyor;
        let aktif = self.winget_sekme;
        let sekmeler = [
            (WingetSekme::Guncellemeler, "Güncellemeler"),
            (WingetSekme::Magaza, "Mağaza"),
            (WingetSekme::Kurulu, "Kurulu"),
            (WingetSekme::Kaynaklar, "Kaynaklar"),
            (WingetSekme::Liste, "Liste"),
            (WingetSekme::Profiller, "Profiller"),
        ];

        let icerik = match aktif {
            WingetSekme::Guncellemeler => self.winget_guncellemeler_bolumu(cx),
            WingetSekme::Magaza => self.winget_magaza_bolumu(window, cx),
            WingetSekme::Kurulu => self.winget_kurulu_bolumu(cx),
            WingetSekme::Kaynaklar => self.winget_kaynaklar_bolumu(window, cx),
            WingetSekme::Liste => self.winget_liste_bolumu(window, cx),
            WingetSekme::Profiller => self.winget_profiller_bolumu(cx),
        };
        let komutlar = self.winget_komutlar.clone();

        v_flex()
            .gap_4()
            .child(
                h_flex()
                    .justify_end()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-kaynak")
                                    .label("Kaynakları Güncelle")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_kaynak_guncelle(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-tara")
                                    .primary()
                                    .label(if taraniyor { "İşlem sürüyor..." } else { "Tara" })
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_tara(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children(sekmeler.iter().map(|(sekme, etiket)| {
                        let sekme = *sekme;
                        let buton = Button::new(format!("winget-sekme-{etiket}")).label(*etiket);
                        let buton = if aktif == sekme {
                            buton.primary()
                        } else {
                            buton.ghost()
                        };
                        buton.on_click(cx.listener(move |this, _, _, cx| {
                            this.winget_sekme = sekme;
                            cx.notify();
                        }))
                    })),
            )
            .child(if let Some(hata) = &self.winget_hata {
                self.hata_kutusu(hata, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if let Some(mesaj) = &self.winget_mesaj {
                self.mesaj_kutusu(mesaj, cx).into_any_element()
            } else {
                div().into_any_element()
            })
            .child(if self.winget_onay.is_some() {
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
                    .child(div().text_sm().child(self.winget_onay_metni()))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("winget-onayla")
                                    .primary()
                                    .label("Onayla")
                                    .disabled(taraniyor)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_onayla(cx);
                                    })),
                            )
                            .child(
                                Button::new("winget-vazgec")
                                    .label("Vazgeç")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.winget_onay = None;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .into_any_element()
            } else {
                div().into_any_element()
            })
            .child(
                v_flex()
                    .p_3()
                    .gap_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Sözleşme Onayları"),
                            )
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("winget-lisans")
                                            .label(if lisans {
                                                "Paket lisansı: Açık"
                                            } else {
                                                "Paket lisansı: Kapalı"
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.winget_lisans_degistir(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("winget-kaynak-onay")
                                            .label(if kaynak {
                                                "Kaynak onayı: Açık"
                                            } else {
                                                "Kaynak onayı: Kapalı"
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.winget_kaynak_degistir(cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Paket lisansı ve kaynak sözleşmeleri varsayılan olarak kapalıdır; kurulum ve güncelleme için açık olmalıdır."),
                    ),
            )
            .child(icerik)
            .child(
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Komut Günlüğü"),
                    )
                    .child(if komutlar.is_empty() {
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Henüz komut çalıştırılmadı.")
                            .into_any_element()
                    } else {
                        v_flex()
                            .gap_1()
                            .children(komutlar.iter().rev().take(10).map(|komut| {
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(komut.clone())
                            }))
                            .into_any_element()
                    }),
            )
            .into_any_element()
    }

    fn sayfaya_git(&mut self, sayfa: Sayfa, window: &mut Window, cx: &mut Context<Self>) {
        if self.sayfa == sayfa {
            return;
        }
        self.sayfa = sayfa;
        self.merkez_onay = None;
        self.merkez_mesaj = None;
        self.merkez_hata = None;
        if sayfa == Sayfa::Android {
            self.android_arama_olustur(window, cx);
        }
        if sayfa.arac_aramasi_gerekli() {
            self.merkez_sorgu = None;
            self.merkez_sorgu_abonelik = None;
            self.merkez_sorgu_olustur(window, cx);
        }
        cx.notify();
    }

    fn dokunma_hedefi(buton: Button) -> Button {
        buton.h(px(DOKUNMA_HEDEFI_PX)).min_w(px(DOKUNMA_HEDEFI_PX))
    }

    fn bolum_butonu(&self, bolum: Bolum, cx: &mut Context<Self>) -> AnyElement {
        let buton = Button::new(format!("bolum-{}", bolum.kimlik()))
            .icon(bolum.ikon())
            .label(bolum.baslik())
            .with_size(gpui_kit::component::Size::Small)
            .px_2();
        let buton = if self.sayfa.bolum() == Some(bolum) {
            buton.primary()
        } else {
            buton.ghost()
        };
        Self::dokunma_hedefi(buton)
            .on_click(cx.listener(move |this, _, window, cx| {
                let hedef = bolum
                    .sayfalar()
                    .first()
                    .copied()
                    .unwrap_or(Sayfa::GenelBakis);
                this.sayfaya_git(hedef, window, cx);
            }))
            .into_any_element()
    }

    fn tema_dugmesi(
        &self,
        tema: Tema,
        kimlik: &'static str,
        etiket: &'static str,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let buton = Button::new(format!("tema-{kimlik}"))
            .label(etiket)
            .with_size(gpui_kit::component::Size::Small)
            .px_3();
        let buton = if self.ayarlar.tema == tema {
            buton.primary()
        } else {
            buton.ghost()
        };
        Self::dokunma_hedefi(buton)
            .on_click(cx.listener(move |this, _, _, cx| this.tema_ayarla(tema, cx)))
            .into_any_element()
    }

    fn tema_secici(&self, cx: &mut Context<Self>) -> AnyElement {
        h_flex()
            .gap_1()
            .items_center()
            .child(self.tema_dugmesi(Tema::Sistem, "sistem", "Sistem", cx))
            .child(self.tema_dugmesi(Tema::Acik, "acik", "Açık", cx))
            .child(self.tema_dugmesi(Tema::Koyu, "koyu", "Koyu", cx))
            .into_any_element()
    }

    fn ust_gezinme(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut sol = h_flex().flex_shrink_0().gap_1().items_center();
        for bolum in Bolum::HEPSI {
            sol = sol.child(self.bolum_butonu(bolum, cx));
        }
        let ayarlar = Button::new("ayarlar-ust")
            .icon(IconName::Settings)
            .tooltip("Ayarlar")
            .accessibility_label("Ayarlar");
        let ayarlar = if self.sayfa == Sayfa::Ayarlar {
            ayarlar.primary()
        } else {
            ayarlar.ghost()
        };
        h_flex()
            .h(px(UST_CUBUK_YUKSEKLIGI_PX))
            .px_4()
            .gap_2()
            .items_center()
            .flex_shrink_0()
            .bg(cx.theme().sidebar)
            .border_b_1()
            .border_color(cx.theme().border)
            .child(sol)
            .child(div().flex_1())
            .child(Self::dokunma_hedefi(
                Button::new("tercih-ice")
                    .icon(IconName::ArrowDown)
                    .ghost()
                    .tooltip("Tercihleri içe aktar")
                    .accessibility_label("Tercihleri içe aktar")
                    .on_click(cx.listener(|this, _, _, cx| this.tercihleri_ice_aktar(cx))),
            ))
            .child(Self::dokunma_hedefi(
                Button::new("tercih-disa")
                    .icon(IconName::ArrowUp)
                    .ghost()
                    .tooltip("Tercihleri dışa aktar")
                    .accessibility_label("Tercihleri dışa aktar")
                    .on_click(cx.listener(|this, _, _, cx| this.tercihleri_disa_aktar(cx))),
            ))
            .child(Self::dokunma_hedefi(
                Button::new("geri-al-ust")
                    .icon(IconName::Undo2)
                    .ghost()
                    .tooltip("Son işlemi geri al")
                    .accessibility_label("Son işlemi geri al")
                    .on_click(cx.listener(|this, _, _, cx| this.son_islemi_geri_al(cx))),
            ))
            .child(
                Self::dokunma_hedefi(ayarlar).on_click(cx.listener(|this, _, window, cx| {
                    this.sayfaya_git(Sayfa::Ayarlar, window, cx);
                })),
            )
    }

    fn baslik_cubugu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let satir = h_flex()
            .h(px(BASLIK_CUBUGU_YUKSEKLIGI_PX))
            .px_4()
            .gap_3()
            .items_center()
            .flex_shrink_0()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                Icon::new(self.sayfa.ikon())
                    .with_size(gpui_kit::component::Size::Medium)
                    .text_color(cx.theme().muted_foreground),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_0p5()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.sayfa.baslik()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(self.sayfa.aciklama()),
                    ),
            );
        satir
    }

    fn kart(
        baslik: &str,
        deger: String,
        ikon: IconName,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_w_0()
            .p_3()
            .gap_1()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().colors.list)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(ikon).text_color(cx.theme().muted_foreground))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(baslik.to_string()),
                    ),
            )
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(deger),
            )
    }

    fn durum_rozeti(durum: &KuralDurumu, cx: &mut Context<Self>) -> impl IntoElement {
        let renk = match durum {
            KuralDurumu::Uygulandi => cx.theme().success,
            KuralDurumu::Uygulanmadi => cx.theme().muted_foreground,
            KuralDurumu::Kismi => cx.theme().warning,
            KuralDurumu::Desteklenmiyor => cx.theme().muted_foreground,
            KuralDurumu::Bilinmiyor => cx.theme().info,
            KuralDurumu::Hata => cx.theme().danger,
        };
        h_flex()
            .gap_1()
            .items_center()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(renk)
            .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(renk))
            .child(div().text_sm().child(durum.metin().to_string()))
    }

    fn risk_rozeti(risk: Risk, cx: &mut Context<Self>) -> impl IntoElement {
        let renk = match risk {
            Risk::Dusuk => cx.theme().success,
            Risk::Orta => cx.theme().warning,
            Risk::Yuksek => cx.theme().danger,
            Risk::Kritik => cx.theme().danger,
        };
        h_flex()
            .gap_1()
            .items_center()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(renk)
            .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(renk))
            .child(div().text_sm().child(risk.metin().to_string()))
    }

    fn hata_kutusu(&self, hata: &str, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .items_center()
            .p_3()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(cx.theme().danger)
            .child(Icon::new(IconName::CircleAlert).text_color(cx.theme().danger))
            .child(div().text_sm().child(hata.to_string()))
    }

    fn uyari_kutusu(&self, uyari: &str, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .items_center()
            .p_3()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(cx.theme().warning)
            .child(Icon::new(IconName::TriangleAlert).text_color(cx.theme().warning))
            .child(div().text_sm().child(uyari.to_string()))
    }

    fn mesaj_kutusu(&self, mesaj: &str, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .items_center()
            .p_3()
            .rounded_md()
            .bg(cx.theme().muted)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(cx.theme().success)
            .child(Icon::new(IconName::CircleCheck).text_color(cx.theme().success))
            .child(div().text_sm().child(mesaj.to_string()))
    }

    fn bayt_metni(bayt: u64) -> String {
        let (deger, birim) = if bayt >= 1024 * 1024 * 1024 {
            (bayt as f64 / 1024.0 / 1024.0 / 1024.0, "GB")
        } else if bayt >= 1024 * 1024 {
            (bayt as f64 / 1024.0 / 1024.0, "MB")
        } else if bayt >= 1024 {
            (bayt as f64 / 1024.0, "KB")
        } else {
            (bayt as f64, "B")
        };
        format!("{deger:.1} {birim}").replace('.', ",")
    }

    fn sistem_paketi_mi(ad: &str, yayinci: &str) -> bool {
        let ad = ad.to_lowercase();
        let yayinci = yayinci.to_lowercase();
        ad.starts_with("microsoft.")
            || ad.starts_with("windows.")
            || ad.starts_with("microsoftwindows.")
            || yayinci.contains("cn=microsoft windows")
            || yayinci.contains("microsoft corporation")
    }

    fn modul_satiri(
        &self,
        baslik: String,
        alt: String,
        detay: String,
        aksiyonlar: Vec<AnyElement>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut sag = h_flex().gap_2().items_center();
        for aksiyon in aksiyonlar {
            sag = sag.child(aksiyon);
        }
        h_flex()
            .justify_between()
            .items_center()
            .p_3()
            .gap_4()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(baslik))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(alt),
                    )
                    .child(if detay.is_empty() {
                        div().into_any_element()
                    } else {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(detay)
                            .into_any_element()
                    }),
            )
            .child(sag)
    }

    fn bolum_karti(
        &self,
        baslik: String,
        aciklama: String,
        icerik: AnyElement,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(baslik),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(aciklama),
                    ),
            )
            .child(icerik)
            .into_any_element()
    }

    fn bolum_basligi(baslik: &str, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(cx.theme().muted_foreground)
            .child(baslik.to_string())
    }

    fn durum_alani(baslik: &str, deger: String, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_1()
            .min_w_0()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(baslik.to_string()),
            )
            .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(deger))
    }

    fn sistem_durumu(&self, cx: &mut Context<Self>) -> AnyElement {
        let sistem = self.motor.sistem();
        v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().colors.list)
            .child(
                h_flex()
                    .gap_6()
                    .child(Self::durum_alani(
                        "İşletim sistemi",
                        format!(
                            "Windows {}.{}.{}",
                            sistem.surum.major, sistem.surum.minor, sistem.surum.build
                        ),
                        cx,
                    ))
                    .child(Self::durum_alani(
                        "Yönetici",
                        if sistem.yonetici {
                            "Evet".to_string()
                        } else {
                            "Hayır".to_string()
                        },
                        cx,
                    ))
                    .child(Self::durum_alani("Sürüm", surum_etiketi(), cx)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Veri dizini: {}", self.dizinler.veri.display())),
            )
            .into_any_element()
    }

    fn modul_karti(&self, sayfa: Sayfa, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .flex_1()
            .min_w_0()
            .p_3()
            .gap_2()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().colors.list)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .min_w_0()
                    .child(
                        Icon::new(sayfa.ikon())
                            .with_size(gpui_kit::component::Size::Medium)
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(sayfa.baslik()),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(sayfa.aciklama()),
            )
            .child(Self::dokunma_hedefi(
                Button::new(format!("modul-{}", sayfa.kimlik()))
                    .icon(IconName::ChevronRight)
                    .label("Aç")
                    .px_3()
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.sayfaya_git(sayfa, window, cx);
                    })),
            ))
            .into_any_element()
    }

    fn modul_vitrini(&self, cx: &mut Context<Self>) -> AnyElement {
        let moduller = Sayfa::HEPSI
            .iter()
            .copied()
            .filter(|sayfa| *sayfa != Sayfa::GenelBakis)
            .collect::<Vec<_>>();
        let satirlar = moduller
            .chunks(3)
            .map(|cift| {
                h_flex()
                    .gap_3()
                    .children(cift.iter().map(|sayfa| self.modul_karti(*sayfa, cx)))
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        v_flex()
            .gap_2()
            .child(Self::bolum_basligi("Modüller", cx))
            .children(satirlar)
            .into_any_element()
    }

    fn son_islemler(&self, cx: &mut Context<Self>) -> AnyElement {
        let icerik = match self.gunluk.son_kayitlar(5) {
            Ok(kayitlar) if kayitlar.is_empty() => h_flex()
                .gap_2()
                .items_center()
                .child(Icon::new(IconName::Info).text_color(cx.theme().muted_foreground))
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Henüz işlem yok."),
                )
                .into_any_element(),
            Ok(kayitlar) => v_flex()
                .gap_2()
                .children(
                    kayitlar
                        .iter()
                        .rev()
                        .map(|kayit| self.gecmis_satiri(kayit, cx)),
                )
                .into_any_element(),
            Err(gecmis_hatasi) => self
                .hata_kutusu(&gecmis_hatasi.to_string(), cx)
                .into_any_element(),
        };
        v_flex()
            .gap_2()
            .child(Self::bolum_basligi("Son işlemler", cx))
            .child(icerik)
            .into_any_element()
    }

    fn genel_bakis(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_4()
            .child(self.sistem_durumu(cx))
            .child(self.donanim_ozeti_karti(cx))
            .child(self.tarama_karti(cx))
            .child(self.modul_vitrini(cx))
            .child(self.son_islemler(cx))
            .into_any_element()
    }

    fn ince_ayarlar(&self, cx: &mut Context<Self>) -> AnyElement {
        let kurallar = self.kurallar.clone();
        let toplam = kurallar.len();
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
        let desteklenmeyen = self
            .durumlar
            .values()
            .filter(|durum| **durum == KuralDurumu::Desteklenmiyor)
            .count();
        let mut gruplar = v_flex().flex_1().min_w_0().gap_6();
        for kategori in KuralKategorisi::HEPSI {
            let bolum_kurallari = kurallar
                .iter()
                .filter(|kural| kural_kategorisi(kural) == kategori)
                .collect::<Vec<_>>();
            if bolum_kurallari.is_empty() {
                continue;
            }
            let mut bolum = v_flex().gap_2().child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Icon::new(kategori.ikon()).text_color(cx.theme().muted_foreground))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(cx.theme().muted_foreground)
                            .child(kategori.baslik()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{} kural", bolum_kurallari.len())),
                    ),
            );
            for kural in bolum_kurallari {
                bolum = bolum.child(self.kural_satiri(kural, cx));
            }
            gruplar = gruplar.child(bolum);
        }
        v_flex()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "Toplam {toplam} kural · {uygulanan} uygulanan · {bekleyen} bekleyen · {desteklenmeyen} desteklenmeyen"
                    )),
            )
            .child(
                h_flex()
                    .gap_6()
                    .items_start()
                    .child(gruplar)
                    .child(self.detay_paneli(cx)),
            )
            .into_any_element()
    }

    fn kural_satiri(&self, kural: &Kural, cx: &mut Context<Self>) -> impl IntoElement {
        let durum = self
            .durumlar
            .get(&kural.id)
            .cloned()
            .unwrap_or(KuralDurumu::Bilinmiyor);
        let id = kural.id.clone();
        h_flex()
            .justify_between()
            .items_center()
            .p_3()
            .gap_4()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(kural.baslik.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(kural.ozet.clone()),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(Self::durum_rozeti(&durum, cx))
                    .child(Self::risk_rozeti(kural.risk, cx))
                    .child(
                        Button::new(format!("sec-{}", kural.id))
                            .ghost()
                            .label("İncele")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.secili = Some(id.clone());
                                this.onay = None;
                                cx.notify();
                            })),
                    ),
            )
    }

    fn detay_paneli(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let icerik = match self.secili_kural() {
            Some(kural) => {
                let durum = self
                    .durumlar
                    .get(&kural.id)
                    .cloned()
                    .unwrap_or(KuralDurumu::Bilinmiyor);
                let id = kural.id.clone();
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(kural.baslik.clone()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(kural.ozet.clone()),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Self::durum_rozeti(&durum, cx))
                            .child(Self::risk_rozeti(kural.risk, cx)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(kural.kanit.clone()),
                    )
                    .child(if kural.yeniden_baslatma {
                        div()
                            .text_xs()
                            .text_color(cx.theme().warning)
                            .child("Explorer yeniden başlatma gerekebilir.")
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    })
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new(format!("uygula-{}", kural.id))
                                    .primary()
                                    .label("Uygula")
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.uygula_secili(&id, cx);
                                    })),
                            )
                            .child(Button::new("geri-al-detay").label("Geri Al").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.son_islemi_geri_al(cx);
                                }),
                            )),
                    )
                    .child(self.onay_paneli(cx))
                    .into_any_element()
            }
            None => v_flex()
                .gap_2()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Kural seçilmedi"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("İncelemek için soldaki listeden bir kural seçin."),
                )
                .into_any_element(),
        };

        v_flex()
            .w(px(360.))
            .gap_3()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().colors.list)
            .child(icerik)
    }

    fn onay_paneli(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(plan) = &self.onay else {
            return div().into_any_element();
        };
        v_flex()
            .gap_2()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().warning)
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Değişiklik önizlemesi"),
            )
            .children(
                plan.adimlar
                    .iter()
                    .map(|adim| div().text_sm().child(adim.aciklama.clone())),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("onayla")
                            .primary()
                            .label("Onayla")
                            .on_click(cx.listener(|this, _, _, cx| this.onayla(cx))),
                    )
                    .child(Button::new("vazgec").label("Vazgeç").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.onay = None;
                            cx.notify();
                        },
                    ))),
            )
            .into_any_element()
    }

    fn gecmis(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut liste = v_flex().gap_2();
        match self.gunluk.son_kayitlar(50) {
            Ok(kayitlar) if kayitlar.is_empty() => {
                liste = liste.child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(Icon::new(IconName::Info).text_color(cx.theme().muted_foreground))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Henüz işlem yok."),
                        ),
                );
            }
            Ok(kayitlar) => {
                for kayit in kayitlar.iter().rev() {
                    liste = liste.child(self.gecmis_satiri(kayit, cx));
                }
            }
            Err(gecmis_hatasi) => {
                liste = liste.child(self.hata_kutusu(&gecmis_hatasi.to_string(), cx));
            }
        }
        liste.into_any_element()
    }

    fn gecmis_satiri(&self, kayit: &GunlukKaydi, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .justify_between()
            .gap_4()
            .p_3()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .min_w_0()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(IconName::Undo2).text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(kayit.kural_id.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(kayit.eylem.clone()),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{} · {}", kayit.zaman, kayit.sonuc)),
            )
    }

    fn tarama_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let suruyor = self.tarama_suruyor();
        let ozet = self.tarama_ozeti();
        let durum = if suruyor {
            self.tarama
                .mesaj
                .clone()
                .unwrap_or_else(|| "Tüm sistem taranıyor...".to_string())
        } else if self.tarama.etkin {
            self.tarama
                .mesaj
                .clone()
                .unwrap_or_else(|| "Son tarama tamamlandı.".to_string())
        } else {
            "Henüz tarama yapılmadı.".to_string()
        };
        let dugme = Self::dokunma_hedefi(
            Button::new("tum-sistemi-tara")
                .icon(IconName::RefreshCw)
                .label(if suruyor {
                    "Taranıyor..."
                } else {
                    "Tüm Sistemi Tara"
                })
                .loading(suruyor)
                .primary()
                .px_4()
                .on_click(cx.listener(|this, _, _, cx| this.tum_tarama_baslat(cx))),
        );
        let mut kart = v_flex()
            .p_4()
            .gap_3()
            .rounded_lg()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().colors.list)
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        Icon::new(IconName::RefreshCw)
                            .with_size(gpui_kit::component::Size::Large)
                            .text_color(cx.theme().primary),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_1()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Sistem Taraması"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Kurallar, uygulamalar, güncellemeler, servisler, donanım, temizlik ve bakım kayıtları tek tıkla taranır."),
                            ),
                    )
                    .child(dugme),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(durum),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(ozet),
            );
        if let Some(hata) = &self.tarama.hata {
            kart = kart.child(self.hata_kutusu(hata, cx));
        }
        kart.into_any_element()
    }

    fn tercih_karti(&self, cx: &mut Context<Self>) -> AnyElement {
        let ice = Self::dokunma_hedefi(
            Button::new("tercih-ice-ayar")
                .icon(IconName::ArrowDown)
                .label("İçe Aktar")
                .px_4()
                .on_click(cx.listener(|this, _, _, cx| this.tercihleri_ice_aktar(cx))),
        );
        let disa = Self::dokunma_hedefi(
            Button::new("tercih-disa-ayar")
                .icon(IconName::ArrowUp)
                .label("Dışa Aktar")
                .px_4()
                .on_click(cx.listener(|this, _, _, cx| this.tercihleri_disa_aktar(cx))),
        );
        self.bolum_karti(
            "Tercihler".to_string(),
            "Tema bu bölümden seçilir; bu dosya kalan tüm WinNexus tercihlerini taşır.".to_string(),
            v_flex()
                .gap_3()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .child("Tema"),
                        )
                        .child(self.tema_secici(cx)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("Tercih dosyası; tema, winget onayları, ADB yolu ve paket eşlemelerini JSON olarak saklar."),
                )
                .child(h_flex().gap_2().child(ice).child(disa))
                .into_any_element(),
            cx,
        )
    }

    fn ayarlar_sayfasi(&self, cx: &mut Context<Self>) -> AnyElement {
        v_flex()
            .gap_4()
            .child(self.tarama_karti(cx))
            .child(self.tercih_karti(cx))
            .child(
                self.bolum_karti(
                    "Veri ve Sürüm".to_string(),
                    "Ayarların, günlüklerin ve yedeklerin tutulduğu yerel konumlar.".to_string(),
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "Ayarlar: {}",
                                    self.dizinler.ayarlar_dosyasi().display()
                                )),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("Veri dizini: {}", self.dizinler.veri.display())),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("Günlük: {}", self.gunluk.dosya().display())),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("Sürüm: {}", surum_etiketi())),
                        )
                        .into_any_element(),
                    cx,
                ),
            )
            .child(
                self.bolum_karti(
                    "Yasal Uyarı".to_string(),
                    "Uygulamanın kullanım koşulları, veri kaybı riski ve sorumluluk sınırları."
                        .to_string(),
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!(
                                    "Kabul durumu: {}",
                                    if self.ayarlar.yasal_uyari_kabul_edildi {
                                        "Onaylandı"
                                    } else {
                                        "Onaylanmadı"
                                    }
                                )),
                        )
                        .child(
                            div()
                                .max_h(px(320.))
                                .overflow_y_scrollbar()
                                .p_3()
                                .rounded_md()
                                .border_1()
                                .border_color(cx.theme().border)
                                .child(
                                    v_flex().gap_1().children(
                                        YASAL_UYARI
                                            .lines()
                                            .filter(|satir| !satir.trim().is_empty())
                                            .map(|satir| {
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(satir.to_string())
                                            }),
                                    ),
                                ),
                        )
                        .into_any_element(),
                    cx,
                ),
            )
            .into_any_element()
    }

    fn icerik(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let mut icerik = v_flex().gap_4().w_full();
        if let Some(hata) = &self.hata {
            icerik = icerik.child(self.hata_kutusu(hata, cx));
        }
        if let Some(mesaj) = &self.mesaj {
            icerik = icerik.child(self.mesaj_kutusu(mesaj, cx));
        }
        let sayfa = match self.sayfa {
            Sayfa::GenelBakis => self.genel_bakis(cx),
            Sayfa::Donanim => self.donanim_sayfasi(cx),
            Sayfa::Gecmis => self.gecmis(cx),
            Sayfa::InceAyarlar => self.ince_ayarlar(cx),
            Sayfa::SistemYonetimi => self.sistem_yonetimi_sayfasi(cx),
            Sayfa::BaglantiYoneticisi => self.arac_panosu(self.merkez_baglanti_paneli(cx), cx),
            Sayfa::Temizlik => self.temizlik_sayfasi(cx),
            Sayfa::DiskAnalisti => self.arac_panosu(self.merkez_disk_paneli(cx), cx),
            Sayfa::Bakim => self.bakim_sayfasi(cx),
            Sayfa::WindowsGuncelleme => self.arac_panosu(self.merkez_guncelleme_paneli(cx), cx),
            Sayfa::SorunGiderme => self.sorun_sayfasi(cx),
            Sayfa::Kaldirici => self.kaldirici_sayfasi(window, cx),
            Sayfa::Winget => self.winget_sayfasi(window, cx),
            Sayfa::ZorlaKaldir => self.arac_panosu(self.merkez_zorla_paneli(cx), cx),
            Sayfa::KurulumIzleme => self.arac_panosu(self.merkez_izleme_paneli(cx), cx),
            Sayfa::TarayiciUzantilari => self.arac_panosu(self.merkez_uzanti_paneli(cx), cx),
            Sayfa::Araclar => self.arac_kutusu(cx),
            Sayfa::DosyaKayitBulucu => {
                let panel = self.merkez_bulucu_paneli(window, cx);
                self.arac_panosu(panel, cx)
            }
            Sayfa::GodMode => self.god_mode_sayfasi(cx),
            Sayfa::Android => self.android_sayfasi(window, cx),
            Sayfa::Ayarlar => self.ayarlar_sayfasi(cx),
        };
        icerik.child(sayfa).into_any_element()
    }
}

impl Render for WinNexusApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.baslik_cubugu_guncelle(window, cx);
        if !self.ayarlar.yasal_uyari_kabul_edildi {
            return self.yasal_ekrani(cx).into_any_element();
        }
        let arka = cx.theme().background;
        let on = cx.theme().foreground;
        let icerik = self.icerik(window, cx);
        let baslik = self.baslik_cubugu(cx);
        let govde = v_flex()
            .flex_1()
            .min_h_0()
            .overflow_y_scrollbar()
            .p_4()
            .child(v_flex().w_full().gap_4().child(icerik));
        let sag = v_flex()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .child(baslik)
            .child(govde);
        let alan = if let Some(bolum) = self.sayfa.bolum() {
            h_flex()
                .flex_1()
                .min_h_0()
                .w_full()
                .items_stretch()
                .child(self.yan_menu(bolum, cx))
                .child(sag)
                .into_any_element()
        } else {
            sag.into_any_element()
        };
        v_flex()
            .size_full()
            .bg(arka)
            .text_color(on)
            .child(self.ust_gezinme(cx))
            .child(alan)
            .into_any_element()
    }
}

fn is_parcasi<T>(sonuc: std::thread::Result<sp_core::Sonuc<T>>, ad: &str) -> sp_core::Sonuc<T> {
    sonuc.unwrap_or_else(|_| {
        Err(sp_core::Hata::Beklenmeyen(format!(
            "{ad} taraması beklenmedik şekilde durdu."
        )))
    })
}
