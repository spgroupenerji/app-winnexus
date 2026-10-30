@echo off
setlocal EnableExtensions
title Hizli Erisim Yonetimi

REM HubMode=1 ise Hizli Erisim devre disidir; deger yoksa veya 0 ise etkindir.
net session >nul 2>&1
if errorlevel 1 (
    echo Yonetici haklari gerekiyor. Lutfen yonetici olarak calistirin.
    pause
    exit /b 1
)

reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer" /v HubMode 2>nul | find "0x1" >nul
if not errorlevel 1 (
    echo Hizli Erisim su anda DEVRE DISI.
) else (
    echo Hizli Erisim su anda ETKIN - varsayilan durum.
)

:menu
echo.
echo Secenekler:
echo.
echo 1. Hizli Erisim'i ETKINLESTIR (varsayilana don)
echo 2. Hizli Erisim'i DEVRE DISI BIRAK
echo 3. Cikis
echo.
set /p "secim=Seciminiz (1-3): "

if "%secim%"=="1" goto :etkinlestir
if "%secim%"=="2" goto :devre_disi
if "%secim%"=="3" goto :eof
echo Gecersiz secim. Lutfen 1, 2 veya 3 girin.
pause
goto menu

:etkinlestir
reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer" /v HubMode >nul 2>&1
if errorlevel 1 (
    echo Hizli Erisim zaten varsayilan durumda - ETKIN.
    pause
    goto menu
)
reg delete "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer" /v HubMode /f
if errorlevel 1 (
    echo Hata: Kayit defteri degeri silinemedi.
    pause
    goto menu
)
echo Basarili.
goto :restart_explorer

:devre_disi
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer" /v HubMode /t REG_DWORD /d 1 /f
if errorlevel 1 (
    echo Hata: Kayit defteri degeri ayarlanamadi.
    pause
    goto menu
)
echo Basarili.
goto :restart_explorer

:restart_explorer
echo.
echo Degisikliklerin uygulanmasi icin Explorer yeniden baslatiliyor...
taskkill /f /im explorer.exe >nul
start explorer.exe
echo.
pause
exit /b
