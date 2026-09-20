#!/usr/bin/env python3
"""Publish a Release to GitCode using the official GitCode OpenAPI.

Create Release endpoint (see GitCode OpenAPI docs):

    POST {api_host}/api/v5/repos/{owner}/{repo}/releases?access_token=...

Request body:
    tag_name          (required) tag label, e.g. v1.0.0
    name              (required) release title, defaults to tag_name
    body              (required) release description, Markdown supported
    target_commitish  (optional) branch name or commit SHA; when the tag does
                      not exist yet, it is auto-created from this ref. Falls
                      back to the default branch's latest commit when omitted.
    release_status    (optional) "latest" or "pre" (prerelease)

Attachment upload (optional, --attach) reuses the GitLab-v5-compatible
`releases/{tag}/upload_url` dialect already shipped by
.github/workflows/create_tag_release.py. Override the endpoint with
--upload-url-endpoint when your host uses a different one.

Configuration is injected via env or CLI flags; this tool ships no compiled-in
host, namespace or token.
"""

from __future__ import annotations

import argparse
import json
import os
import ssl
import sys
from urllib import error, parse, request

API_HOST = os.environ.get("RUSTCODE_RELEASE_API_HOST", "https://api.gitcode.com")
REPO_OWNER = os.environ.get("RUSTCODE_RELEASE_OWNER", "SecLab")
# 仓库路径取实际 URL 路径: https://gitcode.com/SecLab/RustCode -> "RustCode".
# 官方文档未说明 owner/repo 是否大小写敏感, 故默认值与 URL 保持一致; 如实际
# 调用报找不到仓库, 显式传 --repo / RUSTCODE_RELEASE_REPO 覆盖即可.
REPO_NAME = os.environ.get("RUSTCODE_RELEASE_REPO", "RustCode")
ACCESS_TOKEN = os.environ.get("RUSTCODE_RELEASE_ACCESS_TOKEN", "")

ALLOWED_RELEASE_STATUSES = ("latest", "pre")

