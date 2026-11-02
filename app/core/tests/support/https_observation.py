"""Disposable TLS smart-advertisement fixture; never serves objects or accepts writes."""
import http.server
import pathlib
import ssl
import sys
import time

certificate, key, marker, mode, oid = sys.argv[1:]
marker = pathlib.Path(marker)

def packet(text):
    value = text.encode()
    return f"{len(value) + 4:04x}".encode() + value

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        with marker.open("a") as stream:
            stream.write(f"{self.path} auth={self.headers.get('Authorization') is not None}\n")
        if mode == "slow":
            time.sleep(60)
        if mode == "authentication":
            body = b"SECRET-SERVER-DIAGNOSTIC"
            self.send_response(401)
            self.send_header("WWW-Authenticate", 'Basic realm="fixture"')
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if mode == "redirect":
            self.send_response(302)
            self.send_header("Location", f"https://127.0.0.1:{self.server.server_port}/forbidden")
            self.send_header("Content-Length", "0")
            self.end_headers()
            return
        body = packet("# service=git-upload-pack\n") + b"0000"
        body += packet(f"{oid} refs/heads/main\0object-format=sha1 agent=gitfleet-fixture\n") + b"0000"
        self.send_response(200)
        self.send_header("Content-Type", "application/x-git-upload-pack-advertisement")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(certificate, key)
server.socket = context.wrap_socket(server.socket, server_side=True)
print(server.server_port, flush=True)
server.serve_forever()
