#!/usr/bin/env python3
"""Unit tests for scripts/gitcode_release.py.

Run from the repo root or from scripts/:
    python3 scripts/test_gitcode_release.py -v
    python3 -m unittest scripts.test_gitcode_release -v
"""

from __future__ import annotations

import contextlib
import io
import json
import os
import sys
import unittest
from unittest import mock
from urllib import error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gitcode_release as gr

HOST = "https://api.gitcode.com"
OWNER = "SecLab"
REPO = "RustCode"
TOKEN = "tok-123"

RELEASE_RESPONSE = {
    "id": 1,
    "tag_name": "v1.0.0",
    "name": "v1.0.0",
    "html_url": "https://gitcode.com/SecLab/RustCode/releases/tag/v1.0.0",
}


def run_main(argv: list[str]) -> tuple[int, str, str]:
    out, err = io.StringIO(), io.StringIO()
    with mock.patch.object(sys, "argv", ["gitcode-release.py", *argv]):
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = gr.main()
    return code, out.getvalue(), err.getvalue()


class NormalizeHostTest(unittest.TestCase):
    def test_bare_host_passes_through(self):
        self.assertEqual(gr.normalize_host("https://api.gitcode.com"), "https://api.gitcode.com")

    def test_v5_suffix_is_stripped(self):
        self.assertEqual(
            gr.normalize_host("https://api.gitcode.com/api/v5"),
            "https://api.gitcode.com",
        )

    def test_trailing_slash_is_stripped(self):
        self.assertEqual(
            gr.normalize_host("https://api.gitcode.com/"),
            "https://api.gitcode.com",
        )


class BuildReleaseUrlTest(unittest.TestCase):
    def test_url_shape_and_token_query(self):
        url = gr.build_release_url(HOST, OWNER, REPO, TOKEN)
        self.assertEqual(
            url,
            f"{HOST}/api/v5/repos/{OWNER}/{REPO}/releases?access_token={TOKEN}",
        )

    def test_api_host_with_v5_suffix_is_not_doubled(self):
        url = gr.build_release_url(f"{HOST}/api/v5", OWNER, REPO, TOKEN)
        self.assertEqual(url.count("/api/v5"), 1)

    def test_token_is_url_encoded(self):
        url = gr.build_release_url(HOST, OWNER, REPO, "a b&c=+")
        self.assertIn("access_token=a%20b%26c%3D%2B", url)


class BuildPayloadTest(unittest.TestCase):
    def test_required_fields(self):
        payload = gr.build_payload("v1.0.0", "", "", "", "")
        self.assertEqual(payload["tag_name"], "v1.0.0")
        self.assertEqual(payload["name"], "v1.0.0")
        self.assertIn("v1.0.0", payload["body"])

    def test_explicit_name_and_body_win(self):
        payload = gr.build_payload("v1.0.0", "正式版", "## 更新内容", "", "")
        self.assertEqual(payload["name"], "正式版")
        self.assertEqual(payload["body"], "## 更新内容")

    def test_optional_fields_omitted_when_empty(self):
        payload = gr.build_payload("v1.0.0", "", "", "", "")
        self.assertNotIn("target_commitish", payload)
        self.assertNotIn("release_status", payload)

    def test_optional_fields_included_when_given(self):
        payload = gr.build_payload(
            "v1.0.0", "", "", "dev", "pre"
        )
        self.assertEqual(payload["target_commitish"], "dev")
        self.assertEqual(payload["release_status"], "pre")

    def test_commitish_sha_supported(self):
        payload = gr.build_payload(
            "v1.0.0", "", "", "0c4a1b2c3", ""
        )
        self.assertEqual(payload["target_commitish"], "0c4a1b2c3")


class BuildUploadUrlTest(unittest.TestCase):
    def test_default_gitlab_v5_dialect(self):
        url = gr.build_upload_url(HOST, OWNER, REPO, TOKEN, "v1.0.0", "rustcode.tar.gz")
        self.assertEqual(
            url,
            f"{HOST}/api/v5/repos/{OWNER}/{REPO}/releases/"
            "v1.0.0/upload_url"
            f"?access_token={TOKEN}&file_name=rustcode.tar.gz",
        )

    def test_tag_is_url_encoded(self):
        url = gr.build_upload_url(HOST, OWNER, REPO, TOKEN, "v 1", "a")
        self.assertIn("releases/v%201/upload_url", url)

    def test_endpoint_override_wins(self):
        url = gr.build_upload_url(
            HOST, OWNER, REPO, TOKEN, "v1.0.0", "a.bin",
            endpoint_override="https://cdn.example.com/upload",
        )
        self.assertEqual(url, "https://cdn.example.com/upload")


