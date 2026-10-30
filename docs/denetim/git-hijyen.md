# Git İzleme Hijyeni ve GitHub Yönetişim Denetimi
Denetim: 2026-10-30 22:04–22:14 (Europe/Istanbul) · Kapsam: HEAD f16c39f "ilk commit" (main = origin/main) · Yöntem: salt-okunur git + GitHub REST API (admin uçları 403).
Not: Denetim sırasında çalışma ağacı başka oturumlarca değiştirildi; bulgular HEAD f16c39f'e aittir (bkz. §8). Bu rapordan başka dosya yazılmadı/silinmedi.

## 1. Özet — 18 bulgu: 2 kritik · 5 yüksek · 7 orta · 4 düşük
| # | Önem | Bulgu | Kanıt | Öneri |
|---|---|---|---|---|
| B1 | Kritik | Remote URL'de düz metin PAT | `.git/config` `remote.origin.url` (maskeli: `https://spgroupenerji:***@github.com/...`); 22:11'de başka oturumca temizlendi | PAT'i revoke/rotate et; URL'ye bir daha gömme |
| B2 | Kritik | 32,37 MB exe HEAD'de izleniyor | `git ls-tree -r -l`: release/WinNexus_beta-v202610301819.exe = 32.369.152 B; pack 15,89 MiB | Geçmişten çıkar (§6-A) + çıktıyı GitHub Releases'e asset yap |
| B3 | Yüksek | Çalışma zamanı `settings.toml` izleniyor | release/settings.toml = 134 B (tema/dil/onaylar); README.md:29,94; dizinler.rs:43 | `git rm --cached`; varsayılanlar kodda (app.rs:488), şablon gerekmez |
| B4 | Yüksek | 32 PNG test çıktısı (5,94 MB); 24'ü README'de yok | tests/ 33 dosya; README.md:130-145 yalnız 8 görsel | 8'i `docs/kanit/gorseller/`e taşı + README güncelle; kalan 24'ü ignore et |
| B5 | Yüksek | `$null` çöp dosyası izleniyor | HEAD blob 6ec3482, 91 B: cmd hatası `'$g' is not recognized...` (§8) | `git rm --cached -- '$null'`; `/$null` ignore |
| B6 | Yüksek | `.gitignore` eksik (HEAD 3 satır) | HEAD: /target, /.cargo-home, *.log; yerel .gitnexus 84,8 MB, .codegraph 10,4 MB | §5 tam içerik; `.git/info/exclude` kuralını paylaşılan dosyaya taşı |
| B7 | Yüksek | GitHub yönetişimi yok | workflows=0, license=null, branch protected=False, rulesets=0, health %14 | §7: CI, LICENSE, ruleset, secret scanning, Dependabot |

## 2. İzleme envanteri
138 dosya · 40.084.458 B (~38,2 MiB) · 1 commit (f16c39f) · 1 pack / 158 nesne · 15,89 MiB (`git ls-files`, `git ls-tree -r -l`, `git count-objects -vH`).
Dağılım: release 32.369.286 (2) · tests 5.944.758 (33) · crates 1.183.221 (56) · Cargo.lock 218.769 · diğer 46 dosya 368.424.
İlk 20 büyük dosya: release exe 32.369.152 · crates/sp-app/src/app.rs 412.527 · 17 × tests/ui-*.png (200.890–245.858) · Cargo.lock 218.769; kalan 118 dosya < 200 KB.

