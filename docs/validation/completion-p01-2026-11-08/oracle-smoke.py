"""Create only disposable Git state and prove the reusable oracle preserves it."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime
from zoneinfo import ZoneInfo

output = Path(__file__).resolve().parent
project = output.parents[2]
sys.path.insert(0, str(project / 'app/core/scripts'))
import completion_evidence as evidence

with tempfile.TemporaryDirectory(prefix='gitfleet-p01-') as temporary:
    root = Path(temporary).resolve()
    os.chmod(root, 0o700)
    repo = root / 'repo'
    repo.mkdir()
    env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
    env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1', GIT_TERMINAL_PROMPT='0')
    def git(*args):
        return subprocess.check_output(['git', '-c', 'init.templateDir=', '-c', 'core.hooksPath=', *args], cwd=repo, env=env)
    git('init', '-q', '-b', 'main')
    (repo / 'selected').write_text('staged\n')
    git('add', '--', 'selected')
    (repo / 'selected').write_text('working\n')
    before = evidence.snapshot(repo)
    after = evidence.snapshot(repo)
    comparison = evidence.compare(before, after)
    assert comparison['preserved']
    for name, value in [('before.json', before), ('after.json', after), ('comparison.json', comparison)]:
        (output / name).write_text(json.dumps(value, indent=2) + '\n')
    def ref(name):
        return dict(path=name, sha256=hashlib.sha256((output / name).read_bytes()).hexdigest())
    result = dict(version=1, case='INF02', package='P01', status='passed', operator='Codex disposable fixture',
        timestamp=datetime.now(ZoneInfo('America/Chicago')).isoformat(),
        source=dict(commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=project, text=True).strip(),
            dirty=True, diff_sha256=hashlib.sha256(subprocess.check_output(['git', 'diff'], cwd=project)).hexdigest()),
        artifact=dict(sha256=hashlib.sha256((project / 'app/core/scripts/completion_evidence.py').read_bytes()).hexdigest(),
            features=[], launch=['python3', str(Path(__file__).resolve())]),
        environment=dict(os=sys.platform, architecture=os.uname().machine, filesystem='local temporary directory',
            tools=dict(python=sys.version.split()[0], git=subprocess.check_output(['git', '--version'], text=True).strip())),
        fixture=str(root), action='Snapshot real staged/working Git fixture twice without Git reads',
        expected='Exact repository/index/files/modes preserved', actual='Independent manifests identical',
        exit_code=0, allowed_deltas=[], oracles={'before':ref('before.json'), 'after':ref('after.json'), 'comparison':ref('comparison.json')},
        phase_proof={'snapshot_complete':ref('comparison.json')}, reason='')
    (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    evidence.validate_result(result, output)
print('Disposable real-Git preservation and hash-bound evidence validation passed')
