# WinNexus Arayüz, Tema ve Sürüm Doğrulama Raporu

| Alan | Değer |
|---|---|
| Tarih | 2026-10-31 |
| Durum | Tamamlandı |
| Kapsam | Pastel tema (açık/koyu), üst panel hızlı eylemleri, tek tıkla sistem taraması, tercih içe/dışa aktarma, 44 px hedefler, sürüm damgası |
| Yöntem | Gerçek uygulama (--test) + UI Automation + System.Drawing; `scripts/ui-palet.ps1` |

## 1. Yapılanlar

- **Tema:** stok siyah-beyaz tema yerine iki pastel tema tanımlandı (`crates/sp-app/src/tema.rs`): Gündüz (krem #F7F3EC zemin, adaçayı #3F6B57 birincil), Gece (#1D2320 zemin, nane #9CCBB2 birincil). Sistem/Açık/Koyu seçimi üst paneldedir.
- **Üst panel:** tema seçici, tercih içe/dışa aktarma, geri al ve Ayarlar 44 × 44 px hedeflerle tek satırda; ikinci satır sayfa başlığı + sekmeler + birincil eylem.
- **Ayarlar:** Tema kartı kaldırıldı; yerine "Sistem Taraması", "Tercihler", "Veri ve Sürüm" ve "Yasal Uyarı" kartları geldi. Tema üst çubuktan seçilir.
- **Tek tıkla tarama:** `Tüm Sistemi Tara` 20 adımlık sıralı kuyruğu başlatır (kurallar, winget, kurulu uygulamalar, servisler, donanım, temizlik, kayıt defteri, kısayollar, bakım listeleri, Windows güncellemeleri, tarayıcı uzantıları, bağlantılar). Adım başına 180 sn zaman aşımı ve `Taranıyor: n/20 · adım` ilerlemesi vardır (`crates/sp-app/src/app/tarama.rs`).
- **Tercih taşıma:** Ayarlar JSON olarak dışa/içe aktarılır (`crates/sp-core/src/ayarlar.rs`, sürümlü `TercihDosyasi`); içe aktarımda tema, winget onayları ve ADB yolu uygulanır, yasal onay korunur.
- **44 px:** Üst çubuk, tema seçici, bölüm/sekme düğmeleri, Ayarlar ve Genel Bakış birincil eylemleri ile modül "Aç" düğmeleri 44 × 44 px'tir (`DOKUNMA_HEDEFI_PX`, `dokunma_hedefi`). Yoğun sayfa içi işlem düğmeleri bileşen varsayılanı olan 32 px yükseklikte bırakıldı; bu boyut WCAG 2.2 2.5.8 AA (24 × 24) tabanını karşılar. Gerekçe: tüm düğmelere zorunlu yükseklik verildiğinde GPUI yerel katmanında `0xC000041D` çökmesi üretildi; kök neden GPUI bileşen/layout katmanında olup ürün kodunda güvenli karşılığı 44 px'i gezinme ve birincil eylemlerle sınırlamaktır.
- **Sürüm:** Sürüm artık tek kaynaktan `vYYYYMMDDHHMM` damgasıyla üretilir (`crates/sp-app/build.rs`: geliştirme derlemesinde UTC damgası, release hattında `release.ps1` ortam değişkeni); aynı damga Windows dosya sürümünde, uygulama içi "Sürüm" alanında ve release dosya adında kullanılır. Sabit `0.1.0.0` sürüm kaynağı kaldırıldı.
- **Koyu tema başlık çubuğu:** Windows yerel başlık çubuğu DWM immersive dark mode ile koyu temaya uyumlandı (`crates/sp-win/src/sistem.rs`).
- **Yoğunluk:** İçerik dolgusu 16 px, kart aralığı 16 px (4 px bareminde simetrik); eski ikinci gezinme satırı ve büyük başlık bloğu kaldırıldı, Genel Bakış modül vitrini 3 sütun oldu.

## 2. Kanıt

- `docs/kanit/ui-palet-gunduz-genel.png`, `ui-palet-gunduz-ayarlar.png`, `ui-palet-gece-ayarlar.png`, `ui-palet-gece-genel.png`: açık/koyu tema ve yeni Ayarlar yerleşimi.
- `docs/kanit/ui-tarama-suruyor.png`: "Tüm Sistemi Tara" sonrası 20 adımlık kuyruk ilerlemesi.
- `docs/kanit/ui-baslik-gece.png`: koyu temada yerel başlık çubuğu.
- Otomatik kapılar: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` temiz.
- Ölçüm: üst çubuk ve ana eylemlerde UI Automation ile 44–162 px genişlik, 44 px yükseklik doğrulandı.

## 3. Bilinen Sınırlar

- "Sistem" tema modu seçim anındaki işletim sistemi görünümünü izler; uygulama açıkken işletim sistemi teması değişirse canlı takip yoktur.
- Sayfa içi yoğun işlem düğmeleri 32 px'tir (yukarıdaki GPUI çökme gerekçesi).
- Winget/Temizlik sayfalarındaki eski üst boşluklar bu geçişte düzenlenmedi.

## 4. Yeniden Üretim

```powershell
cargo build -p sp-app
powershell -ExecutionPolicy Bypass -File scripts/ui-palet.ps1
```

Release sürümü: `scripts/release.ps1` (damga, manifest, SHA-256 ve `release/` artefaktı).
