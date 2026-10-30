# Bağımlılık ve Dokümantasyon Denetimi — WinNexus

**Görev:** task-3 · **Tarih:** 2026-10-30 · **Kapsam:** workspace bağımlılıkları, `Cargo.lock`, `crates/sp-app/build.rs`, sürüm/API uyumu.
**Ortam:** rustc/cargo 1.98.1 (MSVC, `scripts/build.ps1`); derleme çalıştırılmadı (paralel ajan derleme kilidi) — kanıtlar `cargo metadata`, `cargo tree -d`, crates.io API, context7/docs.rs ve kaynak taraması.

## Özet
- 12 doğrudan paket incelendi; hepsi güncel major hattında; yalnız `winreg` (0.55→0.56, kırıcı) ve `encoding_rs` (2 patch) geride.
- `cargo tree -d`: **48 paket adında çift sürüm** (105 giriş); doğrudan kontrol edilebilir tek çift `windows-sys 0.59/0.61` (kaynağı `winreg 0.55`).
- **13 bulgu:** 3 orta (F1–F3), 5 düşük (F4–F8), 5 bilgi/uyum (F9–F13).

## 1. Araç Zinciri ve MSRV
| Bileşen | Değer | Değerlendirme |
|---|---|---|
| rustc / cargo | 1.98.1 / 1.98.1 | Tek sürüm; MSVC hedefi |
| `rust-version` (workspace) | 1.98 | rustc 1.98.1 ile uyumlu ✓ |
| `rust-toolchain.toml` | channel 1.98.1, clippy+rustfmt | İçerik doğru; **izlenmiyor (untracked)** → commit edilmeli (F7) |
| Bağımlılık MSRV üst sınırı | 1.88 (adb_client, time, encoding_rs, image) | 1.98 ≥ 1.88 ✓; 1.98 gereksiz yüksek olabilir (doğrulanmalı) |

## 2. Doğrudan Bağımlılık Sürüm Tablosu
| Paket | Kullanılan | Güncel (crates.io) | Risk | Aksiyon |
|---|---|---|---|---|
| gpui-kit | 0.8.1 (`=0.8.1`) | 0.8.1 (2026-10-26) | Düşük | Pin korunmalı; yükseltme elle (gpui-pre `=0.3.10`) |
| adb_client | 3.2.3 | 3.2.3 (2026-08-02, MSRV 1.88) | Düşük | Feature seti doğru (`usb`); `framebuffer` kapalı |
| windows-sys | 0.61.2 | 0.61.2 (2025-10-06) | Düşük | Kullanılmayan `Security`/`Threading` feature'ları kaldır (F4) |
| directories | 6.0.0 | 6.0.0 (2025-01-12) | **Orta** | Kodda kullanım yok → bağımlılıktan çıkar (F2) |
| rust-embed | 8.13.0 | 8.13.0 (2026-10-07, MSRV 1.80) | Düşük | API uyumlu; aksiyon yok |
| toml | 0.8.23 | 1.1.8+spec-1.1.0 (2026-10-09) | Düşük | 1 major geri; yükseltme çift sürümü azaltmaz (F5) |
| thiserror | 2.0.21 | 2.0.21 (2026-09-23) | Düşük | — |
| time | 0.3.55 | 0.3.55 (2026-08-01, MSRV 1.88) | Düşük | API güncel (F12) |
| tempfile | 3.27.0 | 3.27.0 (2026-03-11) | Düşük | — |
| encoding_rs | 0.8.42 | 0.8.44 (2026-10-30) | Düşük | 2 patch geri: `cargo update -p encoding_rs` (F6) |
| strsim | 0.11.1 | 0.11.1 (2024-04-02) | Düşük | — |
| winreg | 0.55.0 | 0.56.0 (2026-03-14) | **Orta** | 0.56 kırıcı (`RegValue.bytes`: `Cow`) + `windows-sys 0.59` çiftini kapatır (F1) |

Not: `regex 1.13.1`, `serde 1.0.229`, `serde_json 1.0.154`, `tracing 0.1.44`, `tracing-subscriber 0.3.23` güncel; yanked sürüm kullanan paket yok (crates.io API).

