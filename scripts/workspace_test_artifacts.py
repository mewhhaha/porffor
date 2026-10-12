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
