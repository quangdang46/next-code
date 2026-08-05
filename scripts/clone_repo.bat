@echo off
setlocal

REM ============================================================
REM  Clone coding-agent CLI repositories into one folder.
REM  TARGET_DIR = where all repos are cloned to (change if needed)
REM ============================================================
set "TARGET_DIR=%USERPROFILE%\source\github"

if not exist "%TARGET_DIR%" (
    mkdir "%TARGET_DIR%"
)

cd /d "%TARGET_DIR%"

echo =====================================
echo Cloning coding-agent CLI repos into:
echo %TARGET_DIR%
echo =====================================

REM ---- Open source coding-agent CLIs (cloneable) ----
call :clone https://github.com/anomalyco/opencode.git
call :clone https://github.com/1jehuang/jcode.git
call :clone https://github.com/can1357/oh-my-pi.git
call :clone https://github.com/Yeachan-Heo/gajae-code.git
call :clone https://github.com/code-yeongyu/oh-my-openagent.git
call :clone https://github.com/xai-org/grok-build.git
call :clone https://github.com/claude-code-best/claude-code.git
call :clone https://github.com/charmbracelet/crush.git
call :clone https://github.com/openai/codex.git
call :clone https://github.com/earendil-works/pi.git
call :clone https://github.com/Dicklesworthstone/pi_agent_rust.git
call :clone https://github.com/aaif-goose/goose.git
call :clone https://github.com/stakpak/agent.git
call :clone https://github.com/vinhnx/VTCode.git
call :clone https://github.com/autohandai/code-cli.git
call :clone https://github.com/JetBrains/junie.git
call :clone https://github.com/femto/minion-code.git
call :clone https://github.com/mistralai/mistral-vibe.git
call :clone https://github.com/Kilo-Org/kilocode.git
call :clone https://github.com/langchain-ai/deepagents.git
call :clone https://github.com/Hmbown/CodeWhale.git
call :clone https://github.com/MoonshotAI/kimi-code.git
call :clone https://github.com/QwenLM/qwen-code.git
call :clone https://github.com/google-gemini/gemini-cli.git
call :clone https://github.com/NousResearch/hermes-agent.git
call :clone https://github.com/cline/cline.git
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
