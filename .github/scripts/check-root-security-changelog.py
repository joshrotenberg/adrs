#!/usr/bin/env python3
"""Fail when a root-only security fix is missing from a crate changelog.

release-plz builds each crate's changelog only from commits that touch the
crate's directory. A security fix that changes only the workspace-root
Cargo.toml or Cargo.lock never reaches a crate changelog, even when cliff.toml
would classify it as Security (issue #409).

For each workspace crate, this script looks at the commits since the crate's
last release tag. A commit needs a hand-written changelog entry when it:

- names an advisory (RUSTSEC-, CVE-, GHSA-) or uses the `security` scope,
- touches no file under any workspace crate directory, and
- changes a locked package version or a [workspace.dependencies] entry that
  the crate depends on, directly or transitively, outside dev-dependencies.

A commit that touches a crate directory is in that crate's changelog. Whether
a dependent crate's changelog should repeat it is left to the author; release-plz
can include it with `changelog_include`.

The crate's CHANGELOG.md must then contain each advisory ID the commit names,
or the commit's subject description when it names none. Add the entry under
`## [Unreleased]` on main; release-plz carries that section into the next
release. See RELEASING.md.

Run from the repository root: python3 .github/scripts/check-root-security-changelog.py
Requires Python 3.11 or later, git, and cargo.
"""

import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

# Matches the Security rules in cliff.toml.
SECURITY_SUBJECT = re.compile(r"(?i)^[a-z]+\(security\)!?:|\b(RUSTSEC-|CVE-|GHSA-)")
SECURITY_BODY = re.compile(r"\b(RUSTSEC|CVE|GHSA)-")
ADVISORY_ID = re.compile(
    r"\b(RUSTSEC-\d{4}-\d{4}|CVE-\d{4}-\d{4,}|GHSA(?:-[0-9a-z]{4}){3})\b", re.IGNORECASE
)
CONVENTIONAL_PREFIX = re.compile(r"^[a-z]+(\([^)]*\))?!?:\s*", re.IGNORECASE)
TEMPLATE_VAR = re.compile(r"\{\{\s*(\w+)\s*\}\}")


def run(*args: str) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


def git_show_toml(rev: str, path: str) -> dict | None:
    try:
        return tomllib.loads(run("git", "show", f"{rev}:{path}"))
    except (subprocess.CalledProcessError, tomllib.TOMLDecodeError):
        return None


def tag_glob(config: dict, name: str) -> str:
    """Turn the package's release-plz git_tag_name into a git describe pattern."""
    package = next((p for p in config.get("package", []) if p.get("name") == name), {})
    template = package.get(
        "git_tag_name",
        config.get("workspace", {}).get("git_tag_name", "{{ package }}-v{{ version }}"),
    )
    return TEMPLATE_VAR.sub(lambda m: name if m.group(1) == "package" else "*", template)


def last_release_tag(pattern: str) -> str | None:
    try:
        return run("git", "describe", "--tags", "--abbrev=0", "--match", pattern, "HEAD").strip()
    except subprocess.CalledProcessError:
        return None


def locked_versions(lock: dict | None) -> set[tuple[str, str]]:
    if lock is None:
        return set()
    return {(p["name"], p["version"]) for p in lock.get("package", [])}


def workspace_requirements(manifest: dict | None) -> dict[str, str]:
    if manifest is None:
        return {}
    deps = manifest.get("workspace", {}).get("dependencies", {})
    out = {}
    for key, value in deps.items():
        name = value.get("package", key) if isinstance(value, dict) else key
        out[name] = json.dumps(value, sort_keys=True)
    return out


def affected_packages(commit: str, dependencies: set[tuple[str, str]]) -> set[str]:
    """Names of the crate's dependencies that the commit changed.

    A locked package counts when the commit changed its versions and the crate
    now uses a version the parent commit did not lock. That ignores a second
    major version added for another crate, and still catches a fix that a later
    commit bumped again. A changed [workspace.dependencies] entry counts when
    the crate depends on the package at all.
    """
    parent = f"{commit}^"
    before = locked_versions(git_show_toml(parent, "Cargo.lock"))
    after = locked_versions(git_show_toml(commit, "Cargo.lock"))
    changed = {name for name, _ in before ^ after}
    names = {name for name, version in dependencies if name in changed and (name, version) not in before}
    reqs_before = workspace_requirements(git_show_toml(parent, "Cargo.toml"))
    reqs_after = workspace_requirements(git_show_toml(commit, "Cargo.toml"))
    used = {name for name, _ in dependencies}
    for name in (reqs_before.keys() | reqs_after.keys()) & used:
        if reqs_before.get(name) != reqs_after.get(name):
            names.add(name)
    return names


