#!/usr/bin/env python3
"""Check retained diagnostic evidence offline; does not rerun native code."""
import base64
import gzip
import hashlib
import json
from pathlib import Path
import struct
import sys

HEAD = 'eb1ca63c9dbf09148703ccfbd0ca35d9629753e7'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def embedded(row):
    raw = row['raw'].encode()
    assert len(raw) == row['bytes'] and sha(raw) == row['sha256']
    return raw


def run(row, expected):
    owner = json.loads(embedded(row['owner']))
    assert owner['complete'] and owner['exit_code'] == expected
    assert owner['head'] == HEAD and owner['source_unchanged']
    assert owner['source_before'] == owner['source_after']
    assert owner['native_acceptance'] is False
    for stream in ('stdout', 'stderr'):
        archive = base64.b64decode(row[stream]['gzip_base64'], validate=True)
        raw = embedded(row[stream])
        assert gzip.decompress(archive) == raw
        assert sha(archive) == owner[stream]['archive_sha256']
        assert sha(raw) == owner[stream]['raw_sha256']
        assert len(raw) == owner[stream]['raw_bytes']
    return owner


def verify(proof):
    assert proof['schema'] == 'dynamic-aot-same-store-diagnostic-eb1ca63c9-v1'
    assert proof['source_head'] == HEAD and proof['complete']
    assert proof['product_dynamic_source_support'] is False
    assert proof['conformance_or_release_acceptance'] is False
    source = embedded(proof['probe']).decode()
    assert sha(source.encode()) == '9a7b3a75e7607e9f9f4a89f8e63520e41aac0b25e73d66f275b3a1b84a014a41'
    for fragment in ('caller.gc(None)?', 'StructType::eq', 'linker.instantiate(&mut caller',
                     'Module::new(caller.engine(), &artifact.bytes)',
                     'unimplemented proof host callback executed',
                     'NO-GO: existing embedded native R not admitted'):
        assert fragment in source
    embedded(proof['compile_driver'])
    dependencies = json.loads(embedded(proof['dependencies']))
    copies = json.loads(embedded(proof['source_copies_manifest']))
    assert copies['source_head'] == HEAD
    assert dependencies['source_root'] == '/workspace/porffor-next'
    for row in copies['files']:
        pair = proof['selected_sources'][row['source']]
        original = embedded(pair['original'])
        copied = embedded(pair['copy'])
        assert sha(original) == row['source_sha256']
        assert sha(copied) == row['copy_sha256']
        production = original
        for declaration in row['excluded_test_modules']:
            assert production.count(declaration.encode()) == 1
            production = production.replace(declaration.encode(), b'', 1)
        assert production == copied
    producer = json.loads(embedded(proof['ordinary_build_receipt']))
    assert producer['complete'] and producer['passed'] and not producer['oracle']
    assert producer['head'] == HEAD and producer['source_unchanged']
    assert dict(producer['actual_engine_producer']['env']) == dependencies['compile_environment']
    events = producer['compiler_artifacts']
    for name, dependency in dependencies['externs'].items():
        assert dependency['artifact_event_matches_receipt']
        assert any(dependency['rlib'] in event['filenames'] for event in events), name
        actual = proof['extern_observations'][name]
        assert actual['path'] == dependency['rlib'] and actual['bytes'] == dependency['bytes']
        assert actual in producer['dependency_artifacts']
    compile_owner = run(proof['runs']['compile'], 0)
    transport = run(proof['runs']['transport'], 0)
    conflict = run(proof['runs']['conflict'], 101)
    for key, owner in [('transport', transport), ('conflict', conflict)]:
        command = owner['command']
        assert command[-4:] == ['--exact', key, '--nocapture', '--test-threads=1']
        assert command[-5] == proof['binary']['path']
        assert proof['runs'][key]['stdout']['raw'].startswith('\nrunning 1 test\n')
        stderr = proof['runs'][key]['stderr']['raw']
        assert 'runtime-native-bundle hit: 80076128 bytes' in stderr
        assert 'runtime-build-package hit: 40315526 bytes' in stderr
    stdout = proof['runs']['transport']['stdout']['raw']
    for marker in ['baseline-old-P=17', 'reentrant-typed-GC-call=42', 'outer-P-resumed=17',
                   'old-P-hook-context=17', '1 passed; 0 failed; 0 ignored;']:
        assert marker in stdout
    conflict_out = proof['runs']['conflict']['stdout']['raw']
    for marker in ['baseline-old-P=17', 'reentrant-typed-GC-call=42', 'outer-P-resumed=17',
                   '0 passed; 1 failed; 0 ignored;']:
        assert marker in conflict_out
    assert 'old-P-hook-context=17' not in conflict_out
    bits = struct.unpack('<q', struct.pack('<d', 23.0))[0]
    assert f'expected Return(17), got [I32(3), I64({bits}), AnyRef(None), I32(2), I32(0)]' in proof['runs']['conflict']['stderr']['raw']
    assert proof['observations']['hook_conflict_after'] == 23
    assert proof['observations']['hook_conflict_expected'] == 17
    assert proof['observations']['transport_passed'] is True
    assert proof['observations']['hook_ownership_passed'] is False
    for prior in proof['prior_runs']:
        run(prior, prior['expected_exit_code'])
    return {'evidence_verified': True, 'native_reexecuted': False,
            'transport_passed': True, 'hook_ownership_passed': False,
            'conformance_or_release_acceptance': False,
            'compile_seconds': compile_owner['seconds']}


if __name__ == '__main__':
    path = Path(sys.argv[1]) if len(sys.argv) == 2 else Path(__file__).with_name('dynamic-aot-same-store-eb1ca63c9-20261015.json')
    print(json.dumps(verify(json.loads(path.read_text())), sort_keys=True))
