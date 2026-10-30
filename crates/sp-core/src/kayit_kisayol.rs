use crate::error::{Hata, Sonuc};
use serde_json::Value;
use std::path::Path;

pub const KATEGORI_KALDIRMA: &str = "Kaldırma kaydı";
pub const KATEGORI_UYGULAMA_YOLU: &str = "Uygulama yolu";
pub const KATEGORI_PAYLASILAN_DLL: &str = "Paylaşılan DLL";
pub const KATEGORI_YAZI_TIPI: &str = "Yazı tipi kaydı";
pub const KATEGORI_SES_OLAYI: &str = "Ses olayı kaydı";
pub const KATEGORI_MUI: &str = "MUI önbelleği";
pub const KATEGORI_YARDIM: &str = "Yardım dosyası kaydı";

const CLSID_UZUNLUKLARI: [usize; 5] = [8, 4, 4, 4, 12];
const YASAKLI_CALISTIRICILAR: [&str; 5] = ["msiexec", "rundll32", "cmd", "powershell", "pwsh"];
const YAZI_TIPI_UZANTILARI: [&str; 4] = [".ttf", ".otf", ".ttc", ".fon"];
const YARDIM_UZANTILARI: [&str; 2] = [".hlp", ".chm"];
const MUI_SONEKLERI: [&str; 2] = [".FriendlyAppName", ".ApplicationCompany"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KayitDefteriBulgu {
    pub kimlik: String,
    pub kategori: String,
    pub ad: String,
    pub ayrinti: String,
    pub kove: String,
    pub yol: String,
    pub deger: Option<String>,
}

impl KayitDefteriBulgu {
    pub fn kimlik_uret(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.kategori,
            self.kove,
            self.yol.to_ascii_lowercase(),
            self.deger
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KisayolBulgu {
    pub yol: String,
    pub ad: String,
    pub hedef: String,
    pub konum: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaglamOgesi {
    pub clsid: String,
    pub ad: String,
    pub kapsam: String,
    pub kaynak: String,
    pub devre_disi: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KayitTemizlikSonucu {
    pub islenen: usize,
    pub hatalar: Vec<String>,
}

pub fn clsid_gecerli_mi(deger: &str) -> bool {
    let metin = deger.trim();
    let govde = match metin.strip_prefix('{') {
        Some(kalan) => match kalan.strip_suffix('}') {
            Some(icerik) => icerik,
            None => return false,
        },
        None => metin,
    };
    let parcalar = govde.split('-').collect::<Vec<_>>();
    parcalar.len() == CLSID_UZUNLUKLARI.len()
        && parcalar
            .iter()
            .zip(CLSID_UZUNLUKLARI)
            .all(|(parca, uzunluk)| {
                parca.len() == uzunluk && parca.bytes().all(|bayt| bayt.is_ascii_hexdigit())
            })
}

pub fn uninstall_yolu_ayikla(komut: &str) -> Option<String> {
    let komut = komut.trim();
    if komut.is_empty() {
        return None;
    }
    let aday = tirnakli_aday(komut).or_else(|| tirnaksiz_aday(komut))?;
    kaldirma_yolu_uygun_mu(aday).then(|| aday.to_string())
}

fn tirnakli_aday(komut: &str) -> Option<&str> {
    let kalan = komut.strip_prefix('"')?;
    let kapanis = kalan.find('"')?;
    Some(kalan[..kapanis].trim())
}

fn tirnaksiz_aday(komut: &str) -> Option<&str> {
    let ilk = komut
        .find(char::is_whitespace)
        .map(|konum| &komut[..konum])
        .unwrap_or(komut)
        .trim();
    let son_bilesen = ilk.rsplit(['\\', '/']).next().unwrap_or_default();
    if son_bilesen.contains('.') {
        return Some(ilk);
    }
    let kucuk = komut.to_ascii_lowercase();
    let konum = kucuk.find(".exe")?;
    let sonrasi = &kucuk[konum + 4..];
    if !sonrasi.is_empty() && !sonrasi.starts_with(char::is_whitespace) {
        return Some(ilk);
    }
    Some(komut[..konum + 4].trim())
}

fn kaldirma_yolu_uygun_mu(yol: &str) -> bool {
    if yol.is_empty()
        || yol.ends_with('\\')
        || yol.ends_with('/')
        || yol.contains('%')
        || yol.contains("://")
        || yol
            .chars()
            .any(|karakter| matches!(karakter, '\0' | '\r' | '\n'))
    {
        return false;
    }
    if !mutlak_yol_mu(yol) {
        return false;
    }
    let dosya = yol
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let govde = dosya.strip_suffix(".exe").unwrap_or(&dosya);
    !YASAKLI_CALISTIRICILAR.contains(&govde)
}

fn mutlak_yol_mu(yol: &str) -> bool {
    let baytlar = yol.as_bytes();
    let surucu = baytlar.len() > 3
        && baytlar[0].is_ascii_alphabetic()
        && baytlar[1] == b':'
        && matches!(baytlar[2], b'\\' | b'/');
    surucu || (yol.len() > 2 && (yol.starts_with("\\\\") || yol.starts_with("//")))
}

pub fn kayit_yolu_guvenli_mi(yol: &str) -> bool {
    !yol.is_empty()
        && !yol.starts_with('\\')
        && !yol.ends_with('\\')
        && !yol.contains("..")
        && !metin_tehlikeli_mi(yol)
}

pub fn kayit_degeri_guvenli_mi(deger: &str) -> bool {
    !deger.is_empty()
        && !deger.starts_with('/')
        && !deger.contains("..")
        && !metin_tehlikeli_mi(deger)
}

pub fn yazi_tipi_dosya_adi_ayikla(metin: &str) -> Option<String> {
    let aday = tirnaklari_soy(metin);
    if aday.is_empty()
        || aday.contains('%')
        || aday.contains(',')
        || aday.contains(';')
        || metin_tehlikeli_mi(aday)
    {
        return None;
    }
    let kucuk = aday.to_ascii_lowercase();
    if !YAZI_TIPI_UZANTILARI
        .iter()
        .any(|uzanti| kucuk.ends_with(uzanti))
    {
        return None;
    }
    Some(aday.to_string())
}

pub fn yazi_tipi_hedef_yolu(metin: &str, font_klasoru: &str) -> Option<String> {
    let aday = yazi_tipi_dosya_adi_ayikla(metin)?;
    if mutlak_yol_mu(&aday) {
        return Some(aday);
    }
    if aday.contains('\\') || aday.contains('/') {
        return None;
    }
    klasore_ekle(font_klasoru, &aday)
}

pub fn ses_olayi_hedef_yolu(metin: &str, sistem_kok: &str) -> Option<String> {
    let ham = tirnaklari_soy(metin);
    if ham.is_empty() || metin_tehlikeli_mi(ham) {
        return None;
    }
    let hedef = ortam_degiskenlerini_genislet(ham, sistem_kok)?;
    if !hedef.to_ascii_lowercase().ends_with(".wav") || !mutlak_yol_mu(&hedef) {
        return None;
    }
    Some(hedef)
}

pub fn mui_uygulama_yolu_ayikla(ad: &str) -> Option<String> {
    let ad = ad.trim();
    if ad.is_empty() || metin_tehlikeli_mi(ad) {
        return None;
    }
    let govde = MUI_SONEKLERI
        .iter()
        .find_map(|sonek| sonek_ayikla(ad, sonek))?
        .trim();
    let kucuk = govde.to_ascii_lowercase();
    if !kucuk.starts_with("c:\\") && !govde.starts_with("\\\\") {
        return None;
    }
    if !kucuk.ends_with(".exe") || govde.split(['\\', '/']).any(|parca| parca == "..") {
        return None;
    }
    Some(govde.to_string())
}

pub fn yardim_dosyasi_hedef_yolu(metin: &str, yardim_klasoru: &str) -> Option<String> {
    let aday = tirnaklari_soy(metin);
    if aday.is_empty() || aday.contains('%') || metin_tehlikeli_mi(aday) {
        return None;
    }
    let kucuk = aday.to_ascii_lowercase();
    if !YARDIM_UZANTILARI
        .iter()
        .any(|uzanti| kucuk.ends_with(uzanti))
    {
        return None;
    }
    if mutlak_yol_mu(aday) {
        return Some(aday.to_string());
    }
    if aday.contains('\\') || aday.contains('/') {
        return None;
    }
    klasore_ekle(yardim_klasoru, aday)
}

fn metin_tehlikeli_mi(metin: &str) -> bool {
    metin
        .chars()
        .any(|karakter| matches!(karakter, '"' | '\r' | '\n' | '\0'))
}

fn tirnaklari_soy(metin: &str) -> &str {
    let metin = metin.trim();
    match metin
        .strip_prefix('"')
        .and_then(|kalan| kalan.strip_suffix('"'))
    {
        Some(icerik) => icerik.trim(),
        None => metin,
    }
}

fn klasore_ekle(klasor: &str, ad: &str) -> Option<String> {
    let klasor = klasor.trim().trim_end_matches(['\\', '/']);
    if klasor.is_empty() || ad.is_empty() {
        return None;
    }
    Some(format!("{klasor}\\{ad}"))
}

fn sonek_ayikla<'a>(metin: &'a str, sonek: &str) -> Option<&'a str> {
    let bas = metin.len().checked_sub(sonek.len())?;
    let kuyruk = metin.get(bas..)?;
    kuyruk.eq_ignore_ascii_case(sonek).then(|| &metin[..bas])
}

fn ortam_degiskenlerini_genislet(metin: &str, sistem_kok: &str) -> Option<String> {
    let kok = sistem_kok.trim().trim_end_matches(['\\', '/']);
    if kok.is_empty() {
        return None;
    }
    let mut sonuc = String::new();
    let mut kalan = metin;
    while let Some(bas) = kalan.find('%') {
        sonuc.push_str(&kalan[..bas]);
        let sonra = &kalan[bas + 1..];
        let son = sonra.find('%')?;
        let degisken = &sonra[..son];
        if !degisken.eq_ignore_ascii_case("SystemRoot") && !degisken.eq_ignore_ascii_case("WINDIR")
        {
            return None;
        }
        sonuc.push_str(kok);
        kalan = &sonra[son + 1..];
    }
    sonuc.push_str(kalan);
    Some(sonuc)
}

pub fn kisayol_json_ayristir(json: &str) -> Sonuc<Vec<KisayolBulgu>> {
    let metin = json.trim().trim_start_matches('\u{feff}').trim();
    if metin.is_empty() {
        return Ok(Vec::new());
    }
    let deger: Value = serde_json::from_str(metin)
        .map_err(|hata| Hata::Beklenmeyen(format!("Kısayol listesi ayrıştırılamadı: {hata}")))?;
    let satirlar = match &deger {
        Value::Array(satirlar) => satirlar.iter().collect::<Vec<_>>(),
        Value::Object(_) => vec![&deger],
        Value::Null => Vec::new(),
        _ => {
            return Err(Hata::Beklenmeyen(
                "Kısayol listesi beklenen biçimde değil.".into(),
            ))
        }
    };
    Ok(satirlar
        .into_iter()
        .filter_map(kisayol_oku)
        .collect::<Vec<_>>())
}

fn kisayol_oku(satir: &Value) -> Option<KisayolBulgu> {
    let nesne = satir.as_object()?;
    let alan = |ad: &str| {
        nesne
            .iter()
            .find(|(anahtar, _)| anahtar.eq_ignore_ascii_case(ad))
            .and_then(|(_, deger)| deger.as_str())
            .map(str::trim)
    };
    let yol = alan("Yol").filter(|yol| !yol.is_empty())?;
    let hedef = alan("Hedef").filter(|hedef| !hedef.is_empty())?;
    let ad = alan("Ad")
        .filter(|ad| !ad.is_empty())
        .map(str::to_string)
        .or_else(|| {
            Path::new(yol)
                .file_stem()
                .map(|govde| govde.to_string_lossy().into_owned())
        })
        .unwrap_or_default();
    Some(KisayolBulgu {
        yol: yol.to_string(),
        ad,
        hedef: hedef.to_string(),
        konum: alan("Konum").unwrap_or_default().to_string(),
    })
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn tirnakli_kaldirma_yolu_ayiklanir() {
        assert_eq!(
            uninstall_yolu_ayikla(r#""C:\Program Files\Uygulama\unins000.exe" /S"#),
            Some(r"C:\Program Files\Uygulama\unins000.exe".into())
        );
    }

    #[test]
    fn tirnaksiz_kaldirma_yolu_ayiklanir() {
        assert_eq!(
            uninstall_yolu_ayikla(r"C:\Uygulama\unins.exe /S"),
            Some(r"C:\Uygulama\unins.exe".into())
        );
    }

    #[test]
    fn bosluklu_tirnaksiz_yol_ayiklanir() {
        assert_eq!(
            uninstall_yolu_ayikla(r"C:\Program Files\Uygulama\unins.exe /S"),
            Some(r"C:\Program Files\Uygulama\unins.exe".into())
        );
    }

    #[test]
    fn msi_ve_sistem_araclari_reddedilir() {
        assert_eq!(
            uninstall_yolu_ayikla(r"MsiExec.exe /X{12345678-1234-1234-1234-123456789012}"),
            None
        );
        assert_eq!(
            uninstall_yolu_ayikla(r"C:\Windows\System32\msiexec.exe /X{1}"),
            None
        );
        assert_eq!(
            uninstall_yolu_ayikla(r"rundll32.exe setupapi.dll,InstallHinfSection"),
            None
        );
        assert_eq!(
            uninstall_yolu_ayikla(r"C:\Windows\System32\cmd.exe /C unins.bat"),
            None
        );
        assert_eq!(
            uninstall_yolu_ayikla(
                r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -File x.ps1"
            ),
            None
        );
    }

    #[test]
    fn goreli_ortam_ve_adres_yollari_reddedilir() {
        assert_eq!(uninstall_yolu_ayikla(r"unins.exe /S"), None);
        assert_eq!(uninstall_yolu_ayikla(r"%APPDATA%\Uygulama\unins.exe"), None);
        assert_eq!(uninstall_yolu_ayikla("https://ornek.test/unins.exe"), None);
        assert_eq!(uninstall_yolu_ayikla(r"\Uygulama\unins.exe"), None);
        assert_eq!(uninstall_yolu_ayikla(r"C:\Uygulama\"), None);
        assert_eq!(uninstall_yolu_ayikla(""), None);
    }

    #[test]
    fn clsid_bicimleri_dogrulanir() {
        assert!(clsid_gecerli_mi("{86CA1AA0-34AA-4E8B-A509-50C905BAE2A2}"));
        assert!(clsid_gecerli_mi("86ca1aa0-34aa-4e8b-a509-50c905bae2a2"));
        assert!(!clsid_gecerli_mi("{86CA1AA0-34AA-4E8B-A509}"));
        assert!(!clsid_gecerli_mi("{86CA1AA0-34AA-4E8B-A509-50C905BAE2AZ}"));
        assert!(!clsid_gecerli_mi("{86CA1AA0-34AA-4E8B-A509-50C905BAE2A2"));
        assert!(!clsid_gecerli_mi("..\\CLSID"));
    }

    #[test]
    fn kisayol_json_dizisi_ayristirilir() {
        let json = r#"[
            {"Yol":"C:\\Users\\A\\Desktop\\a.lnk","Ad":"a","Hedef":"C:\\App\\a.exe","Konum":"Masaüstü"},
            {"Yol":"C:\\Users\\A\\Desktop\\b.lnk","Ad":"b","Hedef":"","Konum":"Masaüstü"},
            {"yol":"C:\\Users\\A\\Desktop\\c.lnk","hedef":"C:\\App\\c.exe"}
        ]"#;
        let sonuc = kisayol_json_ayristir(json).unwrap();
        assert_eq!(sonuc.len(), 2);
        assert_eq!(sonuc[0].ad, "a");
        assert_eq!(sonuc[0].hedef, r"C:\App\a.exe");
        assert_eq!(sonuc[1].ad, "c");
        assert_eq!(sonuc[1].konum, "");
    }

    #[test]
    fn kisayol_json_tek_nesne_ayristirilir() {
        let json = r#"{"Yol":"C:\\a.lnk","Ad":"a","Hedef":"C:\\a.exe","Konum":"Başlat Menüsü"}"#;
        let sonuc = kisayol_json_ayristir(json).unwrap();
        assert_eq!(sonuc.len(), 1);
        assert_eq!(sonuc[0].konum, "Başlat Menüsü");
    }

    #[test]
    fn bos_kisayol_listesi_bos_doner() {
        assert!(kisayol_json_ayristir("").unwrap().is_empty());
        assert!(kisayol_json_ayristir("null").unwrap().is_empty());
        assert!(kisayol_json_ayristir("[]").unwrap().is_empty());
    }

    #[test]
    fn bozuk_kisayol_json_hata_doner() {
        assert!(kisayol_json_ayristir("{").is_err());
        assert!(kisayol_json_ayristir("42").is_err());
    }

    #[test]
    fn yeni_kategoriler_turkce_sabit_olarak_acilir() {
        assert_eq!(KATEGORI_YAZI_TIPI, "Yazı tipi kaydı");
        assert_eq!(KATEGORI_SES_OLAYI, "Ses olayı kaydı");
        assert_eq!(KATEGORI_MUI, "MUI önbelleği");
        assert_eq!(KATEGORI_YARDIM, "Yardım dosyası kaydı");
    }

    #[test]
    fn kimlik_kategori_kove_yol_ve_deger_icerir() {
        let bulgu = KayitDefteriBulgu {
            kimlik: String::new(),
            kategori: KATEGORI_MUI.into(),
            ad: "Test".into(),
            ayrinti: "test".into(),
            kove: "HKCU".into(),
            yol: r"C:\App\a.exe".into(),
            deger: Some(r"C:\App\a.exe.FriendlyAppName".into()),
        };
        let kimlik = bulgu.kimlik_uret();
        assert!(kimlik.contains(KATEGORI_MUI));
        assert!(kimlik.contains("HKCU"));
        assert!(kimlik.contains(r"c:\app\a.exe"));
        assert!(kimlik.ends_with(r"c:\app\a.exe.friendlyappname"));
    }

    #[test]
    fn kimlik_deger_yoksa_bos_alan_uretir() {
        let bulgu = KayitDefteriBulgu {
            kimlik: String::new(),
            kategori: KATEGORI_YARDIM.into(),
            ad: "Test".into(),
            ayrinti: "test".into(),
            kove: "HKLM".into(),
            yol: r"SOFTWARE\Help".into(),
            deger: None,
        };
        assert_eq!(
            bulgu.kimlik_uret(),
            r"Yardım dosyası kaydı|HKLM|software\help|"
        );
    }

    #[test]
    fn registry_yolu_enjeksiyonunu_reddeder() {
        assert!(kayit_yolu_guvenli_mi(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Fonts"
        ));
        assert!(!kayit_yolu_guvenli_mi(""));
        assert!(!kayit_yolu_guvenli_mi(r"\SOFTWARE\Test"));
        assert!(!kayit_yolu_guvenli_mi(r"SOFTWARE\Test\"));
        assert!(!kayit_yolu_guvenli_mi(r"SOFTWARE\..\Test"));
        assert!(!kayit_yolu_guvenli_mi("SOFTWARE\\Test\"X"));
        assert!(!kayit_yolu_guvenli_mi("SOFTWARE\\Test\rX"));
        assert!(!kayit_yolu_guvenli_mi("SOFTWARE\\Test\nX"));
        assert!(!kayit_yolu_guvenli_mi("SOFTWARE\\Test\0X"));
    }

    #[test]
    fn registry_deger_enjeksiyonunu_reddeder() {
        assert!(kayit_degeri_guvenli_mi(".Current"));
        assert!(kayit_degeri_guvenli_mi("Arial (TrueType)"));
        assert!(kayit_degeri_guvenli_mi(r"C:\App\a.exe.FriendlyAppName"));
        assert!(!kayit_degeri_guvenli_mi(""));
        assert!(!kayit_degeri_guvenli_mi("/va"));
        assert!(!kayit_degeri_guvenli_mi(r"..\a"));
        assert!(!kayit_degeri_guvenli_mi("a\"b"));
        assert!(!kayit_degeri_guvenli_mi("a\rb"));
        assert!(!kayit_degeri_guvenli_mi("a\nb"));
        assert!(!kayit_degeri_guvenli_mi("a\0b"));
    }

    #[test]
    fn font_dosya_adi_ayiklanir() {
        assert_eq!(
            yazi_tipi_dosya_adi_ayikla("arial.ttf").as_deref(),
            Some("arial.ttf")
        );
        assert_eq!(
            yazi_tipi_dosya_adi_ayikla(r#""C:\Windows\Fonts\MSGOTHIC.TTC""#).as_deref(),
            Some(r"C:\Windows\Fonts\MSGOTHIC.TTC")
        );
        assert_eq!(yazi_tipi_dosya_adi_ayikla("Arial (TrueType)"), None);
        assert_eq!(yazi_tipi_dosya_adi_ayikla("readme.txt"), None);
        assert_eq!(yazi_tipi_dosya_adi_ayikla("a.ttf, b.ttf"), None);
        assert_eq!(yazi_tipi_dosya_adi_ayikla(r"%WINDIR%\Fonts\a.ttf"), None);
    }

    #[test]
    fn font_hedef_yolu_fonts_veya_mutlak_yolda_cozulur() {
        assert_eq!(
            yazi_tipi_hedef_yolu("arial.ttf", r"C:\Windows\Fonts").as_deref(),
            Some(r"C:\Windows\Fonts\arial.ttf")
        );
        assert_eq!(
            yazi_tipi_hedef_yolu(r"C:\Fonts\ozel.otf", r"C:\Windows\Fonts").as_deref(),
            Some(r"C:\Fonts\ozel.otf")
        );
        assert_eq!(
            yazi_tipi_hedef_yolu(r"alt\arial.ttf", r"C:\Windows\Fonts"),
            None
        );
    }

    #[test]
    fn ses_yolu_systemroot_genisletilir() {
        assert_eq!(
            ses_olayi_hedef_yolu(r"%SystemRoot%\Media\ding.wav", r"C:\Windows").as_deref(),
            Some(r"C:\Windows\Media\ding.wav")
        );
        assert_eq!(
            ses_olayi_hedef_yolu(r"%windir%\media\ding.WAV", r"C:\Windows").as_deref(),
            Some(r"C:\Windows\media\ding.WAV")
        );
        assert_eq!(
            ses_olayi_hedef_yolu(r"C:\Windows\Media\ding.wav", r"C:\Windows").as_deref(),
            Some(r"C:\Windows\Media\ding.wav")
        );
    }

    #[test]
    fn ses_yolu_diger_degiskenleri_ve_wav_olmayani_atlar() {
        assert_eq!(
            ses_olayi_hedef_yolu(r"%USERPROFILE%\ding.wav", r"C:\Windows"),
            None
        );
        assert_eq!(
            ses_olayi_hedef_yolu(r"%SystemRoot%\Media\ding.mp3", r"C:\Windows"),
            None
        );
        assert_eq!(ses_olayi_hedef_yolu(r"Media\ding.wav", r"C:\Windows"), None);
        assert_eq!(ses_olayi_hedef_yolu("", r"C:\Windows"), None);
        assert_eq!(
            ses_olayi_hedef_yolu(r"%SystemRoot%\Media\ding.wav", ""),
            None
        );
    }

    #[test]
    fn mui_sonegi_ayiklanir_ve_yalnizca_mutlak_exe_kabul_edilir() {
        assert_eq!(
            mui_uygulama_yolu_ayikla(r"C:\App\a.exe.FriendlyAppName").as_deref(),
            Some(r"C:\App\a.exe")
        );
        assert_eq!(
            mui_uygulama_yolu_ayikla(r"c:\app\a.EXE.applicationcompany").as_deref(),
            Some(r"c:\app\a.EXE")
        );
        assert_eq!(
            mui_uygulama_yolu_ayikla(r"\\sunucu\paylasim\a.exe.FriendlyAppName").as_deref(),
            Some(r"\\sunucu\paylasim\a.exe")
        );
        assert_eq!(
            mui_uygulama_yolu_ayikla(r"D:\App\a.exe.FriendlyAppName"),
            None
        );
        assert_eq!(
            mui_uygulama_yolu_ayikla(r"C:\App\a.dll.FriendlyAppName"),
            None
        );
        assert_eq!(mui_uygulama_yolu_ayikla(r"C:\App\a.exe"), None);
        assert_eq!(mui_uygulama_yolu_ayikla(r"a.exe.FriendlyAppName"), None);
        assert_eq!(
            mui_uygulama_yolu_ayikla(r"C:\App\..\a.exe.FriendlyAppName"),
            None
        );
    }

    #[test]
    fn yardim_dosyasi_hedef_yolu_cozulur() {
        assert_eq!(
            yardim_dosyasi_hedef_yolu("windows.hlp", r"C:\Windows\Help").as_deref(),
            Some(r"C:\Windows\Help\windows.hlp")
        );
        assert_eq!(
            yardim_dosyasi_hedef_yolu(r"C:\Apps\yardim.chm", r"C:\Windows\Help").as_deref(),
            Some(r"C:\Apps\yardim.chm")
        );
        assert_eq!(
            yardim_dosyasi_hedef_yolu("readme.txt", r"C:\Windows\Help"),
            None
        );
        assert_eq!(
            yardim_dosyasi_hedef_yolu(r"alt\yardim.hlp", r"C:\Windows\Help"),
            None
        );
        assert_eq!(
            yardim_dosyasi_hedef_yolu(r"%WINDIR%\Help\a.chm", r"C:\Windows\Help"),
            None
        );
    }
}
