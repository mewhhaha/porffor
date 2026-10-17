#!/usr/bin/env python3
"""Verify every raw evidence asset in a shared XZ bundle using the standard library."""
import argparse
import base64
import hashlib
import io
import json
import lzma
from pathlib import Path


def verify(path):
    document = json.loads(Path(path).read_text())
    bundle = document['raw_asset_bundle']
    assert bundle['encoding'] == 'base64+xz'
    compressed = base64.b64decode(bundle['data'], validate=True)
    assert len(compressed) == bundle['compressed_bytes']
    assert hashlib.sha256(compressed).hexdigest() == bundle['compressed_sha256']
    whole = hashlib.sha256()
    offset = 0
    with lzma.LZMAFile(io.BytesIO(compressed), 'rb') as stream:
        for label, asset in document['assets'].items():
            assert asset['encoding'] == 'raw-bundle-slice-v1'
            assert asset['offset'] == offset and type(asset['bytes']) is int and asset['bytes'] >= 0
            digest = hashlib.sha256()
            remaining = asset['bytes']
            while remaining:
                raw = stream.read(min(remaining, 65536))
                assert raw, ('truncated asset', label)
                digest.update(raw)
                whole.update(raw)
                remaining -= len(raw)
            assert digest.hexdigest() == asset['sha256'], ('changed asset', label)
            offset += asset['bytes']
        assert stream.read(1) == b'', 'Unexpected trailing asset bytes'
    assert offset == bundle['raw_bytes']
    assert whole.hexdigest() == bundle['raw_sha256']
    assert len(document['assets']) == bundle['asset_count']
    return dict(assets=len(document['assets']), raw_bytes=offset,
                compressed_bytes=len(compressed), all_raw_asset_checksums_verified=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checkpoint', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.checkpoint), sort_keys=True))
