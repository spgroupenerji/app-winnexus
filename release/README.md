# Dağıtım klasörü

Bu klasör yerel dağıtım çıktısı içindir; depoda ikili dosya tutulmaz (`.gitignore`).

Resmi sürüm, `v*` etiketi itildiğinde `.github/workflows/release.yml` tarafından üretilir:

- Testler çalıştırılır (`cargo test --workspace`).
- Release derlemesi yapılır ve `mt.exe` ile `requireAdministrator` manifesti uygulanır.
- Damga, yönetici manifesti, ikon ve sürüm bilgisi doğrulanır.
- `WinNexus_beta.exe` ve `WinNexus_beta.exe.sha256` GitHub Releases'e yüklenir.
- `releases/latest/download/...` indirme bağlantısı HEAD isteğiyle doğrulanır.

Doğrudan indirme: https://github.com/spgroupenerji/app-winnexus/releases/latest/download/WinNexus_beta.exe
