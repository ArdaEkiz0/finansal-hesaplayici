@echo off
title Finansal Hesaplaci Kurulum
cd /d "%~dp0"

echo.
echo ========================================
echo   Finansal Hesaplaci Kurulum
echo ========================================
echo.

echo [1/3] Bagimliliklar kontrol ediliyor...
if not exist "node_modules" (
    echo    npm install yapiliyor...
    call npm install
    if errorlevel 1 (
        echo HATA: npm install basarisiz!
        pause
        exit /b 1
    )
) else (
    echo    node_modules mevcut.
)

echo.
echo [2/3] Build yapiliyor...
call npm run build
if errorlevel 1 (
    echo HATA: Build basarisiz!
    pause
    exit /b 1
)
npx vite build --config vite.config.electron.ts
if errorlevel 1 (
    echo HATA: Electron build basarisiz!
    pause
    exit /b 1
)

echo.
echo [3/3] Masaustu kisayolu olusturuluyor...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$d=[Environment]::GetFolderPath('Desktop'); $w=New-Object -ComObject WScript.Shell; $l=$w.CreateShortcut(\"$d\Finansal Hesaplaci.lnk\"); $l.TargetPath='%~dp0start.bat'; $l.WorkingDirectory='%~dp0'; $l.Description='Finansal Hesaplaci'; $l.Save(); Write-Host 'Kisayol olusturuldu.'"

echo.
echo ========================================
echo   Kurulum tamamlandi!
echo   Masaustundeki kisayol ile baslatin.
echo ========================================
echo.
pause
