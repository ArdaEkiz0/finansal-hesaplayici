@echo off
title Finansal Hesaplaci Kurulum
cd /d "%~dp0"

echo.
echo ========================================
echo   Finansal Hesaplaci Kurulum
echo ========================================
echo.

echo [1/4] Node.js kontrol ediliyor...
where node >nul 2>&1
if errorlevel 1 (
    echo HATA: Node.js yuklu degil!
    echo https://nodejs.org adresinden indirin.
    goto HATA
)
node -v
echo Node.js hazir.
echo.

echo [2/4] Bagimliliklar yukleniyor...
if exist "node_modules\electron\dist\electron.exe" (
    echo node_modules mevcut, atlaniyor.
) else (
    echo npm install basliyor...
    call npm install
    if errorlevel 1 (
        echo HATA: npm install basarisiz!
        goto HATA
    )
    echo npm install tamamlandi.
)
echo.

echo [3/4] Build aliniyor...
call npm run build
if errorlevel 1 (
    echo HATA: Build basarisiz!
    goto HATA
)
npx vite build --config vite.config.electron.ts
if errorlevel 1 (
    echo HATA: Electron build basarisiz!
    goto HATA
)
echo Build tamamlandi.
echo.

echo [4/4] Masaustu kisayolu olusturuluyor...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$d=[Environment]::GetFolderPath('Desktop'); $w=New-Object -ComObject WScript.Shell; $l=$w.CreateShortcut(\"$d\Finansal Hesaplaci.lnk\"); $l.TargetPath='%~dp0start.bat'; $l.WorkingDirectory='%~dp0'; $l.Description='Finansal Hesaplaci'; $l.Save(); Write-Host 'Kisayol olusturuldu.'"

echo.
echo ========================================
echo   KURULUM TAMAMLANDI!
echo   Masaustunde "Finansal Hesaplaci"
echo   kisayoluna tiklayarak acin.
echo ========================================
echo.
goto SON

:HATA
echo.
echo Bir hata olustu. Yukaridaki hata mesajini kontrol edin.
echo.

:SON
pause
