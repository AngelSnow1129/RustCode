#!/usr/bin/env python3
"""Publish a Release to Gitee using the Gitee OpenAPI (v5).

This is the Gitee counterpart of scripts/gitcode_release.py. The Gitee
pipeline builds the artifacts once and publishes the SAME release assets to
BOTH Gitee and GitCode; GitCode has no pipeline of its own, so the Gitee
pipeline is the single build+publish source for both repositories.

Gitee OpenAPI endpoints used:

  * Create release
        POST {api_host}/api/v5/repos/{owner}/{repo}/releases?access_token=TOKEN
        JSON body: tag_name, name, body, target_commitish, prerelease
  * Get release by tag (idempotency / resolve id for asset upload)
        GET  {api_host}/api/v5/repos/{owner}/{repo}/releases/tags/{tag_name}?access_token=TOKEN
  * Upload asset
        POST {api_host}/api/v5/repos/{owner}/{repo}/releases/{release_id}/attach_files?access_token=TOKEN
        multipart/form-data field "file" = binary content

Configuration is injected via env or CLI flags; this tool ships no
compiled-in host, namespace or token.
"""

from __future__ import annotations

import argparse
import json
import os
import ssl
import sys
from urllib import error, parse, request

# 固定默认值：默认发布只要求提供 GITEE_ACCESS_TOKEN，其余三项均有可用默认值，
# 仍可用 CLI 参数 / 对应环境变量覆盖。
API_HOST = os.environ.get("GITEE_API_HOST", "https://gitee.com")
REPO_OWNER = os.environ.get("GITEE_OWNER", "SecLab")
REPO_NAME = os.environ.get("GITEE_REPO", "RustCode")
ACCESS_TOKEN = os.environ.get("GITEE_ACCESS_TOKEN", "")

DEFAULT_BODY_TEMPLATE = """## 更新内容

RustCode {tag} 发布。

安装与使用说明见仓库 README。
"""


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Publish a Release to Gitee via its OpenAPI."
    )
    parser.add_argument(
        "--api-host",
        default=API_HOST,
        help=(
            "Gitee API host, e.g. https://gitee.com (a trailing /api/v5 is "
            "also accepted and normalized); env GITEE_API_HOST; "
            "default: https://gitee.com"
        ),
    )
    parser.add_argument(
        "--owner",
        default=REPO_OWNER,
        help="Repository owner or namespace (env GITEE_OWNER); default: SecLab",
    )
    parser.add_argument(
        "--repo",
        default=REPO_NAME,
        help="Repository name (env GITEE_REPO); default: RustCode",
    )
    parser.add_argument(
        "--access-token",
        default=ACCESS_TOKEN,
        help=(
            "Personal Access Token with repo permission on the Gitee repo "
            "(env GITEE_ACCESS_TOKEN); required"
        ),
    )
    parser.add_argument("--tag-name", required=True, help="Tag label, e.g. v1.0.0")
    parser.add_argument(
        "--name",
        default="",
        help='Release title (defaults to "--tag-name", e.g. "v1.0.0")',
    )
    parser.add_argument("--body", default="", help="Release description (Markdown)")
    parser.add_argument(
        "--body-file",
        default="",
        help="Read the release description from a file (overrides --body)",
    )
    parser.add_argument(
        "--target-commitish",
        default="",
        help=(
            "Branch name or commit SHA; auto-creates the tag if it does not "
            "exist yet (defaults to the default branch's latest commit)"
        ),
    )
    parser.add_argument(
        "--release-status",
        default="latest",
        choices=("latest", "pre"),
        help="Release status: latest (prerelease=false) or pre (prerelease=true)",
    )
    parser.add_argument(
        "--attach",
        default="",
        help="Optional local file path to upload as a release asset",
    )
    parser.add_argument(
        "--file-name",
        default="",
        help="Asset file name (defaults to --attach basename)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the request that would be sent and exit without calling the API",
    )
    return parser


def normalize_host(host: str) -> str:
    """Strip a trailing /api/v5 so appending /api/v5 below never doubles it."""
    return host.rstrip("/").removesuffix("/api/v5")


def auth_url(host: str, token: str) -> str:
    return f"{normalize_host(host)}/api/v5/repos"


def build_release_url(host: str, owner: str, repo: str, token: str) -> str:
    base = f"{auth_url(host, token)}/{parse.quote(owner)}/{parse.quote(repo)}/releases"
    return f"{base}?access_token={parse.quote(token)}"


def build_release_by_tag_url(
    host: str, owner: str, repo: str, token: str, tag_name: str
) -> str:
    base = (
        f"{auth_url(host, token)}/{parse.quote(owner)}/{parse.quote(repo)}"
        f"/releases/tags/{parse.quote(tag_name)}"
    )
    return f"{base}?access_token={parse.quote(token)}"


