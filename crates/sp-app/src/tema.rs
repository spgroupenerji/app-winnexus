use gpui_kit::component::{Theme, ThemeConfig, ThemeMode};
use gpui_kit::App;
use sp_core::ayarlar::Tema;
use std::rc::Rc;

const GUNDUZ_TEMA: &str = r##"{
  "name": "WinNexus Gündüz",
  "mode": "light",
  "is_default": false,
  "radius": 8,
  "radius.lg": 12,
  "shadow": true,
  "colors": {
    "background": "#F7F3EC",
    "foreground": "#2F3631",
    "popover.background": "#FFFDF8",
    "popover.foreground": "#2F3631",
    "muted.background": "#EDE6DB",
    "muted.foreground": "#5E6A60",
    "border": "#948B7C",
    "input.border": "#8F8576",
    "ring": "#3F6B57",
    "caret": "#2F3631",
    "primary.background": "#3F6B57",
    "primary.hover.background": "#355B4A",
    "primary.active.background": "#2C4C3E",
    "primary.foreground": "#F7FBF8",
    "secondary.background": "#E7E0D3",
    "secondary.hover.background": "#DED6C7",
    "secondary.active.background": "#D5CCBB",
    "secondary.foreground": "#3C423C",
    "accent.background": "#E4DBF0",
    "accent.foreground": "#493D5E",
    "danger.background": "#8F3B33",
    "danger.hover.background": "#98423A",
    "danger.active.background": "#7E332C",
    "danger.foreground": "#FFF8F6",
    "success.background": "#36735A",
    "success.hover.background": "#2F6650",
    "success.active.background": "#295B47",
    "success.foreground": "#F5FBF7",
    "warning.background": "#8A5C1E",
    "warning.hover.background": "#7C521A",
    "warning.active.background": "#6F4917",
    "warning.foreground": "#FFF9EE",
    "info.background": "#356985",
    "info.hover.background": "#2F5D76",
    "info.active.background": "#295268",
    "info.foreground": "#F5FBFF",
    "button.background": "#EDE6DB",
    "button.hover.background": "#E3DACB",
    "button.active.background": "#DAD0BF",
    "button.foreground": "#2F3631",
    "list.background": "#FFFDF8",
    "list.hover.background": "#F2ECE1",
    "list.active.background": "#E1EDE4",
    "list.active.border": "#5E8C74",
    "list.head.background": "#F2ECE1",
    "list.even.background": "#FBF8F2",
    "table.background": "#FFFDF8",
    "table.hover.background": "#F2ECE1",
    "table.active.background": "#E1EDE4",
    "table.active.border": "#5E8C74",
    "table.head.background": "#F2ECE1",
    "table.head.foreground": "#5E6A60",
    "table.even.background": "#FBF8F2",
    "table.row.border": "#E3DCCF",
    "sidebar.background": "#F1EBE1",
    "sidebar.foreground": "#384038",
    "sidebar.border": "#DAD1C2",
    "sidebar.accent.background": "#E3EDE6",
    "sidebar.accent.foreground": "#2F3631",
    "sidebar.primary.background": "#3F6B57",
    "sidebar.primary.foreground": "#F7FBF8",
    "title_bar.background": "#F1EBE1",
    "title_bar.border": "#DAD1C2",
    "status_bar.background": "#F1EBE1",
    "status_bar.border": "#DAD1C2",
    "tab_bar.background": "#F1EBE1",
    "tab_bar.segmented.background": "#E8E1D4",
    "tab.background": "#00000000",
    "tab.active.background": "#FFFDF8",
    "tab.active.foreground": "#2F3631",
    "tab.foreground": "#49544B",
    "switch.background": "#C9BFAE",
    "switch.thumb.background": "#FFFDF8",
    "skeleton.background": "#EDE6DB",
    "selection.background": "#C7DAEC",
    "scrollbar.background": "#F7F3EC00",
    "scrollbar.thumb.background": "#B3A897E6",
    "scrollbar.thumb.hover.background": "#A2977F",
    "group_box.background": "#F2ECE1",
    "group_box.foreground": "#2F3631",
    "description_list.label.background": "#F2ECE1",
    "description_list.label.foreground": "#3C423C",
    "accordion.background": "#F2ECE1",
    "progress.bar.background": "#3F6B57",
    "slider.background": "#3F6B57",
    "slider.thumb.background": "#FFFDF8",
    "overlay": "#2F36310D",
    "window.border": "#DAD1C2",
    "drag.border": "#3F6B57",
    "drop_target.background": "#E1EDE4",
    "link": "#3F6B57",
    "link.active": "#2C4C3E",
    "link.hover": "#355B4A",
    "chart.1": "#7FA8C9",
    "chart.2": "#8FBF9F",
    "chart.3": "#C9A66B",
    "chart.4": "#B18FB8",
    "chart.5": "#D38B7E",
    "chart.grid": "#E3DCCF",
    "chart.bullish": "#36735A",
    "chart.bearish": "#A34E43",
    "base.red": "#B4534B",
    "base.red.light": "#D98C84",
    "base.green": "#4E8A63",
    "base.green.light": "#8FBF9F",
    "base.blue": "#4A7FA0",
    "base.blue.light": "#8FB6CD",
    "base.yellow": "#B8892E",
    "base.yellow.light": "#D8B775",
    "base.magenta": "#8E6B9E",
    "base.magenta.light": "#B99CC6",
    "base.cyan": "#4E8A8A",
    "base.cyan.light": "#8FB8B8"
  }
}"##;

