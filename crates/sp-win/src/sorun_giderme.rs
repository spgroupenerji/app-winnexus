use crate::registry::WindowsAdaptoru;
use crate::winget::{KomutCalistirici, KomutSonucu};
use sp_core::adaptor::SistemAdaptoru;
use sp_core::error::{Hata, Sonuc};
use sp_core::model::{Kove, RegistryKonum, RegistryVeri};
use sp_core::sorun_giderme::{
    execution_policy_ayristir, mmagent_ayristir, npm_global_paketleri_ayristir, path_iceriyor,
    ExecutionPolicyDurumu, MmAgentDurumu, NpmDurumu, PathEklemeSonucu, PolicyKapsami,
};
use std::sync::Arc;
use std::time::Duration;

const KISA_ZAMAN_ASIMI: Duration = Duration::from_secs(30);
const TARAMA_ZAMAN_ASIMI: Duration = Duration::from_secs(120);

fn kisa_hata(metin: &str) -> String {
    metin
        .lines()
        .map(str::trim)
        .find(|satir| !satir.is_empty())
        .unwrap_or("bilinmeyen hata")
        .to_string()
}

fn yetki_hatasi(metin: &str) -> bool {
    let metin = metin.to_lowercase();
    metin.contains("erişim engellendi")
        || metin.contains("access is denied")
        || metin.contains("permissiondenied")
        || metin.contains("yetki")
}

#[derive(Clone)]
pub struct SorunGidermeServisi {
    calistirici: Arc<dyn KomutCalistirici>,
}

impl SorunGidermeServisi {
    pub fn yeni(calistirici: Arc<dyn KomutCalistirici>) -> Self {
        Self { calistirici }
    }

    fn calistir(
        &self,
        program: &str,
        argumanlar: &[String],
        zaman_asimi: Duration,
    ) -> Sonuc<KomutSonucu> {
        self.calistirici.calistir(program, argumanlar, zaman_asimi)
    }

    fn npm_calistir(&self, argumanlar: &[&str]) -> Sonuc<KomutSonucu> {
        let mut tam = vec!["/c".to_string(), "npm".to_string()];
        tam.extend(argumanlar.iter().map(|parca| parca.to_string()));
        self.calistir("cmd.exe", &tam, TARAMA_ZAMAN_ASIMI)
    }

