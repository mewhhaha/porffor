#!/usr/bin/env python3
"""Bind low-memory publication sessions to immutable identities and durable progress.

This records the observed checkout, source bytes and executable bytes. It is
not a build attestation and does not retrofit provenance into native snapshots.
A family with pre-existing results but no manifest must use a new name.
"""
from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import json
import os
import re
from dataclasses import dataclass
from pathlib import Path
import signal
import stat
import subprocess
import sys
import tempfile
from collections.abc import Iterator, Mapping

SCHEMA_VERSION = 2
MAX_MATRIX_COUNT = 10**18 - 1
SOURCE_FILES = ("Cargo.toml", "Cargo.lock", "rust-toolchain", "rust-toolchain.toml")
SOURCE_DIRECTORIES = (".cargo", "crates", "vendor", "scripts")
IGNORED_DIRECTORIES = frozenset({".git", "target", "__pycache__"})
RUNTIME_ENVIRONMENT = (
    "TZ", "LANG", "LC_ALL", "LC_TIME", "RUST_MIN_STACK",
    "LILA_MODULE_MEMORY_CACHE_ENTRIES", "LILA_CACHE_LIMIT_BYTES",
    "LILA_FUNCTION_CACHE_LIMIT_BYTES", "LILA_MODULE_CACHE_LIMIT_BYTES",
    "LILA_PROGRAM_CACHE_LIMIT_BYTES", "LILA_TEST262_FORCE_CASE_RUNNER",
    "LILA_TEST262_DISABLE_CASE_RUNNER",
)
IDENTITY_KEYS = frozenset({
    "repository", "checkout_commit", "checkout_tree", "source_inputs_sha256",
    "source_input_files", "executable", "executable_sha256", "suite_root",
    "suite_sha256", "suite_files", "snapshot_directory", "snapshot_name",
    "execution_backend", "threads", "jobs", "isolate_cases", "environment",
})


class ProvenanceError(Exception):
    """An absent, incompatible or unverifiable publication identity."""


@dataclass(frozen=True)
class MatrixProgress:
    """A checked matrix observation, not evidence of passing test executions."""

    completed: int
    total: int

    def __post_init__(self) -> None:
        if type(self.completed) is not int or type(self.total) is not int:
            raise ProvenanceError("matrix progress counts must be integers, not booleans")
        if not 0 <= self.completed <= MAX_MATRIX_COUNT:
            raise ProvenanceError("invalid matrix progress: completed count")
        if not 1 <= self.total <= MAX_MATRIX_COUNT:
            raise ProvenanceError("invalid matrix progress: total must be positive and bounded")
        if self.completed > self.total:
            raise ProvenanceError("invalid matrix progress: completed exceeds total")

    def as_dict(self) -> dict[str, int]:
        return {"completed": self.completed, "total": self.total}

    @classmethod
    def from_dict(cls, value: object) -> MatrixProgress:
        if not isinstance(value, dict) or set(value) != {"completed", "total"}:
            raise ProvenanceError("invalid publication manifest progress")
        return cls(value["completed"], value["total"])


def parse_matrix_progress(text: str) -> MatrixProgress:
    fields: dict[str, int] = {}
    names = {"matrix_nodes_completed": "completed", "matrix_nodes_total": "total"}
    for line in text.splitlines():
        key, separator, value = line.partition(": ")
        if key not in names:
            continue
        if key in fields:
            raise ProvenanceError(f"duplicate matrix progress field: {key}")
        if not separator or not re.fullmatch(r"0|[1-9][0-9]{0,17}", value):
            raise ProvenanceError(f"invalid matrix progress field: {key}")
        fields[key] = int(value)
    if set(fields) != set(names):
        raise ProvenanceError("invalid matrix progress: expected exactly one completed and total field")
    return MatrixProgress(fields["matrix_nodes_completed"], fields["matrix_nodes_total"])


def _git(root: Path, *arguments: str) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(root), *arguments], stderr=subprocess.PIPE, text=True,
        ).strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise ProvenanceError(f"cannot inspect source checkout: {error}") from error


