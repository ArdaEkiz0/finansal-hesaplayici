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
    echo    Node.js bulunamadi! Indiriliyor...
    echo    Lutfen bekleyin...
    
    REM winget ile kur
    where winget >nul 2>&1
    if not errorlevel 1 (
        echo    winget ile kuruluyor...
        winget install OpenJS.NodeJS.LTS --silent --accept-package-agreements --accept-source-agreements
    ) else (
        REM Manuel indir
        echo    Manuel indiriliyor...
        powershell -NoProfile -ExecutionPolicy Bypass -Command ^
            "$url='https://nodejs.org/dist/v22.17.1/node-v22.17.1-x64.msi'; " ^
            "$out='%TEMP%\node-install.msi'; " ^
            "Write-Host '   Indiriliyor: v22.17.1...'; " ^
            "[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; " ^
            "Invoke-WebRequest -Uri $url -OutFile $out -UseBasicParsing -TimeoutSec 300; " ^
            "Write-Host '   Kuruluyor...'; " ^
            "Start-Process msiexec.exe -ArgumentList '/i', $out, '/qn' -Wait; " ^
            "Remove-Item $out -Force"
    )
    
    REM PATH guncelle
    set "PATH=%PATH%;C:\Program Files\nodejs\"
    
    where node >nul 2>&1
    if errorlevel 1 (
        echo    HATA: Node.js kurulamadi!
        echo    Manuel olarak yukleyin: https://nodejs.org
        pause
        exit /b 1
    )
)
for /f "tokens=*" %%i in ('node -v') do set NODE_VER=%%i
echo    Node.js %NODE_VER% hazir.

REM === ADIM 2: npm install ===
echo.
echo [2/5] Bagimliliklar yukleniyor...
if not exist "node_modules" (
    echo    npm install yapiliyor...
    call npm install
    if errorlevel 1 (
        echo    HATA: npm install basarisiz!
        pause
        exit /b 1
    )
) else (
    echo    node_modules mevcut.
)

REM === ADIM 3: Electron binary kontrol ===
echo.
echo [3/5] Electron kontrol ediliyor...
if not exist "node_modules\electron\dist\electron.exe" (
    echo    Electron indiriliyor...
    
    REM npm postinstall dene
    call node node_modules\electron\install.js
    if errorlevel 1 (
        echo    Otomatik indirme basarisiz, manuel indiriliyor...
        powershell -NoProfile -ExecutionPolicy Bypass -Command ^
            "$url='https://github.com/electron/electron/releases/download/v36.9.5/electron-v36.9.5-win32-x64.zip'; " ^
            "$zip='%TEMP%\electron.zip'; " ^
            "$dest='node_modules\electron\dist'; " ^
            "Write-Host '   Indiriliyor: Electron v36.9.5...'; " ^
            "[Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; " ^
            "Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing -TimeoutSec 600; " ^
            "Write-Host '   Cikariliyor...'; " ^
            "New-Item -ItemType Directory -Path $dest -Force | Out-Null; " ^
            "Expand-Archive -Path $zip -DestinationPath $dest -Force; " ^
            "Remove-Item $zip -Force; " ^
            "Set-Content -Path 'node_modules\electron\path.txt' -Value 'electron.exe' -NoNewline"
    )
    
    if not exist "node_modules\electron\dist\electron.exe" (
        echo    HATA: Electron indirilemedi!
        echo    Manuel olarak yukleyin: https://electronjs.org
        pause
        exit /b 1
    )
)
echo    Electron hazir.

REM === ADIM 4: Build ===
echo.
echo [4/5] Build aliniyor...
call npm run build
if errorlevel 1 (
    echo    HATA: Build basarisiz!
    pause
    exit /b 1
)
npx vite build --config vite.config.electron.ts
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