    fn npm_yolu(&self) -> Sonuc<String> {
        let sonuc = self.calistir(
            "cmd.exe",
            &["/c".to_string(), "where".to_string(), "npm".to_string()],
            KISA_ZAMAN_ASIMI,
        )?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(
                "npm bulunamadı. Node.js kurulumunu ve PATH'i kontrol edin.".into(),
            ));
        }
        sonuc
            .stdout
            .lines()
            .map(str::trim)
            .find(|satir| !satir.is_empty())
            .map(str::to_string)
            .ok_or_else(|| Hata::DisArac("npm yolu boş döndü.".into()))
    }

    fn npm_prefix(&self) -> Sonuc<String> {
        let sonuc = self.npm_calistir(&["prefix", "-g"])?;
        if sonuc.kod != 0 {
            return Err(Hata::DisArac(format!(
                "npm prefix -g başarısız: {}",
                kisa_hata(&sonuc.stderr)
            )));
        }
        let prefix = sonuc.stdout.trim().to_string();
        if prefix.is_empty() {
            return Err(Hata::DisArac("npm global prefix boş döndü.".into()));
        }
        Ok(prefix)
    }

    fn registry_path(&self, kove: Kove, yol: &str) -> Option<String> {
        let konum = RegistryKonum {
            kove,
            yol: yol.to_string(),
            deger: "Path".to_string(),
            gorunum: None,
        };
        match WindowsAdaptoru.registry_oku(&konum) {
            Ok(Some(RegistryVeri::Sz(deger))) | Ok(Some(RegistryVeri::GenisSz(deger))) => {
                Some(deger)
            }
            _ => None,
        }
    }

    pub fn npm_durumu(&self) -> Sonuc<NpmDurumu> {
        let npm_yolu = self.npm_yolu()?;
        let prefix = self.npm_prefix()?;
        let global_sonuc = self.npm_calistir(&["ls", "-g", "--depth=0", "--json"])?;
        let global_paketler = if global_sonuc.kod == 0 {
            npm_global_paketleri_ayristir(&global_sonuc.stdout)?
        } else {
            Vec::new()
        };

        let kullanici = self
            .registry_path(Kove::Hkcu, r"Environment")
            .unwrap_or_default();
        let makine = self
            .registry_path(
                Kove::Hklm,
                r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
            )
            .unwrap_or_default();
        let surec = std::env::var("PATH").unwrap_or_default();

        let path_kaynagi = if path_iceriyor(&kullanici, &prefix) {
            Some("Kullanıcı PATH".to_string())
        } else if path_iceriyor(&makine, &prefix) {
            Some("Makine PATH".to_string())
        } else if path_iceriyor(&surec, &prefix) {
            Some("Süreç PATH".to_string())
        } else {
            None
        };

        Ok(NpmDurumu {
            npm_yolu,
            prefix,
            path_kaynagi,
            global_paketler,
        })
    }

    pub fn path_ekle(&self) -> Sonuc<PathEklemeSonucu> {
        let prefix = self.npm_prefix()?;
        let kullanici = self
            .registry_path(Kove::Hkcu, r"Environment")
            .unwrap_or_default();
        if path_iceriyor(&kullanici, &prefix) {
            return Ok(PathEklemeSonucu {
                onceki: kullanici.clone(),
                yeni: kullanici,
                zaten_var: true,
            });
        }

        let appdata_npm = std::env::var("APPDATA")
            .ok()
            .map(|appdata| format!(r"{appdata}\npm"));
        let eklenecek = if let Some(appdata_npm) = appdata_npm {
            if path_iceriyor(&prefix, &appdata_npm) {
                "%APPDATA%\\npm".to_string()
            } else {
                prefix
            }
        } else {
            prefix
        };
        let yeni = if kullanici.trim().is_empty() {
            eklenecek
        } else {
            format!("{};{}", kullanici.trim_end_matches(';'), eklenecek)
        };
        let konum = RegistryKonum {
            kove: Kove::Hkcu,
            yol: r"Environment".to_string(),
            deger: "Path".to_string(),
            gorunum: None,
        };
        WindowsAdaptoru.registry_yaz(&konum, &RegistryVeri::GenisSz(yeni.clone()))?;
        Ok(PathEklemeSonucu {
            onceki: kullanici,
            yeni,
            zaten_var: false,
        })
    }

    pub fn path_geri_al(&self, onceki: &str) -> Sonuc<()> {
        let konum = RegistryKonum {
            kove: Kove::Hkcu,
            yol: r"Environment".to_string(),
            deger: "Path".to_string(),
            gorunum: None,
        };
        WindowsAdaptoru.registry_yaz(&konum, &RegistryVeri::GenisSz(onceki.to_string()))
    }

    fn powershell(&self, script: &str) -> Sonuc<KomutSonucu> {
        self.calistir(
            "powershell.exe",
            &[
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                script.to_string(),
            ],
            TARAMA_ZAMAN_ASIMI,
        )
    }

    pub fn execution_policy_durumu(&self) -> Sonuc<ExecutionPolicyDurumu> {
        let sonuc = self.powershell(
            "Get-ExecutionPolicy -List | Select-Object @{Name='Scope';Expression={$_.Scope.ToString()}},@{Name='ExecutionPolicy';Expression={$_.ExecutionPolicy.ToString()}} | ConvertTo-Json -Compress",
        )?;
        if sonuc.kod != 0 {
            if yetki_hatasi(&sonuc.stderr) {
                return Err(Hata::Yetki(
                    "Execution policy bilgisi için yönetici izni gerekiyor.".into(),
                ));
            }
            return Err(Hata::DisArac(format!(
                "ExecutionPolicy okunamadı: {}",
                kisa_hata(&sonuc.stderr)
            )));
        }
        execution_policy_ayristir(&sonuc.stdout)
    }

    pub fn execution_policy_ayarla(&self, kapsam: PolicyKapsami) -> Sonuc<KomutSonucu> {
        if kapsam == PolicyKapsami::LocalMachine && !WindowsAdaptoru.yonetici_mi() {
            return Err(Hata::Yetki(
                "LocalMachine execution policy için yönetici izni gerekiyor.".into(),
            ));
        }
        let script = format!(
            "Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope {} -Force",
            kapsam.metin()
        );
        let sonuc = self.powershell(&script)?;
        if sonuc.kod != 0 {
            let alt = sonuc.stderr.to_lowercase();
            if alt.contains("group policy") || alt.contains("grup ilkesi") {
                return Err(Hata::Desteklenmiyor(
                    "Execution policy bir grup ilkesi tarafından yönetiliyor; GPO üzerinden değiştirin.".into(),
                ));
            }
            if yetki_hatasi(&sonuc.stderr) {
                return Err(Hata::Yetki(
                    "Execution policy değiştirmek için yönetici izni gerekiyor.".into(),
                ));
            }
            return Err(Hata::DisArac(format!(
                "ExecutionPolicy ayarlanamadı: {}",
                kisa_hata(&sonuc.stderr)
            )));
        }
        Ok(sonuc)
    }

    pub fn mmagent_durumu(&self) -> Sonuc<MmAgentDurumu> {
        let sonuc = self.powershell(
            "Get-MMAgent | Select-Object MemoryCompression,PageCombining | ConvertTo-Json -Compress",
        )?;
        if sonuc.kod != 0 {
            if yetki_hatasi(&sonuc.stderr) {
                return Err(Hata::Yetki(
                    "Bellek sıkıştırma bilgisi için yönetici izni gerekiyor.".into(),
                ));
            }
            return Err(Hata::DisArac(format!(
                "Get-MMAgent başarısız: {}",
                kisa_hata(&sonuc.stderr)
            )));
        }
        mmagent_ayristir(&sonuc.stdout)
    }

    pub fn mmagent_ayarla(&self, acik: bool) -> Sonuc<KomutSonucu> {
        if !WindowsAdaptoru.yonetici_mi() {
            return Err(Hata::Yetki(
                "Bellek sıkıştırma ayarı için yönetici izni gerekiyor.".into(),
            ));
        }
        let komut = if acik {
            "Enable-MMAgent -mc -Confirm:$false"
        } else {
            "Disable-MMAgent -mc -Confirm:$false"
        };
        let sonuc = self.powershell(komut)?;
        if sonuc.kod != 0 {
            if yetki_hatasi(&sonuc.stderr) {
                return Err(Hata::Yetki(
                    "Bellek sıkıştırma ayarı için yönetici izni gerekiyor.".into(),
                ));
            }
            return Err(Hata::DisArac(format!(
                "MMAgent ayarlanamadı: {}",
                kisa_hata(&sonuc.stderr)
            )));
        }
        Ok(sonuc)
    }
}