def _regular_files(root: Path) -> Iterator[Path]:
    if root.is_symlink():
        raise ProvenanceError(f"input symlinks are not supported: {root}")
    mode = root.stat().st_mode
    if stat.S_ISREG(mode):
        yield root
    elif stat.S_ISDIR(mode):
        for child in sorted(root.iterdir()):
            if child.name in IGNORED_DIRECTORIES and child.is_dir():
                continue
            yield from _regular_files(child)
    else:
        raise ProvenanceError(f"input is not a regular file or directory: {root}")


def _file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _tree_digest(root: Path, paths: list[Path]) -> tuple[str, int]:
    # Length framing distinguishes both paths and contents without depending on
    # directory enumeration order, filesystem times or Git index cleanliness.
    digest = hashlib.sha256(b"lila-publication-inputs-v1\0")
    files = sorted({file for path in paths for file in _regular_files(path)})
    if not files:
        raise ProvenanceError(f"refusing an empty input inventory: {root}")
    for path in files:
        label = path.relative_to(root).as_posix().encode("utf-8")
        size = path.stat().st_size
        digest.update(len(label).to_bytes(8, "big"))
        digest.update(label)
        digest.update(size.to_bytes(8, "big"))
        read = 0
        with path.open("rb") as source:
            for block in iter(lambda: source.read(1024 * 1024), b""):
                read += len(block)
                digest.update(block)
        if read != size:
            raise ProvenanceError(f"input changed while it was being hashed: {path}")
    return digest.hexdigest(), len(files)


def capture_identity(environment: Mapping[str, str]) -> dict:
    root = Path(environment["REPO_ROOT"]).resolve(strict=True)
    binary = Path(environment["LILA_BIN"]).resolve(strict=True)
    suite = Path(environment["SUITE_ROOT"]).resolve(strict=True)
    snapshots = Path(environment["SNAPSHOT_DIR"]).resolve()
    name = environment["SNAPSHOT_NAME"]
    if not name or name in (".", "..") or any(char in name for char in "/\\") or any(ord(char) < 32 for char in name):
        raise ProvenanceError("snapshot name must be a nonempty, non-relative filename component")
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ProvenanceError(f"missing executable or no longer executable: {binary}")
    if not suite.is_dir():
        raise ProvenanceError(f"suite root is not a directory: {suite}")
    paths = [root / name for name in (*SOURCE_FILES, *SOURCE_DIRECTORIES) if (root / name).exists()]
    # The external suite is hashed separately, including all harness and test
    # bytes. Generated snapshots/README status are intentionally not inputs.
    for harness in (root / "test262").glob("*.js"):
        paths.append(harness)
    source_hash, source_count = _tree_digest(root, paths)
    suite_hash, suite_count = _tree_digest(suite, [suite])
    return {
        "repository": str(root),
        "checkout_commit": _git(root, "rev-parse", "--verify", "HEAD"),
        "checkout_tree": _git(root, "rev-parse", "--verify", "HEAD^{tree}"),
        "source_inputs_sha256": source_hash,
        "source_input_files": source_count,
        "executable": str(binary),
        "executable_sha256": _file_digest(binary),
        "suite_root": str(suite),
        "suite_sha256": suite_hash,
        "suite_files": suite_count,
        "snapshot_directory": str(snapshots),
        "snapshot_name": name,
        "execution_backend": "wasm-aot",
        "threads": int(environment["THREADS"]),
        "jobs": int(environment["JOBS"]),
        "isolate_cases": environment["ISOLATE_CASES"] == "1",
        "environment": {name: environment.get(name) for name in RUNTIME_ENVIRONMENT},
    }


def manifest_paths(identity: dict) -> tuple[Path, Path]:
    metadata = Path(identity["snapshot_directory"]) / ".publication-provenance"
    # Snapshot names can contain spaces/non-ASCII; the metadata path must not
    # collide after case folding and must never be considered a node snapshot.
    key = hashlib.sha256(identity["snapshot_name"].encode("utf-8")).hexdigest()
    return metadata / f"{key}.json", metadata / f"{key}.lock"


