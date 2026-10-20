"""Stage a captured native-graph CLI without invoking a second Cargo graph."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def source(root):
    return {
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
        "patch_sha256": hashlib.sha256(subprocess.check_output(
            ["git", "diff", "--binary", "HEAD", "--"], cwd=root)).hexdigest(),
        "untracked": subprocess.check_output(
            ["git", "ls-files", "--others", "--exclude-standard", "-z"], cwd=root).decode(),
    }


def stage(root, target, output, owner_path, *, profile, oracle, share_runtime_images):
    root, target, output, owner_path = map(Path.resolve, map(Path, [root, target, output, owner_path]))
    if not share_runtime_images:
        raise ValueError("captured native worker requires --share-runtime-images")
    if target not in output.parents or output.exists() or target not in owner_path.parents:
        raise ValueError("use a fresh output and an actual build owner under the selected target directory")
    before = source(root)
    if before["patch_sha256"] != hashlib.sha256(b"").hexdigest() or before["untracked"]:
        raise ValueError("freeze a clean source checkout before staging")
    owner_hash = sha(owner_path)
    owned = json.loads(owner_path.read_text())
    if not all(owned.get(key) is True for key in ["complete", "passed", "source_unchanged"]):
        raise ValueError("native build owner is incomplete or failed")
    if owned.get("source_before") != before or owned.get("source_after") != before:
        raise ValueError("native build owner belongs to a different source")
    worker = owned["native_cli_worker"]
    artifact = worker["cargo_artifact"]
    if artifact["target"]["name"] != "scriptw" or artifact["target"]["kind"] != ["bin"] or artifact["profile"]["test"] is not False:
        raise ValueError("captured artifact is not the runnable native-graph CLI")
    expected_opt = {"debug": "0", "release": "3"}[profile]
    if artifact["profile"]["opt_level"] != expected_opt or ("spec-exec-oracle" in artifact["features"]) != oracle:
        raise ValueError("captured CLI profile/oracle features differ from the requested worker")
    captured = Path(worker["path"]).resolve()
    shared = Path(worker["link"]["shared_image"]).resolve()
    if target not in captured.parents or target / "shared-test-runtime-images" not in shared.parents:
        raise ValueError("captured worker and shared image must belong to the selected target")
    if sha(captured) != worker["sha256"] or captured.stat().st_size != worker["bytes"]:
        raise ValueError("captured CLI bytes changed after the native build")
    if sha(shared) != worker["link"]["shared_image_sha256"]:
        raise ValueError("captured shared runtime image changed after the native build")
    if worker["identity"]["source_revision"] != {"kind": "git-commit", "commit": before["head"]}:
        raise ValueError("captured compiler identity belongs to a different source")
    output.mkdir(parents=True)
    receipt = {"complete": False, "source_before": before, "profile": profile,
               "oracle": oracle, "shared_runtime_images": True,
               "build_owner": {"path": str(owner_path), "sha256": owner_hash},
               "driver_sha256": sha(__file__), "worker_graph": worker["graph"], "link": worker["link"],
               "policy": "Copy the exact captured private native-graph CLI. No Cargo, ordinary-product replacement or test verdict. Its actual feature graph remains owned by the original build."}
    def seal():
        (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    seal()
    started = time.monotonic()
    image = output / "scriptw-worker"
    shutil.copy2(captured, image)
    if sha(image) != worker["sha256"]:
        raise ValueError("staged CLI copy differs from its captured image")
    command = [str(image), "--jobs", "1", "compiler-identity"]
    result = subprocess.run(command, cwd=root, capture_output=True, check=True)
    (output / "identity.stdout").write_bytes(result.stdout)
    (output / "identity.stderr").write_bytes(result.stderr)
    identity = json.loads(result.stdout)
    if identity != worker["identity"] or identity["executable_sha256"] != sha(image):
        raise ValueError("actual staged compiler identity differs from the native build")
    after = source(root)
    if after != before or sha(owner_path) != owner_hash or sha(captured) != worker["sha256"] or sha(shared) != worker["link"]["shared_image_sha256"]:
        raise ValueError("source or owned native evidence changed during staging")
    receipt.update(complete=True, source_after=after, source_unchanged=True, cargo_invocations=0,
                   seconds=time.monotonic() - started,
                   worker={"path": str(image), "bytes": image.stat().st_size, "sha256": sha(image), "identity": identity},
                   identity_command=command, identity_stdout_sha256=sha(output / "identity.stdout"),
                   identity_stderr_sha256=sha(output / "identity.stderr"))
    seal()
    return receipt
