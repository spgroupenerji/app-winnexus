use sp_core::error::{Hata, Sonuc};
use sp_core::temizleyici::{
    yol_kok_altinda_mi, KarantinaSonucu, TemizlikHedefi, TemizlikSonucu, TemizlikTarama,
    YinelenenGrup,
};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, Metadata, OpenOptions};
use std::hash::Hasher;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::windows::fs::MetadataExt;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const REPARSE_NOKTASI: u32 = 0x0000_0400;
const TAMPON_BAYT: usize = 64 * 1024;
const AZAMI_KLASOR_DERINLIGI: usize = 12;
const PARCALAMA_GECISI: u8 = 3;
const SIFIR_BAYT: u8 = 0x00;
const DOLU_BAYT: u8 = 0xFF;
const XORSHAFT_SABITI: u64 = 0x9E37_79B9_7F4A_7C15;
const SILINMIS_UZANTI: &str = "winnexus-sil";
const VARSAYILAN_DOSYA_ADI: &str = "dosya";
const KANAL_BEKLEME: Duration = Duration::from_millis(100);
const GERI_DONUSUM_ZAMAN_ASIMI: Duration = Duration::from_secs(120);
const GERI_DONUSUM_BETIGI: &str = "Clear-RecycleBin -Force -ErrorAction Stop";
const CHROMIUM_ONBELLEK_KLASORLERI: [&str; 2] = ["Cache", "Code Cache"];

pub struct TemizleyiciServisi;

impl TemizleyiciServisi {
    pub fn yeni() -> Self {
        Self
    }

    pub fn varsayilan_hedefler(&self) -> Vec<TemizlikHedefi> {
        let mut hedefler = vec![
            hedef(
                "kullanici-gecici",
                "Kullanıcı Geçici Dosyaları",
                &std::env::temp_dir(),
            ),
            hedef(
                "windows-gecici",
                "Windows Geçici Dosyaları",
                &windows_temp_yolu(),
            ),
        ];
        hedefler.extend(tarayici_onbellekleri());
        hedefler
    }

    pub fn tara(&self, hedef: &TemizlikHedefi) -> Sonuc<TemizlikTarama> {
        let mut tarama = TemizlikTarama::default();
        dosyalari_gez(Path::new(&hedef.yol), &mut |_, boyut| {
            tarama.dosya_sayisi += 1;
            tarama.bayt = tarama.bayt.saturating_add(boyut);
        })?;
        Ok(tarama)
    }

    pub fn temizle(&self, hedef: &TemizlikHedefi) -> Sonuc<TemizlikSonucu> {
        let izinli = izinli_kokler();
        let kok = fs::canonicalize(&hedef.yol)
            .map_err(|hata| Hata::Dogrulama(format!("{} bulunamadı: {hata}", hedef.ad)))?;
        if !kok.is_dir() {
            return Err(Hata::Dogrulama(format!("{} bir klasör değil.", hedef.ad)));
        }
        if !kokler_altinda_mi(&izinli, &kok) {
            return Err(Hata::Yetki(format!(
                "{} izinli temizlik köklerinin dışında.",
                hedef.ad
            )));
        }
        let mut sonuc = TemizlikSonucu::default();
        dosyalari_gez(&kok, &mut |yol, boyut| match dosya_sil(yol, &izinli) {
            Ok(()) => {
                sonuc.silinen_dosya += 1;
                sonuc.silinen_bayt = sonuc.silinen_bayt.saturating_add(boyut);
            }
            Err(hata) => sonuc.hatalar.push(hata),
        })?;
        Ok(sonuc)
    }

    pub fn bos_klasorleri_tara(&self, kok: &Path) -> Sonuc<Vec<String>> {
        let meta = fs::symlink_metadata(kok)
            .map_err(|hata| Hata::Dogrulama(format!("{} okunamadı: {hata}", kok.display())))?;
        if !meta.is_dir() {
            return Err(Hata::Dogrulama(format!(
                "{} bir klasör değil.",
                kok.display()
            )));
        }
        let mut klasorler = Vec::new();
        fs::read_dir(kok)
            .map_err(|hata| Hata::Dogrulama(format!("{} okunamadı: {hata}", kok.display())))?;
        bos_klasor_topla(kok, 0, &mut klasorler);
        Ok(klasorler)
    }