## 3. Çift Sürümler (`cargo tree -d --workspace`, Windows hedefi)
| Paket | Sürümler | Kök neden | Aksiyon |
|---|---|---|---|
| windows-sys | 0.59.0 / 0.61.2 | 0.59 yalnız `winreg 0.55` | winreg 0.56 → 0.59 + windows-targets 0.52 zinciri kalkar |
| winreg | 0.55.0 / 0.56.0 | 0.56, `embed-resource 3.0.12` (gpui-pre build-dep) | Üstteki adımla tek sürüme iner |
| toml | 0.5.11 / 0.8.23 / 1.1.8 | 0.5←winres, 0.8←sp-core+rust-i18n, 1.1←embed-resource | winres kalkarsa 0.5 düşer; 0.8 rust-i18n'e bağlı |
| digest/sha2/block-buffer | 0.10.7 / 0.11.3 vb. | adb_client (sha1/sha2 0.10) vs rust-embed (sha2 0.11) | Kontrol dışı; dokümante |
| getrandom | 0.2.17 / 0.3.4 / 0.4.3 | adb_client-rsa, gpui-pre, tempfile | Kontrol dışı |
| windows / windows-core | 0.58.0 / 0.62.2 | gpui-base+gpui-component vs gpui-pre | gpui-kit yükseltmesi beklenir |
| base64 | 0.22.1 / 0.23.1 | adb_client vs rust-i18n/pem | Kontrol dışı |
| syn | 2.0.119 / 3.0.9 | proc-macro ekosistemi | Kontrol dışı |
| rand / rand_core | 0.8.8 / 0.9.5 / 0.10.3 | adb_client-rsa, gpui-pre | Kontrol dışı |
| usvg / resvg | 0.45.1 / 0.46.0 | gpui-component vs gpui-pre | Kontrol dışı |

## 4. API / Doküman Doğrulaması
| Paket | Kaynak kimliği | Doğrulanan kullanım | Sonuç |
|---|---|---|---|
| gpui-kit 0.8.1 | context7 `/longbridge/gpui-kit` (v0.8.0 indeksli), `/websites/rs_gpui-component_gpui_component`; docs.rs/gpui-kit/0.8.1 | `gpui_kit::{application,init,open_window}`, `component::*`, `prelude::FluentBuilder`; docs.rs deps: gpui-pre `=0.3.10`, gpui-component `^0.8.1` | Uyumlu; context7 indeksi 1 patch geride (F11) |
| adb_client 3.2.3 | context7 `/cocool97/adb_client`; docs.rs/adb_client/3.2.3 | `ADBServer::{new_from_path,start,get_device_by_name}`, `usb::ADBUSBDevice::autodetect`, `ADBDeviceExt::shell_command(mut self,..)`; `usb=[dep:rusb]`, default `framebuffer` | Uyumlu; `default-features=false, features=["usb"]` doğru (F10) |
| windows-sys 0.61.2 | docs.rs/windows-sys/0.61.2 + registry kaynağı | `IsUserAnAdmin()->BOOL`, `MultiByteToWideChar(..)`; UI_Shell/Globalization modülleri `Foundation` tiplerine referans veriyor | `Foundation` korunmalı; `Security`/`Threading` gereksiz (F4) |
| time 0.3.55 | docs.rs/time/0.3.55 | `OffsetDateTime`, `Rfc3339`, `format_description!`, `unix_timestamp_nanos`, `Date::from_calendar_date` | Deprecated kullanım yok (F12) |
| rust-embed 8.13.0 | docs.rs/rust-embed/8.13.0 | `#[derive(RustEmbed)]` + `#[folder]` + `iter()`/`get()` (v8 seri API) | Uyumlu |
| toml 1.1.8 (referans) | docs.rs/toml/1.1.8 | `from_str`, `to_string_pretty` mevcut | Yükseltme API açısından mümkün |
| directories 6 | — | Kodda hiç kullanım yok (grep 0 eşleşme) | **Kullanılmıyor** (F2) |

## 5. Kaynak Koddaki Kullanım Noktaları
| API | Dosya:satır | Not |
|---|---|---|
| adb_client | `crates/sp-win/src/android.rs:2-4, 203, 207, 220, 365-369` | `ADBServer`, `ADBUSBDevice`, `ADBDeviceExt`; yalnız burada |
| windows-sys | `crates/sp-win/src/winget.rs:135`, `crates/sp-win/src/registry.rs:143` | 2 çağrı: `MultiByteToWideChar`, `IsUserAnAdmin` |
| time | `sp-core/src/gunluk.rs:7-8,24-26`, `bulucu.rs:185-188`, `gelismis_arac.rs:299`, `dizinler.rs:47` | Güncel 0.3 API |
| rust-embed | `sp-core/src/kural_yukleyici.rs:4-8`, `sp-core/src/winget.rs:604-605` | `rules/tweaks` (7 dosya), `profiles` (4 dosya) mevcut ✓ |
| winreg | `sp-win/src/{registry,kayit_kisayol,bulucu,donanim,kaldirici,onarim,sistem_yonetimi,kurulum_izleme}.rs` | `RegValue` inşası: `registry.rs:38-50`, `kayit_kisayol.rs:376,409`, `bulucu.rs:607-617` |
| toml | `sp-core/src/ayarlar.rs:58,65`, `kural_yukleyici.rs:21`, `winget.rs:618` | `from_str` / `to_string_pretty` |
| directories | — | `sp-core/Cargo.toml:12`'de tanımlı, kod yok |