class AlreadyExistsTest(unittest.TestCase):
    def _exc(self, code: int, body: str) -> error.HTTPError:
        return error.HTTPError(url="http://x", code=code, msg="err", hdrs=None, fp=io.BytesIO(body.encode()))

    def test_409_already_exists_matches(self):
        self.assertTrue(
            gr.is_release_already_exists(
                self._exc(409, '{"error_message": "Release already exists"}'), '{"error_message": "Release already exists"}'
            )
        )

    def test_400_already_exists_matches(self):
        self.assertTrue(
            gr.is_release_already_exists(
                self._exc(400, '{"message": "release already exist"}'), '{"message": "release already exist"}'
            )
        )

    def test_other_error_does_not_match(self):
        self.assertFalse(gr.is_release_already_exists(self._exc(500, "boom"), "boom"))

    def test_404_does_not_match(self):
        self.assertFalse(gr.is_release_already_exists(self._exc(404, "nope"), "nope"))


class CreateReleaseTest(unittest.TestCase):
    @mock.patch.object(gr, "send_request", return_value=RELEASE_RESPONSE)
    def test_sends_post_with_payload(self, send_request):
        result = gr.create_release(
            HOST, OWNER, REPO, TOKEN, "v1.0.0", "正式版", "## 内容", "dev", "latest"
        )
        self.assertEqual(result, RELEASE_RESPONSE)
        url, kwargs = send_request.call_args
        self.assertIn(f"repos/{OWNER}/{REPO}/releases", url[0])
        self.assertEqual(kwargs["method"], "POST")
        payload = kwargs["payload"]
        self.assertEqual(payload["tag_name"], "v1.0.0")
        self.assertEqual(payload["name"], "正式版")
        self.assertEqual(payload["target_commitish"], "dev")
        self.assertEqual(payload["release_status"], "latest")


class UploadAssetTest(unittest.TestCase):
    def test_upload_put_success(self):
        upload_result = {
            "url": "https://cdn.example.com/put-here",
            "headers": {"x-auth": "1"},
        }
        tmp = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_upload_fixture.bin")
        with open(tmp, "wb") as f:
            f.write(b"payload-bytes")
        try:
            with mock.patch.object(gr.request, "urlopen") as urlopen:
                resp = mock.Mock()
                resp.read.return_value = b"ok"
                resp.status = 201
                urlopen.return_value.__enter__.return_value = resp

                out = gr.upload_release_asset(upload_result, tmp)
        finally:
            os.remove(tmp)

        self.assertEqual(out, {"status_code": 201, "body": "ok"})
        req = urlopen.call_args[0][0]
        self.assertEqual(req.method, "PUT")
        normalized = {k.lower(): v for k, v in req.headers.items()}
        self.assertEqual(normalized, {"x-auth": "1"})
        self.assertEqual(req.data, b"payload-bytes")

    def test_missing_upload_url_raises(self):
        with self.assertRaises(ValueError):
            gr.upload_release_asset({}, "/x")