    pub fn bos_klasorleri_sil(&self, kok: &Path, klasorler: &[String]) -> Sonuc<TemizlikSonucu> {
        let kok = fs::canonicalize(kok)
            .map_err(|hata| Hata::Dogrulama(format!("{} bulunamadı: {hata}", kok.display())))?;
        if !kok.is_dir() {
            return Err(Hata::Dogrulama(format!(
                "{} bir klasör değil.",
                kok.display()
            )));
        }
        let mut sonuc = TemizlikSonucu::default();
        let mut adaylar = Vec::new();
        let mut gorulen = HashSet::new();
        for metin in klasorler {
            let Ok(aday) = fs::canonicalize(metin) else {
                continue;
            };
            if aday == kok {
                sonuc
                    .hatalar
                    .push(format!("{metin}: kök klasör silinemez."));
                continue;
            }
            if !yol_kok_altinda_mi(&kok, &aday) {
                sonuc.hatalar.push(format!("{metin}: kökün dışında."));
                continue;
            }
            if gorulen.insert(aday.clone()) {
                adaylar.push(aday);
            }
        }
        adaylar.sort_by_key(|aday| std::cmp::Reverse(aday.components().count()));
        for aday in adaylar {
            if !bos_dizin_mi(&aday) {
                continue;
            }
            match fs::remove_dir(&aday) {
                Ok(()) => sonuc.silinen_dosya += 1,
                Err(hata) => sonuc
                    .hatalar
                    .push(format!("{} silinemedi: {hata}", aday.display())),
            }
        }
        Ok(sonuc)
    }

    pub fn yinelenenleri_tara(&self, kok: &Path, min_bayt: u64) -> Sonuc<Vec<YinelenenGrup>> {
        let esik = min_bayt.max(1);
        let mut boyuta_gore: HashMap<u64, Vec<PathBuf>> = HashMap::new();
        dosyalari_gez(kok, &mut |yol, boyut| {
            if boyut >= esik {
                boyuta_gore
                    .entry(boyut)
                    .or_default()
                    .push(yol.to_path_buf());
            }
        })?;
        let mut gruplar = Vec::new();
        for (boyut, dosyalar) in boyuta_gore {
            if dosyalar.len() < 2 {
                continue;
            }
            for icerik in icerige_gore_grupla(&dosyalar) {
                gruplar.push(YinelenenGrup {
                    boyut,
                    dosyalar: icerik,
                });
            }
        }
        gruplar.sort_by(|a, b| {
            b.boyut
                .cmp(&a.boyut)
                .then_with(|| a.dosyalar[0].cmp(&b.dosyalar[0]))
        });
        Ok(gruplar)
    }

    pub fn yinelenenleri_karantinaya_tasi(
        &self,
        kok: &Path,
        dosyalar: &[String],
        karantina: &Path,
    ) -> Sonuc<KarantinaSonucu> {
        let kok = fs::canonicalize(kok)
            .map_err(|hata| Hata::Dogrulama(format!("{} bulunamadı: {hata}", kok.display())))?;
        if !kok.is_dir() {
            return Err(Hata::Dogrulama(format!(
                "{} bir klasör değil.",
                kok.display()
            )));
        }
        let mut sonuc = KarantinaSonucu {
            tasinan: 0,
            hatalar: Vec::new(),
        };
        if dosyalar.len() < 2 {
            return Ok(sonuc);
        }
        fs::create_dir_all(karantina)
            .map_err(|hata| Hata::Dogrulama(format!("Karantina klasörü oluşturulamadı: {hata}")))?;
        let karantina = fs::canonicalize(karantina)
            .map_err(|hata| Hata::Dogrulama(format!("Karantina klasörü açılamadı: {hata}")))?;
        for (sira, metin) in dosyalar.iter().enumerate() {
            let Ok(kaynak) = fs::canonicalize(metin) else {
                sonuc.hatalar.push(format!("{metin}: dosya bulunamadı."));
                continue;
            };
            if !yol_kok_altinda_mi(&kok, &kaynak) {
                sonuc.hatalar.push(format!("{metin}: kökün dışında."));
                continue;
            }
            if yol_kok_altinda_mi(&karantina, &kaynak) {
                sonuc
                    .hatalar
                    .push(format!("{metin}: dosya zaten karantinada."));
                continue;
            }
            let Ok(meta) = fs::symlink_metadata(&kaynak) else {
                sonuc.hatalar.push(format!("{metin}: dosya okunamadı."));
                continue;
            };
            if !meta.is_file() || yeniden_ayristirma_mi(&meta) {
                sonuc.hatalar.push(format!("{metin}: normal dosya değil."));
                continue;
            }
            if sira == 0 {
                continue;
            }
            let hedef = benzersiz_yol(&karantina, &kaynak);
            match tasi(&kaynak, &hedef) {
                Ok(()) => sonuc.tasinan += 1,
                Err(hata) => sonuc.hatalar.push(format!("{metin}: {hata}")),
            }
        }
        Ok(sonuc)
    }

