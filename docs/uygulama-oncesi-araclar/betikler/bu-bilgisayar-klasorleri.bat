@echo off
setlocal EnableExtensions EnableDelayedExpansion
title Bu Bilgisayar Klasorlerini Yonet

REM Bu Bilgisayar altindaki klasorlerin gorunurlugu, klasor GUID'inin
REM FolderDescriptions\PropertyBag anahtarindaki ThisPCPolicy degeri ile yonetilir (Show/Hide).

net session >nul 2>&1
if errorlevel 1 (
    echo Yonetici haklari gerekiyor. Lutfen yonetici olarak calistirin.
    pause
    exit /b 1
)

set "GUIDs[3DObjects]={31C0DD25-9439-4F12-BF41-7FF4EDA38722}"
set "GUIDs[Documents]={f42ee2d3-909f-4907-8871-4c22fc0bf756}"
set "GUIDs[Downloads]={7d83ee9b-2244-4e70-b1f5-5393042af1e4}"
set "GUIDs[Music]={a0c69a99-21c8-4671-8703-7934162fcf1d}"
set "GUIDs[Pictures]={0ddd015d-b06c-45d5-8c4c-f59713854639}"
set "GUIDs[Videos]={35286a68-3c57-41a1-bbb1-0eae73d76c95}"

set "FOLDERS=3DObjects Documents Downloads Music Pictures Videos"

:menu
cls
echo.
echo Mevcut Durum:
echo.
for %%a in (%FOLDERS%) do call :ShowStatus "%%a"
echo.
echo Secenekler:
echo.
echo 1. Tum klasorleri GOSTER
echo 2. Tum klasorleri GIZLE
echo 3. Tek tek secerek duzenle
echo 4. Cikis
echo.
set /p "secim=Seciminiz (1-4): "

if "%secim%"=="1" (
    for %%a in (%FOLDERS%) do call :SetFolderVisibility "%%a" "Show"
    goto :son
)
if "%secim%"=="2" (
    for %%a in (%FOLDERS%) do call :SetFolderVisibility "%%a" "Hide"
    goto :son
)
if "%secim%"=="3" (
    for %%a in (%FOLDERS%) do call :AskFolder "%%a"
    goto :son
)
if "%secim%"=="4" goto :eof
echo Gecersiz secim. Lutfen 1 ile 4 arasinda bir sayi girin.
pause
goto menu

:ShowStatus
set "ad=%~1"
set "g=!GUIDs[%ad%]!"
set "durum=Gorunur"
reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FolderDescriptions\%g%\PropertyBag" /v "ThisPCPolicy" 2>nul | findstr /i "Hide" >nul
if not errorlevel 1 set "durum=Gizli"
echo %ad%: %durum%
goto :eof

:AskFolder
set "ad=%~1"
:tekrar
set "goster="
set /p "goster=%ad% klasorunu goster (E/H)?: "
if /i "%goster%"=="E" (
    call :SetFolderVisibility "%ad%" "Show"
    goto :eof
)
if /i "%goster%"=="H" (
    call :SetFolderVisibility "%ad%" "Hide"
    goto :eof
)
echo Lutfen E (Evet) veya H (Hayir) girin.
goto tekrar

:SetFolderVisibility
set "ad=%~1"
set "g=!GUIDs[%ad%]!"
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FolderDescriptions\%g%\PropertyBag" /v "ThisPCPolicy" /t REG_SZ /d "%~2" /f >nul
if errorlevel 1 (
    echo %ad%: Hata - kayit defteri degeri ayarlanamadi.
) else (
    echo %ad%: %~2 uygulandi.
)
goto :eof

:son
echo.
echo Degisikliklerin gorunmesi icin Explorer'i yeniden baslatin.
echo.
pause
exit /b
