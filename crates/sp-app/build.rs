fn main() {
    println!("cargo:rerun-if-env-changed=WINNEXUS_BUILD_STAMP");
    let damga = std::env::var("WINNEXUS_BUILD_STAMP")
        .ok()
        .filter(|deger| damga_gecerli(deger))
        .unwrap_or_else(utc_damgasi);
    println!("cargo:rustc-env=WINNEXUS_BUILD_STAMP={damga}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let (yil, ay, gun, saat, dakika) = damga_alanlari(&damga);
        let mut kaynak = winres::WindowsResource::new();
        kaynak.set_icon("../../assets/winnexus.ico");
        kaynak.set("FileDescription", "SP GROUP WinNexus");
        kaynak.set("ProductName", "SP GROUP WinNexus");
        let dosya_surumu = format!("{yil}.{ay}.{gun}.{saat}{dakika}");
        let urun_surumu = format!("v{damga}");
        kaynak.set("FileVersion", &dosya_surumu);
        kaynak.set("ProductVersion", &urun_surumu);
        kaynak.set("LegalCopyright", "SP GROUP");
        kaynak
            .compile()
            .expect("Windows sürüm kaynakları derlenemedi");
    }
}

fn damga_gecerli(deger: &str) -> bool {
    deger.len() == 12 && deger.bytes().all(|bayt| bayt.is_ascii_digit())
}

fn damga_alanlari(damga: &str) -> (i64, u32, u32, u32, u32) {
    let alan = |baslangic: usize, uzunluk: usize| -> u32 {
        damga[baslangic..baslangic + uzunluk].parse().unwrap_or(0)
    };
    (
        i64::from(alan(0, 4)),
        alan(4, 2),
        alan(6, 2),
        alan(8, 2),
        alan(10, 2),
    )
}

fn utc_damgasi() -> String {
    let saniye = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|sure| sure.as_secs())
        .unwrap_or(0);
    let gun_sayisi = (saniye / 86_400) as i64;
    let (yil, ay, gun) = tarih(gun_sayisi);
    let saat = (saniye % 86_400) / 3_600;
    let dakika = (saniye % 3_600) / 60;
    format!("{yil:04}{ay:02}{gun:02}{saat:02}{dakika:02}")
}

fn tarih(gun_sayisi: i64) -> (i64, u32, u32) {
    let z = gun_sayisi + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let yil = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let gun = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let ay = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if ay <= 2 { yil + 1 } else { yil }, ay, gun)
}
