#[derive(Debug, thiserror::Error)]
pub enum Hata {
    #[error("Kural dosyası okunamadı: {0}")]
    KuralOkunamadi(String),
    #[error("Kural doğrulaması başarısız: {0}")]
    KuralGecersiz(String),
    #[error("Kayıt defteri işlemi başarısız: {0}")]
    Registry(String),
    #[error("Doğrulama başarısız: {0}")]
    Dogrulama(String),
    #[error("Günlük işlemi başarısız: {0}")]
    Gunluk(String),
    #[error("Ayar işlemi başarısız: {0}")]
    Ayar(String),
    #[error("Geri alma başarısız: {0}")]
    GeriAlma(String),
    #[error("Yönetici izni gerekiyor: {0}")]
    Yetki(String),
    #[error("Desteklenmeyen işlem: {0}")]
    Desteklenmiyor(String),
    #[error("Dış araç hatası: {0}")]
    DisArac(String),
    #[error("Beklenmeyen hata: {0}")]
    Beklenmeyen(String),
}

pub type Sonuc<T> = Result<T, Hata>;
