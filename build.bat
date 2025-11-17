@echo off
REM
REM

echo =============================
echo SystemScript Compiler - Build
echo =============================
echo.

echo [1/2] Building compiler...
cargo build --release
if errorlevel 1 (
    echo ERROR: Compiler build failed
    exit /b 1
)
echo       Compiler built successfully
echo.

echo [2/2] Checking for example programs...
if not exist example (
    echo No example directory found. Skipping compilation tests.
    echo.
    echo =============================
    echo Build completed successfully!
    echo =============================
    goto :end
)

echo       Found example directory
echo.

echo Compiling example programs...
echo.

REM Loop through all .ss files
setlocal enabledelayedexpansion
set SUCCESS=0
set FAILED=0

for %%f in (example\*.ss) do (
    echo    Compiling %%~nxf...
    target\release\ssc.exe "%%f" -o "example\%%~nf.exe" --emit-ir
    if errorlevel 1 (
        echo    ERROR: %%~nxf compilation failed
        set /a FAILED+=1
    ) else (
        echo    Success!
        set /a SUCCESS+=1
    )
    echo.
)

echo ================
echo Build completed!
echo ================
echo.
echo Compilation Results:
echo   Success: !SUCCESS!
echo   Failed:  !FAILED!
echo.
echo Check the example\ directory for:
echo   - .ss files (source code)
echo   - .exe files (compiled executables)
echo   - .ir files (intermediate representation)
echo   - .asm files (assembly output)
echo.

:end
endlocal