const GECE_TEMA: &str = r##"{
  "name": "WinNexus Gece",
  "mode": "dark",
  "is_default": false,
  "radius": 8,
  "radius.lg": 12,
  "shadow": true,
  "colors": {
    "background": "#1D2320",
    "foreground": "#EDE8DE",
    "popover.background": "#242B27",
    "popover.foreground": "#EDE8DE",
    "muted.background": "#29312C",
    "muted.foreground": "#A9B3AA",
    "border": "#6D796F",
    "input.border": "#7A867C",
    "ring": "#9CCBB2",
    "caret": "#EDE8DE",
    "primary.background": "#9CCBB2",
    "primary.hover.background": "#ADD8C1",
    "primary.active.background": "#8ABFA5",
    "primary.foreground": "#14281D",
    "secondary.background": "#323B35",
    "secondary.hover.background": "#3A443D",
    "secondary.active.background": "#424D46",
    "secondary.foreground": "#E4EAE3",
    "accent.background": "#3B3447",
    "accent.foreground": "#E8DDF4",
    "danger.background": "#E3A199",
    "danger.hover.background": "#ECB0A8",
    "danger.active.background": "#D69189",
    "danger.foreground": "#2A1412",
    "success.background": "#84C29B",
    "success.hover.background": "#90CBA6",
    "success.active.background": "#76B58D",
    "success.foreground": "#10241A",
    "warning.background": "#D8B775",
    "warning.hover.background": "#E1C285",
    "warning.active.background": "#CDA95F",
    "warning.foreground": "#2A2110",
    "info.background": "#8FBBD4",
    "info.hover.background": "#9CC5DB",
    "info.active.background": "#82AFCA",
    "info.foreground": "#0E1E28",
    "button.background": "#2C342F",
    "button.hover.background": "#333C36",
    "button.active.background": "#3A443D",
    "button.foreground": "#EDE8DE",
    "list.background": "#242B27",
    "list.hover.background": "#2C342F",
    "list.active.background": "#33453A",
    "list.active.border": "#7FA98F",
    "list.head.background": "#2C342F",
    "list.even.background": "#262E29",
    "table.background": "#242B27",
    "table.hover.background": "#2C342F",
    "table.active.background": "#33453A",
    "table.active.border": "#7FA98F",
    "table.head.background": "#2C342F",
    "table.head.foreground": "#A9B3AA",
    "table.even.background": "#262E29",
    "table.row.border": "#39433C",
    "sidebar.background": "#212824",
    "sidebar.foreground": "#DDE5DD",
    "sidebar.border": "#39433C",
    "sidebar.accent.background": "#2F3B34",
    "sidebar.accent.foreground": "#EDE8DE",
    "sidebar.primary.background": "#9CCBB2",
    "sidebar.primary.foreground": "#14281D",
    "title_bar.background": "#212824",
    "title_bar.border": "#39433C",
    "status_bar.background": "#212824",
    "status_bar.border": "#39433C",
    "tab_bar.background": "#212824",
    "tab_bar.segmented.background": "#2A332D",
    "tab.background": "#00000000",
    "tab.active.background": "#2C342F",
    "tab.active.foreground": "#EDE8DE",
    "tab.foreground": "#C9D2CA",
    "switch.background": "#4D5A50",
    "switch.thumb.background": "#EDE8DE",
    "skeleton.background": "#29312C",
    "selection.background": "#3B5568",
    "scrollbar.background": "#1D232000",
    "scrollbar.thumb.background": "#5A665DE6",
    "scrollbar.thumb.hover.background": "#6D796F",
    "group_box.background": "#29312C",
    "group_box.foreground": "#EDE8DE",
    "description_list.label.background": "#29312C",
    "description_list.label.foreground": "#DDE5DD",
    "accordion.background": "#29312C",
    "progress.bar.background": "#9CCBB2",
    "slider.background": "#9CCBB2",
    "slider.thumb.background": "#EDE8DE",
    "overlay": "#0B0F0D66",
    "window.border": "#39433C",
    "drag.border": "#9CCBB2",
    "drop_target.background": "#33453A",
    "link": "#9CCBB2",
    "link.active": "#ADD8C1",
    "link.hover": "#B7DCC6",
    "chart.1": "#8FB6CD",
    "chart.2": "#9CCBB2",
    "chart.3": "#D8B775",
    "chart.4": "#B99CC6",
    "chart.5": "#D99087",
    "chart.grid": "#39433C",
    "chart.bullish": "#84C29B",
    "chart.bearish": "#D99087",
    "base.red": "#D99087",
    "base.red.light": "#E8ADA6",
    "base.green": "#84C29B",
    "base.green.light": "#A5D4B7",
    "base.blue": "#8FBBD4",
    "base.blue.light": "#ADCDE0",
    "base.yellow": "#D8B775",
    "base.yellow.light": "#E5CD98",
    "base.magenta": "#B99CC6",
    "base.magenta.light": "#CDB8D6",
    "base.cyan": "#8FB8B8",
    "base.cyan.light": "#ACD0D0"
  }
}"##;

