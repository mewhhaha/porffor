"""Bind every native scope to its exact Cargo package and feature cohort."""

from pathlib import Path


def package_selection(packages=None, excluded=()):
    """Select an explicit Cargo graph without widening test-only dependencies."""
    excluded = set(excluded)
    if not packages:
        return ["--workspace", *[word for package in sorted(excluded)
                                 for word in ("--exclude", package)]]
    selected = [package for package in packages if package not in excluded]
    if not selected:
        raise ValueError("Cargo package selection must remain nonempty")
    return [word for package in selected for word in ("-p", package)]


def selector_key(case, isolated_packages=()):
    """Isolated packages retain their own cohort even for same-name targets."""
    target = "*" if case["kind"] == "lib" else case["target"]
    if isolated_packages and case["package"] in isolated_packages:
        target = case["package"] + "::" + target
    return (case["kind"], target)


def doc_cohorts(inventory, packages=None, isolated_packages=()):
    """Cover all documentation packages without merging isolated Cargo roots.

    A cohort retains all its doc siblings even on a partial continuation. The
    caller executes it only when it contains pending work; previously completed
    siblings are evidence of the cohort, never replacement verdicts.
    """
    docs = [case["package"] for case in inventory if case["kind"] == "doc"]
    if len(docs) != len(set(docs)):
        raise ValueError("duplicate documentation package")
    isolated = set(isolated_packages)
    normal = set(docs) - isolated
    cohorts = []
    if normal:
        cohorts.append(("doc:*", package_selection(packages, isolated), normal))
    cohorts.extend(("doc:" + package + "::*", ["-p", package], {package})
                   for package in sorted(set(docs) & isolated))
    return cohorts


def test_artifacts(messages, expected):
    """Reject missing, ambiguous and unexpected executables in one selector.

    `expected` contains exact (Cargo package ID, target name, target kind) keys.
    Normal binaries/dependencies are not libtest executables. Two packages may
    deliberately have the same target name; that never makes them interchangeable.
    """
    expected = set(expected)
    found = {}
    for message in messages:
        if (message.get("reason") != "compiler-artifact"
                or not message.get("executable")
                or not message.get("profile", {}).get("test")):
            continue
        target = message["target"]
        keys = [(message["package_id"], target["name"], kind)
                for kind in target["kind"]]
        matches = [key for key in keys if key in expected]
        if len(matches) != 1:
            raise ValueError(f"unexpected test artifact: {keys}")
        key = matches[0]
        path = Path(message["executable"])
        if key in found:
            raise ValueError(f"duplicate test artifact: {key}")
        found[key] = path
    missing = expected - found.keys()
    if missing:
        raise ValueError(f"missing test artifacts: {sorted(missing)}")
    return found


def build_script_artifact(messages, package_id, target_name):
    """Select one exact successful Cargo producer, independently of libtest.

    Cargo's custom-build row has executable=null. Its single filenames member
    owns the producer; optimization/profile claims must use the actual row.
    Other packages' build scripts may coexist in the same successful graph.
    File existence, bytes and hashes are sealed by the caller.
    """
    messages = list(messages)
    finished = [row.get("success") for row in messages
                if row.get("reason") == "build-finished"]
    if finished != [True]:
        raise ValueError("build script requires one successful Cargo finish")
    selected = [row for row in messages
                if row.get("reason") == "compiler-artifact"
                and row.get("package_id") == package_id
                and row.get("target", {}).get("name") == target_name
                and row["target"].get("kind") == ["custom-build"]]
    if len(selected) != 1:
        raise ValueError("missing or duplicate exact build script artifact")
    row = selected[0]
    files = row.get("filenames", [])
    if (row.get("executable") is not None
            or row.get("profile", {}).get("test") is not False
            or len(files) != 1 or not isinstance(files[0], str) or not files[0]):
        raise ValueError("invalid custom-build producer framing")
    return Path(files[0])
