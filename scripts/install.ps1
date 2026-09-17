# RustCode installer for Windows — PowerShell
#
# Obtain install.ps1 from your distribution channel, then point it at the
# location that hosts the rustcode binaries:
#
#   $env:RUSTCODE_RELEASE_BASE = "https://<your-distribution-host>/releases/download"
#   powershell -ExecutionPolicy Bypass -File install.ps1
#
# Env overrides:
#   $env:RUSTCODE_RELEASE_BASE        download root hosting
#                                       "rustcode-<tag>-windows-<arch>.exe" (required)
#   $env:RUSTCODE_RELEASE_LATEST_API  optional JSON endpoint whose "tag_name" field
#                                       gives the latest release tag (auto-detection)
#   $env:RUSTCODE_VERSION             release tag to install (default: latest release,
#                                       auto-detected from RUSTCODE_RELEASE_LATEST_API when set)
#   $env:RUSTCODE_PREFIX              install dir (default: %LOCALAPPDATA%\RustCode)
# IMPORTANT: when changing install paths, registry edits, or filenames here,
# also update scripts/uninstall.ps1 AND
# crates/rustcode-cli/src/uninstall/paths.rs. The CI parity test guards
# the manifest, but binary path / PATH edit are not checked.

param(
    [string]$Url = "",
    [string]$Key = "",
    [string]$Model = "",
    [string]$Provider = ""
)

$ErrorActionPreference = "Stop"

# --- optional provider injection: -Url/-Key/-Model flags or RUSTCODE_PROVIDER_* env ---
# Maps to the [providers.<name>] block in config.toml (type = "openai-compatible").
$ProviderUrl = if ($Url) { $Url } else { $env:RUSTCODE_PROVIDER_URL }
$ProviderKey = if ($Key) { $Key } else { $env:RUSTCODE_PROVIDER_KEY }
$ProviderModel = if ($Model) { $Model } else { $env:RUSTCODE_PROVIDER_MODEL }
$ProviderName = if ($Provider) { $Provider } else { "custom" }

# `model` is a required field in config.toml's [providers.<name>] block (there is
# no sensible default for an OpenAI-compatible endpoint). Fail closed before any
# download when a provider is being injected without a model.
if ((-not $ProviderModel) -and ($ProviderUrl -or $ProviderKey)) {
    Write-Host "Error: -Model is required when injecting a provider (-Url/-Key given)." -ForegroundColor Red
    exit 1
}

# Release source: provided by the operator/distribution channel via env; there
# is no compiled-in vendor host.
$RepoBase = $env:RUSTCODE_RELEASE_BASE
$RepoLatestApi = $env:RUSTCODE_RELEASE_LATEST_API

# --- detect arch ---
# Prefer PROCESSOR_ARCHITEW6432 (set only when a 32-bit process runs on a 64-bit
# OS — it holds the real OS arch). Fall back to PROCESSOR_ARCHITECTURE.
# Avoids RuntimeInformation::OSArchitecture which is empty on older PS 5.1/.NET.
$RealArch = if ($env:PROCESSOR_ARCHITEW6432) {
    $env:PROCESSOR_ARCHITEW6432
} else {
    $env:PROCESSOR_ARCHITECTURE
}

switch ($RealArch) {
    "AMD64" { $ArchTag = "x64" }
    "ARM64" { $ArchTag = "arm64" }
    default {
        Write-Host "Unsupported architecture: $RealArch (supported: AMD64, ARM64)" -ForegroundColor Red
        exit 1
    }
}

