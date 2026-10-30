@echo off
setlocal EnableExtensions
title Windows Fotograf Goruntuleyici Etkinlestirme

net session >nul 2>&1
if errorlevel 1 (
    echo Yonetici haklari gerekiyor. Lutfen yonetici olarak calistirin.
    pause
    exit /b 1
)

echo Windows Fotograf Goruntuleyici icin dosya iliskilendirmeleri ekleniyor...
echo.
for %%e in (.tif .tiff .png .bmp .jpeg .jpg .ico .gif .jfif .dib .wdp .jxr) do (
    reg add "HKLM\SOFTWARE\Microsoft\Windows Photo Viewer\Capabilities\FileAssociations" /v "%%e" /t REG_SZ /d "PhotoViewer.FileAssoc.Tiff" /f >nul
    if errorlevel 1 echo Hata: %%e iliskilendirmesi eklenemedi.
)
echo.
echo Islemler tamamlandi. Goruntuleyiciyi varsayilan yapmak icin
echo Ayarlar - Uygulamalar - Varsayilan uygulamalar bolumunden secebilirsiniz.
pause
