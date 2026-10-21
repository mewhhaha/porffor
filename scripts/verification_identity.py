"""Source and file identities shared by verification drivers.

Keep receipt schemas in their owners; these primitives bind the same Git bytes
regardless of the caller's working directory. Ignored build outputs are excluded.
"""

import hashlib
import os
from pathlib import Path
import subprocess


def file_sha256(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def git_source_bytes(root):
    def git(*arguments):
        return subprocess.check_output(["git", *arguments], cwd=root)

    return (
        git("rev-parse", "HEAD").decode().strip(),
        hashlib.sha256(git("diff", "--binary", "HEAD", "--")).hexdigest(),
        git("ls-files", "--others", "--exclude-standard", "-z"),
    )


def source_map(root):
    head, patch, untracked = git_source_bytes(root)
    return {
        "head": head,
        "tracked_patch_sha256": patch,
        "untracked": {
            os.fsdecode(path): file_sha256(Path(root) / os.fsdecode(path))
            for path in untracked.split(b"\0")
            if path
        },
    }
