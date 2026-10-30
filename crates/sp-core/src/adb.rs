use crate::error::{Hata, Sonuc};
use regex::Regex;
use std::sync::OnceLock;

fn paket_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Za-z0-9][A-Za-z0-9._]{0,127}$").expect("sabit regex"))
}

pub fn paket_gecerli(id: &str) -> bool {
    paket_regex().is_match(id)
}

pub fn paket_dogrula(id: &str) -> Sonuc<()> {
    if paket_gecerli(id) {
        Ok(())
    } else {
        Err(Hata::Beklenmeyen(format!(
            "Geçersiz Android paket kimliği: {id}"
        )))
    }
}

pub fn seri_gecerli(seri: &str) -> bool {
    !seri.is_empty()
        && seri.len() <= 128
        && seri.chars().all(|karakter| {
            karakter.is_ascii_alphanumeric() || matches!(karakter, '.' | '_' | '-' | ':')
        })
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn paket_kimligi_dogrulanir() {
        assert!(paket_gecerli("com.miui.analytics"));
        assert!(paket_gecerli("com.google.android.youtube"));
        assert!(!paket_gecerli("com.miui;rm -rf"));
        assert!(!paket_gecerli("kötü paket"));
        assert!(!paket_gecerli(""));
    }

    #[test]
    fn seri_dogrulanir() {
        assert!(seri_gecerli("192.168.1.5:5555"));
        assert!(seri_gecerli("ABC123DEF"));
        assert!(!seri_gecerli("kötü seri"));
        assert!(!seri_gecerli(""));
    }
}
