"""Requires explicit approval: ephemeral read-only token on the owned loopback fixture.

No production server/account is used. Revoke the token, restore the exact generated
Keychain fixture credential, and remove secret files in finally, including on failure.
"""
from pathlib import Path
import base64, json, os, ssl, subprocess, urllib.request, urllib.parse, uuid

out = Path(__file__).resolve().parent
project = out.parents[2]
meta = json.loads((out / 'gitea-fixture.json').read_text())
root = Path(meta['root'])
assert (root / '.gitfleet-gitea-fixture').is_file()
assert (root / 'keychain-owned.json').is_file()
url = urllib.parse.urlsplit(meta['url'])
assert url.scheme == 'https' and url.hostname == '127.0.0.1'
assert meta['account'].startswith('gitfleet-fixture-')
original = json.loads(Path(meta['credential_file']).read_text())
assert original['account'] == meta['account']
base = url.scheme + '://' + url.netloc
frame = ('protocol=https\nhost=' + url.netloc + '\npath=' + url.path[1:] +
         '\nusername=' + meta['account'] + '\n')

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args):
        return None

opener = urllib.request.build_opener(
    urllib.request.ProxyHandler({}), NoRedirect(),
    urllib.request.HTTPSHandler(context=ssl.create_default_context(cafile=meta['ca_file'])))
authorization = 'Basic ' + base64.b64encode(
    (original['account'] + ':' + original['password']).encode()).decode()

def api(path, method='GET', data=None):
    request = urllib.request.Request(base + '/api/v1' + path, method=method,
        data=json.dumps(data).encode() if data is not None else None,
        headers={'Authorization': authorization, 'Content-Type': 'application/json'})
    with opener.open(request, timeout=10) as response:
        body = response.read()
        return json.loads(body) if body else None

def store(password):
    data = (frame + 'password=' + password + '\n\n').encode()
    result = subprocess.run([meta['helper'], 'store'], input=data,
                            capture_output=True, timeout=10)
    assert result.returncode == 0, 'Fixture helper refused store; output withheld'

token = None
secret_file = root / 'temporary-read-token.json'
fixture_file = root / 'temporary-token-fixture.json'
result = None
summary = {'fixture_only': True, 'scopes': ['read:repository'],
           'token_revoked': False, 'original_fixture_credential_restored': False}
try:
    token = api('/users/' + meta['account'] + '/tokens', 'POST',
                {'name': 'gitfleet-disposable-' + uuid.uuid4().hex,
                 'scopes': ['read:repository']})
    secret_file.write_text(json.dumps({'account': meta['account'], 'password': token['sha1']}))
    secret_file.chmod(0o600)
    selected = dict(meta, credential_file=str(secret_file))
    fixture_file.write_text(json.dumps(selected))
    fixture_file.chmod(0o600)
    store(token['sha1'])
    environment = dict(os.environ, GITFLEET_GITEA_FIXTURE=str(fixture_file),
                       CARGO_NET_OFFLINE='true')
    with (out / 'gitea-keychain-token-integration.txt').open('wb') as log:
        result = subprocess.run(['cargo', 'test', '--manifest-path', 'app/core/Cargo.toml',
            '--features', 'application-services', '--test', 'remote_observation',
            'actual_gitea_and_osxkeychain', '--locked', '--', '--ignored'],
            cwd=project, env=environment, stdout=log, stderr=log)
        summary['integration_exit_code'] = result.returncode
finally:
    cleanup_errors = []
    try:
        store(original['password'])
        summary['original_fixture_credential_restored'] = True
    except Exception:
        cleanup_errors.append('fixture credential restore failed')
    try:
        if token is not None:
            api('/users/' + meta['account'] + '/tokens/' + str(token['id']), 'DELETE')
            summary['token_revoked'] = True
    except Exception:
        cleanup_errors.append('fixture token revocation failed')
    for file in [secret_file, fixture_file]:
        try:
            file.unlink(missing_ok=True)
        except Exception:
            cleanup_errors.append('temporary secret-file removal failed')
    summary['cleanup_errors'] = cleanup_errors
    (out / 'gitea-token-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
if cleanup_errors:
    raise RuntimeError('Fixture cleanup failed; inspect the nonsecret token summary')
if result is None or result.returncode:
    raise SystemExit(1)
print('Disposable read-only token integration passed; token revoked and fixture credential restored.')
