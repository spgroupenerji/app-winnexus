# Kod Kalitesi ve Risk Denetimi — app-winnexus

- Görev: task-2 | Kapsam: 52 `.rs` dosyası, 32.389 satır (sp-app 13.080, sp-core 6.776, sp-win 12.533) | Commit: f16c39f
- Yöntem: `scripts/build.ps1` doğrulama hattı + regex tabanlı statik tarama + brace tabanlı fonksiyon/nesting ölçümü + komut, registry ve dosya silme odaklı manuel güvenlik incelemesi.
- Not: İnceleme f16c39f commit'i baz alınarak yapıldı; çalışma ağacındaki eşzamanlı değişiklikler (`motor.rs`, `scripts/`, `AGENTS.md`) kapsam dışıdır. `motor.rs` üzerinde bulgu 3-4'e yönelik bir düzeltme başlatılmış görünüyor.

## 1. Doğrulama hattı

| Komut | Sonuç | Kanıt |
|---|---|---|
| `fmt --all -- --check` | Temiz | çıkış 0, biçim farkı yok |
| `clippy --workspace --all-targets -- -D warnings` | Temiz | çıkış 0, uyarı yok |
| `test --workspace` | 338 test geçti, 0 başarısız | sp-core 153, sp-win 185, sp-app 0; çıkış 0 |

Derleyici ve lint kapıları temiz; aşağıdaki bulgular bu kapılardan geçen yapısal/davranışsal risklerdir.

## 2. Statik risk taraması

