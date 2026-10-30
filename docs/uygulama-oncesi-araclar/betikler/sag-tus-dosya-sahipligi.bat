@echo off
setlocal EnableExtensions
title Sahipligi Al (Sag Tus Menusu) - EKLE/KALDIR

net session >nul 2>&1
if errorlevel 1 (
    echo Yonetici haklari gerekiyor. Lutfen yonetici olarak calistirin.
    pause
    exit /b 1
)

:menu
echo.
echo Secenekler:
echo.
echo 1. "Sahipligi Al" secenegini EKLE (normal sag tik)
echo 2. "Sahipligi Al" secenegini EKLE (sadece Shift + sag tik)
echo 3. "Sahipligi Al" secenegini KALDIR
echo 4. Cikis
echo.
set /p "secim=Seciminiz (1-4): "

if "%secim%"=="1" goto :ekle
if "%secim%"=="2" goto :ekle_shift
if "%secim%"=="3" goto :kaldir
if "%secim%"=="4" goto :eof
echo Gecersiz secim. Lutfen 1 ile 4 arasinda bir sayi girin.
pause
goto menu

:ekle
call :EkleAnahtarlar
reg delete "HKCR\*\shell\runas" /v Extended /f >nul 2>&1
reg delete "HKCR\Directory\shell\runas" /v Extended /f >nul 2>&1
reg delete "HKCR\Directory\Background\shell\runas" /v Extended /f >nul 2>&1
echo "Sahipligi Al" secenegi sag tus menusune eklendi.
goto :son

:ekle_shift
call :EkleAnahtarlar
reg add "HKCR\*\shell\runas" /v Extended /t REG_SZ /d "" /f >nul
reg add "HKCR\Directory\shell\runas" /v Extended /t REG_SZ /d "" /f >nul
reg add "HKCR\Directory\Background\shell\runas" /v Extended /t REG_SZ /d "" /f >nul
echo "Sahipligi Al" secenegi Shift + sag tik menusune eklendi.
goto :son

:EkleAnahtarlar
reg add "HKCR\*\shell\runas" /ve /t REG_SZ /d "Sahipligi Al" /f >nul
reg add "HKCR\*\shell\runas" /v HasLUAShield /t REG_SZ /d "" /f >nul
reg add "HKCR\*\shell\runas" /v NoWorkingDirectory /t REG_SZ /d "" /f >nul
reg add "HKCR\*\shell\runas\command" /ve /t REG_EXPAND_SZ /d "cmd.exe /c takeown /f \"%%1\" && icacls \"%%1\" /grant *S-1-5-32-544:F" /f >nul
reg add "HKCR\*\shell\runas\command" /v IsolatedCommand /t REG_EXPAND_SZ /d "cmd.exe /c takeown /f \"%%1\" && icacls \"%%1\" /grant *S-1-5-32-544:F" /f >nul
reg add "HKCR\Directory\shell\runas" /ve /t REG_SZ /d "Sahipligi Al" /f >nul
reg add "HKCR\Directory\shell\runas" /v HasLUAShield /t REG_SZ /d "" /f >nul
reg add "HKCR\Directory\shell\runas" /v NoWorkingDirectory /t REG_SZ /d "" /f >nul
reg add "HKCR\Directory\shell\runas\command" /ve /t REG_EXPAND_SZ /d "cmd.exe /c takeown /f \"%%1\" /r /d y && icacls \"%%1\" /grant *S-1-5-32-544:F /t /c /l /q" /f >nul
reg add "HKCR\Directory\shell\runas\command" /v IsolatedCommand /t REG_EXPAND_SZ /d "cmd.exe /c takeown /f \"%%1\" /r /d y && icacls \"%%1\" /grant *S-1-5-32-544:F /t /c /l /q" /f >nul
reg add "HKCR\Directory\Background\shell\runas" /ve /t REG_SZ /d "Sahipligi Al" /f >nul
reg add "HKCR\Directory\Background\shell\runas" /v HasLUAShield /t REG_SZ /d "" /f >nul
reg add "HKCR\Directory\Background\shell\runas" /v NoWorkingDirectory /t REG_SZ /d "" /f >nul
reg add "HKCR\Directory\Background\shell\runas\command" /ve /t REG_EXPAND_SZ /d "cmd.exe /c takeown /f \"%%V\" /r /d y && icacls \"%%V\" /grant *S-1-5-32-544:F /t /c /l /q" /f >nul
reg add "HKCR\Directory\Background\shell\runas\command" /v IsolatedCommand /t REG_EXPAND_SZ /d "cmd.exe /c takeown /f \"%%V\" /r /d y && icacls \"%%V\" /grant *S-1-5-32-544:F /t /c /l /q" /f >nul
goto :eof

:kaldir
call :KaldirAnahtar "HKCR\*\shell\runas" "Dosyalar"
call :KaldirAnahtar "HKCR\Directory\shell\runas" "Klasorler"
call :KaldirAnahtar "HKCR\Directory\Background\shell\runas" "Klasor arka plani"
echo "Sahipligi Al" secenegi kaldirildi.
goto :son

:KaldirAnahtar
reg query "%~1" >nul 2>&1
if errorlevel 1 (
    echo %~2: Zaten kaldirilmis.
) else (
    echo %~2: Kaldiriliyor...
    reg delete "%~1" /f >nul
)
goto :eof

:son
echo.
pause
exit /b
