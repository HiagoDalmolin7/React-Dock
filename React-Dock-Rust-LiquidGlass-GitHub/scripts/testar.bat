@echo off
cd /d "%~dp0.."
where cargo >nul 2>&1 || (echo Instale Rust de https://rustup.rs && pause && exit /b 1)
where npm >nul 2>&1 || (echo Instale Node.js 22+ && pause && exit /b 1)
call npm install || (pause && exit /b 1)
call npm run tauri dev
pause