def _unique_object(pairs: list[tuple[str, object]]) -> dict:
    output = {}
    for key, value in pairs:
        if key in output:
            raise ProvenanceError(f"duplicate manifest field: {key}")
        output[key] = value
    return output


def _read_manifest_document(path: Path) -> dict:
    if path.is_symlink():
        raise ProvenanceError(f"publication manifest must not be a symlink: {path}")
    try:
        value = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=_unique_object)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ProvenanceError(f"cannot read publication manifest {path}: {error}") from error
    if not isinstance(value, dict) or "schema_version" not in value:
        raise ProvenanceError(f"invalid publication manifest envelope: {path}")
    if type(value["schema_version"]) is not int or value["schema_version"] != SCHEMA_VERSION:
        raise ProvenanceError(
            f"unsupported publication manifest schema: {path}; "
            "retain the existing results and use a fresh snapshot name"
        )
    if set(value) != {"schema_version", "identity", "progress"}:
        raise ProvenanceError(f"invalid publication manifest envelope: {path}")
    identity = value["identity"]
    if not isinstance(identity, dict) or set(identity) != IDENTITY_KEYS:
        raise ProvenanceError(f"invalid publication manifest identity: {path}")
    if value["progress"] is not None:
        MatrixProgress.from_dict(value["progress"])
    return value


def read_manifest(path: Path) -> dict:
    return _read_manifest_document(path)["identity"]


def require_identity(path: Path, expected: dict) -> None:
    recorded = read_manifest(path)
    # Canonical JSON also distinguishes booleans from integers (True == 1 in
    # Python), so malformed typed values cannot compare equal accidentally.
    differences = [key for key in sorted(IDENTITY_KEYS)
                   if json.dumps(recorded[key], sort_keys=True) != json.dumps(expected[key], sort_keys=True)]
    if differences:
        raise ProvenanceError(
            "publication provenance mismatch for " + ", ".join(differences)
            + "; retain the existing results and use a fresh snapshot name"
        )


def family_result_names(identity: dict) -> list[str]:
    name = identity["snapshot_name"]
    return sorted(
        file.name for file in Path(identity["snapshot_directory"]).iterdir()
        if file.suffix in {".json", ".txt", ".jsonl"}
        and (file.stem == name or file.name.startswith(name + "-"))
    )


def require_fresh_matrix(path: Path) -> None:
    document = _read_manifest_document(path)
    if document["progress"] is not None:
        raise ProvenanceError("matrix checkpoint is unavailable after previously recorded progress")
    if family_result_names(document["identity"]):
        raise ProvenanceError("matrix checkpoint is unavailable for a family with existing results")