# This build ships no compiled-in release host. The download root must be
# supplied by the operator/distribution channel; fail with guidance instead of
# guessing a vendor URL.
if (-not $RepoBase) {
    Write-Host "Error: no release download source configured." -ForegroundColor Red
    Write-Host "       Set `$env:RUSTCODE_RELEASE_BASE to the directory that hosts the" -ForegroundColor Red
    Write-Host "       rustcode-<tag>-windows-<arch>.exe binaries, then re-run, e.g.:" -ForegroundColor Red
    Write-Host "         `$env:RUSTCODE_RELEASE_BASE = 'https://<your-distribution-host>/releases/download'" -ForegroundColor Red
    Write-Host "         powershell -ExecutionPolicy Bypass -File install.ps1" -ForegroundColor Red
    Write-Host "       Optionally set `$env:RUSTCODE_RELEASE_LATEST_API for automatic" -ForegroundColor Red
    Write-Host "       latest-version detection, or pin `$env:RUSTCODE_VERSION = '<tag>'." -ForegroundColor Red
    exit 1
}
$RepoBase = $RepoBase.TrimEnd('/')

# --- resolve version ---
# Honor $env:RUSTCODE_VERSION if set; otherwise auto-detect the latest release
# tag from RUSTCODE_RELEASE_LATEST_API. Without either we cannot guess a tag
# (this build has no built-in release host), so fail with guidance.
if ($env:RUSTCODE_VERSION) {
    $Version = $env:RUSTCODE_VERSION
} elseif ($RepoLatestApi) {
    Write-Host "==> Detecting latest version"
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $ProgressPreference = 'SilentlyContinue'
        $Latest = Invoke-RestMethod -Uri $RepoLatestApi -UseBasicParsing -TimeoutSec 10
        $Version = $Latest.tag_name
    } catch {
        $Version = $null
    }
    if (-not $Version) {
        Write-Host "Error: could not determine the latest release from:" -ForegroundColor Red
        Write-Host "       $RepoLatestApi" -ForegroundColor Red
        Write-Host "       Set `$env:RUSTCODE_VERSION explicitly (e.g. '<tag>')." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Error: no version specified and no release API configured." -ForegroundColor Red
    Write-Host "       Pin a tag with `$env:RUSTCODE_VERSION = '<tag>', or set" -ForegroundColor Red
    Write-Host "       `$env:RUSTCODE_RELEASE_LATEST_API to a JSON endpoint that returns" -ForegroundColor Red
    Write-Host "       a 'tag_name' field for automatic latest-release detection." -ForegroundColor Red
    exit 1
}

$BinName = "rustcode-$Version-windows-$ArchTag.exe"
$Url = "$RepoBase/$Version/$BinName"

# --- pick install dir ---
$Prefix = if ($env:RUSTCODE_PREFIX) {
    $env:RUSTCODE_PREFIX
} else {
    Join-Path $env:LOCALAPPDATA "RustCode"
}

if (-not (Test-Path $Prefix)) {
    New-Item -ItemType Directory -Path $Prefix -Force | Out-Null
}

# --- download ---
$Dest = Join-Path $Prefix "rustcode.exe"
$TmpFile = Join-Path $env:TEMP "rustcode-download.exe"

Write-Host "==> Downloading $BinName"
Write-Host "    from $Url"

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $Url -OutFile $TmpFile -UseBasicParsing
} catch {
    Write-Host "Error: download failed." -ForegroundColor Red
    Write-Host "       $_" -ForegroundColor Red
    Write-Host "       URL: $Url" -ForegroundColor Red
    exit 1
}

# Sanity check: must not be an HTML page
$Header = [System.IO.File]::ReadAllBytes($TmpFile)[0..3]
if ([char]$Header[0] -eq '<') {
    Write-Host "Error: download looks like an HTML page, not a binary." -ForegroundColor Red
    Write-Host "       The release may not exist, or the URL is wrong." -ForegroundColor Red
    Write-Host "       URL: $Url" -ForegroundColor Red
    Remove-Item $TmpFile -Force -ErrorAction SilentlyContinue
    exit 1
}