    #[allow(clippy::permissions_set_readonly_false)]
    pub fn parcala(&self, dosya: &Path) -> Sonuc<()> {
        let meta = fs::symlink_metadata(dosya)
            .map_err(|hata| Hata::Dogrulama(format!("{} bulunamadı: {hata}", dosya.display())))?;
        if !meta.is_file() || yeniden_ayristirma_mi(&meta) {
            return Err(Hata::Dogrulama(format!(
                "{} normal bir dosya değil.",
                dosya.display()
            )));
        }
        if meta.permissions().readonly() {
            let mut izinler = meta.permissions();
            izinler.set_readonly(false);
            fs::set_permissions(dosya, izinler).map_err(|hata| {
                Hata::Dogrulama(format!(
                    "{} salt okunur bayrağı kaldırılamadı: {hata}",
                    dosya.display()
                ))
            })?;
        }
        uzerine_yaz(dosya, meta.len())?;
        let silinecek = silme_yolu(dosya);
        if silinecek != dosya {
            fs::rename(dosya, &silinecek).map_err(|hata| {
                Hata::Dogrulama(format!(
                    "{} silinmek üzere yeniden adlandırılamadı: {hata}",
                    dosya.display()
                ))
            })?;
        }
        fs::remove_file(&silinecek)
            .map_err(|hata| Hata::Dogrulama(format!("{} silinemedi: {hata}", silinecek.display())))
    }

    pub fn geri_donusum_temizle(&self) -> Sonuc<()> {
        let mut surec = geri_donusum_sureci()?;
        let cikti_okuyucu = surec
            .stdout
            .take()
            .map(|kanal| std::thread::spawn(move || kanal_metni(kanal)));
        let hata_okuyucu = surec
            .stderr
            .take()
            .map(|kanal| std::thread::spawn(move || kanal_metni(kanal)));
        let durum = surec_bekle(&mut surec)?;
        let cikti = cikti_okuyucu
            .and_then(|okuyucu| okuyucu.join().ok())
            .unwrap_or_default();
        let hata_ciktisi = hata_okuyucu
            .and_then(|okuyucu| okuyucu.join().ok())
            .unwrap_or_default();
        if durum.success() {
            return Ok(());
        }
        Err(Hata::DisArac(geri_donusum_hatasi(&cikti, &hata_ciktisi)))
    }
}

fn hedef(kimlik: &str, ad: &str, yol: &Path) -> TemizlikHedefi {
    TemizlikHedefi {
        kimlik: kimlik.to_string(),
        ad: ad.to_string(),
        yol: yol.to_string_lossy().into_owned(),
    }
}

fn windows_temp_yolu() -> PathBuf {
    std::env::var_os("SystemRoot")
        .map(|kok| PathBuf::from(kok).join("Temp"))
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows\Temp"))
}

