#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import ssl
import sys
from urllib import error, parse, request

# Release-target configuration. This build ships no compiled-in release host or
# namespace: the operator/distributor supplies them via env or CLI flags.
# The API dialect is GitLab-v5-compatible (releases + upload_url endpoints).
REPO_OWNER = os.environ.get("RUSTCODE_RELEASE_OWNER", "SecLab")
# 仓库路径取实际 URL 路径: https://gitcode.com/SecLab/RustCode -> "RustCode".
# 官方文档未说明 owner/repo 是否大小写敏感, 故默认值与 URL 保持一致; 如实际
# 调用报找不到仓库, 显式传 --repo / RUSTCODE_RELEASE_REPO 覆盖即可.
REPO_NAME = os.environ.get("RUSTCODE_RELEASE_REPO", "RustCode")
ACCESS_TOKEN = os.environ.get("RUSTCODE_RELEASE_ACCESS_TOKEN", "")
API_HOST = os.environ.get("RUSTCODE_RELEASE_API_HOST", "https://api.gitcode.com")
# GitCode 官方文档仅为 release_status 定义两个值: latest(最新) / pre(预发布).
ALLOWED_RELEASE_STATUSES = ("latest", "pre")
BODY_TEMPLATE = """
Release  Note

使用于mac和linux的当前最新版本，安装命令如下

```bash
# 进入 ~/.local/bin 目录
cd ~/.local/bin

# 通过 Finder 打开目录
open .

# 将下载后的 rustcode-xxx 文件放入到目录，重命名为 rustcode

# 设置 rustcode 的运行权限
chmod +x rustcode
```

最后，在终端中输入 rustcode 并运行即可
"""



def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Create a release through a GitLab-v5-compatible release API, "
            "then fetch the attachment upload URL."
        )
    )
    parser.add_argument(
        "--api-host",
        default=API_HOST,
        help=(
            "Release API host, e.g. https://api.gitcode.com (a trailing "
            "/api/v5 is also accepted and normalized); "
            "env RUSTCODE_RELEASE_API_HOST; default: https://api.gitcode.com"
        ),
    )
    parser.add_argument(
        "--owner",
        default=REPO_OWNER,
        help="Repository owner or namespace (env RUSTCODE_RELEASE_OWNER); default: SecLab",
    )
    parser.add_argument(
        "--repo",
        default=REPO_NAME,
        help="Repository name (env RUSTCODE_RELEASE_REPO); default: RustCode",
    )
    parser.add_argument(
        "--access-token",
        default=ACCESS_TOKEN,
        help="Release API access token (env RUSTCODE_RELEASE_ACCESS_TOKEN); required",
    )
    parser.add_argument("--tag-name", required=True, help="Tag name to create")
    parser.add_argument(
        "--body", default="", help="Release description (Markdown); required by the GitCode API, falls back to BODY_TEMPLATE"
    )
    # 官方文档: target_commitish 非必填. tag 不存在时用于新建 tag; 省略则
    # 默认取默认分支的最新提交.
    parser.add_argument(
        "--target-commitish",
        default="",
        help="Branch name or commit SHA; auto-creates the tag if it does not exist (defaults to the default branch's latest commit)",
    )
    # 官方文档: release_status 非必填, 仅允许 pre / latest.
    parser.add_argument(
        "--release-status",
        default="",
        choices=ALLOWED_RELEASE_STATUSES,
        help="Release status: latest (default at API level) or pre (prerelease)",
    )
    parser.add_argument(
        "--file-name",
        required=True,
        help="Attachment file name used to fetch the upload URL",
    )
    parser.add_argument(
        "--file-path",
        required=True,
        help="Local file path to upload",
    )
    parser.add_argument(
        "--insecure",
        action="store_true",
        help="Retained for compatibility; SSL verification is already skipped by default",
    )
    return parser


def send_request(url: str, method: str = "GET", payload: dict | None = None) -> dict:
    data = None
    headers = {}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"

    req = request.Request(
        url=url,
        data=data,
        headers=headers,
        method=method,
    )
    ssl_context = ssl._create_unverified_context()

    with request.urlopen(req, timeout=30, context=ssl_context) as resp:
        body = resp.read().decode("utf-8")
        return json.loads(body) if body else {}