## 3. Karar tablosu — izlenmemesi gerekenler
| Dosya | Boyut | Gerekçe | Aksiyon |
|---|---|---|---|
| release/WinNexus_beta-v202610301819.exe | 32.369.152 | Derleme çıktısı; `scripts/release.ps1:49-72` üretir | `git rm --cached`; `/release/*.exe`; GitHub Releases |
| release/settings.toml | 134 | Çalışma zamanı tercihi; README.md:29,94; dizinler.rs:43 | `git rm --cached`; `/release/settings.toml` |
| `$null` | 91 | cmd yönlendirme artığı (blob 6ec3482) | `git rm --cached -- '$null'`; `/$null` |
| 24 kullanılmayan tests PNG | 4.304.697 | `scripts/ui-*.ps1` üretir; README referansı yok | `/tests/ui-*.png` (+ exe-icon.png) ignore |
| README'deki 8 PNG | 1.638.663 | Doküman görseli ama test klasöründe | `docs/kanit/gorseller/`e taşı; README.md:130-145 güncelle |
| .gitnexus, .codegraph, graft, graphify-out | 110,7 MB | Yerel indeks/araç çıktısı; .gitnexus yalnız `.git/info/exclude:7`'de | `.gitignore`a taşı |
| CLAUDE.md / .claude yerel | 3,6 KB | Türetilmiş/kişisel ajan durumu | `/CLAUDE.md`; `/.claude/*` + `!/.claude/skills/` |
Not: `release/` klasöründe izlenen dosya kalmamalı (yalnız `release/README.md` klasör açıklaması). `settings.toml` şablonu gerekmez: eksik dosyada `Ayarlar::default()` devreye girer (app.rs:488; ayarlar.rs `eksik_dosya_varsayilan` testi); istenirse `docs/ornek/settings.ornek.toml` tutulur.

## 4. Karar tablosu — izlenmesi gerekenler / eksikler
| Dosya | Mevcut durum (HEAD) | Aksiyon |
|---|---|---|
| LICENSE + NOTICE | LICENSE yok (Apache-2.0 beyanı Cargo.toml:8, README.md:189); NOTICE yok; taslak 201 satır (izlenmemiş) | Commit et; `/license` 404 sorunu kapanır |
| .github/workflows/ci.yml + dependabot.yml | Yok; ikisinin de taslağı var (izlenmemiş) | §7 iskeletiyle commit et; alerts/security updates aç |
| rust-toolchain.toml | Yok; taslak 1.98.1 | Commit (MSRV `Cargo.toml:9`) |
| SECURITY.md + .editorconfig | Yok; taslakları var | Commit + private vulnerability reporting aç |
| AGENTS.md + .claude/skills | İkisi de izlenmiyor; AGENTS.md `.claude/skills/*`'e atıf yapıyor | AGENTS.md'yi izle; 6 SKILL.md'yi izle veya atıfları kaldır |
| CODEOWNERS, issue/PR şablonları | Yok (tek geliştirici) | Opsiyonel; ekip büyüyünce ekle |

## 5. Önerilen .gitignore (tam içerik)
```gitignore
/target/
/.cargo-home/
*.log
*.pdb
/release/*.exe
/release/settings.toml
/release/veri/
/veri/
/settings.toml
/tests/ui-*.png
/tests/exe-icon.png
/.gitnexus/
/.codegraph/
/graphify-out/
/graft/
/.claude/*
!/.claude/skills/
/CLAUDE.md
/.idea/
/.vscode/
Thumbs.db
desktop.ini
.env
.env.*
/$null
```
Not: README.md:130-145'teki 8 görsel önce `docs/kanit/gorseller/`e taşınmalı; aksi hâlde `/tests/ui-*.png` kuralı onları kapsar (izlenen dosyalar ignore'dan etkilenmez ama yeniden üretimde kaybolurlar).

## 6. Geçmiş temizliği seçenekleri (uygulanmadı; onay gerektirir)
| Seçenek | Kazanç | Risk |
|---|---|---|
| A) `git commit --amend` (exe/settings/$null/24 PNG çıkar) + `git push --force-with-lease` | ~38 MB geçmişten silinir; pack 15,89 MiB → ~2-3 MiB (tahmini, doğrulanmalı) | SHA değişir; eski commit GitHub cache/klonlarında bir süre erişilebilir; force-push sonrası GC beklenmeli; `protected=False`, forks=0, releases=0 → teknik engel/dış bağımlılık düşük |
| B) `git rm --cached` + takip commit | Hızlı, geçmiş bozulmaz | 15,89 MiB pack kalıcı; her klon 32 MB exe indirir → önerilmez |
| C) `git filter-repo --path release/ --invert-paths` / BFG | Tüm geçmişten siler; SHA'lar değişir | Tek commit'te gereksiz; koordinasyon ister |
LFS: gerekmiyor — exe geçmişten çıkınca en büyük nesne 412.527 B (app.rs), PNG'ler 160-246 KB. Öneri: A; tek commit + 0 fork + release/tag yokken en düşük maliyetli pencere.

