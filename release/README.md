# Dağıtım klasörü

Bu klasör yerel dağıtım çıktısı içindir; depoda ikili dosya tutulmaz (`.gitignore`).

`scripts/release.ps1` üretir:

- `WinNexus_beta-vYYYYMMDDHHMM.exe` — tarih damgalı yerel kopya (uygulama içi sürüm etiketiyle aynı)
- `WinNexus_beta.exe` — sabit adlı kopya (GitHub Releases asset adı)
- `WinNexus_beta.exe.sha256` — SHA-256 özeti

Sürüm yayını `.github/workflows/release.yml` ile `v*` etiketi itildiğinde otomatik yapılır:
GitHub Actions testleri çalıştırır, release derlemesini yapar, `requireAdministrator` manifestini
uygular, SHA-256 doğrular ve GitHub Releases'e `WinNexus_beta.exe` + `.sha256` asset'lerini yükler.

Elle yayın (yerel derleme sonrası):

```powershell
gh release create v2026.10.30 release/WinNexus_beta.exe release/WinNexus_beta.exe.sha256 --title "WinNexus beta v2026.10.30" --generate-notes --latest
```
