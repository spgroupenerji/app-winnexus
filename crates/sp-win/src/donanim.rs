use crate::winget::KomutCalistirici;
use sp_core::donanim::serde_json::{self, Value};
use sp_core::donanim::{
    CpuBilgisi, DiskBilgisi, GpuBilgisi, RamBilgisi, RamModulu, RdpDurumu, SistemOzeti, UacDurumu,
    WlanProfili,
};
use sp_core::error::{Hata, Sonuc};
use std::sync::Arc;
use std::time::Duration;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, REG_DWORD};
use winreg::RegKey;

const ZAMAN_ASIMI: Duration = Duration::from_secs(90);
const UAC_YOLU: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System";
const RDP_YOLU: &str = r"SYSTEM\CurrentControlSet\Control\Terminal Server";
const BAYT_MB: u64 = 1024 * 1024;
const BAYT_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const BILINMIYOR: &str = "Bilinmiyor";

const SISTEM_BETIGI: &str = r#"
$cs = Get-CimInstance Win32_ComputerSystem
$os = Get-CimInstance Win32_OperatingSystem
$bios = Get-CimInstance Win32_BIOS
$sure = (Get-Date) - $os.LastBootUpTime
if ($sure.TotalDays -ge 1) { $uptime = '{0} gün {1} saat' -f [int][math]::Floor($sure.TotalDays), $sure.Hours } else { $uptime = '{0} saat' -f [int][math]::Floor($sure.TotalHours) }
[pscustomobject]@{ Bilgisayar = [string]$cs.Name; IsletimSistemi = [string]$os.Caption; Surum = [string]$os.Version; Uretici = [string]$cs.Manufacturer; Model = [string]$cs.Model; Bios = ('{0} {1}' -f $bios.Manufacturer, $bios.SMBIOSBIOSVersion).Trim(); Uptime = $uptime } | ConvertTo-Json -Compress
"#;

const CPU_BETIGI: &str = r#"
Get-CimInstance Win32_Processor | Select-Object @{Name='Ad';Expression={$_.Name}},@{Name='Cekirdek';Expression={$_.NumberOfCores}},@{Name='Mantiksal';Expression={$_.NumberOfLogicalProcessors}},@{Name='Hiz';Expression={$_.MaxClockSpeed}} | ConvertTo-Json -Compress
"#;

const GPU_BETIGI: &str = r#"
Get-CimInstance Win32_VideoController | Select-Object @{Name='Ad';Expression={$_.Name}},@{Name='Bellek';Expression={$_.AdapterRAM}},@{Name='Surucu';Expression={$_.DriverVersion}} | ConvertTo-Json -Compress
"#;

const RAM_BETIGI: &str = r#"
Get-CimInstance Win32_PhysicalMemory | Select-Object @{Name='Ad';Expression={$_.DeviceLocator}},@{Name='Boyut';Expression={$_.Capacity}},@{Name='Hiz';Expression={$_.Speed}} | ConvertTo-Json -Compress
"#;

const DISK_BETIGI: &str = r#"
$diskler = Get-PhysicalDisk | Select-Object @{Name='Ad';Expression={$_.FriendlyName}},@{Name='Boyut';Expression={$_.Size}},@{Name='Medya';Expression={[string]$_.MediaType}},@{Name='Saglik';Expression={[string]$_.HealthStatus}}
ConvertTo-Json -InputObject @($diskler) -Compress
"#;

const DISK_YEDEK_BETIGI: &str = r#"
$diskler = Get-CimInstance Win32_DiskDrive | Select-Object @{Name='Ad';Expression={$_.Model}},@{Name='Boyut';Expression={$_.Size}},@{Name='Medya';Expression={[string]$_.MediaType}},@{Name='Saglik';Expression={[string]$_.Status}}
ConvertTo-Json -InputObject @($diskler) -Compress
"#;