## 6. Tedarik Zinciri
- **Cargo.lock:** v4, git'te izleniyor (`git ls-files` ✓); 841 crates.io + 3 workspace paketi, **git bağımlılığı 0**, `[patch]` yok (F13).
- **Lisans:** 844/844 pakette lisans alanı dolu; GPL-only yok (`self_cell`: Apache-2.0 OR GPL-2.0-only → Apache seçilir). MPL-2.0 3 paket: `dwrote` (ikiliye girer), `cbindgen` (build-only), `option-ext` → NOTICE/üçüncü taraf lisans dosyası önerilir.
- **build.rs (`crates/sp-app/build.rs`):** `winres 0.1.12` ile `assets/winnexus.ico` (81 KB ✓), FileDescription/ProductName/Version/Copyright gömüyor; `CARGO_CFG_TARGET_OS=windows` koşullu, `WINNEXUS_BUILD_STAMP` rerun izliyor. winres son sürüm 2021-09-28 → **bakımsız**; `toml 0.5.11 + winapi 0.3.9` transitifini getiriyor (F3).
- **CI/governance:** `.github/workflows/ci.yml` fmt+clippy+test (windows-latest, 1.98.1), `.github/dependabot.yml` haftalık gruplu cargo. Eksik: `cargo-deny`/`cargo-audit` + `deny.toml` (advisory/lisans/ban/source kapısı yok) (F8).
- **İzlenmeyen dosyalar:** `.github/`, `rust-toolchain.toml`, `LICENSE`, `SECURITY.md` denetim anında untracked → commit edilmedikçe CI pinlemesi klonda yok (F7).

## 7. Bulgular (öncelik sıralı)
| # | Bulgu | Önem |
|---|---|---|
| F1 | `winreg 0.55→0.56`: `RegValue.bytes: Vec<u8>` → `Cow<[u8]>` ve serileştirme izinleri değişti; 8 inşa noktası uyarlanmalı; kazanç: `windows-sys 0.59` çifti kapanır | Orta |
| F2 | `directories 6.0.0` sp-core'da kullanılmıyor → kaldırılmalı (`dirs-sys`/`option-ext` zinciri küçülür) | Orta |
| F3 | `winres 0.1.12` (2021) bakımsız build-dep → `embed-resource 3` (ağaçta) veya `tauri-winres` | Orta |
| F4 | windows-sys'te `Win32_Security`, `Win32_System_Threading` doğrudan kullanılmıyor; `Win32_Foundation` modül bağımlılığı nedeniyle korunmalı (kaldırma `cargo check` ile doğrulanmalı) | Düşük |
| F5 | `toml 0.8.23` (1 major geri); ağaçta 3 major; rust-i18n 0.8 tuttuğu için yükseltme çift sürümü azaltmaz | Düşük |
| F6 | `encoding_rs 0.8.42 → 0.8.44` (2 patch; 2026-10-30 yayımlandı) | Düşük |
| F7 | `rust-toolchain.toml` içeriği doğru (1.98.1) ama izlenmiyor; CI 1.98.1'i elle sabitliyor | Düşük |
| F8 | CI'da tedarik zinciri kapısı yok (`cargo-deny`/`cargo-audit`, `deny.toml`) | Düşük |
| F9 | 48 pakette çift sürüm (105 giriş); çoğu gpui-kit zincirinden, kontrol dışı | Bilgi |
| F10 | adb_client 3.2.3 `usb` feature/API kullanımı doğrulandı (docs.rs + context7) | Bilgi |
| F11 | gpui-kit 0.8.1 güncel ve pinli; context7 indeksi v0.8.0 (doküman 1 patch geride) | Bilgi |
| F12 | windows-sys 0.61 / time 0.3.55 / rust-embed 8.13 API kullanımı sürümle uyumlu, deprecated yok | Bilgi |
| F13 | Cargo.lock izleniyor, git/patch bağımlılığı yok, 844/844 lisans dolu; MPL-2.0 bildirimi önerilir | Bilgi |

## 8. Aksiyon Planı
1. **P1:** `winreg 0.56` yükselt + `RegValue` uyarlaması → `windows-sys 0.59` çiftini kapat.
2. **P1:** `directories` bağımlılığını kaldır.
3. **P2:** `winres`→`embed-resource` geçişi.
4. **P2:** windows-sys gereksiz feature kırpma + `cargo check` doğrulaması.
5. **P3:** `cargo update -p encoding_rs`; CI'a `cargo-deny`; `rust-toolchain.toml`+`.github`+`LICENSE`/`SECURITY.md` commit.

## 9. Kaynaklar
- context7 (3 çağrı): `/longbridge/gpui-kit`, `/websites/rs_gpui-component_gpui_component`, `/cocool97/adb_client`.
- docs.rs: `gpui-kit/0.8.1`, `adb_client/3.2.3` (usb, ADBServer, ADBDeviceExt), `windows-sys/0.61.2`, `time/0.3.55`, `rust-embed/8.13.0`, `toml/1.1.8`; winreg CHANGELOG (0.56).
- crates.io API v1 (`/api/v1/crates/<ad>`): 20 paket sürüm tarihi/MSRV/yanked; yerel `cargo metadata`, `cargo tree -d`, `cargo tree --depth 1`, `git ls-files`.
