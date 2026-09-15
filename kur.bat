@echo off
title Finansal Hesaplaci Kurulum
cd /d "%~dp0"
chcp 65001 >nul 2>&1

echo.
echo ========================================
echo   Finansal Hesaplaci Kurulum
echo ========================================
echo.

REM === ADIM 1: Node.js kontrol ===
echo [1/5] Node.js kontrol ediliyor...
where node >nul 2>&1
if errorlevel 1 (
    echo    Node.js bulunamadi!
    echo    Manuel olarak yukleyin: https://nodejs.org
    pause
    exit /b 1
)
for /f "tokens=*" %%i in ('node -v') do set NODE_VER=%%i
echo    Node.js %NODE_VER% hazir.

REM === ADIM 2: npm install (OneDrive sorunu icin C:\ dizinine gecici kur) ===
echo.
echo [2/5] Bagimliliklar yukleniyor...
if exist "node_modules\electron\dist\electron.exe" (
    echo    node_modules mevcut.
) else (
    echo    Gecici klasore kuruluyor (OneDrive kilitleme sorunu)...
    set "TEMP_DIR=C:\_fh_build"
    if exist "%TEMP_DIR%" rmdir /s /q "%TEMP_DIR%" 2>nul
    mkdir "%TEMP_DIR%" 2>nul
    
    REM Dosyalari kopyala
    robocopy "%~dp0" "%TEMP_DIR%" /E /XD node_modules dist dist-electron .git /XF package-lock.json /NFL /NDL /NJH /NJS /NC /NS /NP >nul 2>&1
    
    REM Orada npm install yap
    cd /d "%TEMP_DIR%"
    call npm install --loglevel error
    if errorlevel 1 (
        echo    HATA: npm install basarisiz!
        cd /d "%~dp0"
        pause
        exit /b 1
    )
    
    REM node_modules'i geri kopyala
    cd /d "%~dp0"
    robocopy "%TEMP_DIR%\node_modules" "%~dp0node_modules" /E /NFL /NDL /NJH /NJS /NC /NS /NP >nul 2>&1
    
    REM Temizle
    rmdir /s /q "%TEMP_DIR%" 2>nul
    echo    Bagimliliklar yuklendi.
)

REM === ADIM 3: Electron binary kontrol ===
echo.
echo [3/5] Electron kontrol ediliyor...
if not exist "node_modules\electron\dist\electron.exe" (
    echo    HATA: Electron bulunamadi!
    echo    node_modules\electron\ klasorunu kontrol edin.
    pause
    exit /b 1
)
echo    Electron hazir.

REM === ADIM 4: Build ===
echo.
echo [4/5] Build aliniyor...
call npm run build 2>nul
if errorlevel 1 (
    echo    HATA: Build basarisiz!
    pause
    exit /b 1
)
npx vite build --config vite.config.electron.ts 2>nul
if errorlevel 1 (
    echo    HATA: Electron build basarisiz!
    pause
    exit /b 1
)
echo    Build tamamlandi.

REM === ADIM 5: Kisayol ===
echo.
echo [5/5] Masaustu kisayolu olusturuluyor...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$d=[Environment]::GetFolderPath('Desktop'); $w=New-Object -ComObject WScript.Shell; $l=$w.CreateShortcut(\"$d\Finansal Hesaplaci.lnk\"); $l.TargetPath='%~dp0start.bat'; $l.WorkingDirectory='%~dp0'; $l.Description='Finansal Hesaplaci'; $l.Save(); Write-Host 'Kisayol olusturuldu.'"

echo.
echo ========================================
echo   Kurulum tamamlandi!
echo   Masaustundeki kisayol ile baslatin.
echo ========================================
echo.
pause
