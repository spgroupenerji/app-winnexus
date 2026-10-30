@echo off
:: Yönetici izni kontrolü ve otomatik yetki isteme
net session >nul 2>&1
if %errorLevel% neq 0 (
    powershell -Command "Start-Process '%~f0' -Verb RunAs"
    exit /b
)

:: Bilgisayarı yeniden başlatıp doğrudan BIOS ekranını açan komut
shutdown /r /fw /t 0
