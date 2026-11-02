#!/usr/bin/env python3
"""Fail early when validation tools do not satisfy GitFleet's declared floors."""
import argparse
import os
import platform
import re
import shutil
import subprocess
import sys


def require(program, minimum, flag="--version"):
    executable = shutil.which(program)
    if not executable:
        raise RuntimeError("Required executable not found: " + program)
    output = subprocess.check_output([executable, flag], text=True, stderr=subprocess.STDOUT)
    match = re.search(r"(?<!\d)(\d+)\.(\d+)(?:\.(\d+))?", output)
    if not match:
        raise RuntimeError("Cannot identify version of " + program)
    version = tuple(int(part or 0) for part in match.groups())
    if version < minimum:
        raise RuntimeError("{} {} is below required {}".format(program, version, minimum))
    print(output.splitlines()[0])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--desktop", action="store_true")
    args = parser.parse_args()
    require("git", (2, 56, 0))
    require("rustc", (1, 99, 0))
    require("cargo", (1, 99, 0))
    if args.desktop:
        require("node", (22, 12, 0))
        require("npm", (10, 0, 0))
    elif platform.system() == "Darwin":
        require("openssl", (1, 1, 0), "version")
        signer = os.environ.get("GITFLEET_TEST_GPG") or shutil.which("gpg") or shutil.which("gpg2")
        if not signer:
            raise RuntimeError("Set GITFLEET_TEST_GPG to an installed GPG executable for isolated signing tests")
        require(signer, (2, 0, 0))


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.SubprocessError) as error:
        print("GitFleet prerequisite check failed: " + str(error), file=sys.stderr)
        sys.exit(1)