DEFAULT_BODY_TEMPLATE = """## 更新内容

RustCode {tag} 发布。

安装与使用说明见仓库 README。
"""


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Publish a Release to GitCode via its OpenAPI."
    )
    parser.add_argument(
        "--api-host",
        default=API_HOST,
        help=(
            "GitCode API host, e.g. https://api.gitcode.com (a trailing "
            "/api/v5 is also accepted and normalized); "
            "env RUSTCODE_RELEASE_API_HOST; required"
        ),
    )
    parser.add_argument(
        "--owner",
        default=REPO_OWNER,
        help="Repository owner or namespace (env RUSTCODE_RELEASE_OWNER); required",
    )
    parser.add_argument(
        "--repo",
        default=REPO_NAME,
        help="Repository name (env RUSTCODE_RELEASE_REPO); default: rustcode",
    )
    parser.add_argument(
        "--access-token",
        default=ACCESS_TOKEN,
        help=(
            "Personal Access Token with write/admin permission on the repo "
            "(env RUSTCODE_RELEASE_ACCESS_TOKEN); required"
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
        default="",
        choices=ALLOWED_RELEASE_STATUSES,
        help="Release status: latest (default at API level) or pre",
    )
    parser.add_argument(
        "--attach",
        default="",
        help="Optional local file path to upload as a release asset",
    )
    parser.add_argument(
        "--file-name",
        default="",
        help='Asset file name used for the upload URL (defaults to --attach basename)',
    )
    parser.add_argument(
        "--upload-url-endpoint",
        default="",
        help=(
            "Full upload URL endpoint for release assets; defaults to the "
            "GitLab-v5-compatible "
            "{api}/api/v5/repos/{owner}/{repo}/releases/{tag}/upload_url"
        ),
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


def build_release_url(host: str, owner: str, repo: str, token: str) -> str:
    base = f"{normalize_host(host)}/api/v5/repos/{owner}/{repo}/releases"
    return f"{base}?access_token={parse.quote(token)}"


def build_payload(
    tag_name: str,
    name: str,
    body: str,
    target_commitish: str,
    release_status: str,
) -> dict:
    payload: dict = {
        "tag_name": tag_name,
        "name": name or tag_name,
        "body": body or DEFAULT_BODY_TEMPLATE.format(tag=tag_name),
    }
    if target_commitish:
        payload["target_commitish"] = target_commitish
    if release_status:
        payload["release_status"] = release_status
    return payload


def send_request(url: str, method: str = "GET", payload: dict | None = None) -> dict:
    data = None
    headers = {}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
        headers["Accept"] = "application/json"

    req = request.Request(
        url=url,
        data=data,
        headers=headers,
        method=method,
    )
    ssl_context = ssl._create_unverified_context()

    with request.urlopen(req, timeout=30, context=ssl_context) as resp:
        response_body = resp.read().decode("utf-8", errors="replace")
        return json.loads(response_body) if response_body else {}


def create_release(
    host: str,
    owner: str,
    repo: str,
    token: str,
    tag_name: str,
    name: str,
    body: str,
    target_commitish: str,
    release_status: str,
) -> dict:
    url = build_release_url(host, owner, repo, token)
    payload = build_payload(
        tag_name, name, body, target_commitish, release_status
    )
    return send_request(url, method="POST", payload=payload)


def is_release_already_exists(exc: error.HTTPError, details: str) -> bool:
    if exc.code not in {400, 409}:
        return False
    try:
        payload = json.loads(details) if details else {}
    except json.JSONDecodeError:
        payload = {}
    messages = " ".join(
        str(payload.get(key, "")) for key in (
            "error_message", "message", "error", "errors"
        )
    )
    return "already exists" in messages or "already exist" in messages


def build_upload_url(
    host: str,
    owner: str,
    repo: str,
    token: str,
    tag_name: str,
    file_name: str,
    endpoint_override: str = "",
) -> str:
    if endpoint_override:
        return endpoint_override
    base = (
        f"{normalize_host(host)}/api/v5/repos/{owner}/{repo}/releases/"
        f"{parse.quote(tag_name)}/upload_url"
    )
    return (
        f"{base}?access_token={parse.quote(token)}"
        f"&file_name={parse.quote(file_name)}"
    )


def list_release_links(
    host: str, owner: str, repo: str, token: str, tag_name: str
) -> list[dict]:
    """List a release's asset links (GitLab-v5 compatible) for de-dup/overwrite."""
    base = (
        f"{normalize_host(host)}/api/v5/repos/{parse.quote(owner)}/"
        f"{parse.quote(repo)}/releases/{parse.quote(tag_name)}/assets/links"
    )
    url = f"{base}?access_token={parse.quote(token)}"
    try:
        data = send_request(url)
    except error.HTTPError:
        return []
    return data if isinstance(data, list) else data.get("assets", []) or []


def delete_release_link(
    host: str, owner: str, repo: str, token: str, tag_name: str, link_id: str
) -> None:
    base = (
        f"{normalize_host(host)}/api/v5/repos/{parse.quote(owner)}/"
        f"{parse.quote(repo)}/releases/{parse.quote(tag_name)}/assets/links/"
        f"{parse.quote(str(link_id))}"
    )
    url = f"{base}?access_token={parse.quote(token)}"
    send_request(url, method="DELETE")


def upload_release_asset(upload_result: dict, file_path: str) -> dict:
    upload_url = upload_result.get("url")
    upload_headers = upload_result.get("headers", {})
    if not upload_url:
        raise ValueError("Missing upload URL in upload_url response.")

    with open(file_path, "rb") as file_obj:
        file_data = file_obj.read()

    req = request.Request(
        url=upload_url,
        data=file_data,
        headers=upload_headers,
        method="PUT",
    )
    ssl_context = ssl._create_unverified_context()

    with request.urlopen(req, timeout=60, context=ssl_context) as resp:
        response_body = resp.read().decode("utf-8", errors="replace").strip()
        return {
            "status_code": resp.status,
            "body": response_body or "success",
        }


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    missing = []
    if not args.api_host:
        missing.append("API host (set RUSTCODE_RELEASE_API_HOST or pass --api-host)")
    if not args.owner:
        missing.append("repository owner (set RUSTCODE_RELEASE_OWNER or pass --owner)")
    # A dry run never touches the network, so the token stays optional there.
    if not args.dry_run and not args.access_token:
        missing.append(
            "access token (set RUSTCODE_RELEASE_ACCESS_TOKEN or pass --access-token)"
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

    url = build_release_url(args.api_host, args.owner, args.repo, args.access_token)
    payload = build_payload(
        args.tag_name,
        args.name,
        body,
        args.target_commitish,
        args.release_status,
    )

    if args.dry_run:
        summary = {
            "dry_run": True,
            "method": "POST",
            "url": url,
            "payload": payload,
        }
        if args.attach:
            summary["asset_upload"] = {
                "file_path": args.attach,
                "file_name": file_name,
                "upload_url": build_upload_url(
                    args.api_host,
                    args.owner,
                    args.repo,
                    args.access_token,
                    args.tag_name,
                    file_name,
                    args.upload_url_endpoint,
                ),
            }
        print(json.dumps(summary, ensure_ascii=False, indent=2))
        return 0

    try:
        release_result = create_release(
            args.api_host,
            args.owner,
            args.repo,
            args.access_token,
            args.tag_name,
            args.name,
            body,
            args.target_commitish,
            args.release_status,
        )
    except error.HTTPError as exc:
        details = exc.read().decode("utf-8", errors="replace")
        if is_release_already_exists(exc, details):
            release_result = {
                "skipped": True,
                "reason": "release already exists",
                "details": json.loads(details) if details else {},
            }
        else:
            print(f"HTTP {exc.code}: {details}", file=sys.stderr)
            return 1
    except error.URLError as exc:
        print(f"Request failed: {exc}", file=sys.stderr)
        return 1
    except OSError as exc:  # e.g. body file read failure
        print(str(exc), file=sys.stderr)
        return 1

    result: dict = {"release": release_result}

    if args.attach:
        # Idempotent overwrite: drop any existing asset link with the same file
        # name so re-running the pipeline does not append duplicate assets.
        try:
            for lk in list_release_links(
                args.api_host,
                args.owner,
                args.repo,
                args.access_token,
                args.tag_name,
            ):
                if lk.get("name") == file_name:
                    lid = lk.get("id")
                    print(
                        f"[*] Removing existing asset link '{file_name}' "
                        f"(id={lid}) before re-upload (overwrite)...",
                        file=sys.stderr,
                    )
                    delete_release_link(
                        args.api_host,
                        args.owner,
                        args.repo,
                        args.access_token,
                        args.tag_name,
                        lid,
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
                args.api_host,
                args.owner,
                args.repo,
                args.access_token,
                args.tag_name,
                file_name,
                args.upload_url_endpoint,
            )
            upload_result = send_request(upload_url)
            result["asset_upload"] = upload_release_asset(upload_result, args.attach)
        except ValueError as exc:
            print(str(exc), file=sys.stderr)
            return 1
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