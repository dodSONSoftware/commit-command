use super::Fixture;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct TlsServer {
    child: std::process::Child,
    pub port: u16,
    pub marker: PathBuf,
}
impl Drop for TlsServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
pub fn start(f: &Fixture, remote: &Path, mode: &str) -> TlsServer {
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };
    let cert = f.root.join("certificate.pem");
    let key = f.root.join("key.pem");
    let config = f.root.join("certificate.cnf");
    fs::write(&config,b"[req]\ndistinguished_name=dn\nx509_extensions=ext\nprompt=no\n[dn]\nCN=GitFleet TLS Fixture\n[ext]\nsubjectAltName=IP:127.0.0.1\nbasicConstraints=critical,CA:TRUE\nkeyUsage=critical,digitalSignature,keyCertSign\n").unwrap();
    let certificate = Command::new("openssl")
        .args([
            "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1", "-config",
        ])
        .arg(&config)
        .arg("-keyout")
        .arg(&key)
        .arg("-out")
        .arg(&cert)
        .output()
        .unwrap();
    assert!(
        certificate.status.success(),
        "{}",
        String::from_utf8_lossy(&certificate.stderr)
    );
    let marker = f.root.join("network-requests");
    let log = fs::File::create(f.root.join("https-server.txt")).unwrap();
    let oid = String::from_utf8(f.git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    let mut child = Command::new("python3")
        .args(["-I", "-c", include_str!("https_observation.py")])
        .arg(&cert)
        .arg(&key)
        .arg(&marker)
        .arg(mode)
        .arg(oid)
        .arg(remote)
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stdout).read_line(&mut line);
        let _ = sender.send((result, line));
    });
    let ready = receiver.recv_timeout(std::time::Duration::from_secs(10));
    if ready.is_err() {
        let _ = child.kill();
    }
    reader.join().unwrap();
    let (result, line) = ready.unwrap();
    result.unwrap();
    let port = line.trim().parse().unwrap_or_else(|_| {
        panic!(
            "TLS server did not start: {}",
            fs::read_to_string(f.root.join("https-server.txt")).unwrap()
        )
    });
    TlsServer {
        child,
        port,
        marker,
    }
}