def build_upload_url(
    host: str, owner: str, repo: str, token: str, release_id: str
) -> str:
    base = (
        f"{auth_url(host, token)}/{parse.quote(owner)}/{parse.quote(repo)}"
        f"/releases/{parse.quote(str(release_id))}/attach_files"
    )
    return f"{base}?access_token={parse.quote(token)}"


def build_payload(
    tag_name: str,
    name: str,
    body: str,
    target_commitish: str,
    prerelease: bool,
) -> dict:
    payload: dict = {
        "tag_name": tag_name,
        "name": name or tag_name,
        "body": body or DEFAULT_BODY_TEMPLATE.format(tag=tag_name),
        "prerelease": prerelease,
    }
    if target_commitish:
        payload["target_commitish"] = target_commitish
    return payload


def send_json(url: str, method: str = "GET", payload: dict | None = None) -> dict:
    data = None
    headers = {}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
        headers["Accept"] = "application/json"

    req = request.Request(url=url, data=data, headers=headers, method=method)
    ssl_context = ssl._create_unverified_context()
    with request.urlopen(req, timeout=30, context=ssl_context) as resp:
        response_body = resp.read().decode("utf-8", errors="replace")
        return json.loads(response_body) if response_body else {}


def send_multipart(url: str, file_path: str, file_name: str) -> dict:
    boundary = "----gitee_release_boundary_7Q4x"
    with open(file_path, "rb") as file_obj:
        file_data = file_obj.read()

    head = (
        f"--{boundary}\r\n"
        f'Content-Disposition: form-data; name="file"; filename="{file_name}"\r\n'
        f"Content-Type: application/octet-stream\r\n\r\n"
    ).encode("utf-8")
    tail = f"\r\n--{boundary}--\r\n".encode("utf-8")
    body = head + file_data + tail

    req = request.Request(url=url, data=body, method="POST")
    req.add_header("Content-Type", f"multipart/form-data; boundary={boundary}")
    ssl_context = ssl._create_unverified_context()
    with request.urlopen(req, timeout=60, context=ssl_context) as resp:
        response_body = resp.read().decode("utf-8", errors="replace").strip()
        return json.loads(response_body) if response_body else {}


def is_already_exists(exc: error.HTTPError, details: str) -> bool:
    if exc.code not in {400, 403, 409}:
        return False
    try:
        payload = json.loads(details) if details else {}
    except json.JSONDecodeError:
        payload = {}
    messages = " ".join(
        str(payload.get(key, "")) for key in ("message", "error", "errors")
    )
    return (
        "already exist" in messages
        or "already has" in messages
        or "exist" in messages
    )


def resolve_release_id(
    host: str, owner: str, repo: str, token: str, tag_name: str
) -> int | None:
    """Return existing release id for tag_name, or None if not found."""
    url = build_release_by_tag_url(host, owner, repo, token, tag_name)
    try:
        resp = send_json(url, method="GET")
    except error.HTTPError as exc:
        if exc.code == 404:
            return None
        details = exc.read().decode("utf-8", errors="replace")
        print(f"HTTP {exc.code}: {details}", file=sys.stderr)
        return None
    rid = resp.get("id")
    return int(rid) if rid is not None else None


def build_release_by_id_url(
    host: str, owner: str, repo: str, token: str, release_id: str
) -> str:
    base = (
        f"{auth_url(host, token)}/{parse.quote(owner)}/{parse.quote(repo)}"
        f"/releases/{parse.quote(str(release_id))}"
    )
    return f"{base}?access_token={parse.quote(token)}"


def list_release_assets(
    host: str, owner: str, repo: str, token: str, release_id: str
) -> list[dict]:
    """Return the release's attached assets (id + name) for de-dup/overwrite."""
    url = build_release_by_id_url(host, owner, repo, token, release_id)
    try:
        resp = send_json(url, method="GET")
    except error.HTTPError as exc:
        details = exc.read().decode("utf-8", errors="replace")
        print(f"[WARN] list_release_assets failed: HTTP {exc.code}: {details}",
              file=sys.stderr)
        return []
    return resp.get("assets", []) or []


def delete_release_asset(
    host: str, owner: str, repo: str, token: str, release_id: str, asset_id: str
) -> None:
    """Remove an existing attached asset so a re-upload overwrites it."""
    url = (
        f"{auth_url(host, token)}/{parse.quote(owner)}/{parse.quote(repo)}"
        f"/releases/{parse.quote(str(release_id))}/attach_files/"
        f"{parse.quote(str(asset_id))}?access_token={parse.quote(token)}"
    )
    send_json(url, method="DELETE")