fn tema_yapilandirmasi_coz(ham: &str, ad: &str) -> Option<ThemeConfig> {
    match serde_json::from_str::<ThemeConfig>(ham) {
        Ok(yapilandirma) => Some(yapilandirma),
        Err(hata) => {
            tracing::error!("{ad} teması çözümlenemedi: {hata}");
            None
        }
    }
}

pub fn tema_ayarla(cx: &mut App, tema: Tema) {
    let (Some(gunduz), Some(gece)) = (
        tema_yapilandirmasi_coz(GUNDUZ_TEMA, "Gündüz"),
        tema_yapilandirmasi_coz(GECE_TEMA, "Gece"),
    ) else {
        return;
    };
    let mod_ = match tema {
        Tema::Sistem => ThemeMode::from(cx.window_appearance()),
        Tema::Acik => ThemeMode::Light,
        Tema::Koyu => ThemeMode::Dark,
    };
    let gunduz = Rc::new(gunduz);
    let gece = Rc::new(gece);
    let secilen = if mod_ == ThemeMode::Dark {
        gece.clone()
    } else {
        gunduz.clone()
    };
    Theme::update(cx, |tema_nesnesi| {
        tema_nesnesi.light_theme = gunduz;
        tema_nesnesi.dark_theme = gece;
        tema_nesnesi.apply_config(&secilen);
    });
}

#[cfg(test)]
mod testler {
    use super::{GECE_TEMA, GUNDUZ_TEMA};
    use gpui_kit::component::{ThemeConfig, ThemeMode};
    use gpui_kit::SharedString;

    fn tema_coz(ham: &str) -> ThemeConfig {
        serde_json::from_str(ham).expect("sabit tema JSON'u çözümlenmeli")
    }

    fn metin(deger: &SharedString) -> &str {
        <SharedString as AsRef<str>>::as_ref(deger)
    }

    fn renk(deger: &Option<SharedString>) -> &str {
        let ham = deger.as_ref().expect("sabit tema rengi tanımlı olmalı");
        <SharedString as AsRef<str>>::as_ref(ham)
    }

    fn kanal_parlakligi(kanal: u8) -> f64 {
        let oran = f64::from(kanal) / 255.0;
        if oran <= 0.04045 {
            oran / 12.92
        } else {
            ((oran + 0.055) / 1.055).powf(2.4)
        }
    }

    fn parlaklik(altigen: &str) -> f64 {
        let ham = altigen.trim_start_matches('#');
        let kanal = |baslangic: usize| {
            u8::from_str_radix(&ham[baslangic..baslangic + 2], 16)
                .expect("sabit renk geçerli olmalı")
        };
        0.2126 * kanal_parlakligi(kanal(0))
            + 0.7152 * kanal_parlakligi(kanal(2))
            + 0.0722 * kanal_parlakligi(kanal(4))
    }

