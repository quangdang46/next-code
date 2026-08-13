#!/usr/bin/env bash
# ============================================================
#  Clone coding-agent CLI repositories into one folder.
#  TARGET_DIR = where all repos are cloned to (change if needed)
# ============================================================
set -euo pipefail

# ---- Change this to your destination folder ----
TARGET_DIR="${TARGET_DIR:-$HOME/source/github}"

mkdir -p "$TARGET_DIR"
cd "$TARGET_DIR"

echo "====================================="
echo "Cloning coding-agent CLI repos into:"
echo "$TARGET_DIR"
echo "====================================="

clone() {
    url="$1"
    echo
    echo "-------------------------------------"
    echo "Cloning $url"
    if git clone "$url"; then
        echo "Success: $url"
    else
        echo "Failed: $url"
    fi
}

# ---- Open source coding-agent CLIs (cloneable) ----
clone https://github.com/anomalyco/opencode.git
clone https://github.com/1jehuang/jcode.git
clone https://github.com/can1357/oh-my-pi.git
clone https://github.com/Yeachan-Heo/gajae-code.git
clone https://github.com/code-yeongyu/oh-my-openagent.git
clone https://github.com/xai-org/grok-build.git
clone https://github.com/claude-code-best/claude-code.git
clone https://github.com/charmbracelet/crush.git
clone https://github.com/openai/codex.git
clone https://github.com/earendil-works/pi.git
clone https://github.com/Dicklesworthstone/pi_agent_rust.git
clone https://github.com/aaif-goose/goose.git
clone https://github.com/stakpak/agent.git
clone https://github.com/vinhnx/VTCode.git
clone https://github.com/autohandai/code-cli.git
clone https://github.com/femto/minion-code.git
clone https://github.com/mistralai/mistral-vibe.git
clone https://github.com/Kilo-Org/kilocode.git
clone https://github.com/langchain-ai/deepagents.git
clone https://github.com/Hmbown/CodeWhale.git
clone https://github.com/MoonshotAI/kimi-code.git
clone https://github.com/QwenLM/qwen-code.git
clone https://github.com/google-gemini/gemini-cli.git
clone https://github.com/NousResearch/hermes-agent.git
clone https://github.com/cline/cline.git
echo
echo "====================================="
echo "Done!"
echo "====================================="
