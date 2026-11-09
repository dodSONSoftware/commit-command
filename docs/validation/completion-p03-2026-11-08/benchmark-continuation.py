"""Read-only release CLI continuation measurements on new generated repositories."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

out = Path(__file__).resolve().parent
project = out.parents[2]
sys.path.insert(0, str(project / 'app/core/scripts'))
from completion_evidence import snapshot, compare
cli = project / 'app/core/target/release/gitfleet-cli'
env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1', GIT_TERMINAL_PROMPT='0')
results = []
with tempfile.TemporaryDirectory(prefix='gitfleet-continuation-') as temporary:
    root = Path(temporary).resolve()
    os.chmod(root, 0o700)
    (root / '.gitfleet-scale-fixture').write_text('Disposable continuation acceptance\n')
    template = root / 'template'
    subprocess.run(['git', '-c', 'init.templateDir=', 'init', '-q', '-b', 'main', str(template)], env=env, check=True)
    workspace = root / 'workspace'
    workspace.mkdir()
    for index in range(100):
        shutil.copytree(template, workspace / f'repo-{index:03}')
    before = snapshot(workspace)
    for repetition in range(3):
        state = root / f'state-{repetition}'
        definition = json.loads(subprocess.check_output([str(cli), '--data-dir', str(state), 'workspace', 'create', 'Continuation benchmark', str(workspace)], env=env))
        start = time.monotonic()
        result = json.loads(subprocess.check_output([str(cli), '--data-dir', str(state), 'workspace', 'scan-complete', definition['id'], '10'], env=env, timeout=180))
        scan = result['inventory']['scan']
        record = dict(repetition=repetition, seconds=time.monotonic()-start, slices=result['slices'], complete=scan['complete'], repositories=len(scan['repositories']), issues=len(scan['issues']))
        assert record['complete'] and record['repositories'] == 100 and record['issues'] == 0, record
        results.append(record)
        print(json.dumps(record), flush=True)
    preservation = compare(before, snapshot(workspace))
    assert preservation['preserved'], preservation
    value = dict(scope='Release Core CLI only, 100 generated empty independent repos, first pass after creation is not OS-cache-cold; no renderer/native/watcher/1000/10000 capacity claim',
                 cli_sha256=hashlib.sha256(cli.read_bytes()).hexdigest(), results=results, preservation=preservation)
    (out / 'benchmark.json').write_text(json.dumps(value, indent=2)+'\n')