| Kalıp | Sayı | Not |
|---|---:|---|
| `unwrap()` | 571 | 569'u test kodunda; üretimde 2 (`sp-win/src/winget.rs:196,215`, Mutex kilidi) |
| `expect(` | 7 | 4'ü sabit regex (`validation.rs:23`, `adb.rs:7`, `winget.rs:149,154`), 2'si girişte (`main.rs:57,125`), 1 `build.rs:13` |
| `panic!` / `unreachable!` / `todo!` / `unimplemented!` | 0 | — |
| `unsafe {` | 3 | `registry.rs:143`, `winget.rs:139,153` |
| `#[allow(...)]` | 3 | `ptr_arg` (`bulucu.rs:173`), `permissions_set_readonly_false` (`onarim.rs:639`, `temizleyici.rs:241`); `allow(dead_code)` yok |
| `println!` / `eprintln!` / `dbg!` | 1 / 0 / 0 | yalnız `build.rs:2` |
| `let _ =` (sessiz yutma) | 83 | app 58, sp-win 25 (7'si test); kritik örnekler §3 |
| `.ok();` | 7 | tamamı sp-app (`app.rs:3377,3483,6039,7543,7554,7565,7568`) |
| TODO / FIXME / XXX | 0 | — |
| Devre dışı kod (`cfg(any())`, `if false`) | 0 | — |

## 3. En riskli 25 bulgu

| Öncelik | Bulgu | Kanıt (dosya:satır) | Öneri |
|---|---|---|---|
| Kritik | Registry `UninstallString` değeri `cmd /C` + `raw_arg` ile ham çalıştırılıyor; tek koruma CR/LF/NUL → `&`, `\|`, `%VAR%` kabuk metakarakteri enjeksiyonu | `sp-win/src/kaldirici.rs:80-84`; `sp-win/src/winget.rs:57-66`; `sp-core/src/kaldirici.rs:77-84` | `cmd` kaldır; exe yolu + argüman allowlist, MSI GUID doğrulaması, metakarakter reddi |
| Yüksek | Windows yerleşik `*\shell\runas` ("Yönetici olarak çalıştır") üzerine yazılıyor; kaldırmada anahtar tamamen siliniyor → kalıcı OS davranış kaybı | `sp-win/src/sahiplik.rs:42-55,57-66` | Benzersiz verb (`WinNexus.SahipligiAl`); önceki `command` değerini yedekle ve geri yükle |
| Yüksek | Eylem başarılı olduktan sonra günlük yazımı `?` ile hata verirse `adimlari_geri_al` çağrılmadan dönülüyor; UI da telafi etmiyor → izsiz kalıcı değişiklik | `sp-core/src/motor.rs:141-147,158-163`; `sp-app/src/app.rs:756-768` | Başarı günlüğü hatasını geri alma tetikleyicisine çevir; UI `Err` yolunda `motor.geri_al` çağır |
| Yüksek | Geri alma döngüsünde günlük yazımı `?` ile kesiliyor (kısmi telafi); `hatali` günlük yazımı başarısızsa `GeriAlma` hatası maskeleniyor | `sp-core/src/motor.rs:213-252` | Günlük hatalarını biriktir, telafiye devam et, sonuçta raporla; asıl hatayı koru |
| Yüksek | HKCU\Environment PATH kalıcı yedek olmadan yazılıyor; geri alma yalnız bellek içi `onceki` değerine bağlı; uzunluk/parça doğrulaması yok | `sp-win/src/sorun_giderme.rs:147-189,191-199`; `sp-app/src/app.rs:2213-2232` | Yazma öncesi dosyaya yedek + açılışta kurtarma; 32K sınırı; PATH parça doğrulaması |
| Orta | Platform-tools arşivi HTTPS dışında bütünlük doğrulaması olmadan indirilip `adb.exe` çalıştırılıyor (hash/imza yok) | `sp-win/src/android.rs:30-42,81-98` | SHA-256 sabit hash doğrulaması; doğrulanmadan arşiv açma/çalıştırma yok |
| Orta | Uygulama taraması hataları `.ok()` ile yutuluyor; kullanıcı boş liste görüyor, hata gösterilmiyor | `sp-app/src/app.rs:3377,3483` | `Err` yolunda `self.hata` doldur |
| Orta | HKCU bağlam menüsü blok anahtarına yedeksiz yazma; `klasik_baglam_menu_ayarla(false)` anahtarı yedeksiz siliyor | `sp-win/src/kayit_kisayol.rs:370-385,402-425` | Export/yedek + geri yükleme; silmede değer bazlı `delete_value` |
| Orta | Karantina manifest yazımı `let _ =` ile yutuluyor; taşınan kısayol kaydı tutulmazsa geri alınamaz | `sp-win/src/kayit_kisayol.rs:320-324` | Hatayı döndür; manifest yazılamazsa taşımayı geri al |
| Orta | Ayar migrasyon kopyası hatası yutuluyor | `sp-app/src/app.rs:484` | Hatayı kullanıcıya/günlüğe bildir |
| Orta | Explorer yeniden başlatmada `taskkill` sonucu yutuluyor; komut başarısız olsa da "yeniden başlatıldı" mesajı verilebilir | `sp-app/src/app.rs:2640-2656` | Sonucu kontrol et, hata durumunu raporla |
| Orta | Boş klasör silme hataları yutuluyor (`let _ = fs::remove_dir`) | `sp-win/src/onarim.rs:629-636` | Hataları sonuç listesine ekle |
| Orta | Üretim yolunda `Mutex` kilidi `unwrap` (zehirleme → panik) | `sp-win/src/winget.rs:196,215` | `lock().unwrap_or_else(PoisonError::into_inner)` veya `Sonuc` |
| Orta | Üretim `expect`: günlük writer kopyalama ve pencere açma panik üretir | `sp-app/src/main.rs:57,125` | Yedek writer'a düş; pencere hatasını raporla |
| Orta | Temizlik izinli kökleri `%TEMP%` (ortam değişkeniyle değiştirilebilir) ve zayıf `guvenli_kok_mu` (tek `Normal` bileşen) | `sp-win/src/temizleyici.rs:407-424` | Sabit kullanıcı profili yolları + derinlik sınırı; env tabanlı kökü kaldır |
| Orta | `komut_guvenli_mi` yalnız uzunluk + CR/LF/NUL denetliyor; `%`, `&`, `\|`, `>` serbest | `sp-core/src/kaldirici.rs:77-84` | Satır 1'deki allowlist yaklaşımı |
| Orta | Tüm PowerShell çağrılarında `-ExecutionPolicy Bypass` (13 çağrı) | `sp-win/src/sistem_yonetimi.rs:266-279`; `bakim.rs:150-162`; `temizleyici.rs:701-716`; `kayit_kisayol.rs:238-249` | İmzalı `.ps1` + `-File`; Bypass'ı kaldır |
| Orta | `npm` çağrıları `cmd.exe /c` üzerinden | `sp-win/src/sorun_giderme.rs:52-63` | `npm.cmd` tam yolu + argüman dizisi |
| Orta | PowerShell kanal okuma hatası yutuluyor; boş çıktı/yanıltıcı hata metni | `sp-win/src/temizleyici.rs:743-753` | Hatayı sonuca ekle |
| Orta | Monolit: app.rs 9.986 satır / 152 fonksiyon; 40 satır üstü 58, 3+ nesting 77, ikisi 42 (anayasa: 40 satır / 3 seviye) | `sp-app/src/app.rs:6251-6933` (temizlik_sayfasi, 683 satır) vb. | §5'teki 9 modüllü bölme planı |
| Orta | En derin fonksiyonlar: nesting 11 ve 7 | `sp-app/src/app.rs:3272-3429` (kaldirici_toplu_kaldir); `4856-5146` (sistem_onayla); `1208-1289` (winget_profilleri_tara) | Erken dönüş + alt fonksiyonlara ayır |
| Orta | `tests/` yalnız 32 PNG + 1 manifest; Rust entegrasyon testi yok | `tests/` | Kritik akışlar (kaldırma, geri yükleme, karantina) için integration test |
| Orta | sp-app 0 test; güvenlik doğrulayıcı `validation.rs` 0 test | `sp-app/src/*`; `sp-core/src/validation.rs` | Doğrulama tablosu testleri + UI işleyicileri için saf fn testleri |
| Düşük | `unsafe`: `MultiByteToWideChar` boyut dönüşümlerinde i32 taşma kontrolü yok | `sp-win/src/winget.rs:139-166` | `try_into()` + hata dönüşü |
| Düşük | İki `allow(clippy::permissions_set_readonly_false)` gerekçesiz; canonicalize+allowlist sonrası kabul edilebilir | `sp-win/src/onarim.rs:639-660`; `temizleyici.rs:241-260` | Gerekçeli allow veya ortak yardımcı fn |

## 4. Güvenlik incelemesi — doğru uygulanan kontroller

- Registry silmeden önce export + doğrulama: `sp-win/src/zorla.rs:88-134`, `artik.rs:111-145`, `kayit_kisayol.rs:428-451`.
- Dosya silmede canonicalize + izinli kök + reparse noktası atlama: `temizleyici.rs:430-470`, `onarim.rs:639-660`, `sp-core/src/zorla.rs:28-102`.
- Sürücü kurulumunda Authenticode imza doğrulaması: `android_driver.rs:106-111,189-200`; gömülü PowerShell değişkenlerinde sanitizasyon: `sp-core/src/onarim.rs:185-199`, `sistem_yonetimi.rs:573-604`, `bakim.rs:174-200`.
- Servis/süreç koruması: servis adı allowlist (`sistem_yonetimi.rs:560-571`), PID ≤ 4 reddi (`:248-253`).
- Kural doğrulama: registry yolunda `..`/`\0`/uzunluk reddi (`validation.rs:96-122`); tüm winget mutasyonlarında paket kimliği doğrulaması (`sp-core/src/winget.rs:188-196`).

## 5. app.rs monolit ölçümü (412.527 bayt)

Toplam 9.986 satır, 152 üst düzey fonksiyon; 40 satırı aşan 58 (%38), 3 seviyeden derin 77 (%51), her ikisi 42. Ölçüm: yorum/dize ayıklanmış brace sayımı.

| # | Fonksiyon | Satır aralığı | Uzunluk | Nesting |
|---:|---|---|---:|---:|
| 1 | temizlik_sayfasi | 6251-6933 | 683 | 4 |
| 2 | kaldirici_sayfasi | 3755-4344 | 590 | 6 |
| 3 | bakim_sayfasi | 7048-7500 | 453 | 4 |
| 4 | sorun_sayfasi | 2295-2596 | 302 | 5 |
| 5 | sistem_onayla | 4856-5146 | 291 | 7 |
| 6 | sistem_yonetimi_sayfasi | 5148-5435 | 288 | 6 |
| 7 | sistem_onay_metni | 4538-4798 | 261 | 4 |
| 8 | new | 469-724 | 256 | 3 |
| 9 | donanim_sayfasi | 7577-7787 | 211 | 3 |
| 10 | winget_sayfasi | 8940-9143 | 204 | 3 |
| 11 | winget_onerilenler_karti | 8026-8204 | 179 | 4 |
| 12 | winget_kurulu_bolumu | 8403-8571 | 169 | 5 |
| 13 | baglam_paneli | 2711-2874 | 164 | 4 |
| 14 | kaldirici_toplu_kaldir | 3272-3429 | 158 | 11 |
| 15 | winget_liste_bolumu | 8718-8868 | 151 | 5 |

**Bölme fizibilitesi:** 9 modül önerilir — `app/mod.rs` (durum + render iskeleti, ~700), `app/durum.rs` (App alanları/onay enumları, ~500), `app/sayfa/{kaldirici, winget, temizlik, bakim, sorun, sistem, donanim, ayarlar}.rs` (~600-1.500), `app/bilesen.rs` (kart/rozet/paneller, ~600). Bağımlılık yönü tek yönlü: `sayfa/*` → `durum.rs` + `sp-core`/`sp-win` servisleri; `mod.rs` → sayfa render dağıtımı; sayfalar birbirine bağımlı olmaz. GitNexus: 3.247 sembol/96 modül, "App" kümesi 106 sembol (%72 cohesion) — ağırlığın app.rs'te olduğunu doğrular.

## 6. Test kapsamı

| Kapsam | Test sayısı | Not |
|---|---:|---|
| sp-core | 153 | motor 5, winget 15, onarim 21, kayit_kisayol 15 vb.; `validation.rs` ve `model.rs` 0 |
| sp-win | 185 | winget 20, onarim 21, sistem_yonetimi 14, kayit_kisayol 15, zorla 7, registry 3 |
| sp-app | 0 | app.rs (9.986), app/android.rs (1.385), app/arac_merkezi.rs (1.565), main.rs test içermiyor |
| tests/ | 0 | 32 PNG + 1 manifest; Rust entegrasyon testi yok |

- Motorun hata yolları (günlük yazımı başarısızken telafi) test edilmiyor; §3'teki 3-4 numaralı bulgular bu boşlukta.
- Öneri: karantina/geri yükleme/kaldırma için 3-5 entegrasyon senaryosu; onay işleyicilerini saf fonksiyonlara ayırıp birim testi.

## 7. Sonuç

Biçim/lint/test kapıları temiz; ancak sessiz hata yutma, komut/registry mutasyonlarında telafi boşluğu ve app.rs monoliti kabul edilemez teknik borçtur. Öncelik: §3'teki ilk 5 bulgu (Kritik/Yüksek), ardından monolit bölme ve test kapsamı.
