"""Optional libtest debug stripping, preserving every allocated ELF section."""
import hashlib
import mmap
import struct
import subprocess
import time
from pathlib import Path


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def allocated_sections(path):
    with path.open("rb") as stream, mmap.mmap(stream.fileno(), 0, access=mmap.ACCESS_READ) as data:
        header = struct.unpack_from("<16sHHIQQQIHHHHHH", data)
        assert header[0][:6] == b"\x7fELF\x02\x01" and header[2] == 62
        sections = [struct.unpack_from("<IIQQQQIIQQ", data, header[6] + index * header[11])
                    for index in range(header[12])]
        strings = sections[header[13]]
        result = {}
        for section in sections:
            if not section[2] & 2:
                continue
            start = strings[4] + section[0]
            name = data[start:data.find(b"\0", start)].decode()
            assert name not in result
            result[name] = {
                "kind": section[1], "flags": section[2], "address": section[3],
                "bytes": section[5], "alignment": section[8],
                "sha256": None if section[1] == 8 else hashlib.sha256(
                    memoryview(data)[section[4]:section[4] + section[5]]).hexdigest(),
            }
        return result


def strip_debug(output):
    before = allocated_sections(output)
    original_sha256 = digest(output)
    original_bytes = output.stat().st_size
    tool = Path("/usr/bin/objcopy")
    command = [str(tool), "--strip-debug", str(output)]
    start = time.monotonic()
    subprocess.run(command, check=True)
    assert allocated_sections(output) == before, "debug stripping changed allocated ELF data"
    return {
        "command": command, "tool_sha256": digest(tool),
        "original_sha256": original_sha256, "original_bytes": original_bytes,
        "stripped_sha256": digest(output), "stripped_bytes": output.stat().st_size,
        "seconds": round(time.monotonic() - start, 3),
        "allocated_sections": before,
        "diagnostics": "DWARF source lines removed; global symbols and compiled panic locations retained",
    }