fn tarayici_onbellekleri() -> Vec<TemizlikHedefi> {
    let mut hedefler = Vec::new();
    if let Some(yerel) = std::env::var_os("LOCALAPPDATA") {
        let yerel = PathBuf::from(yerel);
        let tarayicilar = [
            (
                "chrome",
                "Google Chrome",
                yerel.join("Google").join("Chrome").join("User Data"),
            ),
            (
                "edge",
                "Microsoft Edge",
                yerel.join("Microsoft").join("Edge").join("User Data"),
            ),
            (
                "brave",
                "Brave",
                yerel
                    .join("BraveSoftware")
                    .join("Brave-Browser")
                    .join("User Data"),
            ),
        ];
        for (kimlik, ad, kok) in tarayicilar {
            for profil in alt_klasorler(&kok) {
                let profil_adi = profil_adi(&profil);
                if !tarayici_profili_mi(&profil_adi) {
                    continue;
                }
                for alt in CHROMIUM_ONBELLEK_KLASORLERI {
                    let yol = profil.join(alt);
                    if yol.is_dir() {
                        hedefler.push(hedef(
                            &onbellek_kimligi(kimlik, &profil_adi, alt),
                            &format!("{ad} Önbelleği ({profil_adi})"),
                            &yol,
                        ));
                    }
                }
            }
        }
    }
    if let Some(gezgin) = std::env::var_os("APPDATA") {
        let kok = PathBuf::from(gezgin)
            .join("Mozilla")
            .join("Firefox")
            .join("Profiles");
        for profil in alt_klasorler(&kok) {
            let profil_adi = profil_adi(&profil);
            let yol = profil.join("cache2");
            if yol.is_dir() {
                hedefler.push(hedef(
                    &format!("firefox-onbellek-{}", profil_adi.to_ascii_lowercase()),
                    &format!("Firefox Önbelleği ({profil_adi})"),
                    &yol,
                ));
            }
        }
    }
    hedefler
}

fn onbellek_kimligi(tarayici: &str, profil: &str, alt_klasor: &str) -> String {
    format!(
        "{tarayici}-onbellek-{}-{}",
        profil.to_ascii_lowercase(),
        alt_klasor.to_ascii_lowercase().replace(' ', "-")
    )
}

fn alt_klasorler(kok: &Path) -> Vec<PathBuf> {
    let Ok(kayitlar) = fs::read_dir(kok) else {
        return Vec::new();
    };
    kayitlar
        .flatten()
        .map(|kayit| kayit.path())
        .filter(|yol| yol.is_dir())
        .collect()
}