    fn kontrast(ust: &str, alt: &str) -> f64 {
        let ust_parlakligi = parlaklik(ust);
        let alt_parlakligi = parlaklik(alt);
        let (acik, koyu) = if ust_parlakligi >= alt_parlakligi {
            (ust_parlakligi, alt_parlakligi)
        } else {
            (alt_parlakligi, ust_parlakligi)
        };
        (acik + 0.05) / (koyu + 0.05)
    }

    #[test]
    fn gunduz_temasi_cozulur() {
        let tema = tema_coz(GUNDUZ_TEMA);
        assert_eq!(metin(&tema.name), "WinNexus Gündüz");
        assert_eq!(tema.mode, ThemeMode::Light);
        assert_eq!(tema.radius, Some(8));
        assert_eq!(tema.radius_lg, Some(12));
        assert_eq!(tema.shadow, Some(true));
        assert_eq!(renk(&tema.colors.background), "#F7F3EC");
        assert_eq!(renk(&tema.colors.foreground), "#2F3631");
        assert_eq!(renk(&tema.colors.primary), "#3F6B57");
        assert_eq!(renk(&tema.colors.accent), "#E4DBF0");
    }

    #[test]
    fn gece_temasi_cozulur() {
        let tema = tema_coz(GECE_TEMA);
        assert_eq!(metin(&tema.name), "WinNexus Gece");
        assert_eq!(tema.mode, ThemeMode::Dark);
        assert_eq!(tema.radius, Some(8));
        assert_eq!(tema.radius_lg, Some(12));
        assert_eq!(tema.shadow, Some(true));
        assert_eq!(renk(&tema.colors.background), "#1D2320");
        assert_eq!(renk(&tema.colors.foreground), "#EDE8DE");
        assert_eq!(renk(&tema.colors.primary), "#9CCBB2");
        assert_eq!(renk(&tema.colors.accent), "#3B3447");
    }

    fn karisim(ust: &str, alt: &str, oran: f64) -> String {
        let ust_ham = ust.trim_start_matches('#');
        let alt_ham = alt.trim_start_matches('#');
        let kanal = |baslangic: usize| {
            f64::from(
                u8::from_str_radix(&ust_ham[baslangic..baslangic + 2], 16)
                    .expect("sabit renk geçerli olmalı"),
            ) * oran
                + f64::from(
                    u8::from_str_radix(&alt_ham[baslangic..baslangic + 2], 16)
                        .expect("sabit renk geçerli olmalı"),
                ) * (1.0 - oran)
        };
        format!(
            "#{:02X}{:02X}{:02X}",
            kanal(0).round() as u8,
            kanal(2).round() as u8,
            kanal(4).round() as u8
        )
    }

    #[test]
    fn pastel_temalar_kontrast_esiklerini_karsilar() {
        for (ad, ham) in [("gündüz", GUNDUZ_TEMA), ("gece", GECE_TEMA)] {
            let tema = tema_coz(ham);
            let arkaplan = renk(&tema.colors.background);
            assert!(
                kontrast(renk(&tema.colors.foreground), arkaplan) >= 7.0,
                "{ad}: metin/arkaplan kontrastı yetersiz"
            );
            assert!(
                kontrast(renk(&tema.colors.border), arkaplan) >= 3.0,
                "{ad}: kenar/arkaplan kontrastı yetersiz"
            );
            assert!(
                kontrast(renk(&tema.colors.muted_foreground), arkaplan) >= 4.5,
                "{ad}: soluk metin/arkaplan kontrastı yetersiz"
            );
            assert!(
                kontrast(
                    renk(&tema.colors.primary_foreground),
                    renk(&tema.colors.primary)
                ) >= 4.5,
                "{ad}: birincil metin/birincil kontrastı yetersiz"
            );
            let tehlike = renk(&tema.colors.danger);
            let yumusak_zemin = karisim(tehlike, arkaplan, 0.2);
            assert!(
                kontrast(tehlike, &yumusak_zemin) >= 4.5,
                "{ad}: yumuşak tehlike metni/zemin kontrastı yetersiz"
            );
            assert!(
                kontrast(renk(&tema.colors.danger_foreground), tehlike) >= 4.5,
                "{ad}: tehlike metni/zemini kontrastı yetersiz"
            );
        }
    }
}