def create_tag_release(args: argparse.Namespace) -> dict:
    # Accept either a bare host (https://gitlab.example.com) or the v5 base
    # (https://gitlab.example.com/api/v5); normalize to bare so /api/v5 below is
    # not doubled. Keeps this tool consistent with the shell packagers.
    host = args.api_host.rstrip("/").removesuffix("/api/v5")
    base_url = f"{host}/api/v5/repos/{args.owner}/{args.repo}/releases"
    url = f"{base_url}?access_token={parse.quote(args.access_token)}"

    # 官方文档: tag_name / name / body 均为 required. body 缺失时回退到模板,
    # 但仍须保证非空, 否则按文档属不合格请求.
    body = args.body or BODY_TEMPLATE
    if not body.strip():
        raise ValueError("release body is required by the GitCode API but is empty")

    payload: dict = {
        "tag_name": args.tag_name,
        "name": args.tag_name,
        "body": body,
    }
    # 官方文档: target_commitish / release_status 均为非必填, 仅在提供时加入.
    # tag 不存在时 target_commitish 用于新建 tag; 省略则默认默认分支最新提交.
    if args.target_commitish:
        payload["target_commitish"] = args.target_commitish
    if args.release_status:
        if args.release_status not in ALLOWED_RELEASE_STATUSES:
            raise ValueError(
                f"release_status must be one of {ALLOWED_RELEASE_STATUSES}, "
                f"got {args.release_status!r}"
            )
        payload["release_status"] = args.release_status
    return send_request(url, method="POST", payload=payload)


def get_release_upload_url(args: argparse.Namespace) -> dict:
    # Accept either a bare host (https://gitlab.example.com) or the v5 base
    # (https://gitlab.example.com/api/v5); normalize to bare so /api/v5 below is
    # not doubled. Keeps this tool consistent with the shell packagers.
    host = args.api_host.rstrip("/").removesuffix("/api/v5")
    url = (
        f"{host}/api/v5/repos/{args.owner}/{args.repo}/releases/"
        f"{parse.quote(args.tag_name)}/upload_url"
        f"?access_token={parse.quote(args.access_token)}"
        f"&file_name={parse.quote(args.file_name)}"
    )
    return send_request(url)


def upload_release_asset(upload_url_result: dict, file_path: str) -> dict:
    upload_url = upload_url_result.get("url")
    upload_headers = upload_url_result.get("headers", {})
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


def is_release_already_exists(exc: error.HTTPError, details: str) -> bool:
    if exc.code not in {400, 409}:
        return False

    try:
        payload = json.loads(details) if details else {}
    except json.JSONDecodeError:
        payload = {}

    error_message = str(payload.get("error_message", ""))
    return "Release already exists" in error_message


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    missing = []
    if not args.api_host:
        missing.append("API host (set RUSTCODE_RELEASE_API_HOST or pass --api-host)")
    if not args.owner:
        missing.append("repository owner (set RUSTCODE_RELEASE_OWNER or pass --owner)")
    if not args.access_token:
        missing.append(
            "access token (set RUSTCODE_RELEASE_ACCESS_TOKEN or pass --access-token)"
        )
    if missing:
        print("Missing required release configuration:", file=sys.stderr)
        for item in missing:
            print(f"  - {item}", file=sys.stderr)
        return 1
    if not os.path.isfile(args.file_path):
        print(f"File not found: {args.file_path}", file=sys.stderr)
        return 1

    try:
        release_result = create_tag_release(args)
    except error.HTTPError as exc:
        details = exc.read().decode("utf-8", errors="replace")
        if is_release_already_exists(exc, details):
            release_result = {
                "skipped": True,
                "reason": "release already exists",
                "details": json.loads(details),
            }
        else:
            print(f"HTTP {exc.code}: {details}", file=sys.stderr)
            return 1
    except error.URLError as exc:
        print(f"Request failed: {exc}", file=sys.stderr)
        return 1

    try:
        upload_url_result = get_release_upload_url(args)
        upload_result = upload_release_asset(upload_url_result, args.file_path)
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

    print(
        json.dumps(
            {
                "release": release_result,
                "upload_url": upload_url_result,
                "upload_result": upload_result,
            },
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