fn profil_adi(profil: &Path) -> String {
    profil
        .file_name()
        .map(|ad| ad.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn tarayici_profili_mi(ad: &str) -> bool {
    let ad = ad.to_ascii_lowercase();
    ad == "default" || ad.starts_with("profile ")
}

fn izinli_kokler() -> Vec<PathBuf> {
    let mut kokler = vec![std::env::temp_dir(), windows_temp_yolu()];
    kokler.extend(
        tarayici_onbellekleri()
            .into_iter()
            .map(|hedef| PathBuf::from(hedef.yol)),
    );
    kokler
        .into_iter()
        .filter_map(|kok| fs::canonicalize(kok).ok())
        .filter(|kok| guvenli_kok_mu(kok))
        .collect()
}

fn guvenli_kok_mu(kok: &Path) -> bool {
    kok.components()
        .any(|bilesen| matches!(bilesen, std::path::Component::Normal(_)))
}

fn kokler_altinda_mi(kokler: &[PathBuf], yol: &Path) -> bool {
    kokler.iter().any(|kok| yol_kok_altinda_mi(kok, yol))
}

fn dosya_sil(yol: &Path, izinli: &[PathBuf]) -> Result<(), String> {
    let kanonik = fs::canonicalize(yol).map_err(|hata| format!("{}: {hata}", yol.display()))?;
    if !kokler_altinda_mi(izinli, &kanonik) {
        return Err(format!("{} izinli kökün dışında.", yol.display()));
    }
    fs::remove_file(&kanonik).map_err(|hata| format!("{} silinemedi: {hata}", yol.display()))
}

fn dosyalari_gez(kok: &Path, islem: &mut impl FnMut(&Path, u64)) -> Sonuc<()> {
    let meta = fs::symlink_metadata(kok)
        .map_err(|hata| Hata::Dogrulama(format!("{} okunamadı: {hata}", kok.display())))?;
    if !meta.is_dir() {
        return Err(Hata::Dogrulama(format!(
            "{} bir klasör değil.",
            kok.display()
        )));
    }
    fs::read_dir(kok)
        .map_err(|hata| Hata::Dogrulama(format!("{} okunamadı: {hata}", kok.display())))?;
    dizin_gez(kok, islem);
    Ok(())
}

fn dizin_gez(dizin: &Path, islem: &mut impl FnMut(&Path, u64)) {
    let Ok(kayitlar) = fs::read_dir(dizin) else {
        return;
    };
    for kayit in kayitlar.flatten() {
        let yol = kayit.path();
        let Ok(meta) = fs::symlink_metadata(&yol) else {
            continue;
        };
        if yeniden_ayristirma_mi(&meta) {
            continue;
        }
        if meta.is_dir() {
            dizin_gez(&yol, islem);
        } else if meta.is_file() {
            islem(&yol, meta.len());
        }
    }
}

fn yeniden_ayristirma_mi(meta: &Metadata) -> bool {
    meta.file_attributes() & REPARSE_NOKTASI != 0
}

fn bos_klasor_topla(dizin: &Path, derinlik: usize, sonuc: &mut Vec<String>) -> bool {
    let Ok(kayitlar) = fs::read_dir(dizin) else {
        return false;
    };
    let mut bos = true;
    for kayit in kayitlar {
        let Ok(kayit) = kayit else {
            bos = false;
            continue;
        };
        let yol = kayit.path();
        let Ok(meta) = fs::symlink_metadata(&yol) else {
            bos = false;
            continue;
        };
        if yeniden_ayristirma_mi(&meta) || !meta.is_dir() || derinlik >= AZAMI_KLASOR_DERINLIGI {
            bos = false;
            continue;
        }
        if !bos_klasor_topla(&yol, derinlik + 1, sonuc) {
            bos = false;
        }
    }
    if bos && derinlik > 0 {
        sonuc.push(dizin.to_string_lossy().into_owned());
    }
    bos
}

fn bos_dizin_mi(yol: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(yol) else {
        return false;
    };
    meta.is_dir()
        && !yeniden_ayristirma_mi(&meta)
        && fs::read_dir(yol)
            .map(|mut kayitlar| kayitlar.next().is_none())
            .unwrap_or(false)
}

fn icerige_gore_grupla(dosyalar: &[PathBuf]) -> Vec<Vec<String>> {
    let mut kovalar: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for yol in dosyalar {
        if let Some(ozet) = dosya_ozeti(yol) {
            kovalar.entry(ozet).or_default().push(yol.clone());
        }
    }
    let mut gruplar = Vec::new();
    for kova in kovalar.into_values() {
        if kova.len() < 2 {
            continue;
        }
        let mut alt_gruplar: Vec<Vec<PathBuf>> = Vec::new();
        for yol in kova {
            let eslesen = alt_gruplar
                .iter()
                .position(|grup| ayni_icerik_mi(&grup[0], &yol));
            match eslesen {
                Some(sira) => alt_gruplar[sira].push(yol),
                None => alt_gruplar.push(vec![yol]),
            }
        }
        for grup in alt_gruplar {
            if grup.len() < 2 {
                continue;
            }
            let mut metinler: Vec<String> = grup
                .iter()
                .map(|yol| yol.to_string_lossy().into_owned())
                .collect();
            metinler.sort();
            gruplar.push(metinler);
        }
    }
    gruplar
}

fn dosya_ozeti(yol: &Path) -> Option<u64> {
    let mut dosya = File::open(yol).ok()?;
    let mut ozet = DefaultHasher::new();
    let mut tampon = vec![0u8; TAMPON_BAYT];
    loop {
        let okunan = dosya.read(&mut tampon).ok()?;
        if okunan == 0 {
            break;
        }
        ozet.write(&tampon[..okunan]);
    }
    Some(ozet.finish())
}

fn ayni_icerik_mi(ilk: &Path, ikinci: &Path) -> bool {
    let (Ok(mut ilk), Ok(mut ikinci)) = (File::open(ilk), File::open(ikinci)) else {
        return false;
    };
    let mut ilk_tampon = vec![0u8; TAMPON_BAYT];
    let mut ikinci_tampon = vec![0u8; TAMPON_BAYT];
    loop {
        let ilk_okunan = match ilk.read(&mut ilk_tampon) {
            Ok(okunan) => okunan,
            Err(_) => return false,
        };
        let ikinci_okunan = match ikinci.read(&mut ikinci_tampon) {
            Ok(okunan) => okunan,
            Err(_) => return false,
        };
        if ilk_okunan != ikinci_okunan || ilk_tampon[..ilk_okunan] != ikinci_tampon[..ikinci_okunan]
        {
            return false;
        }
        if ilk_okunan == 0 {
            return true;
        }
    }
}

fn benzersiz_yol(klasor: &Path, kaynak: &Path) -> PathBuf {
    let ad = kaynak
        .file_name()
        .map(|ad| ad.to_string_lossy().into_owned())
        .unwrap_or_else(|| VARSAYILAN_DOSYA_ADI.to_string());
    let aday = klasor.join(&ad);
    if !aday.exists() {
        return aday;
    }
    let yol = Path::new(&ad);
    let govde = yol
        .file_stem()
        .map(|govde| govde.to_string_lossy().into_owned())
        .unwrap_or_else(|| VARSAYILAN_DOSYA_ADI.to_string());
    let uzanti = yol
        .extension()
        .map(|uzanti| format!(".{}", uzanti.to_string_lossy()))
        .unwrap_or_default();
    let mut sayac = 1u32;
    loop {
        let aday = klasor.join(format!("{govde}-{sayac}{uzanti}"));
        if !aday.exists() {
            return aday;
        }
        sayac += 1;
    }
}

fn tasi(kaynak: &Path, hedef: &Path) -> Result<(), String> {
    if fs::rename(kaynak, hedef).is_ok() {
        return Ok(());
    }
    if let Err(hata) = fs::copy(kaynak, hedef) {
        let _ = fs::remove_file(hedef);
        return Err(format!("kopyalanamadı: {hata}"));
    }
    fs::remove_file(kaynak).map_err(|hata| format!("kopyalandı ancak kaynak silinemedi: {hata}"))
}

fn uzerine_yaz(dosya: &Path, uzunluk: u64) -> Sonuc<()> {
    let mut akis = OpenOptions::new().write(true).open(dosya).map_err(|hata| {
        Hata::Dogrulama(format!("{} yazma için açılamadı: {hata}", dosya.display()))
    })?;
    let mut durum = tohum(uzunluk);
    let mut tampon = vec![0u8; TAMPON_BAYT];
    for gecis in 0..PARCALAMA_GECISI {
        akis.seek(SeekFrom::Start(0)).map_err(|hata| {
            Hata::Dogrulama(format!("{} konumlandırılamadı: {hata}", dosya.display()))
        })?;
        let mut kalan = uzunluk;
        while kalan > 0 {
            let parca = kalan.min(TAMPON_BAYT as u64) as usize;
            match gecis {
                0 => tampon[..parca].fill(SIFIR_BAYT),
                1 => tampon[..parca].fill(DOLU_BAYT),
                _ => xorshift_doldur(&mut tampon[..parca], &mut durum),
            }
            akis.write_all(&tampon[..parca]).map_err(|hata| {
                Hata::Dogrulama(format!("{} üzerine yazılamadı: {hata}", dosya.display()))
            })?;
            kalan -= parca as u64;
        }
        akis.sync_all()
            .map_err(|hata| Hata::Dogrulama(format!("{} eşitlenemedi: {hata}", dosya.display())))?;
    }
    Ok(())
}

fn tohum(uzunluk: u64) -> u64 {
    let zaman = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|sure| sure.as_nanos() as u64)
        .unwrap_or(XORSHAFT_SABITI);
    (zaman ^ uzunluk ^ XORSHAFT_SABITI) | 1
}

fn xorshift_doldur(tampon: &mut [u8], durum: &mut u64) {
    for bayt in tampon {
        let mut deger = *durum;
        deger ^= deger << 13;
        deger ^= deger >> 7;
        deger ^= deger << 17;
        *durum = deger;
        *bayt = (deger & 0xFF) as u8;
    }
}

fn silme_yolu(dosya: &Path) -> PathBuf {
    let mut aday = dosya.to_path_buf();
    aday.set_extension(SILINMIS_UZANTI);
    if aday != dosya && !aday.exists() {
        return aday;
    }
    let govde = dosya
        .file_stem()
        .map(|govde| govde.to_string_lossy().into_owned())
        .unwrap_or_else(|| VARSAYILAN_DOSYA_ADI.to_string());
    let klasor = dosya.parent().unwrap_or_else(|| Path::new("."));
    let mut sayac = 1u32;
    loop {
        let aday = klasor.join(format!("{govde}-{sayac}.{SILINMIS_UZANTI}"));
        if !aday.exists() {
            return aday;
        }
        sayac += 1;
    }
}

fn geri_donusum_sureci() -> Sonuc<Child> {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            GERI_DONUSUM_BETIGI,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|hata| Hata::DisArac(format!("PowerShell başlatılamadı: {hata}")))
}