def create_release(
    host: str,
    owner: str,
    repo: str,
    token: str,
    tag_name: str,
    name: str,
    body: str,
    target_commitish: str,
    prerelease: bool,
) -> dict:
    url = build_release_url(host, owner, repo, token)
    payload = build_payload(tag_name, name, body, target_commitish, prerelease)
    return send_json(url, method="POST", payload=payload)


def ensure_release(
    host: str,
    owner: str,
    repo: str,
    token: str,
    tag_name: str,
    name: str,
    body: str,
    target_commitish: str,
    prerelease: bool,
) -> dict:
    """Idempotently ensure a release exists for tag_name; return its record."""
    existing_id = resolve_release_id(host, owner, repo, token, tag_name)
    if existing_id is not None:
        return {"id": existing_id, "skipped": True, "reason": "release already exists"}

    try:
        return create_release(
            host, owner, repo, token, tag_name, name, body, target_commitish, prerelease
        )
    except error.HTTPError as exc:
        details = exc.read().decode("utf-8", errors="replace")
        if is_already_exists(exc, details):
            existing_id = resolve_release_id(host, owner, repo, token, tag_name)
            if existing_id is not None:
                return {
                    "id": existing_id,
                    "skipped": True,
                    "reason": "release already exists (recovered)",
                }
        print(f"HTTP {exc.code}: {details}", file=sys.stderr)
        raise


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    missing = []
    if not args.api_host:
        missing.append("API host (set GITEE_API_HOST or pass --api-host)")
    if not args.owner:
        missing.append("repository owner (set GITEE_OWNER or pass --owner)")
    if not args.dry_run and not args.access_token:
        missing.append(
            "access token (set GITEE_ACCESS_TOKEN or pass --access-token)"
        )
    if missing:
        print("Missing required release configuration:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        return 1

    body = args.body
    if args.body_file:
        try:
            with open(args.body_file, "r", encoding="utf-8") as file_obj:
                body = file_obj.read()
        except OSError as exc:
            print(str(exc), file=sys.stderr)
            return 1

    file_name = args.file_name or os.path.basename(args.attach or "")
    if args.attach and not file_name:
        print("Missing asset file name (pass --file-name).", file=sys.stderr)
        return 1
    if args.attach and not os.path.isfile(args.attach):
        print(f"File not found: {args.attach}", file=sys.stderr)
        return 1

    prerelease = args.release_status == "pre"
    host = args.api_host
    owner = args.owner
    repo = args.repo
    token = args.access_token

    release_url = build_release_url(host, owner, repo, token)
    payload = build_payload(
        args.tag_name, args.name, body, args.target_commitish, prerelease
    )

    if args.dry_run:
        summary = {
            "dry_run": True,
            "method": "POST",
            "url": release_url,
            "payload": payload,
        }
        if args.attach:
            summary["asset_upload"] = {
                "file_path": args.attach,
                "file_name": file_name,
                "note": "multipart POST to .../releases/{id}/attach_files",
            }
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 0

    try:
        release_result = ensure_release(
            host,
            owner,
            repo,
            token,
            args.tag_name,
            args.name,
            body,
            args.target_commitish,
            prerelease,
        )
    except error.HTTPError as exc:  # already surfaced in ensure_release
        return 1
    except error.URLError as exc:
        print(f"Request failed: {exc}", file=sys.stderr)
        return 1
    except OSError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    release_id = release_result.get("id")
    if release_id is None:
        print(
            "Could not resolve release id from response; cannot upload assets.",
            file=sys.stderr,
        )
        print(json.dumps(release_result, ensure_ascii=False), file=sys.stderr)
        return 1

    result: dict = {"release": release_result}

    if args.attach:
        # Idempotent overwrite: drop any existing asset with the same file name
        # so re-running the pipeline does not append duplicate attachments.
        try:
            for a in list_release_assets(host, owner, repo, token, release_id):
                if a.get("name") == file_name:
                    aid = a.get("id")
                    print(
                        f"[*] Removing existing asset '{file_name}' "
                        f"(id={aid}) before re-upload (overwrite)...",
                        file=sys.stderr,
                    )
                    delete_release_asset(
                        host, owner, repo, token, release_id, aid
                    )
                    break
        except error.HTTPError as exc:
            print(
                f"[WARN] pre-upload de-dup check failed: HTTP {exc.code}; "
                "proceeding with upload (may duplicate).",
                file=sys.stderr,
            )
        try:
            upload_url = build_upload_url(
                host, owner, repo, token, release_id
            )
            result["asset_upload"] = send_multipart(
                upload_url, args.attach, file_name
            )
        except error.HTTPError as exc:
            details = exc.read().decode("utf-8", errors="replace")
            print(f"HTTP {exc.code}: {details}", file=sys.stderr)
            return 1
        except error.URLError as exc:
            print(f"Request failed: {exc}", file=sys.stderr)
            return 1

    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