const WLAN_BETIGI: &str = r#"
$kok = Join-Path $env:ProgramData 'Microsoft\Wlansvc\Profiles\Interfaces'
$liste = @()
if (Test-Path -LiteralPath $kok) {
  foreach ($dosya in (Get-ChildItem -LiteralPath $kok -Recurse -Filter *.xml -ErrorAction SilentlyContinue)) {
    $xml = $null
    try { $xml = [xml](Get-Content -LiteralPath $dosya.FullName -Raw -Encoding UTF8) } catch { $xml = $null }
    if ($null -ne $xml -and $null -ne $xml.WLANProfile) {
      $ad = [string]$xml.WLANProfile.name
      $ssid = [string]$xml.WLANProfile.SSIDConfig.SSID.name
      if (-not [string]::IsNullOrWhiteSpace($ad)) { $liste += [pscustomobject]@{ Ad = $ad; Ssid = $ssid } }
    }
  }
}
ConvertTo-Json -InputObject $liste -Compress
"#;

fn kisa_hata(metin: &str) -> String {
    metin
        .lines()
        .map(str::trim)
        .find(|satir| !satir.is_empty())
        .unwrap_or("bilinmeyen hata")
        .to_string()
}

fn json_degeri(json: &str) -> Sonuc<Value> {
    serde_json::from_str(json.trim())
        .map_err(|hata| Hata::Beklenmeyen(format!("Donanım çıktısı JSON olarak okunamadı: {hata}")))
}

fn kayitlar(json: &str) -> Sonuc<Vec<Value>> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(match json_degeri(json)? {
        Value::Array(kayitlar) => kayitlar,
        Value::Null => Vec::new(),
        deger => vec![deger],
    })
}

fn metin(nesne: &Value, alan: &str) -> Option<String> {
    nesne
        .get(alan)?
        .as_str()
        .map(str::trim)
        .filter(|deger| !deger.is_empty())
        .map(str::to_string)
}

fn sayi(nesne: &Value, alan: &str) -> Option<u64> {
    match nesne.get(alan)? {
        Value::Number(deger) => deger.as_u64().or_else(|| {
            deger
                .as_f64()
                .filter(|sayi| *sayi >= 0.0)
                .map(|sayi| sayi as u64)
        }),
        Value::String(deger) => deger.trim().parse().ok(),
        _ => None,
    }
}

fn u32_sayi(nesne: &Value, alan: &str) -> u32 {
    sayi(nesne, alan)
        .and_then(|deger| u32::try_from(deger).ok())
        .unwrap_or_default()
}

pub fn sistem_ozeti_ayristir(json: &str) -> Sonuc<SistemOzeti> {
    let deger = json_degeri(json)?;
    if !deger.is_object() {
        return Err(Hata::Beklenmeyen(
            "Sistem özeti çıktısı JSON nesnesi değil.".into(),
        ));
    }
    Ok(SistemOzeti {
        bilgisayar: metin(&deger, "Bilgisayar").unwrap_or_default(),
        isletim_sistemi: metin(&deger, "IsletimSistemi").unwrap_or_default(),
        surum: metin(&deger, "Surum").unwrap_or_default(),
        uretici: metin(&deger, "Uretici").unwrap_or_default(),
        model: metin(&deger, "Model").unwrap_or_default(),
        bios: metin(&deger, "Bios").unwrap_or_default(),
        uptime: metin(&deger, "Uptime").unwrap_or_default(),
    })
}

pub fn cpu_ayristir(json: &str) -> Sonuc<Vec<CpuBilgisi>> {
    Ok(kayitlar(json)?
        .into_iter()
        .filter_map(|kayit| {
            Some(CpuBilgisi {
                ad: metin(&kayit, "Ad")?,
                cekirdek: u32_sayi(&kayit, "Cekirdek"),
                mantiksal: u32_sayi(&kayit, "Mantiksal"),
                hiz_mhz: u32_sayi(&kayit, "Hiz"),
            })
        })
        .collect())
}

