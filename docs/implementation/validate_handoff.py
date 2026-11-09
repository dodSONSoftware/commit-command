#!/usr/bin/env python3
"""Validate completion-document references and identifier coverage; no app acceptance."""
import json
import re
from pathlib import Path


def validate():
    folder = Path(__file__).resolve().parent
    root = folder.parents[1]
    files = sorted(folder.glob('*.md'))
    errors = []
    catalog = (folder / 'COMPLETION_TEST_CATALOG.md').read_text()
    cases = re.findall(r'^\| ([A-Z]+\d{2}) \|', catalog, re.M)
    case_set = set(cases)
    families = {re.match(r'[A-Z]+', case).group() for case in cases}
    if len(case_set) != len(cases):
        errors.append('Duplicate catalog case ID')
    for path in files:
        content = path.read_text()
        if content.count('```') % 2:
            errors.append(f'{path.name}: unbalanced code fences')
        for number, line in enumerate(content.splitlines(), 1):
            if line.rstrip() != line:
                errors.append(f'{path.name}:{number}: trailing whitespace')
        for target in re.findall(r'\]\(([^)]+)\)', content):
            if '://' in target or target.startswith('#'):
                continue
            if not (path.parent / target.split('#')[0]).exists():
                errors.append(f'{path.name}: missing file link {target}')
        for case in set(re.findall(r'\b[A-Z]+\d{2}\b', content)):
            family = re.match(r'[A-Z]+', case).group()
            if family in families and case not in case_set:
                errors.append(f'{path.name}: unknown case {case}')
        for start, end in re.findall(r'\b([A-Z]+\d{2})–([A-Z]+\d{2})\b', content):
            first_family, first = re.fullmatch(r'([A-Z]+)(\d+)', start).groups()
            last_family, last = re.fullmatch(r'([A-Z]+)(\d+)', end).groups()
            if first_family not in families:
                continue
            if first_family != last_family or int(first) > int(last):
                errors.append(f'{path.name}: invalid case range {start}–{end}')
                continue
            for number in range(int(first), int(last) + 1):
                if f'{first_family}{number:02}' not in case_set:
                    errors.append(f'{path.name}: missing range member {first_family}{number:02}')
    coverage = (folder / 'COMPLETION_REQUIREMENT_COVERAGE.md').read_text()
    pattern = r'^\| ((?:WS|UI|GIT|SEC|BULK|PUSH|AI|ACT|STATE|RELEASE)-\d{3}) \|'
    expected = set(re.findall(pattern, (root / 'docs/REQUIREMENT_EXECUTION_CROSSWALK.md').read_text(), re.M))
    actual = re.findall(pattern, coverage, re.M)
    if len(actual) != 109 or len(set(actual)) != 109 or set(actual) != expected:
        errors.append('Original requirement mapping mismatch')
    for prefix, width, count in [('INV-', 3, 15), ('M3-A', 2, 20)]:
        found = re.findall(rf'^\| ({prefix}\d{{{width}}}) \|', coverage, re.M)
        required = {f'{prefix}{number:0{width}}' for number in range(1, count + 1)}
        if len(found) != count or set(found) != required:
            errors.append(f'{prefix} coverage mismatch')
    runbook = (folder / 'COMPLETION_RUNBOOK.md').read_text()
    for family in sorted(families):
        if not re.search(rf'\b{family}(?:\d|/| cases| case)', runbook):
            errors.append(f'Unrouted case family {family}')
    ledger_path = folder / 'execution-ledger.json'
    if ledger_path.exists():
        ledger = json.loads(ledger_path.read_text())
        requirement_rows = ledger['requirements']
        case_rows = ledger['cases']
        if len(requirement_rows) != 109 or {row['id'] for row in requirement_rows} != expected:
            errors.append('Execution ledger requirement coverage mismatch')
        if len(case_rows) != len(cases) or {row['id'] for row in case_rows} != case_set:
            errors.append('Execution ledger case coverage mismatch')
        for row in requirement_rows:
            for field in ['code', 'existing_tests', 'retained_evidence_candidates']:
                if not row[field]:
                    errors.append(f"{row['id']}: empty {field}")
                for name in row[field]:
                    if not (root / name).is_file():
                        errors.append(f"{row['id']}: missing {field} file {name}")
        for row in case_rows:
            if row['status'] == 'passed' and not row['evidence']:
                errors.append(f"{row['id']}: passed without evidence")
            for name in row['evidence']:
                if not (root / name).is_file():
                    errors.append(f"{row['id']}: missing evidence file {name}")
    return {
        'scope': 'Documentation structure and references only; no application/native/release acceptance',
        'markdown_files': len(files),
        'words': sum(len(path.read_text().split()) for path in files),
        'catalog_cases': len(cases),
        'original_requirements': len(actual),
        'release_invariants': 15,
        'm3_criteria': 20,
        'errors': errors,
    }


if __name__ == '__main__':
    result = validate()
    print(json.dumps(result, indent=2))
    raise SystemExit(bool(result['errors']))
