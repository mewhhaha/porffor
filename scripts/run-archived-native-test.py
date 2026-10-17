#!/usr/bin/env python3
"""Run a byte-verified archived ELF without restoring a disk copy (Linux).

The decoded image occupies inherited cgroup memory. Use the normal verification
limiter. Test binaries which spawn themselves by pathname need a regular file.
"""
import argparse
import fcntl
import hashlib
import os
from pathlib import Path
import re
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive', type=Path, required=True)
    parser.add_argument('--sha256', required=True)
    parser.add_argument('--bytes', type=int, required=True)
    parser.add_argument('arguments', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if sys.platform != 'linux' or not hasattr(os, 'memfd_create'):
        parser.error('sealed-memory execution requires Linux memfd support')
    if not re.fullmatch('[0-9a-f]{64}', args.sha256) or args.bytes <= 0:
        parser.error('provide the expected SHA-256 and positive decoded byte count')
    arguments = args.arguments
    if arguments[:1] == ['--']:
        arguments = arguments[1:]
    fd = os.memfd_create('lila-native-test', os.MFD_ALLOW_SEALING | os.MFD_CLOEXEC)
    child = None
    try:
        child = subprocess.Popen(['zstd', '-q', '-d', '-c', str(args.archive)], stdout=subprocess.PIPE)
        digest = hashlib.sha256()
        size = 0
        magic = b''
        while chunk := child.stdout.read(1024 * 1024):
            if not magic:
                magic = chunk[:4]
            size += len(chunk)
            if size > args.bytes:
                raise ValueError('decoded image exceeds the expected byte count')
            digest.update(chunk)
            view = memoryview(chunk)
            while view:
                view = view[os.write(fd, view):]
        if child.wait() != 0:
            raise ValueError('archive decompression failed')
        if size != args.bytes or digest.hexdigest() != args.sha256:
            raise ValueError('decoded image SHA-256 or byte count mismatch')
        if magic != b'\x7fELF':
            raise ValueError('the decoded native test must be an ELF executable')
        os.fchmod(fd, 0o500)
        # Linux UAPI values also support Python builds lacking these constants.
        add_seals = getattr(fcntl, 'F_ADD_SEALS', 1033)
        get_seals = getattr(fcntl, 'F_GET_SEALS', 1034)
        seals = 0x0F  # SEAL_SEAL | SEAL_SHRINK | SEAL_GROW | SEAL_WRITE
        fcntl.fcntl(fd, add_seals, seals)
        if fcntl.fcntl(fd, get_seals) & seals != seals:
            raise ValueError('decoded image sealing failed')
        executable = f'/proc/self/fd/{fd}'
        return subprocess.run([executable, *arguments], pass_fds=(fd,)).returncode
    except (OSError, ValueError) as error:
        print(f'archived native test: {error}', file=sys.stderr)
        return 2
    finally:
        if child is not None and child.poll() is None:
            child.kill()
            child.wait()
        os.close(fd)


if __name__ == '__main__':
    sys.exit(main())
