@echo off
echo ========================================================
echo   J12 Forensic Suite - Windows Production Build Script
echo ========================================================

cd /d "%~dp0\.."

where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not found. Please install Node.js 18+.
    exit /b 1
)

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] Rust/Cargo is not found. Please install Rust from https://rustup.rs
    exit /b 1
)

if not exist "node_modules\" (
    echo [INFO] Installing npm dependencies...
    call npm install
)

echo [INFO] Building production Tauri release (.msi and .exe)...
call npx tauri build

echo.
echo ========================================================
echo   [SUCCESS] Build Finished!
echo   Outputs located at:
echo   src-tauri\target\release\bundle\msi\
echo   src-tauri\target\release\bundle\nsis\
echo ========================================================
