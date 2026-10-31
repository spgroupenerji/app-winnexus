# Dağıtım klasörü

Bu klasör yerel dağıtım çıktısı içindir; depoda ikili dosya tutulmaz (`.gitignore`).

- Derleme `scripts/release.ps1` ile yapılır. Sürüm damgası `vYYYYMMDDHHMM` biçiminde derleme anında üretilir ve exe içine gömülür.
- Klasörde yalnızca en güncel derleme kalır: `WinNexus_beta-vYYYYMMDDHHMM.exe` ve aynı adlı `.sha256` özeti. Yeni artefakt doğrulanmadan önceki dosya silinmez; eski damgalı dosyalar yeni derleme doğrulandıktan sonra temizlenir.
- `settings.toml`, `veri/` ve günlük dosyaları uygulamanın çalışma zamanı verisidir; artefaktın parçası değildir ve dağıtılmaz.

Resmi sürüm, `v*` etiketi itildiğinde `.github/workflows/release.yml` tarafından üretilir:

- Testler çalıştırılır (`cargo test --workspace`).
- Release derlemesi yapılır ve `mt.exe` ile `requireAdministrator` manifesti uygulanır.
- Damga, yönetici manifesti, ikon ve sürüm bilgisi doğrulanır.
- GitHub Releases'e sabit adlı `WinNexus_beta.exe` ve `WinNexus_beta.exe.sha256` yüklenir; sabit ad, `releases/latest/download/...` bağlantısının her sürümde çalışmasını sağlar.
- İndirme bağlantısı HEAD isteğiyle doğrulanır.

Doğrudan indirme: https://github.com/spgroupenerji/app-winnexus/releases/latest/download/WinNexus_beta.exe
