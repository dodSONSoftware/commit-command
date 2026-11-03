"""Disposable read-only TLS Git fixture; never accepts receive-pack or writes."""
import base64
import hmac
import json
import http.server
import os
import pathlib
import ssl
import subprocess
import sys
import time
import urllib.parse

certificate, key, marker, mode, oid, *repositories = sys.argv[1:]
marker = pathlib.Path(marker)

def packet(text):
    value = text.encode()
    return f"{len(value) + 4:04x}".encode() + value

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def authenticated(self):
        policy=marker.parent / "basic-auth.json"
        if not policy.exists():return True
        credentials=json.loads(policy.read_text())
        expected="Basic "+base64.b64encode((credentials["account"]+":"+credentials["password"]).encode()).decode()
        matched=hmac.compare_digest(self.headers.get("Authorization", ""),expected)
        with (marker.parent / "authentication-events").open("a") as stream:
            scheme="basic" if self.headers.get("Authorization", "").startswith("Basic ") else "none-or-other"
            stream.write(f"{self.command} {self.path} matched={matched} scheme={scheme}\n")
        if matched:return True
        self.send_response(401)
        self.send_header("WWW-Authenticate", 'Basic realm="GitFleet disposable fixture"')
        self.send_header("Content-Length", "0")
        self.end_headers()
        return False

    def do_GET(self):
        with marker.open("a") as stream:
            stream.write(f"{self.path} auth={self.headers.get('Authorization') is not None}\n")
        if not self.authenticated():return
        override = marker.parent / "network-mode"
        selected = override.read_text().strip() if override.exists() else mode
        if selected == "slow":
            time.sleep(60)
        if selected == "authentication":
            body = b"SECRET-SERVER-DIAGNOSTIC"
            self.send_response(401)
            self.send_header("WWW-Authenticate", 'Basic realm="fixture"')
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if selected == "redirect":
            self.send_response(302)
            self.send_header("Location", f"https://127.0.0.1:{self.server.server_port}/forbidden")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        if selected == "git":
            self.git_backend(b"")
            return
        body = packet("# service=git-upload-pack\n") + b"0000"
        body += packet(f"{oid} refs/heads/main\0object-format=sha1 agent=gitfleet-fixture\n") + b"0000"
        self.send_response(200)
        self.send_header("Content-Type", "application/x-git-upload-pack-advertisement")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        with marker.open("a") as stream:
            stream.write(f"{self.path} auth={self.headers.get('Authorization') is not None}\n")
        if not self.authenticated():return
        if mode != "git":
            self.send_error(403)
            return
        length = int(self.headers.get("Content-Length", "0"))
        if length < 0 or length > 1024 * 1024:
            self.send_error(413)
            return
        self.git_backend(self.rfile.read(length))

    def git_backend(self, body):
        parsed = urllib.parse.urlsplit(self.path)
        if not repositories or (self.command, parsed.path, parsed.query) not in [
            ("GET", "/repo.git/info/refs", "service=git-upload-pack"),
            ("POST", "/repo.git/git-upload-pack", ""),
        ]:
            self.send_error(403)
            return
        environment = {
            "PATH": os.environ["PATH"], "HOME": str(marker.parent),
            "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_PROJECT_ROOT": str(pathlib.Path(repositories[0]).parent),
            "GIT_HTTP_EXPORT_ALL": "1", "REQUEST_METHOD": self.command,
            "PATH_INFO": parsed.path.replace("/repo.git/", "/remote.git/", 1),
            "QUERY_STRING": parsed.query, "CONTENT_TYPE": self.headers.get("Content-Type", ""),
            "CONTENT_LENGTH": str(len(body)), "SERVER_PROTOCOL": "HTTP/1.1",
            "SERVER_NAME": "127.0.0.1", "SERVER_PORT": str(self.server.server_port),
            "REMOTE_ADDR": "127.0.0.1", "GIT_PROTOCOL": self.headers.get("Git-Protocol", ""),
        }
        result = subprocess.run(["git", "http-backend"], input=body, env=environment,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10, check=True)
        headers, response = result.stdout.split(b"\r\n\r\n", 1)
        pairs = [line.decode().split(":", 1) for line in headers.split(b"\r\n")]
        status = next((int(value.split()[0]) for key, value in pairs if key.lower() == "status"), 200)
        self.send_response(status)
        for key, value in pairs:
            if key.lower() not in ["status", "content-length"]:
                self.send_header(key, value.strip())
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        self.wfile.write(response)

server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(certificate, key)
server.socket = context.wrap_socket(server.socket, server_side=True)
print(server.server_port, flush=True)
server.serve_forever()
