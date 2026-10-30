# WinNexus — Harita, Denetim ve İyileştirme Raporu

**Tarih:** 2026-10-30 · **Depo:** `C:\01-github\app-winnexus` · **Baz commit:** `f16c39f` "ilk commit" (main = origin/main) · **İndeks:** gitnexus/codegraph güncel (HEAD)

## 0. Yönetici özeti

Depo derlenebilir, 341 testi geçen, clippy/fmt kapıları temiz bir Rust/GPUI masaüstü uygulaması. Ancak **depo ve tedarik zinciri hijyeni zayıftı**: çalışan bir GitHub PAT'i git yapılandırmasında düz metin duruyordu, 32,4 MB derleme çıktısı ve çalışma zamanı ayarı git'te izleniyordu, CI/lisans/güvenlik politikası yoktu. Kod tarafında en önemli kusur, sistem mutasyon kuyruğunun (**IsMotoru**) günlük yazımı başarısız olduğunda telafiyi atlamasıydı; bu düzeltildi ve iki regresyon testiyle kilitlendi. Bağlam haritaları (gitnexus, codegraph, graphify, trail) mimari merkezleri doğruladı: tek monolit `app.rs` (~9,8 bin satır) ve `WinNexusApp` düğümü (240 kenar).

| Alan | Önce | Şimdi | Kalan iş |
|---|---|---|---|
| Git izleme hijyeni | 2/10 | 7/10 | 32,4 MB exe + 4,3 MB PNG + `$null` izlemeden çıkarılmalı (onay) |
| Sır yönetimi | 2/10 | 6/10 | PAT düz metindi → URL temizlendi; **token rotasyonu kullanıcıda** |
| GitHub yönetişimi | 1/10 | 7/10 | CI + Dependabot + LICENSE eklendi; ruleset/secret scanning admin gerektirir |
| Kod kalitesi | 7/10 | 8/10 | fmt/clippy/test temiz; UninstallString enjeksiyonu ve PATH yedeği açık |
| Mimari | 6/10 | 6/10 | `app.rs` bölünmesi plan onayı bekliyor |
| Bağımlılıklar | 8/10 | 9/10 | `directories` kaldırıldı; `winreg 0.56` ve `winres → embed-resource` planlı |

**Yeni doğrulama:** `fmt --all -- --check` = 0 · `clippy --workspace --all-targets -- -D warnings` = 0 (uyarısız) · `test --workspace` = **341 test geçti** (sp-core 155, sp-win 186, sp-app 0).

## 1. Kritik güvenlik — çözüldü / rotasyon bekliyor

| # | Bulgu | Kanıt | Durum |
|---|---|---|---|
| S1 | `.git/config` remote URL'de **canlı GitHub PAT** (`github_pat_…`, 93 karakter, `git ls-remote` başarılı) | `git remote -v`, `gh api` | **URL tokensiz hale getirildi** (`https://github.com/spgroupenerji/app-winnexus.git`); eski token iptal edilmeli |
| S2 | PAT, ilk keşif çıktısında göründü (oturum günlüğü) | araç çıktısı | **Rotasyon zorunlu** — GitHub > Settings > Developer settings > PAT |
| S3 | `runas` bağlam menüsü fiili eziliyor ve kaldırmada siliniyordu; makinede önceden kurulu "Take Ownership" tweak'i vardı | `sp-win/src/sahiplik.rs:41-66`; `reg query HKCR\*\shell\runas` → `Take Ownership` | **Düzeltildi:** benzersiz fiil `WinNexus.SahipligiAl`; `runas` artık yazılmıyor/silinmiyor + test |

## 2. Git izleme hijyeni — "takip edilmesi / edilmemesi gerekenler"

Denetim: 138 izlenen dosya, 40,08 MB, tek pack 15,89 MiB (`git ls-files`, `ls-tree -r -l`, `count-objects -vH`). Ayrıntı: [git-hijyen raporu](git-hijyen.md).

