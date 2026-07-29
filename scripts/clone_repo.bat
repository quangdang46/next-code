@echo off
setlocal

REM ==== Change this to your destination folder ====
set "TARGET_DIR=%USERPROFILE%\source\github"

if not exist "%TARGET_DIR%" (
    mkdir "%TARGET_DIR%"
)

cd /d "%TARGET_DIR%"

echo =====================================
echo Cloning repositories into:
echo %TARGET_DIR%
echo =====================================

call :clone https://github.com/anomalyco/opencode.git
call :clone https://github.com/can1357/oh-my-pi.git
call :clone https://github.com/Yeachan-Heo/gajae-code.git
call :clone https://github.com/code-yeongyu/oh-my-openagent.git
call :clone https://github.com/xai-org/grok-build.git
call :clone https://github.com/claude-code-best/claude-code.git
call :clone https://github.com/charmbracelet/crush.git
call :clone https://github.com/openai/codex.git
call :clone https://github.com/earendil-works/pi.git
call :clone https://github.com/Dicklesworthstone/pi_agent_rust.git

echo.
echo =====================================
echo Done!
echo =====================================
pause
exit /b

:clone
echo.
echo -------------------------------------
echo Cloning %~1
git clone %~1
if errorlevel 1 (
    echo Failed: %~1
) else (
    echo Success: %~1
)
exit /b