# --- install ---
# Move-Item -Force is unreliable on Windows PowerShell 5.1 when the destination
# already exists (see: fails with "当文件已存在时，无法创建该文件"). Do an explicit
# Remove-Item first, and surface a clear message if the old binary is locked
# (rustcode.exe still running in another terminal).
Write-Host "==> Installing to $Dest"
if (Test-Path $Dest) {
    try {
        Remove-Item $Dest -Force -ErrorAction Stop
    } catch {
        Write-Host "Error: cannot replace existing $Dest" -ForegroundColor Red
        Write-Host "       It may be in use. Close any running rustcode.exe and re-run this installer." -ForegroundColor Red
        Write-Host "       $_" -ForegroundColor Red
        Remove-Item $TmpFile -Force -ErrorAction SilentlyContinue
        exit 1
    }
}
Move-Item -Path $TmpFile -Destination $Dest

# --- add to PATH ---
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
# Compare COMPLETE, normalized PATH entries — never a substring of the raw string.
# A raw `-like "*$Prefix*"` false-positives when another entry merely CONTAINS the
# prefix (e.g. "...\RustCodeBackup" for prefix "...\RustCode"), silently skipping the
# real add so rustcode isn't on PATH in a new terminal. Split on ';', trim trailing
# '\' + whitespace, match case-insensitively (Windows paths are case-insensitive).
# Also avoids `-like` treating the prefix as a wildcard pattern (e.g. a literal '[').
$PrefixNorm = $Prefix.TrimEnd('\').Trim()
$InPath = $false
if ($UserPath) {
    foreach ($entry in ($UserPath -split ';')) {
        if ($entry.TrimEnd('\').Trim() -ieq $PrefixNorm) { $InPath = $true; break }
    }
}
if (-not $InPath) {
    $NewPath = if ($UserPath) { "$Prefix;$UserPath" } else { $Prefix }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    # Also update current session so user can use it immediately
    $env:Path = "$Prefix;$env:Path"
    Write-Host ""
    Write-Host "Added $Prefix to user PATH." -ForegroundColor Green
    Write-Host "New terminal windows will pick it up automatically."
}

# --- done ---
Write-Host ""
Write-Host "Installed: $Dest" -ForegroundColor Green
try {
    & $Dest --version
} catch {
    # ignore
}

# --- write custom provider config (optional -Url/-Key/-Model) ---
if ($ProviderUrl -or $ProviderKey -or $ProviderModel) {
    $CfgDir = if ($env:RUSTCODE_HOME) { $env:RUSTCODE_HOME } else { Join-Path $HOME ".rustcode" }
    $Cfg = Join-Path $CfgDir "config.toml"
    if (-not (Test-Path $CfgDir)) { New-Item -ItemType Directory -Path $CfgDir -Force | Out-Null }

    $Esc = { param($s) ($s -replace '\\', '\\' -replace '"', '\"') }

    $Existing = $false
    if (Test-Path $Cfg) {
        $Content = Get-Content -Path $Cfg -Raw -ErrorAction SilentlyContinue
        $Marker = "[providers.""$ProviderName""]"
        $Existing = ($Content -match [regex]::Escape($Marker))
    }

    if ($Existing) {
        Write-Host "[INFO] provider '$ProviderName' already present in $Cfg; skipped."
    } else {
        $Lines = New-Object System.Collections.Generic.List[string]
        if (-not (Test-Path $Cfg)) {
            $Lines.Add("default_provider = `"$ProviderName`"")
            $Lines.Add("")
        }
        $Lines.Add("[providers.`"$ProviderName`"]")
        $Lines.Add('type = "openai-compatible"')
        if ($ProviderUrl)   { $Lines.Add("base_url = `"$(& $Esc $ProviderUrl)`"") }
        if ($ProviderKey)   { $Lines.Add("api_key = `"$(& $Esc $ProviderKey)`"") }
        if ($ProviderModel) { $Lines.Add("model = `"$(& $Esc $ProviderModel)`"") }
        $Lines.Add("")
        Add-Content -Path $Cfg -Value $Lines -Encoding utf8
        Write-Host "[INFO] wrote provider '$ProviderName' to $Cfg"
        Write-Host "[INFO] run 'rustcode' and use /provider to add more models if needed."
    }
}

Write-Host ""
Write-Host "Run 'rustcode' to get started." -ForegroundColor Cyan
