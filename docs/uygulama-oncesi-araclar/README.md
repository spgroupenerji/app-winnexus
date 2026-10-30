# Uygulama Öncesi Araçlar

WinNexus geliştirilmeden önce elle kullanılan betiklerin ve o dönemdeki uygulama envanterinin arşividir. Buradaki betikler artık bakım görmüyor; aynı işleri uygulama içindeki ilgili özellikler yapar. Arşiv, hangi elle işlemin hangi uygulama özelliğine dönüştüğünü izlemek için tutulur.

## Betik eşlemesi

| Arşivdeki dosya | WinNexus karşılığı |
| --- | --- |
| [betikler/bios-yeniden-baslat.bat](betikler/bios-yeniden-baslat.bat) | Araç Kutusu > "BIOS'a yeniden başlat" (`AracOnay::BiosYenidenBaslat`) |
| [betikler/explorer-yeniden-baslat.lnk](betikler/explorer-yeniden-baslat.lnk) | Araç Kutusu > "Explorer'ı yeniden başlat" (`AracOnay::ExplorerYenidenBaslat`) |
| [betikler/sag-tus-dosya-sahipligi.bat](betikler/sag-tus-dosya-sahipligi.bat) | Araç Kutusu > "Sahipliği Al menüsü ekle/kaldır" (`AracOnay::SahiplikEkle`, `AracOnay::SahiplikKaldir`) |
| [betikler/bu-bilgisayar-klasorleri.bat](betikler/bu-bilgisayar-klasorleri.bat) | İnce ayar `sistem.bu_bilgisayar_gizle` (`rules/tweaks/arac_kutusu.toml`) |
| [betikler/hizli-erisim-ac-kapat.bat](betikler/hizli-erisim-ac-kapat.bat) | İnce ayar `sistem.hizli_erisim_kapat` (`rules/tweaks/arac_kutusu.toml`) |
| [betikler/fotograf-goruntuleyici-etkinlestir.bat](betikler/fotograf-goruntuleyici-etkinlestir.bat) | İnce ayar `sistem.fotograf_goruntuleyici` (`rules/tweaks/arac_kutusu.toml`) |
| [betikler/miui-bloatware-kaldir-adb.bat](betikler/miui-bloatware-kaldir-adb.bat) | Android Yönetimi > Paket Veritabanı (UAD-ng) |

Kaldırılan dört `Windows_WinGet_*.bat` kurulum betiği, içerikleri aynı paket kümeleriyle eşleştiği için gömülü profillere birleştirildi: [gelistirici.toml](../../profiles/gelistirici.toml), [is.toml](../../profiles/is.toml), [magaza.toml](../../profiles/magaza.toml), [uygulama.toml](../../profiles/uygulama.toml). Kurulum artık Winget Merkezi > Profiller bölümünden yapılır.

## Uygulama envanteri

[kullandigim-uygulamalar.md](kullandigim-uygulamalar.md) bu makinede winget tarafından yönetilen uygulamaların listesidir; Windows'un yönettiği MSIX/sistem paketleri kapsam dışıdır. [profiles/kullandigim.toml](../../profiles/kullandigim.toml) aynı listedeki winget yönetimli paketlerin gömülü profili; uygulama bu profili Winget Merkezi > Profiller bölümünde tarar ve eksikleri tek seçimle kurar.

Envanteri ve profili yenilemek için:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File docs/uygulama-oncesi-araclar/uygulama-envanteri.ps1 -Yenile
```

Yalnızca güncellik denetimi için `-Denetle` kullanılır. Betik `winget list` çıktısını okur; winget kurulu ve erişilebilir olmalıdır. Windows tarafından yönetilen MSIX/sistem paketleri profile alınmaz.
