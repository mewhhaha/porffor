"""Share the verified immutable runtime-image object for opt-in Linux tests."""
import ctypes
import fcntl
import hashlib
import json
import mmap
import os
import platform
import struct
import subprocess
from pathlib import Path

SYMBOLS = {"SCRIPTW_RUNTIME_" + prefix + suffix
           for prefix in ("", "PORTABLE_")
           for suffix in ("PACKAGE", "NATIVE", "MANIFEST")}


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def archive_symbols(archive):
    result = {}
    with archive.open("rb") as stream, mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as data:
        assert data[:8] == b"!<arch>\n"
        position = 8
        while position < len(data):
            member_size = int(data[position + 48:position + 58])
            start = position + 60
            position = start + member_size + (member_size % 2)
            if data[start:start + 4] != b"\x7fELF":
                continue
            header = struct.unpack_from("<16sHHIQQQIHHHHHH", data, start)
            assert header[0][4:6] == b"\x02\x01" and header[1] == 1 and header[2] == 62
            sections = [struct.unpack_from("<IIQQQQIIQQ", data, start + header[6] + i * header[11]) for i in range(header[12])]
            for section in sections:
                if section[1] != 2:
                    continue
                strings = sections[section[6]]
                strings_start = start + strings[4]
                for offset in range(0, section[5], section[9]):
                    name_offset, info, other, index, value, size = struct.unpack_from("<IBBHQQ", data, start + section[4] + offset)
                    name_start = strings_start + name_offset
                    name = data[name_start:data.find(b"\0", name_start)].decode()
                    if name not in SYMBOLS:
                        continue
                    assert info & 15 == 1 and info >> 4 == 1 and name not in result
                    owner = sections[index]
                    assert owner[2] & 2 and not owner[2] & 1
                    begin = start + owner[4] + value
                    assert begin + size <= start + member_size
                    result[name] = {"bytes": size, "sha256": hashlib.sha256(memoryview(data)[begin:begin + size]).hexdigest()}
    assert set(result) == SYMBOLS
    return result


def shared_image(archive, cache):
    assert platform.system() == "Linux" and platform.machine() == "x86_64"
    sysroot = Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
    linker = sysroot / "lib/rustlib/x86_64-unknown-linux-gnu/bin/rust-lld"
    identity = {"archive_sha256": digest(archive), "linker_sha256": digest(linker), "builder_sha256": digest(Path(__file__)), "target": "x86_64-unknown-linux-gnu", "schema": 2}
    key = hashlib.sha256(json.dumps(identity, sort_keys=True).encode()).hexdigest()
    cache.mkdir(parents=True, exist_ok=True)
    output = cache / ("libscriptw_runtime_images_" + key + ".so")
    receipt_path = output.with_suffix(".json")
    with (cache / (key + ".lock")).open("a+b") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        if output.exists():
            try:
                receipt = json.loads(receipt_path.read_text())
                if receipt["identity"] == identity and receipt["shared_sha256"] == digest(output):
                    return output, receipt
            except (OSError, ValueError, KeyError):
                pass
            # A crashed producer may leave only half the pair. No new link can
            # admit it; reconstruct and verify both files under the cache lock.
        expected = archive_symbols(archive)
        temporary = output.with_suffix(".so.tmp")
        command = [str(linker), "-flavor", "gnu", "-shared", "--whole-archive", str(archive), "--no-whole-archive", "-soname", output.name, "-o", str(temporary)]
        subprocess.run(command, check=True)
        library = ctypes.CDLL(str(temporary))
        observed = {}
        for name, row in expected.items():
            array = (ctypes.c_ubyte * row["bytes"]).in_dll(library, name)
            observed[name] = {"bytes": len(array), "sha256": hashlib.sha256(memoryview(array)).hexdigest()}
        assert observed == expected
        temporary.chmod(0o444)
        temporary.replace(output)
        receipt = {"identity": identity, "archive": str(archive), "command": command, "symbols": observed, "shared_sha256": digest(output), "shared_bytes": output.stat().st_size}
        temporary_receipt = receipt_path.with_suffix('.json.tmp')
        temporary_receipt.write_text(json.dumps(receipt, indent=2) + "\n")
        temporary_receipt.replace(receipt_path)
        return output, receipt
