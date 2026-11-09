"""Disposable generated-source native fixture; loopback only, no credentials."""
import hashlib
import json
import plistlib
import shutil
import socket
import subprocess
import tempfile
import time
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parent
REPOSITORY = EVIDENCE.parents[2]
root = Path(tempfile.mkdtemp(prefix="gitfleet-p09-editor-", dir="/private/tmp"))
app = root / "Commit Command AI Editor Test.app"
shutil.copytree("/private/tmp/gitfleet-m3-native-cb8nbd9g/Commit Command Activation.app", app)
shutil.copytree("/private/tmp/gitfleet-p04-composer-i8ihccx_/state", root / "state")
assert not (root / "state/ai").exists()
binary = app / "Contents/MacOS/gitfleet-desktop"
shutil.copy2(REPOSITORY / "app/desktop/src-tauri/target/debug/gitfleet-desktop", binary)
info = app / "Contents/Info.plist"
metadata = plistlib.loads(info.read_bytes())
metadata["CFBundleName"] = "Commit Command AI Editor Test"
metadata["CFBundleDisplayName"] = "Commit Command AI Editor Test"
info.write_bytes(plistlib.dumps(metadata))
subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(app)], check=True, capture_output=True)
listener = socket.socket()
listener.bind(("127.0.0.1", 0))
listener.listen(4)
listener.settimeout(0.25)
fixture = {"root": str(root), "app": str(app), "binary": str(binary), "state": str(root / "state"), "workspace": "/private/tmp/gitfleet-p04-filter-uhe_cyb2/workspace", "endpoint": "http://127.0.0.1:%d/" % listener.getsockname()[1], "features": ["custom-protocol"], "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
(EVIDENCE / "native-fixture.json").write_text(json.dumps(fixture, indent=2) + "\n")
print(json.dumps(fixture), flush=True)
requests = []
errors = []
deadline = time.monotonic() + 1200
while time.monotonic() < deadline and not (root / "finish.native").exists():
    try:
        stream, _ = listener.accept()
    except socket.timeout:
        continue
    with stream:
        stream.settimeout(5)
        try:
            data = bytearray()
            while b"\r\n\r\n" not in data:
                chunk = stream.recv(4096)
                assert chunk
                data.extend(chunk)
                assert len(data) <= 2 * 1024 * 1024
            head, body = bytes(data).split(b"\r\n\r\n", 1)
            headers = head.decode("ascii")
            assert headers.startswith("POST /api/chat HTTP/1.1\r\n")
            assert "authorization:" not in headers.lower()
            length = int(next(line.split(":", 1)[1].strip() for line in headers.split("\r\n") if line.lower().startswith("content-length:")))
            assert 0 < length <= 2 * 1024 * 1024
            while len(body) < length:
                chunk = stream.recv(min(65536, length - len(body)))
                assert chunk
                body += chunk
            payload = json.loads(body[:length])
            requests.append(payload)
            (EVIDENCE / "native-requests.json").write_text(json.dumps(requests, indent=2) + "\n")
            assert payload["model"] == "fixture-editor-model"
            assert payload["messages"][0]["content"] == "UNSAVED_EDITOR_SYSTEM"
            user = json.loads(payload["messages"][1]["content"])
            assert user["template"]["rules"]["instructions"] == "UNSAVED_EDITOR_TEMPLATE"
            assert "UNSAVED_EDITOR_INSTRUCTIONS" in user["additive_instructions"]
            assert "disposable staged fixture" in user["context"]
            assert "disposable child fixture" not in user["context"]
            context = json.loads(user["context"])
            candidates = {"candidates": [{"message": {"subject": "fix: describe disposable staged fixture", "body": "Generated fixture response only"}, "rationale": "Observed generated changes", "summary": "Disposable staged fixture", "concerns": [], "covered_chunks": [chunk["id"] for chunk in context["chunks"]]}]}
            response = json.dumps({"model": "fixture-editor-model", "done": True, "done_reason": "stop", "message": {"role": "assistant", "content": json.dumps(candidates)}}).encode()
            stream.sendall(b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: " + str(len(response)).encode() + b"\r\n\r\n" + response)
        except Exception as error:
            errors.append(type(error).__name__ + ": " + str(error))
            response = b'{"error":"fixture assertion failed"}'
            stream.sendall(b"HTTP/1.1 500 Fixture\r\nConnection: close\r\nContent-Length: " + str(len(response)).encode() + b"\r\n\r\n" + response)
listener.close()
(EVIDENCE / "native-provider-summary.json").write_text(json.dumps({"requests": len(requests), "errors": errors, "finished_explicitly": (root / "finish.native").exists()}, indent=2) + "\n")
assert not errors, errors
