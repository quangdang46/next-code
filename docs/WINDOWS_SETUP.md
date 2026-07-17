# Windows Setup and Troubleshooting Guide

This guide provides Windows-specific setup instructions and troubleshooting for next-code users on Windows 10/11.

## Installation

### Quick Install (PowerShell)

```powershell
irm https://raw.githubusercontent.com/quangdang46/next-code/master/scripts/install.ps1 | iex
```

### Installation Details

The PowerShell installer:
- Downloads the latest next-code release for your architecture (x86_64 or ARM64)
- Installs to `%LOCALAPPDATA%\next-code\bin\next-code.exe` (launcher)
- Stores versioned binaries in `%LOCALAPPDATA%\next-code\builds\versions\<version>\next-code.exe`
- Adds the installation directory to your user PATH
- Optionally installs Alacritty terminal emulator
- Optionally sets up Alt+; global hotkey to launch next-code

### Manual Installation

If the automated installer fails:

1. Download the appropriate release from [GitHub Releases](https://github.com/quangdang46/next-code/releases)
2. Extract `next-code.exe` to a directory of your choice
3. Add that directory to your system PATH
4. Verify installation: `next-code --version`

## Configuration Locations

Windows-specific configuration paths:

| File/Directory | Windows Path |
|----------------|--------------|
| Main config | `%USERPROFILE%\.next-code\config.toml` |
| Auth credentials | `%USERPROFILE%\.next-code\auth.json` |
| Provider env files | `%APPDATA%\next-code\` |
| Build artifacts | `%LOCALAPPDATA%\next-code\builds\` |
| Browser components | `%LOCALAPPDATA%\next-code\browser\` |
| Logs | `%USERPROFILE%\.next-code\logs\` |

## Login and Authentication

### CLI Login (Recommended for Windows)

The CLI provides flags to avoid interactive TUI issues:

```powershell
# OpenAI-compatible provider with all options
next-code login --provider openai-compatible --api-base https://api.deepseek.com --model deepseek-v4-flash --api-key YOUR_API_KEY

# Skip API key prompt (will prompt securely)
next-code login --provider openai-compatible --api-base https://api.deepseek.com --model deepseek-v4-flash
```

Available login flags:
- `--provider <PROVIDER>`: Provider to use (openai, claude, openrouter, etc.)
- `--api-base <URL>`: OpenAI-compatible API base URL
- `--api-key <KEY>`: API key (omitted = secure prompt)
- `--model <MODEL>`: Model to use
- `--account <LABEL>`: Account label for multi-account support
- `--no-browser`: Skip browser OAuth (for headless/SSH)
- `--json`: Machine-readable JSON output for scripting

### Handling Credential Conflicts

next-code may detect credentials from other tools (OpenCode, Codex, etc.):

```
Found existing OpenRouter credentials from OpenCode auth.json at C:\Users\...\auth.json.
next-code will only read that source in place after you approve it.
Trust this auth source for future next-code sessions? [y/N]:
```

Type `y` to approve, or manage credentials manually in the config locations listed above.

### TUI Login

If using the TUI `/login` command:
- Use arrow keys or type to select providers (mouse support varies by terminal)
- Press Enter to confirm selection
- Follow the prompts for your chosen provider

## Terminal Compatibility

### Recommended Terminals

- **Alacritty**: Excellent compatibility, installed automatically by default
- **Windows Terminal**: Good compatibility, use with PowerShell or pwsh
- **PowerShell 7**: Recommended over Windows PowerShell 5.1
- **Git Bash**: Works well for Unix-style workflows

### Terminal Issues

If you experience:
- **Cursor positioning problems**: Try a different terminal (Alacritty recommended)
- **Colors not displaying**: Ensure your terminal supports 256-color mode
- **Input lag**: Check terminal performance settings

## Known Windows Issues

### Issue #82: Login Flow Problems

**Symptoms**: API key input fails, unclear TUI selection, shows OpenCode docs

**Workaround**: Use CLI login with flags (see "CLI Login" above)

**Status**: Partially addressed with `--api-base`, `--api-key`, `--model` flags

### Issue #118: Installer Architecture Error

**Symptoms**: "Unsupported architecture" error during installation

**Workaround**: Ensure you're using PowerShell 5.1+, check your architecture with `$env:PROCESSOR_ARCHITECTURE`

### Issue #140: Browser Setup Missing Binary

**Symptoms**: `next-code browser setup` fails with "Host binary not found"

**Status**: Firefox Agent Bridge native messaging host not included in Windows releases

**Workaround**: Browser automation currently not available on Windows

## Troubleshooting

### Installation Issues

**Problem**: `next-code` command not found after installation
- **Solution**: Open a new terminal window (PATH changes require new session)
- **Alternative**: Add `%LOCALAPPDATA%\next-code\bin` to PATH manually

**Problem**: Installation fails with "PowerShell 5.1 or later required"
- **Solution**: Update PowerShell or use Windows Terminal with PowerShell 7

**Problem**: Download fails or times out
- **Solution**: Manually download from GitHub Releases and extract

### Authentication Issues

**Problem**: "No models are available" after login
- **Solution**: Check that your API key is valid and the model name is correct
- **Solution**: Run `next-code auth-test` to verify credentials

**Problem**: Credential conflicts with other tools
- **Solution**: Approve the credential source when prompted, or manage credentials manually

**Problem**: OAuth flow fails or browser doesn't open
- **Solution**: Use `--no-browser` flag and copy-paste the auth URL manually

### Runtime Issues

**Problem**: next-code won't start or crashes immediately
- **Solution**: Check logs in `%USERPROFILE%\.next-code\logs\`
- **Solution**: Ensure Windows Defender isn't blocking the executable
- **Solution**: Try running as administrator (not recommended for regular use)

**Problem**: Slow performance or high memory usage
- **Solution**: This is expected for local embeddings; disable with config if needed
- **Solution**: Close other sessions to reduce memory footprint

**Problem**: File permission errors
- **Solution**: Windows file permissions work differently than Unix; most operations are no-ops on Windows

## Getting Help

If you encounter issues not covered here:

1. Check existing [GitHub Issues](https://github.com/quangdang46/next-code/issues) for similar problems
2. Search for your error message in the issue tracker
3. File a new issue with:
   - Your Windows version and architecture
   - next-code version (`next-code --version`)
   - Terminal type (PowerShell, Windows Terminal, etc.)
   - Full error message and reproduction steps
   - Relevant logs from `%USERPROFILE%\.next-code\logs\`

## Additional Resources

- [Windows Architecture Notes](WINDOWS.md) - Technical implementation details
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [GitHub Issues](https://github.com/quangdang46/next-code/issues) - Bug reports and feature requests