#[cfg(test)]
mod testler {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    struct KuyrukCalistirici {
        cevaplar: Mutex<VecDeque<KomutSonucu>>,
        cagrilar: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl KomutCalistirici for KuyrukCalistirici {
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

    fn cevap(stdout: &str, kod: i32) -> KomutSonucu {
        KomutSonucu {
            komut: "test".into(),
            kod,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    #[test]
    fn npm_durumunu_okur() {
        let calistirici = Arc::new(KuyrukCalistirici {
            cevaplar: Mutex::new(VecDeque::from([
                cevap("C:\\Node\\npm.cmd\n", 0),
                cevap("C:\\Test\\npm\n", 0),
                cevap(r#"{"dependencies":{"npm":{"version":"11.0.0"}}}"#, 0),
            ])),
            cagrilar: Mutex::new(Vec::new()),
        });
        let servis = SorunGidermeServisi::yeni(calistirici);
        let durum = servis.npm_durumu().unwrap();
        assert_eq!(durum.prefix, "C:\\Test\\npm");
        assert_eq!(durum.global_paketler, vec!["npm"]);
    }

    #[test]
    fn execution_policy_ayar_komutu_dogru() {
        let calistirici = Arc::new(KuyrukCalistirici {
            cevaplar: Mutex::new(VecDeque::from([cevap("", 0)])),
            cagrilar: Mutex::new(Vec::new()),
        });
        let servis = SorunGidermeServisi::yeni(calistirici.clone());
        if servis
            .execution_policy_ayarla(PolicyKapsami::CurrentUser)
            .is_ok()
        {
            let cagri = &calistirici.cagrilar.lock().unwrap()[0];
            assert!(cagri.1.iter().any(|parca| parca.contains("RemoteSigned")));
            assert!(cagri.1.iter().any(|parca| parca.contains("CurrentUser")));
        }
    }
}