def shipped_dependencies(metadata: dict, member_id: str) -> set[tuple[str, str]]:
    """Every package the member depends on outside dev-dependencies, as (name, version)."""
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    names = {pkg["id"]: (pkg["name"], pkg["version"]) for pkg in metadata["packages"]}
    seen = set()
    stack = [member_id]
    while stack:
        node = nodes[stack.pop()]
        for dep in node["deps"]:
            if dep["pkg"] in seen:
                continue
            if all(kind["kind"] == "dev" for kind in dep["dep_kinds"]):
                continue
            seen.add(dep["pkg"])
            stack.append(dep["pkg"])
    return {names[pkg_id] for pkg_id in seen}


def required_tokens(message: str) -> list[str]:
    ids = sorted({m.upper() for m in ADVISORY_ID.findall(message)})
    if ids:
        return ids
    subject = message.splitlines()[0]
    return [CONVENTIONAL_PREFIX.sub("", subject)]


def is_security_commit(message: str) -> bool:
    subject, _, body = message.partition("\n")
    return bool(SECURITY_SUBJECT.search(subject) or SECURITY_BODY.search(body))


def normalize(text: str) -> str:
    return " ".join(text.split()).lower()


def main() -> int:
    root = Path(run("git", "rev-parse", "--show-toplevel").strip())
    config_path = root / "release-plz.toml"
    config = tomllib.loads(config_path.read_text()) if config_path.exists() else {}
    metadata = json.loads(
        run("cargo", "metadata", "--format-version", "1", "--all-features", "--manifest-path",
            str(root / "Cargo.toml"))
    )
    packages = {pkg["id"]: pkg for pkg in metadata["packages"]}
    crate_dirs = [
        Path(packages[member_id]["manifest_path"]).parent.relative_to(root).as_posix()
        for member_id in metadata["workspace_members"]
    ]

    failures = []
    for member_id in metadata["workspace_members"]:
        package = packages[member_id]
        name = package["name"]
        crate_dir = Path(package["manifest_path"]).parent.relative_to(root).as_posix()
        tag = last_release_tag(tag_glob(config, name))
        if tag is None:
            print(f"{name}: no release tag, skipping")
            continue

        commits = run("git", "rev-list", "--no-merges", f"{tag}..HEAD").split()
        dependencies = None
        checked = 0
        for commit in commits:
            message = run("git", "log", "-1", "--format=%B", commit).strip()
            if not is_security_commit(message):
                continue
            files = run("git", "diff-tree", "--no-commit-id", "--name-only", "-r", commit).split("\n")
            if any(f.startswith(f"{d}/") for f in files for d in crate_dirs):
                # release-plz lists this commit in that crate's changelog.
                continue
            if dependencies is None:
                dependencies = shipped_dependencies(metadata, member_id)
            affected = sorted(affected_packages(commit, dependencies))
            if not affected:
                continue
            checked += 1
            changelog_path = root / crate_dir / "CHANGELOG.md"
            changelog = normalize(changelog_path.read_text()) if changelog_path.exists() else ""
            missing = [t for t in required_tokens(message) if normalize(t) not in changelog]
            if missing:
                subject = message.splitlines()[0]
                failures.append(
                    f"{name}: {commit[:7]} \"{subject}\" changes {', '.join(affected)} "
                    f"but {crate_dir}/CHANGELOG.md does not mention {', '.join(missing)}"
                )
        print(f"{name}: {len(commits)} commits since {tag}, {checked} root-only security fixes")

    if failures:
        print()
        for failure in failures:
            print(f"error: {failure}")
        print()
        print("Add a Security entry under `## [Unreleased]` in each crate changelog above.")
        print("See \"Root-only security fixes\" in RELEASING.md.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
