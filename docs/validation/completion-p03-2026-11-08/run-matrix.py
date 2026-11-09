"""Run unchanged safety matrix with a fresh private fixture root and isolated signer."""
from pathlib import Path
import json
import os
import subprocess
import tempfile
import uuid
out=Path(__file__).resolve().parent
project=out.parents[2]
tools=project/'docs/validation/completion-tools-2026-11-08'
prefix=Path((tools/'tool-root.txt').read_text().strip())/'install'
assert (prefix/'bin/gpg').is_file()
fixtures=Path(tempfile.mkdtemp(prefix='gitfleet-completion-fixtures-')).resolve()
os.chmod(fixtures,0o700)
alias=Path('/private/tmp')/('gf-'+uuid.uuid4().hex[:10])
alias.symlink_to(fixtures)
env=dict(os.environ,PATH=str(prefix/'bin')+':'+os.environ['PATH'],TMPDIR=str(alias),GITFLEET_TEST_GPG=str(prefix/'bin/gpg'),CARGO_NET_OFFLINE='true',CARGO_BUILD_JOBS='4',RUST_TEST_THREADS='4')
(out/'matrix-launch.json').write_text(json.dumps(dict(fixture_directory=str(fixtures),temp_alias=str(alias),signer=str(prefix/'bin/gpg'),features='default,experimental-writes,controlled-writes,application-services,all-features',replay=False),indent=2)+'\n')
with (out/'core-matrix.txt').open('w') as log:
    result=subprocess.run(['sh','app/core/scripts/validate.sh',str(out/'core-matrix')],cwd=project,env=env,stdout=log,stderr=subprocess.STDOUT)
(out/'matrix-exit.json').write_text(json.dumps({'exit_code':result.returncode})+'\n')
raise SystemExit(result.returncode)
