#![windows_subsystem = "windows"]

mod app;
mod tema;

use app::WinNexusApp;
use gpui_kit::*;
use sp_core::ayarlar::Ayarlar;
use sp_core::dizinler::Dizinler;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) const ICON_PNG: &[u8] = include_bytes!("../../../assets/winnexus-icon.png");

pub(crate) const SURUM: &str = match option_env!("WINNEXUS_BUILD_STAMP") {
    Some(damga) => damga,
    None => env!("CARGO_PKG_VERSION"),
};

fn main() {
    if let Some(bayrak) = std::env::args().nth(1) {
        if bayrak == "--sahipligi-al" {
            match std::env::args().nth(2) {
                Some(yol) => match sp_win::sahiplik::sahipligi_al(std::path::Path::new(&yol)) {
                    Ok(()) => std::process::exit(0),
                    Err(hata) => {
                        sp_win::sahiplik::hata_kaydet(&hata.to_string());
                        std::process::exit(1);
                    }
                },
                None => {
                    sp_win::sahiplik::hata_kaydet("Sahipliği Al: yol verilmedi.");
                    std::process::exit(2);
                }
            }
        }
    }

    let dizinler = Dizinler::varsayilan();
    let log_yolu = dizinler
        .as_ref()
        .map(|dizinler| dizinler.log_dizini().join("winnexus.log"))
        .unwrap_or_else(|_| PathBuf::from("winnexus.log"));
    if let Some(anne) = log_yolu.parent() {
        let _ = std::fs::create_dir_all(anne);
    }
    if let Ok(dosya) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_yolu)
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with_writer(move || dosya.try_clone().expect("günlük dosyası açılamadı"))
            .with_ansi(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .with_ansi(false)
            .init();
    }

    let panic_yolu = dizinler
        .as_ref()
        .map(|dizinler| dizinler.log_dizini().join("panic.log"))
        .unwrap_or_else(|_| PathBuf::from("panic.log"));
    if let Some(anne) = panic_yolu.parent() {
        let _ = std::fs::create_dir_all(anne);
    }
    std::panic::set_hook(Box::new(move |bilgi| {
        use std::io::Write as _;
        if let Ok(mut dosya) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&panic_yolu)
        {
            let geri_iz = std::backtrace::Backtrace::force_capture();
            let _ = writeln!(dosya, "{bilgi}\n{geri_iz}");
        }
    }));

    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            if sp_win::animasyonlar_kapali_mi() {
                cx.set_reduce_motion(true);
            }

            let tema = dizinler
                .as_ref()
                .ok()
                .and_then(|dizinler| Ayarlar::yukle(&dizinler.ayarlar_dosyasi()).ok())
                .map(|ayarlar| ayarlar.tema)
                .unwrap_or_default();
            tema::tema_ayarla(cx, tema);

            let ikon = image::load_from_memory(ICON_PNG)
                .ok()
                .map(|goruntu| Arc::new(goruntu.to_rgba8()));
            let pencere = WindowOptions {
                icon: ikon,
                window_bounds: Some(WindowBounds::Maximized(Bounds::new(
                    point(px(100.), px(100.)),
                    size(px(1280.), px(860.)),
                ))),
                window_min_size: Some(size(px(960.), px(640.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("SP GROUP WinNexus".into()),
                    ..Default::default()
                }),
                ..Default::default()
            };

            gpui_kit::open_window(pencere, cx, |_, cx| cx.new(WinNexusApp::new))
                .expect("Pencere açılamadı");
        });
}
