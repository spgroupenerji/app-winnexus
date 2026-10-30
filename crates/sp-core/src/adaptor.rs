use crate::error::Sonuc;
use crate::model::{RegistryKonum, RegistryVeri};
use crate::os::Surum;
use std::sync::Arc;

pub trait SistemAdaptoru: Send + Sync {
    fn registry_oku(&self, konum: &RegistryKonum) -> Sonuc<Option<RegistryVeri>>;
    fn registry_yaz(&self, konum: &RegistryKonum, veri: &RegistryVeri) -> Sonuc<()>;
    fn registry_sil(&self, konum: &RegistryKonum) -> Sonuc<()>;
    fn yonetici_mi(&self) -> bool;
}

pub type AdaptorRef = Arc<dyn SistemAdaptoru>;

#[derive(Debug, Clone)]
pub struct SistemBilgisi {
    pub surum: Surum,
    pub yonetici: bool,
}
