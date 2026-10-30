@echo off
setlocal EnableExtensions
title MIUI Bloatware Kaldirma (ADB)
color a
echo Bu betik ADB uzerinden MIUI bloatware paketlerini kaldirir.
echo Telefonda gelistirici secenekleri ve USB hata ayiklama acik olmali.
echo.
set /p "Y=adb.exe klasorunun tam yolu: "
cd /d "%Y%"
if errorlevel 1 (
    echo Gecersiz klasor yolu.
    pause
    exit /b 1
)
where adb >nul 2>&1
if errorlevel 1 (
    echo adb.exe bulunamadi. Platform Tools klasorunu kontrol edin.
    pause
    exit /b 1
)
adb devices
echo Listede cihaz gorunuyorsa devam etmek icin bir tusa basin.
pause
echo -----------------------------------------------
echo Elle kaldirma komutu: pm uninstall -k --user 0 [paket]
echo -----------------------------------------------
for %%X in (
"com.android.bips"
"com.android.calendar"
"com.android.dreams.phototable"
"com.android.hotwordenrollment.okgoogle"
"com.android.hotwordenrollment.xgoogle"
"com.android.protips"
"com.android.providers.calendar"
"com.android.providers.downloads.ui"
"com.android.wallpaper.livepicker"
"com.facebook.appmanager"
"com.facebook.services"
"com.facebook.system"
"com.google.android.apps.googleassistant"
"com.google.android.apps.restore"
"com.google.android.apps.subscriptions.red"
"com.google.android.apps.wellbeing"
"com.google.android.googlequicksearchbox"
"com.google.android.marvin.talkback"
"com.google.android.setupwizard"
"com.google.android.syncadapters.calendar"
"com.google.android.youtube"
"com.google.ar.lens"
"com.mi.android.globalminusscreen"
"com.mi.globalbrowser"
"com.mipay.wallet.id"
"com.mipay.wallet.in"
"com.miui.analytics"
"com.miui.backup"
"com.miui.bugreport"
"com.miui.calculator"
"com.miui.cloudbackup"
"com.miui.cloudservice"
"com.miui.cloudservice.sysbase"
"com.miui.compass"
"com.miui.daemon"
"com.miui.global.packageinstaller"
"com.miui.micloudsync"
"com.miui.miservice"
"com.miui.mishare.connectivity"
"com.miui.msa.global"
"com.miui.notes"
"com.miui.player"
"com.miui.screenrecorder"
"com.miui.videoplayer"
"com.miui.weather2"
"com.miui.yellowpage"
"com.xiaomi.calendar"
"com.xiaomi.glgm"
"com.xiaomi.mi_connect_service"
"com.xiaomi.midrop"
"com.xiaomi.payment"
"com.xiaomi.simactivate.service"
) do (
    adb shell pm uninstall -k --user 0 %%X
)
echo -----------------------------------------------
echo Islem tamam.
echo Paket geri yuklemek icin: adb shell cmd package install-existing [paket]
echo -----------------------------------------------
pause