pub fn gpu_ayristir(json: &str) -> Sonuc<Vec<GpuBilgisi>> {
    Ok(kayitlar(json)?
        .into_iter()
        .filter_map(|kayit| {
            Some(GpuBilgisi {
                ad: metin(&kayit, "Ad")?,
                bellek_mb: sayi(&kayit, "Bellek")
                    .filter(|bayt| *bayt > 0)
                    .map(|bayt| bayt / BAYT_MB),
                surucu: metin(&kayit, "Surucu").unwrap_or_else(|| BILINMIYOR.to_string()),
            })
        })
        .collect())
}

pub fn ram_ayristir(json: &str) -> Sonuc<RamBilgisi> {
    let moduller = kayitlar(json)?
        .into_iter()
        .filter_map(|kayit| {
            let boyut_mb = sayi(&kayit, "Boyut").filter(|bayt| *bayt > 0)? / BAYT_MB;
            Some(RamModulu {
                ad: metin(&kayit, "Ad").unwrap_or_else(|| "Bellek".to_string()),
                boyut_mb,
                hiz_mhz: sayi(&kayit, "Hiz")
                    .and_then(|hiz| u32::try_from(hiz).ok())
                    .filter(|hiz| *hiz > 0),
            })
        })
        .collect::<Vec<_>>();
    Ok(RamBilgisi {
        toplam_mb: moduller.iter().map(|modul| modul.boyut_mb).sum(),
        moduller,
    })
}

pub fn disk_ayristir(json: &str) -> Sonuc<Vec<DiskBilgisi>> {
    Ok(kayitlar(json)?
        .into_iter()
        .filter_map(|kayit| {
            let boyut_bayt = sayi(&kayit, "Boyut").filter(|bayt| *bayt > 0)?;
            Some(DiskBilgisi {
                ad: metin(&kayit, "Ad").unwrap_or_else(|| BILINMIYOR.to_string()),
                boyut_gb: ((boyut_bayt as f64 / BAYT_GB) * 10.0).round() / 10.0,
                medya: metin(&kayit, "Medya").unwrap_or_else(|| BILINMIYOR.to_string()),
                saglik: metin(&kayit, "Saglik").unwrap_or_else(|| BILINMIYOR.to_string()),
            })
        })
        .collect())
}

pub fn wlan_profilleri_ayristir(json: &str) -> Sonuc<Vec<WlanProfili>> {
    Ok(kayitlar(json)?
        .into_iter()
        .filter_map(|kayit| {
            let ad = metin(&kayit, "Ad")?;
            Some(WlanProfili {
                ssid: metin(&kayit, "Ssid").unwrap_or_else(|| ad.clone()),
                ad,
            })
        })
        .collect())
}

pub fn uac_yorumla(
    enable_lua: Option<u32>,
    prompt: Option<u32>,
    secure_desktop: Option<u32>,
) -> UacDurumu {
    UacDurumu {
        etkin: enable_lua.unwrap_or(1) != 0,
        prompt: prompt.unwrap_or(5),
        secure_desktop: secure_desktop.unwrap_or(1) != 0,
    }
}

pub fn rdp_yorumla(deny_connections: Option<u32>) -> RdpDurumu {
    RdpDurumu {
        etkin: deny_connections == Some(0),
    }
}

fn dword_oku(yol: &str, ad: &str) -> Sonuc<Option<u32>> {
    let anahtar = match RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(yol, KEY_READ | KEY_WOW64_64KEY)
    {
        Ok(anahtar) => anahtar,
        Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(hata) => return Err(Hata::Registry(format!("{yol} açılamadı: {hata}"))),
    };
    match anahtar.get_raw_value(ad) {
        Ok(ham) if ham.vtype == REG_DWORD && ham.bytes.len() >= 4 => {
            Ok(Some(u32::from_le_bytes([
                ham.bytes[0],
                ham.bytes[1],
                ham.bytes[2],
                ham.bytes[3],
            ])))
        }
        Ok(_) => Ok(None),
        Err(hata) if hata.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(hata) => Err(Hata::Registry(format!("{yol}\\{ad} okunamadı: {hata}"))),
    }
}

