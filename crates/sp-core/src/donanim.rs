pub use serde_json;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SistemOzeti {
    pub bilgisayar: String,
    pub isletim_sistemi: String,
    pub surum: String,
    pub uretici: String,
    pub model: String,
    pub bios: String,
    pub uptime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuBilgisi {
    pub ad: String,
    pub cekirdek: u32,
    pub mantiksal: u32,
    pub hiz_mhz: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuBilgisi {
    pub ad: String,
    pub bellek_mb: Option<u64>,
    pub surucu: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RamModulu {
    pub ad: String,
    pub boyut_mb: u64,
    pub hiz_mhz: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RamBilgisi {
    pub toplam_mb: u64,
    pub moduller: Vec<RamModulu>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiskBilgisi {
    pub ad: String,
    pub boyut_gb: f64,
    pub medya: String,
    pub saglik: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WlanProfili {
    pub ad: String,
    pub ssid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UacDurumu {
    pub etkin: bool,
    pub prompt: u32,
    pub secure_desktop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RdpDurumu {
    pub etkin: bool,
}
