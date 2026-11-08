#!/usr/bin/env python3
"""Build checksum-pinned Git in an explicit disposable prefix; never installs globally."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tarfile
import urllib.request

VERSION = '2.56.0'
SHA256 = '26c56c296b38c0695b26fa95f475f1d01704d2d38e73465ca30b0b2f5dc789d3'
URL = 'https://www.kernel.org/pub/software/scm/git/git-' + VERSION + '.tar.xz'


def validate_members(members):
    base = 'git-' + VERSION
    # These are the pinned upstream archive's three repository-internal links.
    links = {base + '/RelNotes': 'Documentation/RelNotes/' + VERSION + '.adoc',
             base + '/subprojects/git-gui': '../git-gui',
             base + '/subprojects/gitk': '../gitk-git'}
    seen = set()
    for member in members:
        parts = Path(member.name).parts
        if not parts or parts[0] != base or '..' in parts or member.name in seen:
            raise ValueError('Unexpected Git archive path')
        seen.add(member.name)
        if any(member.name.startswith(link + '/') for link in links):
            raise ValueError('Git archive entry descends through a link')
        if member.issym():
            if links.get(member.name) != member.linkname:
                raise ValueError('Unexpected Git archive link')
        elif not (member.isdir() or member.isfile()):
            raise ValueError('Unexpected Git archive member type')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    root = args.directory.resolve()
    if root.exists():
        raise SystemExit('Provisioning directory must be new; refusing to reuse or overwrite it')
    root.mkdir(parents=True)
    archive = root / ('git-' + VERSION + '.tar.xz')
    with urllib.request.urlopen(URL, timeout=60) as response:
        archive.write_bytes(response.read())
    actual = hashlib.sha256(archive.read_bytes()).hexdigest()
    if actual != SHA256:
        raise SystemExit('Git source checksum mismatch; not extracting or executing')
    source = root / ('git-' + VERSION)
    with tarfile.open(archive) as stream:
        validate_members(stream.getmembers())
        stream.extractall(root)
    prefix = root / 'install'
    build = ['make', '-j4', 'prefix=' + str(prefix), 'NO_GETTEXT=YesPlease',
             'NO_TCLTK=YesPlease', 'NO_INSTALL_HARDLINKS=YesPlease']
    subprocess.run(build, cwd=source, check=True)
    subprocess.run(build + ['install'], cwd=source, check=True)
    binary = prefix / 'bin/git'
    version = subprocess.check_output([str(binary), '--version'], text=True).strip()
    if version != 'git version ' + VERSION:
        raise SystemExit('Installed Git version does not match pinned version')
    metadata = dict(url=URL, source_sha256=actual, version=version,
                    prefix=str(prefix), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
    (root / 'provenance.json').write_text(json.dumps(metadata, indent=2) + '\n')
    github_path = os.environ.get('GITHUB_PATH')
    if github_path:
        with open(github_path, 'a') as output:
            output.write(str(prefix / 'bin') + '\n')
    print(json.dumps(metadata), flush=True)


if __name__ == '__main__':
    main()
