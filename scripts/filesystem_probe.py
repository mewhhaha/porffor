"""Probe a physical filesystem without creating missing output directories."""

from pathlib import Path
import shutil


def probe_filesystem(path, *, require_directory=False):
    physical = Path(path).resolve(strict=require_directory)
    if require_directory and not physical.is_dir():
        raise ValueError("scratch must be an existing directory")
    existing = physical
    while not existing.exists():
        parent = existing.parent
        if parent == existing:
            raise OSError(f"no existing ancestor for {physical}")
        existing = parent
    # On POSIX shutil.disk_usage.free is statvfs.f_bavail * f_frsize: bytes
    # available to this user, excluding blocks reserved by the filesystem.
    return physical, existing, existing.stat().st_dev, shutil.disk_usage(existing).free
