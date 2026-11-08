#!/usr/bin/env python3
"""Measure read-only discovery over disposable local repositories; never user roots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--timeout', type=float, default=30)
    parser.add_argument('--sizes', type=int, nargs='+', default=[100, 1000, 10000])
    args = parser.parse_args()
    if args.timeout <= 0 or any(size <= 0 or size > 10000 for size in args.sizes):
        parser.error('Require positive timeout and sizes between 1 and 10000')
    core = Path(__file__).resolve().parents[1]
    cli = core / 'target/debug/gitfleet-cli'
    root = Path(tempfile.mkdtemp(prefix='gitfleet-scale-', dir='/private/tmp'))
    (root / '.gitfleet-scale-fixture').write_text('Disposable read-only benchmark\n')
    env = {key: value for key, value in os.environ.items() if not key.startswith('GIT_')}
    env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1', GIT_TERMINAL_PROMPT='0')
    template = root / 'template'
    subprocess.run(['git', '-c', 'init.templateDir=', 'init', '-q', '-b', 'main', str(template)], env=env, check=True)
    results = []
    try:
        for size in args.sizes:
            workspace = root / ('repositories-' + str(size))
            workspace.mkdir()
            for index in range(size):
                shutil.copytree(template, workspace / ('repo-%05d' % index))
            for temperature in ['first_pass', 'warm']:
                started = time.monotonic()
                process = subprocess.Popen([str(cli), 'scan', str(workspace)], env=env,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                           start_new_session=True)
                timed_out = False
                try:
                    stdout, stderr = process.communicate(timeout=args.timeout)
                except subprocess.TimeoutExpired:
                    timed_out = True
                    os.killpg(process.pid, signal.SIGKILL)
                    stdout, stderr = process.communicate()
                result = dict(repositories=size, run=temperature, seconds=time.monotonic()-started,
                              timeout_seconds=args.timeout, timed_out=timed_out,
                              exit=process.returncode, output_bytes=len(stdout))
                if not timed_out and process.returncode == 0:
                    value = json.loads(stdout)
                    result.update(complete=value.get('complete'),
                                  returned_repositories=len(value.get('repositories', [])),
                                  issues=len(value.get('issues', [])),
                                  issue_kinds=[issue['error']['kind'] for issue in value.get('issues', [])])
                    if result['complete'] and result['returned_repositories'] != size:
                        result['unexpected_result'] = 'Complete scan omitted repositories'
                    if not result['complete'] and not result['issues']:
                        result['unexpected_result'] = 'Partial scan has no issue'
                else:
                    result['partial_json_available'] = False
                    if not timed_out:
                        result['error'] = stderr.decode(errors='replace')[:2000]
                results.append(result)
                print(json.dumps(result), flush=True)
            shutil.rmtree(workspace)
    finally:
        metadata = dict(platform=platform.platform(), machine=platform.machine(),
                        cpu_count=os.cpu_count(), fixture=str(root),
                        git=subprocess.check_output(['git', '--version'], text=True).strip(),
                        cli_sha256=hashlib.sha256(cli.read_bytes()).hexdigest(),
                        scope='Core CLI scan only; empty independent repos; no renderer/IPC/watcher/memory budget claim',
                        first_pass_is_not_os_cache_cold=True, results=results)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(metadata, indent=2)+'\n')
        if (root / '.gitfleet-scale-fixture').is_file():
            shutil.rmtree(root)


if __name__ == '__main__':
    main()
