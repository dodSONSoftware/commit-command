#!/usr/bin/env python3
"""Read-only completion fixture oracles and strict evidence validation.

No Git invocation, cleanup, process signaling, source bytes or credential output.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
from datetime import datetime

MAX_ENTRIES = 100000
MAX_FILE_BYTES = 256 * 1024 * 1024
MAX_TOTAL_BYTES = 1024 * 1024 * 1024
HEX256 = re.compile(r'[0-9a-f]{64}')


def _stamp(value):
    return (value.st_dev, value.st_ino, value.st_mode, value.st_nlink,
            value.st_size, value.st_mtime_ns, value.st_ctime_ns)


def _root(path):
    """Open every absolute path component without following a symlink."""
    path = Path(path)
    if not path.is_absolute() or '..' in path.parts:
        raise ValueError('Snapshot root must be an absolute canonical directory')
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    fd = os.open('/', flags)
    try:
        for part in path.parts[1:]:
            next_fd = os.open(part, flags, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        return fd
    except BaseException:
        os.close(fd)
        raise


def snapshot(path, *, max_entries=MAX_ENTRIES, max_file_bytes=MAX_FILE_BYTES,
             max_total_bytes=MAX_TOTAL_BYTES):
    """Hash all entries including Git metadata; race/limit/special-file failures abort.

    Hex-encoded native relative paths and symlink targets preserve non-UTF8 bytes.
    File content is streamed and never returned. Directory changes during traversal
    cause failure rather than a misleading internally inconsistent preservation pass.
    """
    if min(max_entries, max_file_bytes, max_total_bytes) < 1:
        raise ValueError('Snapshot limits must be positive')
    entries = {}
    total = 0

    def visit(parent, name, relative):
        nonlocal total
        if len(entries) >= max_entries:
            raise ValueError('Snapshot entry limit exceeded')
        before = os.stat(name, dir_fd=parent, follow_symlinks=False)
        value = dict(mode=stat.S_IMODE(before.st_mode), device=before.st_dev,
                     inode=before.st_ino, links=before.st_nlink, owner=before.st_uid, group=before.st_gid)
        entries[relative.hex()] = value
        if stat.S_ISLNK(before.st_mode):
            value.update(kind='symlink', target_hex=os.fsencode(os.readlink(name, dir_fd=parent)).hex())
        elif stat.S_ISDIR(before.st_mode):
            fd = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)
            try:
                if _stamp(before) != _stamp(os.fstat(fd)):
                    raise ValueError('Directory changed before snapshot')
                value['kind'] = 'directory'
                for child in sorted(os.listdir(fd), key=os.fsencode):
                    child_bytes = os.fsencode(child)
                    visit(fd, child, relative + (b'/' if relative else b'') + child_bytes)
                if _stamp(before) != _stamp(os.fstat(fd)):
                    raise ValueError('Directory changed during snapshot')
            finally:
                os.close(fd)
        elif stat.S_ISREG(before.st_mode):
            if before.st_size > max_file_bytes or total + before.st_size > max_total_bytes:
                raise ValueError('Snapshot byte limit exceeded')
            fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=parent)
            try:
                if _stamp(before) != _stamp(os.fstat(fd)):
                    raise ValueError('File changed before snapshot')
                digest = hashlib.sha256()
                size = 0
                while chunk := os.read(fd, 65536):
                    size += len(chunk)
                    if size > max_file_bytes or total + size > max_total_bytes:
                        raise ValueError('Snapshot byte limit exceeded')
                    digest.update(chunk)
                if size != before.st_size or _stamp(before) != _stamp(os.fstat(fd)):
                    raise ValueError('File changed during snapshot')
                total += size
                value.update(kind='file', bytes=size, sha256=digest.hexdigest())
            finally:
                os.close(fd)
        else:
            raise ValueError('Snapshot refuses special files')
        if _stamp(before) != _stamp(os.stat(name, dir_fd=parent, follow_symlinks=False)):
            raise ValueError('Entry changed during snapshot')

    fd = _root(path)
    try:
        visit(fd, '.', b'')
    finally:
        os.close(fd)
    return dict(version=1, root=str(path), entries=entries, total_bytes=total)


def compare(before, after, allowed_paths=()):
    """Return exact changed native paths; allowed deltas are literal, never globs."""
    if before.get('version') != 1 or after.get('version') != 1 or before.get('root') != after.get('root'):
        raise ValueError('Snapshot versions or roots differ')
    allowed = set(allowed_paths)
    for path in allowed:
        parts = path.split(b'/')
        if path.startswith(b'/') or any(part in (b'', b'.', b'..') for part in parts):
            raise ValueError('Allowed delta must be a literal relative path')
    left, right = before['entries'], after['entries']
    changed = sorted(key for key in left.keys() | right.keys() if left.get(key) != right.get(key))
    unexpected = [key for key in changed if bytes.fromhex(key) not in allowed]
    return dict(changed_paths_hex=changed, unexpected_paths_hex=unexpected, preserved=not unexpected)


def validate_result(result, evidence_root):
    """Require explicit provenance/oracles for pass claims; never infer a pass."""
    if not isinstance(result, dict):
        raise ValueError('Result must be an object')
    fields = {'version', 'case', 'package', 'status', 'operator', 'timestamp', 'source',
              'artifact', 'environment', 'fixture', 'action', 'expected', 'actual',
              'exit_code', 'allowed_deltas', 'oracles', 'phase_proof', 'reason'}
    if set(result) != fields or result['version'] != 1:
        raise ValueError('Unknown, missing or unsupported result fields')
    if result['status'] not in ('passed', 'failed', 'blocked', 'unsupported', 'not_run'):
        raise ValueError('Unknown result status')
    if not re.fullmatch(r'[A-Z]+[0-9]{2}', result['case']) or not re.fullmatch(r'P[0-9]{2}[AB]?', result['package']):
        raise ValueError('Invalid case or package identifier')
    when = datetime.fromisoformat(result['timestamp'])
    if when.tzinfo is None:
        raise ValueError('Timestamp requires a timezone')
    for field in ('operator', 'fixture', 'action', 'expected', 'actual'):
        if not isinstance(result[field], str) or not result[field].strip():
            raise ValueError('Missing result description: ' + field)
    if not isinstance(result['allowed_deltas'], list) or not all(isinstance(v, str) for v in result['allowed_deltas']):
        raise ValueError('Allowed deltas must be an explicit string list')
    if not isinstance(result['reason'], str):
        raise ValueError('Reason must be text')
    source, artifact = result['source'], result['artifact']
    if not isinstance(source, dict) or set(source) != {'commit', 'dirty', 'diff_sha256'}:
        raise ValueError('Source must record exact commit and dirty diff fingerprint')
    if not re.fullmatch(r'[0-9a-f]{40}|[0-9a-f]{64}', source['commit']) or type(source['dirty']) is not bool or not HEX256.fullmatch(source['diff_sha256']):
        raise ValueError('Invalid source fingerprint')
    if not isinstance(artifact, dict) or set(artifact) != {'sha256', 'features', 'launch'} or not HEX256.fullmatch(artifact['sha256']):
        raise ValueError('Missing artifact hash/features/launch')
    if not all(isinstance(artifact[key], list) and all(isinstance(v, str) for v in artifact[key]) for key in ('features', 'launch')):
        raise ValueError('Artifact features and launch must be string lists')
    environment = result['environment']
    if not isinstance(environment, dict) or set(environment) != {'os', 'architecture', 'filesystem', 'tools'} or any(not environment[key] for key in environment):
        raise ValueError('Missing environment/tool provenance')
    if not isinstance(result['oracles'], dict) or not isinstance(result['phase_proof'], dict):
        raise ValueError('Oracles and phase proof must be objects')
    if type(result['exit_code']) is not int:
        raise ValueError('Exit code must be explicit')
    if result['status'] == 'passed':
        if result['exit_code'] != 0 or result['reason']:
            raise ValueError('Pass requires zero exit and no failure reason')
        if not result['oracles'] or not result['phase_proof']:
            raise ValueError('Pass requires oracle artifacts and phase/action proof')
    elif not result['reason'].strip():
        raise ValueError('Nonpass requires an explicit reason')
    base = Path(evidence_root)
    for name, ref in list(result['oracles'].items()) + list(result['phase_proof'].items()):
        if not isinstance(name, str) or not name or not isinstance(ref, dict) or set(ref) != {'path', 'sha256'}:
            raise ValueError('Evidence references require path and hash')
        relative = ref['path']
        if not isinstance(relative, str) or not relative or '\\' in relative or relative.startswith('/') or any(part in ('', '.', '..') for part in relative.split('/')):
            raise ValueError('Evidence path must be a literal contained relative file')
        if not isinstance(ref['sha256'], str) or not HEX256.fullmatch(ref['sha256']):
            raise ValueError('Invalid evidence hash')
        parent = _root(base)
        try:
            parts = relative.split('/')
            for part in parts[:-1]:
                child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=parent)
                os.close(parent)
                parent = child
            fd = os.open(parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK | os.O_CLOEXEC, dir_fd=parent)
            try:
                before = os.fstat(fd)
                if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > MAX_FILE_BYTES:
                    raise ValueError('Evidence must be a bounded single-link regular file')
                digest = hashlib.sha256()
                size = 0
                while chunk := os.read(fd, 65536):
                    size += len(chunk)
                    if size > MAX_FILE_BYTES:
                        raise ValueError('Evidence exceeds byte limit')
                    digest.update(chunk)
                if _stamp(before) != _stamp(os.fstat(fd)) or digest.hexdigest() != ref['sha256']:
                    raise ValueError('Evidence changed or hash mismatched')
            finally:
                os.close(fd)
        finally:
            os.close(parent)
    return {'case': result['case'], 'status': result['status'], 'validated': True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='command', required=True)
    snap = commands.add_parser('snapshot')
    snap.add_argument('root', type=Path)
    check = commands.add_parser('validate')
    check.add_argument('result', type=Path)
    diff = commands.add_parser('compare')
    diff.add_argument('before', type=Path)
    diff.add_argument('after', type=Path)
    diff.add_argument('--allow', action='append', default=[])
    args = parser.parse_args()
    if args.command == 'snapshot':
        result = snapshot(args.root)
    elif args.command == 'validate':
        result = validate_result(json.loads(args.result.read_text()), args.result.resolve().parent)
    else:
        result = compare(json.loads(args.before.read_text()), json.loads(args.after.read_text()), [os.fsencode(p) for p in args.allow])
    print(json.dumps(result, indent=2, sort_keys=True))
    return 1 if result.get('preserved') is False else 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, TypeError, KeyError) as error:
        raise SystemExit('Completion evidence refused: ' + str(error))