| Dosya | Boyut | Karar | Durum |
|---|---|---|---|
| `release/WinNexus_beta-v202610301819.exe` | 32.369.152 B | **İzlenmemeli** — `scripts/release.ps1` üretir; GitHub Releases asset'i olmalı | **Index'ten çıkarıldı**; indirme hattı eklendi |
| `release/settings.toml` | 134 B | **İzlenmemeli** — çalışma zamanı tercihi (tema/dil/onaylar) | **Index'ten çıkarıldı**, ignore eklendi |
| `$null` | 91 B | **İzlenmemeli** — cmd yönlendirme artığı (blob `6ec3482`) | **Index'ten çıkarıldı**; ortamda otomatik silinme nedeni belirsiz (doğrulanmalı) |
| `tests/ui-*.png` (32 adet) | 5,94 MB | 8'i README'de kullanılıyor (`README.md:130-145`), 24'ü referanssız | **24'ü izlemeden çıkarıldı**; 8'i izleniyor (ignore istisnalarıyla) |
| `tests/exe-icon.png`, `tests/asinvoker.manifest` | 3,1 KB | Kod/betiklerde referans yok (orphan) | İsteğe bağlı temizlik |
| `.gitnexus/` (84,8 MB), `.codegraph/` (10,4 MB), `graphify-out/`, `graft/` | — | Yerel indeks/araç çıktısı; yeniden üretilebilir | `.gitignore`'a eklendi |
| `CLAUDE.md`, `.claude/*` | 3,6 KB | gitnexus üretimi; `.claude/skills` istisna (izlenebilir) | ignore + istisna eklendi |
| `AGENTS.md` | 3,6 KB | Proje ajan talimatı (GitNexus bloğu korunarak yeniden yazıldı) | İzlenecek |
| `LICENSE`, `.github/workflows/ci.yml`, `.github/dependabot.yml`, `SECURITY.md`, `.editorconfig`, `rust-toolchain.toml`, `release/README.md` | — | Eksikti | **Eklendi** (Apache-2.0 metni resmî kaynaktan) |

**Zorunlu minimum:** 32,4 MB exe HEAD'de olduğu sürece klon/pack boyutu ve GitHub uyarıları şişer. Geçmiş temizliği seçenekleri:
- **A (önerilen):** `git rm --cached` + `git commit --amend` + `git push --force-with-lease` — tek commit olduğu için temiz.
- **B:** Düzeltme commit'i — exe tarihte kalır (pack ~15,9 MiB).
- **C:** Yalnız çalışma ağacı — depo etkilenmez.

## 3. GitHub yönetişimi (API kanıtı)

| Kontrol | Sonuç |
|---|---|
| Görünürlük | **public** (`private:false`), `archived:false`, tek kullanıcı `spgroupenerji` |
| Workflows | **0** → CI eklendi (`ci.yml`: fmt + clippy + test, Windows) |
| Lisans | `null` → **LICENSE eklendi** (Cargo.toml ile uyumlu Apache-2.0) |
| Dependabot alerts | **kapalı** → `dependabot.yml` eklendi; Ayarlar'dan alerts/security updates açılmalı |
| Branch protection / rulesets / secret scanning / actions izinleri / deploy keys | PAT yetkisi 403: **doğrulanmalı**; admin token ile ruleset + secret scanning önerilir |
| Wiki / Issues | wiki açık (kullanılmıyorsa kapatılmalı), issues açık |
| Tag / Release / hook / collaborator | 0 / 0 / doğrulanamadı / yalnız sahip |

## 4. Mimari harita (araç kanıtlı)

| Krate | Dosya | Satır (yaklaşık) | Rol |
|---|---|---|---|
| `sp-core` | 27 | 6,2 bin | Domain, TOML kural motoru (41 kural), modeller, doğrulama, günlük, karantina |
| `sp-win` | 20 | 11,7 bin | Windows adaptörleri: registry, PowerShell/sc/taskkill, winget, Android, DISM/SFC |
| `sp-app` | 5 | 12,8 bin | GPUI arayüzü; `app.rs` **9.747 satır / 152 fn** (monolit), `app/android.rs`, `app/arac_merkezi.rs` |

