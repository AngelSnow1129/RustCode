# RustCode installer for Windows — PowerShell
#
#   irm https://gitcode.com/api/v5/repos/SecLab/RustCode/raw/scripts/install.ps1?ref=dev | iex
#
# Detects architecture automatically, downloads the latest release binary
# from GitCode, and installs it to PATH.
#
# Env overrides (optional):
#   $env:RUSTCODE_RELEASE_BASE        override download root
#   $env:RUSTCODE_RELEASE_LATEST_API  override latest-version API endpoint
#   $env:RUSTCODE_VERSION             pin a specific release tag (default: latest)
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

# Release source: defaults to the GitCode repository so `irm | iex` works
# zero-config. Override via env for alternative distribution channels.
$RepoBase = if ($env:RUSTCODE_RELEASE_BASE) { $env:RUSTCODE_RELEASE_BASE } else { "https://gitcode.com/SecLab/RustCode/releases/download" }
$RepoLatestApi = if ($env:RUSTCODE_RELEASE_LATEST_API) { $env:RUSTCODE_RELEASE_LATEST_API } else { "https://api.gitcode.com/api/v5/repos/SecLab/RustCode/releases/latest" }

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

# --- repo-committed fallback source (pipeline-independent) ---
# GitCode's raw-file endpoint is <base>/<path>?ref=<ref> (ref as a QUERY param).
# The path-segment form (/raw/<ref>/<path>) returns the SPA HTML shell, not the file.
$RepoRawBase = if ($env:RUSTCODE_RELEASE_RAW_BASE) { $env:RUSTCODE_RELEASE_RAW_BASE } else { "https://gitcode.com/api/v5/repos/SecLab/RustCode/raw" }
$RepoRawRef  = if ($env:RUSTCODE_RELEASE_RAW_REF)  { $env:RUSTCODE_RELEASE_RAW_REF }  else { "dev" }

# --- candidate version list ---
# Layer 1 (online Release) is tried first; Layer 2 is the repo-committed
# release/ directory (raw file URL), which does NOT depend on the CI/CD
# pipeline, so a pipeline failure never blocks a download.
$Candidates = @()
if ($env:RUSTCODE_VERSION) {
    $Candidates = @($env:RUSTCODE_VERSION)
} else {
    if ($Version) { $Candidates = @($Version) }
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $ProgressPreference = 'SilentlyContinue'
        $IdxUrl = "$RepoRawBase/release/index.json?ref=$RepoRawRef"
        $Idx = Invoke-RestMethod -Uri $IdxUrl -UseBasicParsing -TimeoutSec 10
        if ($Idx -and $Idx.versions) {
            foreach ($e in $Idx.versions) {
                if ($e.version -and $Candidates -notcontains $e.version) { $Candidates += $e.version }
            }
        }
    } catch {
        # index unreachable; rely on the API latest only
    }
}
if ($Candidates.Count -eq 0) {
    Write-Host "Error: no version resolved." -ForegroundColor Red
    Write-Host "       Set `$env:RUSTCODE_VERSION (e.g. 'vX.Y.Z'), or ensure" -ForegroundColor Red
    Write-Host "       RUSTCODE_RELEASE_LATEST_API / the repo release/index.json is reachable." -ForegroundColor Red
    exit 1
}
Write-Host "==> Candidate versions: $($Candidates -join ', ')"

# --- pick install dir ---
$Prefix = if ($env:RUSTCODE_PREFIX) {
    $env:RUSTCODE_PREFIX
} else {
    Join-Path $env:LOCALAPPDATA "RustCode"
}

if (-not (Test-Path $Prefix)) {
    New-Item -ItemType Directory -Path $Prefix -Force | Out-Null
}

# --- download with multi-source / multi-version fallback ---
$Dest = Join-Path $Prefix "rustcode.exe"
$TmpFile = Join-Path $env:TEMP "rustcode-download.exe"

$Downloaded = $false
$Attempted = @()
foreach ($Ver in $Candidates) {
    $Bin = "rustcode-$Ver-windows-$ArchTag.exe"
    $SrcOnline = "$($RepoBase.TrimEnd('/'))/$Ver/$Bin"
    $SrcRepo   = "$($RepoRawBase.TrimEnd('/'))/release/$Ver/$Bin?ref=$RepoRawRef"
    foreach ($Src in @($SrcOnline, $SrcRepo)) {
        $Attempted += $Src
        Write-Host "==> Trying $Src"
        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            $ProgressPreference = 'SilentlyContinue'
            Invoke-WebRequest -Uri $Src -OutFile $TmpFile -UseBasicParsing
        } catch {
            Remove-Item $TmpFile -Force -ErrorAction SilentlyContinue
            continue
        }
        if (-not (Test-Path $TmpFile) -or (Get-Item $TmpFile).Length -eq 0) {
            Remove-Item $TmpFile -Force -ErrorAction SilentlyContinue
            continue
        }
        $Header = [System.IO.File]::ReadAllBytes($TmpFile)[0..3]
        if ([char]$Header[0] -eq '<') {
            Remove-Item $TmpFile -Force -ErrorAction SilentlyContinue
            continue
        }
        Write-Host "    -> got $Bin ($(Get-Item $TmpFile).Length bytes)"
        $Downloaded = $true
        break
    }
    if ($Downloaded) { break }
}

if (-not $Downloaded) {
    Write-Host "Error: could not download a usable rustcode binary." -ForegroundColor Red
    Write-Host "       ARCH=$ArchTag" -ForegroundColor Red
    Write-Host "       Tried versions: $($Candidates -join ', ')" -ForegroundColor Red
    Write-Host "       Tried sources:" -ForegroundColor Red
    foreach ($a in $Attempted) { Write-Host "         $a" -ForegroundColor Red }
    Write-Host "       A pipeline failure should not block download; the repo release/" -ForegroundColor Red
    Write-Host "       directory may simply be empty. Populate it by running a release" -ForegroundColor Red
    Write-Host "       script on a dev host and committing it." -ForegroundColor Red
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
