# release/ — repo-committed program fallback

This directory is the **pipeline-independent fallback source** for
`scripts/install.sh` / `scripts/install.ps1`. It is NOT a replacement for the
online Release; it is a safety net so that **a CI/CD pipeline failure never
blocks a user from downloading a working binary.**

## Layout (one dir per version, no OS/ARCH sub-split)

```text
release/
├── index.json          # top-level index: versions (newest-first) + targets per version
├── v5.1.0/
│   ├── rustcode-v5.1.0-linux-x64
│   ├── rustcode-v5.1.0-windows-x64.exe
│   ├── rustcode-daemon-v5.1.0-linux-x64
│   ├── manifest.json   # per-version: binaries -> {file, sha256, size}
│   └── ...             # whatever platforms the publishing host built
└── .tmp/               # gitignored staging dir (atomic publish only)
```

The OS and CPU architecture are encoded in the **filename**
(`rustcode-<version>-<os>-<arch>[.exe]`, os ∈ {darwin,linux,ohos,windows},
arch ∈ {arm64,x64}) — they are NOT represented by extra directory levels.

## How it gets populated

Run any release script on a dev host (it builds for that host's environment) and
commit the result:

```sh
scripts/release.sh                # builds + publishes all targets it can
# or: linux-release-linux.sh / macos-release-*.sh / cross-build.sh / release-daemon.sh
git add release/ && git commit -m "chore: publish vX.Y.Z to release/"
```

Every release script now calls `scripts/release-publish.sh` after a successful
build. That helper:

- copies only **verified** binaries (size > 0, not an HTML error page) into
  `release/<version>/`;
- **merges** new binaries into an existing version dir and never deletes a
  previously committed one (so a partial / failed build cannot clobber good
  artifacts);
- writes `manifest.json` and a sorted `index.json` via an atomic temp-dir rename,
  so a crash never leaves a half-written version behind.

## Publish-before-push gate (required)

`.githooks/pre-push` refuses a push that does not carry a **committed**,
build-passing artifact for the version being pushed. The gate never builds —
building inside a hook blocks every push for minutes and can OOM a small host —
it only verifies what you committed and prints the exact fix.

Two strengths, selected with `RUSTCODE_PREPUSH_RELEASE`:

| Value | Requires |
|---|---|
| `on` (default) | The pushed commit contains `release/<version>/manifest.json` **and** a committed binary for the pushing host's OS/ARCH. `<version>` is read from the pushed `Cargo.toml`, so a version bump always forces a fresh artifact while iterating inside a version does not. |
| `strict` | Additionally: `manifest.source.sha` is an ancestor of the pushed commit, the only changes in between are under `release/`, and `manifest.source.dirty == false`. This proves the artifact was built from this exact code. |
| `off` | Skip the gate entirely. |

`manifest.source.sha` can never *equal* the pushed sha: the manifest is written at
build time and the artifact is committed afterwards, so it would have to contain
its own commit hash. "Ancestor, with no source drift in between" is the strongest
satisfiable predicate, and it is what `strict` checks.

Run it by hand at any time:

```sh
bash scripts/prepush-release-check.sh            # check HEAD
bash scripts/prepush-release-check.sh --self-test
```

CI enforces the same predicate (the `release-gate` job in
`.github/workflows/ci.yml`), so `git push --no-verify` does not get an
artifact-less commit onto `dev` either.

## How the download script falls back

`install.sh` (and `install.ps1`) resolve candidate versions
(API latest, then `release/index.json` newest-first) and, for each version, try
sources in order:

1. **Online Release** (GitCode releases/download) — primary.
2. **This repo's `release/<version>/`** via the raw file URL — fallback,
   independent of any pipeline.

If the latest version has no binary for the current OS/ARCH, it walks to older
versions. Only when every candidate/source fails does it print a clear error
with the detected OS, ARCH, the versions tried, and the URLs tried.