- **Döngü yok:** gitnexus `check {cycles:true}` → `status: clean`, 0 bileşen.
- **Merkez düğümler (graphify `god-nodes`):** `WinNexusApp` (240 kenar), `KomutSonucu` (85), `KomutCalistirici` (56), `SistemOnay` (40), `WingetServisi` (32), `RegistryVeri` (31).
- **Çağrı yoğunluğu (gitnexus cypher):** `new` (app.rs) 278 gelen çağrı, `bolum_karti` 43, `modul_satiri` 28, `sistem_onay_iste` 27.
- **En uzun akışlar:** `Icerik → Os_uygun` (8 adım), `Icerik → Adimlari_geri_al` (7), `Winget_sayfasi → Komut_metni` (7).
- **Clusters:** 96 modül; yalnız "App" adlandırılmış (106 sembol, %72 uyum), kalanı `Cluster_N` — isimlendirme zayıf ama işlevsel.
- **README doğrulaması:** "41 ince ayar" = TOML kuralları 3+8+2+1+2+24+1 = **41 ✓**; "21 araç" = `AracKaydi` dizileri 6+4+4+… = **21 ✓**; tüm `kanit` yolları mevcut ✓.
- **Monolit planı:** `app.rs` için 9 modüllü bölme (temizlik/bakım/kaldırıcı/winget/android/araç merkezi/god mode/profil/çekirdek) — **mimari revizyon onayı gerektirir**.

## 5. Kod kusurları

Ayrıntılı 25 bulgu: [kod kalitesi raporu](kod-kalitesi.md). Bu oturumda **düzeltilen** kusurlar:

| # | Kusur | Düzeltme | Doğrulama |
|---|---|---|---|
| K1 | Başarılı mutasyondan sonra günlük yazımı `?` ile dönerse telafi atlanıyor; geri alma döngüsü ilk hatada kesiliyor; `hatali` günlük hatası `GeriAlma` hatasını maskeliyordu (`motor.rs:138-252`; gitnexus impact: **HIGH**, 7 etkilenen) | `telafi_hatasi` + hata biriktiren geri alma; asıl hata korunur; `Hata::Gunluk/GeriAlma` birleşik mesaj | 2 yeni test: `geri_alma_basarisizsa_diger_adimlar_denenir`, `gunluk_yazilamazsa_uygulama_durur` |
| K2 | Yerleşik `runas` fiilinin ezilmesi/silinmesi (işletim sistemi davranış kaybı) | Benzersiz `WinNexus.SahipligiAl` fiili; `menu_kaldir` yalnız kendi anahtarını siler | `menu_anahtarlari_yerlesik_fiilleri_ezmez` testi |
| K3 | Sahiplik hata günlüğü `%LOCALAPPDATA%`'ya yazıyordu (README "portatif, yeni APPDATA verisi yok" iddiasıyla çelişki) | `<exe>/veri/gunluk/sahiplik.log` | Kod incelemesi |
| K4 | 9 `.ps1` dosyası BOM'suz UTF-8; Windows PowerShell 5.1 Türkçe metni bozuyordu | UTF-8 BOM eklendi (13'ü zaten BOM'lu) | `Get-Content`/5.1 doğrulaması: "bulunamadı", "sürüm" doğru |
| K5 | Kullanılmayan `directories` bağımlılığı | `Cargo.toml` + `Cargo.lock` temizlendi (10 satır) | clippy/test temiz |

**Açık (planlanan) riskler:**
1. **Kritik — UninstallString enjeksiyonu:** `cmd /C` + `raw_arg` ile registry'den gelen komut ham çalışıyor; koruma yalnız CR/LF/NUL (`sp-core/kaldirici.rs:77-84`, `sp-win/kaldirici.rs:80-84`, `winget.rs:57-66`). HKCU yazımı + yönetici çalışma = yetki yükseltme vektörü. Öneri: komut satırı ayrıştırıcı + exe/argüman allowlist + metakarakter reddi + MSI GUID doğrulaması.
2. **Yüksek — HKCU PATH:** kalıcı yedek yok, 32K sınırı yok (`sorun_giderme.rs:147-199`, `app.rs:2213-2232`).
3. **Orta — Platform Tools:** hash/imza doğrulaması olmadan indirilip `adb.exe` çalıştırılıyor (`sp-win/android.rs:30-42,81-98`).
4. **Orta — app.rs monoliti:** 58 fonksiyon >40 satır, 77 fonksiyonda 3+ nesting; en derin `kaldirici_toplu_kaldir` (nesting 11).
5. **Orta — gizli hata yutma:** 83 `let _ =`, `app.rs` 7 `.ok()`; UI'da boş liste/hata gösterimi.
6. **Orta — `-ExecutionPolicy Bypass`** 13 PowerShell çağrısında.
7. **Orta — `tests/`:** Rust entegrasyon testi yok; `sp-app` 0 test, `validation.rs` 0 test.

## 6. Bağımlılıklar ve dokümantasyon

