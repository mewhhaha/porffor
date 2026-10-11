"""Keep native test selection on one Cargo feature graph and exact package IDs."""

from pathlib import Path


def package_selection(packages=None):
    """Select the same package set for every selector in a verification run."""
    if not packages:
        return ["--workspace"]
    return [word for package in packages for word in ("-p", package)]


def selector_key(case):
    return (case["kind"], "*" if case["kind"] == "lib" else case["target"])


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