def _replace_manifest(path: Path, document: dict) -> None:
    # Keep identity and its high-water mark in ONE document. Two independently
    # replaced sidecars would permit a crash to lose the last observed count.
    descriptor, temporary = tempfile.mkstemp(prefix=".manifest-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            json.dump(document, output, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
        directory = os.open(path.parent, os.O_RDONLY | getattr(os, "O_DIRECTORY", 0))
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        Path(temporary).unlink(missing_ok=True)


def record_matrix_progress(path: Path, current: MatrixProgress, *, after_report: bool) -> None:
    # The supervisor retains the family lock. A distinct observation lock also
    # serializes direct invocations of this read/modify/write CLI operation.
    with family_lock(path.with_suffix(".progress.lock")):
        document = _read_manifest_document(path)
        previous = (MatrixProgress.from_dict(document["progress"])
                    if document["progress"] is not None else None)
        if previous is not None:
            if current.total != previous.total:
                raise ProvenanceError("matrix total changed from the recorded publication denominator")
            if current.completed < previous.completed:
                raise ProvenanceError("matrix progress regressed below the recorded high-water mark")
        minimum = previous.completed if previous is not None else 0
        if after_report and current.completed <= minimum:
            raise ProvenanceError("report-all did not advance completed matrix nodes")
        if current == previous:
            return
        document["progress"] = current.as_dict()
        _replace_manifest(path, document)


def claim_manifest(path: Path, identity: dict) -> None:
    if path.exists() or path.is_symlink():
        if path.is_symlink():
            raise ProvenanceError(f"publication manifest must not be a symlink: {path}")
        require_identity(path, identity)
        return
    existing = family_result_names(identity)
    if existing:
        raise ProvenanceError(
            "existing results have no publication provenance; refusing to adopt them: "
            + ", ".join(existing[:3]) + "; use a fresh snapshot name"
        )
    descriptor, temporary = tempfile.mkstemp(prefix=".manifest-", dir=path.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            json.dump({"schema_version": SCHEMA_VERSION, "identity": identity, "progress": None},
                      output, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        # The family lock is held by the caller. Link rather than replace still
        # refuses to overwrite a file that appeared outside this supervisor.
        os.link(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)


@contextlib.contextmanager
def family_lock(path: Path):
    descriptor = os.open(path, os.O_CREAT | os.O_RDWR | getattr(os, "O_NOFOLLOW", 0), 0o600)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ProvenanceError("publication family is already locked by another session") from error
        # Keep this inode: unlinking a flock file would allow a second writer to
        # lock a different inode under the same pathname.
        yield descriptor
    finally:
        os.close(descriptor)


def run_session(environment: dict[str, str]) -> int:
    identity = capture_identity(environment)
    manifest, lock = manifest_paths(identity)
    manifest.parent.mkdir(parents=True, exist_ok=True)
    with family_lock(lock) as descriptor:
        claim_manifest(manifest, identity)
        print(f"publication_manifest: {manifest}", flush=True)
        print(f"source_inputs_sha256: {identity['source_inputs_sha256']}", flush=True)
        print(f"suite_sha256: {identity['suite_sha256']}", flush=True)
        command = ["bash", str(Path(identity["repository"]) / "scripts/lib/publish-real-status-driver.sh"),
                   "wasm-aot", identity["snapshot_name"]]
        child_env = {**environment, "LILA_PUBLICATION_MANIFEST": str(manifest)}
        with subprocess.Popen(command, env=child_env, start_new_session=True, pass_fds=(descriptor,)) as child:
            def forward(signum, _frame):
                try:
                    os.killpg(child.pid, signum)
                except ProcessLookupError:
                    pass
            previous = {sig: signal.signal(sig, forward) for sig in (signal.SIGINT, signal.SIGTERM)}
            try:
                status = child.wait()
            finally:
                for sig, handler in previous.items():
                    signal.signal(sig, handler)
        return status if status >= 0 else 128 - status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("run", "verify", "record-progress", "require-fresh"))
    parser.add_argument("manifest", nargs="?", type=Path)
    parser.add_argument("--after-report", action="store_true")
    args = parser.parse_args()
    try:
        if args.after_report and args.operation != "record-progress":
            raise ProvenanceError("--after-report requires record-progress")
        if args.operation != "run":
            if args.manifest is None:
                raise ProvenanceError(f"{args.operation} requires a publication manifest")
            identity = capture_identity(os.environ)
            expected_path, _ = manifest_paths(identity)
            if args.manifest.resolve() != expected_path.resolve():
                raise ProvenanceError("publication manifest does not belong to this snapshot family")
            require_identity(args.manifest, identity)
            if args.operation == "require-fresh":
                require_fresh_matrix(args.manifest)
            elif args.operation == "record-progress":
                progress = parse_matrix_progress(sys.stdin.read())
                record_matrix_progress(args.manifest, progress, after_report=args.after_report)
                print(f"{progress.completed}:{progress.total}")
            return 0
        if args.manifest is not None:
            raise ProvenanceError("run does not accept an existing manifest override")
        return run_session(dict(os.environ))
    except (ProvenanceError, OSError, KeyError, ValueError) as error:
        print(f"publication-session: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