class MainIntegrationTest(unittest.TestCase):
    BASE = [
        "--api-host", HOST,
        "--owner", OWNER,
        "--repo", REPO,
        "--access-token", TOKEN,
        "--tag-name", "v1.0.0",
    ]

    def test_missing_config_fails_closed(self):
        code, _, err = run_main(["--tag-name", "v1.0.0"])
        self.assertEqual(code, 1)
        self.assertIn("Missing required release configuration", err)
        self.assertIn("RUSTCODE_RELEASE_API_HOST", err)

    def test_missing_token_ok_with_dry_run(self):
        code, out, _ = run_main(
            ["--api-host", HOST, "--owner", OWNER, "--tag-name", "v1.0.0", "--dry-run"]
        )
        self.assertEqual(code, 0)
        summary = json.loads(out)
        self.assertTrue(summary["dry_run"])

    def test_dry_run_prints_request_and_calls_nothing(self):
        with mock.patch.object(gr, "send_request") as send_request:
            code, out, _ = run_main([*self.BASE, "--dry-run"])
        self.assertEqual(code, 0)
        send_request.assert_not_called()
        summary = json.loads(out)
        self.assertEqual(summary["method"], "POST")
        self.assertEqual(summary["payload"]["tag_name"], "v1.0.0")
        self.assertIn("access_token=", summary["url"])

    def test_dry_run_reports_attach_upload_url(self):
        tmp = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_dry_attach.bin")
        with open(tmp, "wb") as f:
            f.write(b"x")
        try:
            with mock.patch.object(gr, "send_request") as send_request:
                code, out, _ = run_main(
                    [*self.BASE, "--dry-run", "--attach", tmp, "--file-name", "x.bin"]
                )
            self.assertEqual(code, 0)
            send_request.assert_not_called()
        finally:
            os.remove(tmp)
        summary = json.loads(out)
        self.assertEqual(summary["asset_upload"]["file_name"], "x.bin")

    def test_success_flow_creates_and_uploads(self):
        tmp = os.path.join(os.path.dirname(os.path.abspath(__file__)), "_fixture.bin")
        with open(tmp, "wb") as f:
            f.write(b"data")
        try:
            with mock.patch.object(gr, "send_request", side_effect=[
                RELEASE_RESPONSE,                      # POST /releases
                {"url": "https://cdn.example.com/up", "headers": {}},  # upload_url
            ]) as send_request, mock.patch.object(
                gr, "upload_release_asset", return_value={"status_code": 201, "body": "ok"}
            ) as upload:
                code, out, _ = run_main(
                    [*self.BASE, "--attach", tmp, "--file-name", "fixture.bin"]
                )
                self.assertEqual(code, 0)
                self.assertEqual(send_request.call_count, 2)
                payload = send_request.call_args_list[0].kwargs["payload"]
                self.assertEqual(payload["tag_name"], "v1.0.0")
                upload.assert_called_once()
        finally:
            os.remove(tmp)

        result = json.loads(out)
        self.assertEqual(result["release"], RELEASE_RESPONSE)
        self.assertEqual(result["asset_upload"], {"status_code": 201, "body": "ok"})

    def test_already_exists_is_skipped_not_fatal(self):
        exc = error.HTTPError(
            url="http://x", code=409, msg="conflict", hdrs=None,
            fp=io.BytesIO(b'{"error_message": "Release already exists"}'),
        )
        with mock.patch.object(gr, "send_request", side_effect=exc):
            code, out, _ = run_main(self.BASE)
        self.assertEqual(code, 0)
        result = json.loads(out)
        self.assertTrue(result["release"]["skipped"])
        self.assertEqual(result["release"]["reason"], "release already exists")

    def test_other_http_error_is_fatal(self):
        exc = error.HTTPError(url="http://x", code=500, msg="err", hdrs=None, fp=io.BytesIO(b"boom"))
        with mock.patch.object(gr, "send_request", side_effect=exc):
            code, _, err = run_main(self.BASE)
        self.assertEqual(code, 1)
        self.assertIn("HTTP 500", err)

    def test_attach_missing_file_fails(self):
        code, _, err = run_main(
            [*self.BASE, "--attach", "/definitely/not/here.bin"]
        )
        self.assertEqual(code, 1)
        self.assertIn("File not found", err)

    def test_body_file_is_read(self):
        fd, path = __import__("tempfile").mkstemp(suffix=".md")
        os.write(fd, "## 更新内容\n- 修复 bug".encode())
        os.close(fd)
        try:
            with mock.patch.object(gr, "send_request", return_value=RELEASE_RESPONSE) as send_request:
                code, out, _ = run_main(
                    [*self.BASE, "--body-file", path]
                )
                self.assertEqual(code, 0)
                payload = send_request.call_args.kwargs["payload"]
                self.assertEqual(payload["body"], "## 更新内容\n- 修复 bug")
        finally:
            os.remove(path)

    def test_missing_body_file_fails(self):
        code, _, err = run_main(
            [*self.BASE, "--body-file", "/definitely/not/here.md"]
        )
        self.assertEqual(code, 1)
        self.assertTrue(err)

    def test_bad_release_status_rejected_by_argparse(self):
        with self.assertRaises(SystemExit):
            run_main([*self.BASE, "--release-status", "beta"])


if __name__ == "__main__":
    unittest.main(verbosity=2)