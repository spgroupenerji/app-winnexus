use crate::error::{Hata, Sonuc};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmDurumu {
    pub npm_yolu: String,
    pub prefix: String,
    pub path_kaynagi: Option<String>,
    pub global_paketler: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyScopeDurumu {
    pub scope: String,
    pub policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPolicyDurumu {
    pub scopes: Vec<PolicyScopeDurumu>,
    pub gpo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmAgentDurumu {
    pub memory_compression: bool,
    pub page_combining: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathEklemeSonucu {
    pub onceki: String,
    pub yeni: String,
    pub zaten_var: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyKapsami {
    CurrentUser,
    LocalMachine,
}

impl PolicyKapsami {
    pub fn metin(self) -> &'static str {
        match self {
            Self::CurrentUser => "CurrentUser",
            Self::LocalMachine => "LocalMachine",
        }
    }
}

pub fn npm_global_paketleri_ayristir(json: &str) -> Sonuc<Vec<String>> {
    let deger: Value = serde_json::from_str(json)
        .map_err(|hata| Hata::Beklenmeyen(format!("npm list JSON okunamadı: {hata}")))?;
    let mut paketler = deger
        .get("dependencies")
        .and_then(Value::as_object)
        .map(|nesne| nesne.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    paketler.sort();
    Ok(paketler)
}

pub fn execution_policy_ayristir(json: &str) -> Sonuc<ExecutionPolicyDurumu> {
    let deger: Value = serde_json::from_str(json)
        .map_err(|hata| Hata::Beklenmeyen(format!("ExecutionPolicy JSON okunamadı: {hata}")))?;
    let kayitlar = match deger {
        Value::Array(kayitlar) => kayitlar,
        Value::Object(_) => vec![deger],
        _ => {
            return Err(Hata::Beklenmeyen(
                "ExecutionPolicy çıktısı beklenen biçimde değil.".into(),
            ))
        }
    };
    let mut scopes = Vec::new();
    let mut gpo = false;
    for kayit in kayitlar {
        let Some(scope) = kayit.get("Scope").and_then(Value::as_str) else {
            continue;
        };
        let policy = kayit
            .get("ExecutionPolicy")
            .and_then(Value::as_str)
            .unwrap_or("Undefined");
        if matches!(scope, "MachinePolicy" | "UserPolicy") && policy != "Undefined" {
            gpo = true;
        }
        scopes.push(PolicyScopeDurumu {
            scope: scope.to_string(),
            policy: policy.to_string(),
        });
    }
    Ok(ExecutionPolicyDurumu { scopes, gpo })
}

pub fn mmagent_ayristir(json: &str) -> Sonuc<MmAgentDurumu> {
    let deger: Value = serde_json::from_str(json)
        .map_err(|hata| Hata::Beklenmeyen(format!("Get-MMAgent JSON okunamadı: {hata}")))?;
    let nesne = deger
        .as_object()
        .ok_or_else(|| Hata::Beklenmeyen("Get-MMAgent nesnesi bekleniyordu.".into()))?;
    Ok(MmAgentDurumu {
        memory_compression: nesne
            .get("MemoryCompression")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        page_combining: nesne
            .get("PageCombining")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

pub fn path_iceriyor(path_metni: &str, aranan: &str) -> bool {
    let aranan = normalize_path(&genislet(aranan));
    path_metni
        .split(';')
        .map(genislet)
        .any(|parca| !parca.trim().is_empty() && normalize_path(&parca) == aranan)
}

pub fn genislet(parca: &str) -> String {
    let mut metin = parca.trim().trim_matches('"').to_string();
    for (anahtar, deger) in [
        ("%APPDATA%", std::env::var("APPDATA").ok()),
        ("%LOCALAPPDATA%", std::env::var("LOCALAPPDATA").ok()),
        ("%USERPROFILE%", std::env::var("USERPROFILE").ok()),
    ] {
        if let Some(deger) = deger {
            metin = metin.replace(anahtar, &deger);
        }
    }
    metin
}

fn normalize_path(metin: &str) -> String {
    metin
        .trim()
        .trim_matches('"')
        .trim_end_matches('\\')
        .replace('/', "\\")
        .to_lowercase()
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn npm_global_json_ayristirir() {
        let json = r#"{"name":"lib","dependencies":{"typescript":{"version":"5.0.0"},"npm":{"version":"11.0.0"}}}"#;
        let paketler = npm_global_paketleri_ayristir(json).unwrap();
        assert_eq!(paketler, vec!["npm", "typescript"]);
    }

    #[test]
    fn execution_policy_ayristirir() {
        let json = r#"[{"Scope":"MachinePolicy","ExecutionPolicy":"Undefined"},{"Scope":"CurrentUser","ExecutionPolicy":"RemoteSigned"},{"Scope":"LocalMachine","ExecutionPolicy":"Restricted"}]"#;
        let durum = execution_policy_ayristir(json).unwrap();
        assert_eq!(durum.scopes.len(), 3);
        assert!(!durum.gpo);
        let json_gpo = r#"[{"Scope":"MachinePolicy","ExecutionPolicy":"AllSigned"}]"#;
        assert!(execution_policy_ayristir(json_gpo).unwrap().gpo);
    }

    #[test]
    fn mmagent_ayristirir() {
        let json = r#"{"MemoryCompression":true,"PageCombining":false}"#;
        let durum = mmagent_ayristir(json).unwrap();
        assert!(durum.memory_compression);
        assert!(!durum.page_combining);
    }

    #[test]
    fn path_icerik_kontrolu() {
        let appdata = std::env::var("APPDATA").unwrap();
        let path = format!(r"C:\Windows;{appdata}\npm;C:\Tools");
        assert!(path_iceriyor(&path, &format!(r"{appdata}\npm")));
        assert!(path_iceriyor(&path, "%APPDATA%\\npm"));
        assert!(!path_iceriyor(&path, r"C:\Başka"));
    }
}