#[derive(Clone)]
pub struct DonanimServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl DonanimServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    fn powershell(&self, etiket: &str, betik: &str) -> Sonuc<String> {
        let komut = format!(
            "$ErrorActionPreference='Stop'; & {{ {} }} | ForEach-Object {{ [regex]::Replace([string]$_, '[^\\u0000-\\u007F]', {{ param($m) '\\u{{0:x4}}' -f [int][char]$m.Value }}) }}",
            betik.trim()
        );
        let sonuc = self.calistirici.calistir(
            "powershell",
            &[
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-ExecutionPolicy".to_string(),
                "Bypass".to_string(),
                "-Command".to_string(),
                komut,
            ],
            ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "{etiket} okunamadı: {}",
                kisa_hata(&sonuc.stderr)
            )));
        }
        Ok(sonuc.stdout)
    }

    pub fn sistem_ozeti(&self) -> Sonuc<SistemOzeti> {
        sistem_ozeti_ayristir(&self.powershell("Sistem özeti", SISTEM_BETIGI)?)
    }

    pub fn cpu(&self) -> Sonuc<Vec<CpuBilgisi>> {
        cpu_ayristir(&self.powershell("İşlemci bilgisi", CPU_BETIGI)?)
    }

    pub fn gpu(&self) -> Sonuc<Vec<GpuBilgisi>> {
        gpu_ayristir(&self.powershell("Ekran kartı bilgisi", GPU_BETIGI)?)
    }

    pub fn ram(&self) -> Sonuc<RamBilgisi> {
        ram_ayristir(&self.powershell("Bellek bilgisi", RAM_BETIGI)?)
    }

    pub fn diskler(&self) -> Sonuc<Vec<DiskBilgisi>> {
        let birincil = self
            .powershell("Disk bilgisi", DISK_BETIGI)
            .and_then(|cikti| disk_ayristir(&cikti));
        match birincil {
            Ok(diskler) if !diskler.is_empty() => Ok(diskler),
            _ => disk_ayristir(&self.powershell("Disk bilgisi", DISK_YEDEK_BETIGI)?),
        }
    }

    pub fn wlan_profilleri(&self) -> Sonuc<Vec<WlanProfili>> {
        wlan_profilleri_ayristir(&self.powershell("WLAN profilleri", WLAN_BETIGI)?)
    }

    pub fn uac_durumu(&self) -> Sonuc<UacDurumu> {
        Ok(uac_yorumla(
            dword_oku(UAC_YOLU, "EnableLUA")?,
            dword_oku(UAC_YOLU, "ConsentPromptBehaviorAdmin")?,
            dword_oku(UAC_YOLU, "PromptOnSecureDesktop")?,
        ))
    }

    pub fn rdp_durumu(&self) -> Sonuc<RdpDurumu> {
        Ok(rdp_yorumla(dword_oku(RDP_YOLU, "fDenyTSConnections")?))
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::winget::KomutSonucu;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    const SISTEM_JSON: &str = r#"{"Bilgisayar":"SP-PC","IsletimSistemi":"Microsoft Windows 11 Pro","Surum":"10.0.26100","Uretici":"ASUSTeK COMPUTER INC.","Model":"X570","Bios":"American Megatrends 5003","Uptime":"2 gün 3 saat"}"#;

    struct SahteCalistirici {
        cevaplar: Mutex<VecDeque<KomutSonucu>>,
        cagrilar: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl SahteCalistirici {
        fn yeni(cevaplar: Vec<KomutSonucu>) -> Arc<Self> {
            Arc::new(Self {
                cevaplar: Mutex::new(VecDeque::from(cevaplar)),
                cagrilar: Mutex::new(Vec::new()),
            })
        }
    }

    fn cevap(stdout: &str, kod: i32) -> KomutSonucu {
        KomutSonucu {
            komut: "powershell".into(),
            kod,
            stdout: stdout.into(),
            stderr: if kod == 0 {
                String::new()
            } else {
                "Erişim engellendi.".into()
            },
        }
    }

    impl KomutCalistirici for SahteCalistirici {
        fn calistir(
            &self,
            program: &str,
            argumanlar: &[String],
            _zaman_asimi: Duration,
        ) -> Sonuc<KomutSonucu> {
            self.cagrilar
                .lock()
                .unwrap()
                .push((program.to_string(), argumanlar.to_vec()));
            self.cevaplar
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| Hata::DisArac("test cevabı yok".into()))
        }
    }

    #[test]
    fn sistem_ozetini_ayristirir() {
        let ozet = sistem_ozeti_ayristir(SISTEM_JSON).unwrap();
        assert_eq!(ozet.bilgisayar, "SP-PC");
        assert_eq!(ozet.surum, "10.0.26100");
        assert_eq!(ozet.uptime, "2 gün 3 saat");
    }

    #[test]
    fn eksik_sistem_alanlari_varsayilan_kalir() {
        let ozet = sistem_ozeti_ayristir(r#"{"Bilgisayar":"TEST"}"#).unwrap();
        assert_eq!(ozet.bilgisayar, "TEST");
        assert!(ozet.model.is_empty());
        assert!(ozet.uptime.is_empty());
        assert!(sistem_ozeti_ayristir("[]").is_err());
    }

    #[test]
    fn cpu_tek_nesne_ve_dizi_ayristirir() {
        let tek = r#"{"Ad":"Ryzen 7 5800X","Cekirdek":8,"Mantiksal":16,"Hiz":3800}"#;
        let coklu = format!("[{tek},{tek}]");
        assert_eq!(cpu_ayristir(tek).unwrap().len(), 1);
        assert_eq!(cpu_ayristir(&coklu).unwrap().len(), 2);
        assert_eq!(cpu_ayristir(tek).unwrap()[0].cekirdek, 8);
        assert_eq!(cpu_ayristir(tek).unwrap()[0].hiz_mhz, 3800);
    }

    #[test]
    fn cpu_adsiz_kayit_atlanir_ve_metin_sayi_kabul_edilir() {
        let json =
            r#"[{"Cekirdek":4},{"Ad":"Core i5","Cekirdek":"6","Mantiksal":"12","Hiz":"3200"}]"#;
        let cpu = cpu_ayristir(json).unwrap();
        assert_eq!(cpu.len(), 1);
        assert_eq!(cpu[0].cekirdek, 6);
        assert_eq!(cpu[0].mantiksal, 12);
    }

    #[test]
    fn gpu_bellek_yoksa_none_olur() {
        let json = r#"[{"Ad":"RTX 4070","Bellek":12884901888,"Surucu":"31.0.15.5222"},{"Ad":"Basic Adapter","Bellek":null,"Surucu":""}]"#;
        let gpu = gpu_ayristir(json).unwrap();
        assert_eq!(gpu[0].bellek_mb, Some(12288));
        assert_eq!(gpu[1].bellek_mb, None);
        assert_eq!(gpu[1].surucu, BILINMIYOR);
    }

    #[test]
    fn ram_modulleri_toplanir() {
        let json = r#"[{"Ad":"DIMM_A1","Boyut":17179869184,"Hiz":3200},{"Ad":"DIMM_B1","Boyut":17179869184,"Hiz":null}]"#;
        let ram = ram_ayristir(json).unwrap();
        assert_eq!(ram.toplam_mb, 32768);
        assert_eq!(ram.moduller.len(), 2);
        assert_eq!(ram.moduller[0].boyut_mb, 16384);
        assert_eq!(ram.moduller[1].hiz_mhz, None);
    }

    #[test]
    fn ram_tek_nesne_ve_bos_liste() {
        let tek = r#"{"Ad":"Slot0","Boyut":8589934592,"Hiz":2400}"#;
        assert_eq!(ram_ayristir(tek).unwrap().toplam_mb, 8192);
        assert_eq!(ram_ayristir("[]").unwrap().toplam_mb, 0);
    }

    #[test]
    fn disk_boyutu_gb_olarak_hesaplanir() {
        let json = r#"{"Ad":"Samsung SSD","Boyut":1000204886016,"Medya":"SSD","Saglik":"Healthy"}"#;
        let diskler = disk_ayristir(json).unwrap();
        assert_eq!(diskler.len(), 1);
        assert!((diskler[0].boyut_gb - 931.5).abs() < 0.01);
        assert_eq!(diskler[0].saglik, "Healthy");
    }

    #[test]
    fn disk_eksik_alanlar_toleransli() {
        let json = r#"[{"Boyut":21474836480}]"#;
        let diskler = disk_ayristir(json).unwrap();
        assert_eq!(diskler[0].ad, BILINMIYOR);
        assert_eq!(diskler[0].medya, BILINMIYOR);
        assert!((diskler[0].boyut_gb - 20.0).abs() < 0.01);
    }

    #[test]
    fn wlan_bos_liste_ve_tek_profil() {
        assert!(wlan_profilleri_ayristir("[]").unwrap().is_empty());
        let tek = r#"{"Ad":"EvAgi","Ssid":"Ev Ağı"}"#;
        let profiller = wlan_profilleri_ayristir(tek).unwrap();
        assert_eq!(profiller.len(), 1);
        assert_eq!(profiller[0].ad, "EvAgi");
        assert_eq!(profiller[0].ssid, "Ev Ağı");
    }

    #[test]
    fn uac_varsayilan_ve_ozel_degerler() {
        let varsayilan = uac_yorumla(None, None, None);
        assert_eq!(
            varsayilan,
            UacDurumu {
                etkin: true,
                prompt: 5,
                secure_desktop: true,
            }
        );
        let kapali = uac_yorumla(Some(0), Some(2), Some(0));
        assert!(!kapali.etkin);
        assert_eq!(kapali.prompt, 2);
        assert!(!kapali.secure_desktop);
    }

    #[test]
    fn rdp_engel_degeri_yorumlanir() {
        assert!(rdp_yorumla(Some(0)).etkin);
        assert!(!rdp_yorumla(Some(1)).etkin);
        assert!(!rdp_yorumla(None).etkin);
    }

    #[test]
    fn servis_powershell_ile_okur() {
        let calistirici = SahteCalistirici::yeni(vec![cevap(SISTEM_JSON, 0)]);
        let servis = DonanimServisi::yeni(calistirici.clone());
        let ozet = servis.sistem_ozeti().unwrap();
        assert_eq!(ozet.model, "X570");
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert_eq!(cagrilar[0].0, "powershell");
        assert!(cagrilar[0]
            .1
            .iter()
            .any(|parca| parca.contains("Win32_ComputerSystem")));
        assert!(cagrilar[0]
            .1
            .iter()
            .any(|parca| parca.contains("LastBootUpTime")));
    }

    #[test]
    fn disk_birincil_basarisizsa_yedege_duser() {
        let calistirici = SahteCalistirici::yeni(vec![
            cevap("", 1),
            cevap(
                r#"[{"Ad":"Yedek Disk","Boyut":107374182400,"Medya":"Fixed hard disk media","Saglik":"OK"}]"#,
                0,
            ),
        ]);
        let servis = DonanimServisi::yeni(calistirici.clone());
        let diskler = servis.diskler().unwrap();
        assert_eq!(diskler[0].ad, "Yedek Disk");
        let cagrilar = calistirici.cagrilar.lock().unwrap();
        assert!(cagrilar[0]
            .1
            .iter()
            .any(|parca| parca.contains("Get-PhysicalDisk")));
        assert!(cagrilar[1]
            .1
            .iter()
            .any(|parca| parca.contains("Win32_DiskDrive")));
    }

    #[test]
    fn hata_kodu_turkce_hata_dondurur() {
        let servis = DonanimServisi::yeni(SahteCalistirici::yeni(vec![cevap("", 1)]));
        let hata = servis.cpu().unwrap_err();
        assert!(hata.to_string().contains("İşlemci bilgisi okunamadı"));
        assert!(hata.to_string().contains("Erişim engellendi"));
    }
}
