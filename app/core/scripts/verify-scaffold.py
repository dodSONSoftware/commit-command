#!/usr/bin/env python3
"""Static verification only; does not compile Rust or assert test execution."""
from pathlib import Path
import re
import sys
try:
    import tomllib
except ImportError:
    print('Python 3.11+ is required for static verification')
    sys.exit(2)

root = Path(__file__).resolve().parents[1]
manifest = tomllib.loads((root/'Cargo.toml').read_text())
assert manifest['package']['name'] == 'gitfleet-core'
required = [
    'Cargo.toml', '.gitignore', 'README.md',
    'src/lib.rs', 'src/error.rs', 'src/git.rs', 'src/discovery.rs',
    'src/journal.rs', 'src/commit.rs', 'src/lab.rs',
    'src/bin/gitfleet_cli.rs', 'src/bin/gitfleet_lab.rs',
    'tests/acceptance.rs', 'tests/core_behaviour.rs',
    'docs/ARCHITECTURE.md', 'docs/LIMITATIONS.md', 'docs/TEST_MATRIX.md'
]
assert all((root/p).is_file() for p in required)
text = (root/'tests/acceptance.rs').read_text()
assert len(re.findall(r'^    [a-z_]+,?$', text, flags=re.MULTILINE)) >= 24
assert not any('tauri' in x.name.lower() for x in root.rglob('Cargo.toml'))
print('Static manifest/scaffold checks passed. Rust build/tests: NOT verified by this script.')