Ayrıntı: [bağımlılık raporu](bagimlilik-dokumantasyon.md). context7 ile doğrulanan kütüphaneler: `/longbridge/gpui-kit`, `/websites/rs_gpui-component_gpui_component`, `/cocool97/adb_client` (+ docs.rs).

- rustc/cargo **1.98.1** ↔ `rust-version = 1.98` uyumlu; `rust-toolchain.toml` eklendi.
- `gpui-kit =0.8.1` (gpui-pre `=0.3.10`), `adb_client 3.2.3 usb` API kullanımı doğru; deprecated API yok.
- **48 paket adında çift sürüm**; kök nedenler: `winreg 0.55` (→ windows-sys 0.59), `winres` (toml 0.5), gpui zinciri.
- Öneri: `winreg 0.56` (kırıcı: `RegValue.bytes: Cow`), `winres → embed-resource 3`, `cargo update -p encoding_rs`; CI'ya `cargo-audit`/`cargo-deny` kapısı.
- Tedarik zinciri: Cargo.lock v4 izleniyor, git/patch bağımlılığı yok, 844/844 lisans dolu, GPL-only yok (MPL-2.0 için NOTICE önerilir).

## 7. Uygulanan değişiklikler ve kanıt

| Dosya | Değişiklik |
|---|---|
| `.gitignore` | Kapsamlı hijyen (target, release çıktısı, runtime veri, araç indeksleri, PNG artefaktları, sırlar) |
| `LICENSE` | Apache-2.0 tam metni (apache.org, 11.358 B) |
| `.github/workflows/ci.yml` | Windows CI: fmt --check, clippy -D warnings, test |
| `.github/dependabot.yml` | Cargo + GitHub Actions, haftalık, gruplu |
| `SECURITY.md`, `.editorconfig`, `rust-toolchain.toml`, `release/README.md` | Eksik yönetişim dosyaları |
| `AGENTS.md` | Proje kuralları + korunan GitNexus bloğu |
| `crates/sp-core/src/motor.rs` | Telafi/günlük kusuru düzeltmesi + 2 test |
| `crates/sp-win/src/sahiplik.rs` | Benzersiz bağlam fiili + portatif hata günlüğü + test |
| `crates/sp-core/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | `directories` kaldırıldı |
| `scripts/*.ps1` (9) | UTF-8 BOM (PowerShell 5.1 Türkçe düzeltmesi) |
| `.git/config` | Remote URL'den PAT çıkarıldı |

**Doğrulama (son çalıştırma, çıkış kodları):** `FMT_EXIT=0` · `CLIPPY_EXIT=0` · `TEST_EXIT=0`; toplam **341 test** (sp-core 155, sp-win 186). Release hattı: release.ps1 ikinci çalıştırma **exit 0**, release-check.ps1 **exit 0** (yönetici manifesti + ikon), SHA-256 eşleşmesi doğrulandı. gitnexus `detect_changes`: 11 sembol, 12 dosya, risk **high** (motor değişikliği); etkilenen süreçler `icerik`, `arac_kutusu`, `render` — testlerle kapsandı.

## 8. Araç kanıtları (sp-sozlesme §2.6)

| Araç | Sorgu | Sonuç |
|---|---|---|
| **codegraph** | `codegraph init` + `codegraph_explore "App root view router"` | 52 dosya, 2.378 düğüm, 7.028 kenar (873 ms); `WinNexusApp` app.rs:292, 1 çağıran |
| **gitnexus** | `analyze`, `check`, `cypher` (hotspot/process), `impact`, `detect_changes`, MCP `context/clusters` | 3.247 sembol, 9.257 ilişki, 286 akış, 96 modül; döngü 0; `adimlari_geri_al` impact HIGH (7) |
| **graphify** | `update`, `god-nodes`, `diagnose multigraph`, `query` | 2.585 düğüm, 6.000 kenar, 133 topluluk; 0 çökmüş kenar/0 sarkan uç; 15 dosya sembolsüz (uyarı) |
| **trail** | `build`, `map` | 52 dosya, 1.865 sembol, 2.139 kenar; hub'lar `metin_tehlikeli_mi`, `paket_kimligi_dogrula`; **293.513 token tasarrufu** |
| **context7** | 3 sorgu (gpui-kit, gpui-component, adb_client) | Sürüm/API uyumu doğrulandı; kaynak kimlikleri bağımlılık raporunda |
| **claude-mem** | `mem_search` | **Erişilemez** (fetch failed, 2 deneme) → yedek beyanı: kalıcı belleğe kayıt yapılamadı |
| **GitHub API** | repo, workflows, branch, alerts | public, 0 workflow, dependabot kapalı; admin uçları 403 (**doğrulanmalı**) |

## 9. Release ve herkese açık indirme hattı (yeni)

Kullanıcı isteği: derlenmiş uygulamanın herkese açık sunucudan doğrudan indirilebilmesi. Birincil kaynak doğrulamasıyla seçilen çözüm **GitHub Releases**:

- Release asset limiti: asset başına **< 2 GiB**, sürüm başına 1000 asset, toplam boyut/bandwidth sınırı yok ([GitHub Docs — About releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)). 32 MB exe için uygundur.
- GitHub Pages ikili dağıtım için uygun değil: site ≤ 1 GB, aylık 100 GB soft bant genişliği ([GitHub Docs — Pages limits](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)).
- Kalıcı doğrudan bağlantı biçimi: `https://github.com/{owner}/{repo}/releases/latest/download/{asset}` ([GitHub Docs — Linking to releases](https://docs.github.com/en/repositories/releasing-projects-on-github/linking-to-releases)).
- `gh release create` bayrakları doğrulandı: `--title`, `--notes`, `--generate-notes`, `--latest`, `--verify-tag` ([gh manual](https://cli.github.com/manual/gh_release_create)).

**Eklenen hat:**

| Bileşen | İşlev |
|---|---|
| `.github/workflows/release.yml` | `v*` etiketi veya elle tetikleme; test → release derleme → manifest/ikon doğrulama → SHA-256 → `gh release create` → indirme bağlantısı HEAD doğrulaması |
| `scripts/derleme-ortami.ps1` | MSVC (vswhere + VS2022 fallback) ve `mt.exe` (en yeni Windows SDK) keşfi; sabit yol varsayımı kaldırıldı |
| `scripts/release.ps1` | Sabit adlı `release/WinNexus_beta.exe` + `WinNexus_beta.exe.sha256` üretir (GitHub asset adları) |
| `README.md` | İndir bölümü: sürüm sayfası, doğrudan indirme ve SHA-256 bağlantıları |
| `release/README.md` | Yayın akışı ve elle `gh release create` komutu |

**Yerel doğrulama:** release.ps1 iki kez çalıştırıldı; ikinci çalıştırma **exit 0** (deterministik). Üretilen release/WinNexus_beta.exe: 32.371.200 B, requireAdministrator manifesti **True**, ikon 32x32, SHA-256 92f65c58…eac049 — WinNexus_beta.exe.sha256 ile eşleşiyor. mt.exe ve vcvars64.bat keşfi vswhere/SDK taramasıyla doğrulandı.

**Bekleyen adım:** yerel git kimlik bilgisi geçersiz (`git push --dry-run` → `Authentication failed`); main ve `v*` etiketi push edilemediği için GitHub Actions tetiklenmedi. Kullanıcı `gh auth login -h github.com` (veya `gh auth setup-git`) ile yeniden giriş yaptıktan sonra:

```powershell
git push origin main
git tag v2026.10.30
git push origin v2026.10.30
```

Push sonrası ilk release otomatik oluşur; doğrudan indirme bağlantısı: `https://github.com/spgroupenerji/app-winnexus/releases/latest/download/WinNexus_beta.exe`.

## 10. Onay bekleyen işlemler

1. **Geçmiş temizliği:** İzleme düzeltmeleri yerel commit'lere alındı; 32,4 MB exe eski commit'te duruyor. Geçmişi arındırmak için (A) amend + `push --force-with-lease` **veya** (B) mevcut hâliyle push.
2. **PAT rotasyonu:** eski token iptal edilmeli (bu oturum günlüğünde görüldü).
3. **GitHub ayarları:** ruleset/branch protection, secret scanning, Dependabot alerts, wiki kapatma; repo public→private tercihi.
4. **Mimari:** `app.rs` 9 modüle bölünmesi (ayrı plan + onay).
5. **Güvenlik sertleştirme planı:** UninstallString parser + allowlist; PATH kalıcı yedeği; Platform Tools SHA-256.

---
*Ham denetim raporları: [git hijyeni](git-hijyen.md) · [kod kalitesi](kod-kalitesi.md) · [bağımlılıklar](bagimlilik-dokumantasyon.md)*