## 7. GitHub yönetişimi — kanıt → öneri
| Alan | Kanıt (2026-10-30) | Öneri |
|---|---|---|
| Görünürlük / wiki | `private=false`; forks=0; stars=0; wiki=true; issues=true | Public kalabilir; wiki'yi kapat veya README'yi tek kaynak yap; açıklama/topics ekle |
| Lisans | `license=null`; `/license` 404; health %14 | LICENSE + NOTICE |
| CI | `actions/workflows` total_count=0 | Aşağıdaki iskelet; branch korumasında required check |
| Dependabot | 403 (PAT yetkisiz) → doğrulanmalı; yapılandırma yok | dependabot.yml + alerts/security updates aç |
| Branch koruması | `branches/main` `protected=false`; rulesets=0 | main ruleset: PR zorunlu, CI check zorunlu, force-push/silme yasak |
| Secret scanning / push protection / code scanning / actions izinleri | 403 → doğrulanmalı | Secret scanning + push protection aç (public repoda ücretsiz); Rust code scanning desteği doğrulanmalı, yoksa clippy + `cargo audit`/`cargo deny` |
| Release / tag | tags=0; releases=0 | v0.1.0 tag + exe'yi GitHub Release asset'i yap |
| PAT | Başlangıçta URL'de; şimdi URL temiz + `credential.helper=manager` | PAT'i revoke/rotate et; URL'ye gömme |
CI iskeleti (`.github/workflows/ci.yml`; worktree taslağıyla uyumlu, commit edilmeli):
```yaml
name: CI
on: [push, pull_request]
permissions: { contents: read }
jobs:
  dogrula:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { toolchain: 1.98.1, components: "clippy, rustfmt" }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
```
Not: action major sürümleri (checkout@v4, rust-cache@v2) doğrulanmalı; Windows runner VS 2022/MSVC içerdiği için CI'da `scripts/build.ps1` gerekmez.

## 8. Denetim anı: `$null` tespiti ve eşzamanlı değişiklikler
- **Evet, tespit ettim.** 22:04:5x ilk anlık görüntüde `$null` çalışma ağacında mevcuttu (91 B); `git status --porcelain` yalnızca `?? .claude/ .codegraph/ AGENTS.md CLAUDE.md` gösteriyordu.
- 22:06:03: `git status --porcelain` → ` D $null`; `Test-Path -LiteralPath '$null'` = False; `git ls-files --stage -- '$null'` hâlâ blob `6ec3482` (HEAD'de izlenmeye devam ediyor). Silme benim tarafımdan yapılmadı; tüm komutlarım salt-okunur (`git status/log/ls-tree/ls-files`, `Get-Content/Test-Path`).
- Aynı saatlerde başka oturum(lar): `.gitignore` M, `$null` D, `scripts/*.ps1` M, `motor.rs` M; LICENSE/SECURITY.md/.editorconfig/rust-toolchain.toml/.github/release/README.md eklendi; remote URL'deki PAT temizlendi (`.git/config` mtime 22:11:03). Bu taslaklar izlenmemiş/commit edilmemiştir; rapor HEAD f16c39f'i denetler.
Kanıt komutları: `git ls-tree -r -l HEAD`; `git ls-files`; `git count-objects -vH`; `git log -1 --format=fuller`; `git ls-remote origin`; GitHub API `/repos`, `/branches/main`, `/git/trees/f16c39f...`, `/license`, `/actions/workflows`, `/tags`, `/releases`, `/rulesets` (yetkisiz uçlar 403). Sır taraması (PAT/anahtar desenleri) izlenen dosyalarda eşleşme vermedi.