fn surec_bekle(surec: &mut Child) -> Sonuc<ExitStatus> {
    let baslangic = Instant::now();
    loop {
        match surec.try_wait() {
            Ok(Some(durum)) => return Ok(durum),
            Ok(None) => {}
            Err(hata) => {
                let _ = surec.kill();
                let _ = surec.wait();
                return Err(Hata::DisArac(format!(
                    "Geri dönüşüm kutusu temizliği izlenemedi: {hata}"
                )));
            }
        }
        if baslangic.elapsed() > GERI_DONUSUM_ZAMAN_ASIMI {
            let _ = surec.kill();
            let _ = surec.wait();
            return Err(Hata::DisArac(
                "Geri dönüşüm kutusu temizliği zaman aşımına uğradı.".to_string(),
            ));
        }
        std::thread::sleep(KANAL_BEKLEME);
    }
}

fn kanal_metni(mut kanal: impl Read) -> String {
    let mut baytlar = Vec::new();
    let _ = kanal.read_to_end(&mut baytlar);
    match String::from_utf8(baytlar) {
        Ok(metin) => metin,
        Err(hata) => encoding_rs::WINDOWS_1254
            .decode(hata.as_bytes())
            .0
            .into_owned(),
    }
}

fn geri_donusum_hatasi(cikti: &str, hata_ciktisi: &str) -> String {
    let ayrinti = if hata_ciktisi.trim().is_empty() {
        cikti.trim()
    } else {
        hata_ciktisi.trim()
    };
    if ayrinti.is_empty() {
        "Geri dönüşüm kutusu temizlenemedi.".to_string()
    } else {
        format!("Geri dönüşüm kutusu temizlenemedi: {ayrinti}")
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    fn gecici_hedef(kok: &Path) -> TemizlikHedefi {
        TemizlikHedefi {
            kimlik: "test".to_string(),
            ad: "Test".to_string(),
            yol: kok.to_string_lossy().into_owned(),
        }
    }

    #[test]
    fn onbellek_kimligi_alt_klasorle_benzersizdir() {
        assert_ne!(
            onbellek_kimligi("chrome", "Default", "Cache"),
            onbellek_kimligi("chrome", "Default", "Code Cache")
        );
    }

    #[test]
    fn gecici_klasor_taranir_ve_temizlenir() {
        let kok = tempfile::tempdir().unwrap();
        fs::write(kok.path().join("a.tmp"), b"12345").unwrap();
        fs::write(kok.path().join("b.tmp"), b"123").unwrap();
        let hedef = gecici_hedef(kok.path());
        let servis = TemizleyiciServisi::yeni();
        let tarama = servis.tara(&hedef).unwrap();
        assert_eq!(tarama.dosya_sayisi, 2);
        assert_eq!(tarama.bayt, 8);
        let sonuc = servis.temizle(&hedef).unwrap();
        assert_eq!(sonuc.silinen_dosya, 2);
        assert_eq!(sonuc.silinen_bayt, 8);
        assert!(sonuc.hatalar.is_empty());
        assert!(kok.path().exists());
        assert_eq!(fs::read_dir(kok.path()).unwrap().count(), 0);
    }

    #[test]
    fn izinli_kok_disarisi_temizlenmez() {
        let disari = tempfile::Builder::new()
            .prefix("winnexus-izin-")
            .tempdir_in(std::env::current_dir().unwrap())
            .unwrap();
        let korunacak = disari.path().join("korunacak.txt");
        fs::write(&korunacak, b"veri").unwrap();
        let sonuc = TemizleyiciServisi::yeni().temizle(&gecici_hedef(disari.path()));
        assert!(matches!(sonuc, Err(Hata::Yetki(_))));
        assert!(korunacak.exists());
    }

    #[test]
    fn bos_klasorler_bulunur_ve_silinir() {
        let kok = tempfile::tempdir().unwrap();
        let dal = kok.path().join("a").join("b");
        fs::create_dir_all(&dal).unwrap();
        let dolu = kok.path().join("dolu");
        fs::create_dir_all(&dolu).unwrap();
        fs::write(dolu.join("x.txt"), b"veri").unwrap();
        let servis = TemizleyiciServisi::yeni();
        let bulunan = servis.bos_klasorleri_tara(kok.path()).unwrap();
        assert_eq!(bulunan.len(), 2);
        assert_eq!(bulunan[0], dal.to_string_lossy().into_owned());
        let kok_metni = kok.path().to_string_lossy().into_owned();
        assert!(!bulunan.iter().any(|yol| yol == &kok_metni));
        let sonuc = servis.bos_klasorleri_sil(kok.path(), &bulunan).unwrap();
        assert_eq!(sonuc.silinen_dosya, 2);
        assert!(sonuc.hatalar.is_empty());
        assert!(!kok.path().join("a").exists());
        assert!(kok.path().exists());
        assert!(dolu.join("x.txt").exists());
    }

    #[test]
    fn yinelenen_dosyalar_bulunur_ve_karantinaya_tasinir() {
        let kok = tempfile::tempdir().unwrap();
        let alt = kok.path().join("alt");
        fs::create_dir_all(&alt).unwrap();
        let icerik = "ayni icerik";
        fs::write(kok.path().join("kopya.txt"), icerik).unwrap();
        fs::write(alt.join("kopya.txt"), icerik).unwrap();
        fs::write(kok.path().join("farkli.txt"), b"baska icerik").unwrap();
        fs::write(kok.path().join("bos1.txt"), b"").unwrap();
        fs::write(kok.path().join("bos2.txt"), b"").unwrap();
        let servis = TemizleyiciServisi::yeni();
        let gruplar = servis.yinelenenleri_tara(kok.path(), 1).unwrap();
        assert_eq!(gruplar.len(), 1);
        assert_eq!(gruplar[0].boyut, icerik.len() as u64);
        assert_eq!(gruplar[0].dosyalar.len(), 2);
        assert!(servis
            .yinelenenleri_tara(kok.path(), icerik.len() as u64 + 1)
            .unwrap()
            .is_empty());
        let karantina = kok.path().join("karantina");
        fs::create_dir_all(&karantina).unwrap();
        fs::write(karantina.join("kopya.txt"), b"eski").unwrap();
        let sonuc = servis
            .yinelenenleri_karantinaya_tasi(kok.path(), &gruplar[0].dosyalar, &karantina)
            .unwrap();
        assert_eq!(sonuc.tasinan, 1);
        assert!(sonuc.hatalar.is_empty());
        assert!(karantina.join("kopya-1.txt").exists());
        let kalan = gruplar[0]
            .dosyalar
            .iter()
            .filter(|yol| Path::new(yol).exists())
            .count();
        assert_eq!(kalan, 1);
    }

    #[test]
    fn parcalama_dosyayi_siler() {
        let kok = tempfile::tempdir().unwrap();
        let dosya = kok.path().join("gizli.txt");
        fs::write(&dosya, b"hassas veri").unwrap();
        let mut izinler = fs::metadata(&dosya).unwrap().permissions();
        izinler.set_readonly(true);
        fs::set_permissions(&dosya, izinler).unwrap();
        let eski = kok.path().join("gizli.winnexus-sil");
        fs::write(&eski, b"korunmali").unwrap();
        let servis = TemizleyiciServisi::yeni();
        servis.parcala(&dosya).unwrap();
        assert!(!dosya.exists());
        assert!(eski.exists());
        assert_eq!(fs::read(&eski).unwrap(), b"korunmali".to_vec());
        assert_eq!(fs::read_dir(kok.path()).unwrap().count(), 1);
        assert!(servis.parcala(kok.path()).is_err());
    }
}
