#!/usr/bin/env python3
"""Inventory locked dependency license declarations; no release compliance decision."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def inventory(project: Path, target: str) -> dict:
    packages = {}
    inputs = {}
    for relative in ("app/core", "app/desktop/src-tauri"):
        directory = project / relative
        metadata = json.loads(subprocess.check_output([
            "cargo", "metadata", "--manifest-path", str(directory / "Cargo.toml"),
            "--format-version", "1", "--all-features", "--filter-platform", target, "--locked", "--offline",
        ], text=True))
        resolved = {node["id"] for node in metadata["resolve"]["nodes"]}
        for package in metadata["packages"]:
            if package["id"] not in resolved:
                continue
            key = ("cargo", package["id"])
            value = packages.setdefault(key, {
                "ecosystem": "cargo", "name": package["name"],
                "version": package["version"], "source": package["source"],
                "first_party": package["id"] in metadata["workspace_members"],
                "declared_license": package["license"],
                "license_file_declared": package["license_file"] is not None,
                "graphs": [],
            })
            value["graphs"].append(relative)
        lock = directory / "Cargo.lock"
        inputs[str(lock.relative_to(project))] = hashlib.sha256(lock.read_bytes()).hexdigest()
    lock = project / "app/desktop/package-lock.json"
    data = json.loads(lock.read_text())
    if data["lockfileVersion"] != 3:
        raise ValueError("Expected npm lockfile version 3")
    for path, package in data["packages"].items():
        if package.get("link"):
            raise ValueError("Linked npm packages need explicit inventory support")
        name = package.get("name") or path.rsplit("node_modules/", 1)[-1]
        packages[("npm", path)] = {
            "ecosystem": "npm", "name": name, "version": package["version"],
            "source": package.get("resolved"), "first_party": path == "",
            "declared_license": package.get("license"), "license_file_declared": False,
            "development_only": package.get("dev", False),
            "optional": package.get("optional", False),
            "integrity": package.get("integrity"), "lock_path": path,
            "graphs": ["app/desktop/package-lock.json"],
        }
    inputs[str(lock.relative_to(project))] = hashlib.sha256(lock.read_bytes()).hexdigest()
    values = sorted(packages.values(), key=lambda p: (p["ecosystem"], p["name"], p["version"], str(p["source"])))
    missing = [{"ecosystem": p["ecosystem"], "name": p["name"], "version": p["version"], "first_party": p["first_party"]}
               for p in values if not p["declared_license"] and not p["license_file_declared"]]
    return {
        "version": 1,
        "target": target,
        "scope": "All resolved Cargo features for the named target and complete npm lock graph, including development/test tools. Not the packaged shipping closure or bundled notices.",
        "input_sha256": inputs, "packages": values, "missing_declarations": missing,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--target", default="aarch64-apple-darwin")
    args = parser.parse_args()
    project = Path(__file__).resolve().parents[3]
    result = inventory(project, args.target)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"records": len(result["packages"]), "missing_declarations": result["missing_declarations"]}))


if __name__ == "__main__":
    main()
