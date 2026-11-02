#!/usr/bin/env python3
"""Verify the assembled Core layout and manifest boundaries, not runtime safety."""
from pathlib import Path
import re
import json
import subprocess

core = Path(__file__).resolve().parents[1]
project = core.parents[1]
manifest = json.loads(subprocess.check_output([
    'cargo', 'metadata', '--manifest-path', str(core / 'Cargo.toml'),
    '--no-deps', '--offline', '--locked', '--format-version', '1',
], text=True))['packages'][0]
assert manifest['name'] == 'gitfleet-core'
assert manifest['features']['default'] == []
assert 'experimental-writes' in manifest['features']
assert 'controlled-writes' in manifest['features']
assert 'application-services' in manifest['features']
assert 'controlled-writes' in manifest['features']['application-services']
writer = next(target for target in manifest['targets'] if target['name'] == 'gitfleet-write')
assert writer['required-features'] == ['controlled-writes']
assert not manifest['features']['experimental-writes']
assert not any(name.lower().startswith(('tauri', 'react')) for name in (dependency['name'] for dependency in manifest['dependencies']))
lab = next(binary for binary in manifest['targets'] if binary['name'] == 'gitfleet-lab')
assert lab['required-features'] == ['experimental-writes']
for name in ['acceptance', 'core_behaviour', 'write_regressions']:
    test = next(target for target in manifest['targets'] if target['name'] == name)
    assert test['required-features'] == ['experimental-writes']
for name in [
    'Cargo.toml', 'Cargo.lock', 'README.md', 'src/lib.rs', 'src/error.rs',
    'src/inspection.rs', 'src/domain.rs', 'src/process.rs', 'src/history.rs', 'src/diff.rs', 'src/workspace.rs', 'src/git.rs', 'src/discovery.rs', 'src/journal.rs',
    'src/commit.rs', 'src/lab.rs', 'src/bin/gitfleet_cli.rs',
    'src/bin/gitfleet_lab.rs', 'tests/acceptance.rs', 'tests/core_behaviour.rs',
    'tests/inspection.rs', 'tests/write_regressions.rs', 'tests/m1_read.rs', 'tests/m1_workspace.rs', 'scripts/validate.sh',
]:
    assert (core / name).is_file(), name
for name in ['ARCHITECTURE.md', 'LIMITATIONS.md', 'TEST_MATRIX.md', 'PATCH_NOTES.md']:
    assert (project / 'docs/architecture/core-poc' / name).is_file(), name
assert (project / '.gitignore').is_file()
for name in ['IMPLEMENTATION_HANDOFF.md', 'IMPLEMENTATION_STATUS.md']:
    assert (project / 'docs' / name).is_file(), name
acceptance = (core / 'tests/acceptance.rs').read_text()
assert len(re.findall(r'^    [a-z_]+,?$', acceptance, flags=re.MULTILINE)) >= 23
assert 'fn successful_gpg_signing()' in acceptance
cli = (core / 'src/bin/gitfleet_cli.rs').read_text()
assert 'inspection::snapshot' in cli and 'git::log_state' not in cli
print('Core layout/manifest checks passed. Runtime safety is verified separately by Cargo tests.')
