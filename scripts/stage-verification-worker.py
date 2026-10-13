#!/usr/bin/env python3
"""Stage a smaller private CLI worker and preserve ordinary Cargo output."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

root = Path(__file__).resolve().parent.parent
assert Path.cwd().resolve() == root, 'Run from the workspace root'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--profile', choices=('debug', 'release'), default='debug')
parser.add_argument('--oracle', action='store_true', help='Explicitly enable the developer spec-exec oracle')
parser.add_argument('--share-runtime-images', action='store_true', help='Relink only the private CLI leaf against verified immutable Linux runtime images')
parser.add_argument('--from-native-build-owner', type=Path,
                    help='Copy a source-bound private CLI already captured by a complete native build; run no Cargo')
args = parser.parse_args()
out = args.output.resolve()
if args.from_native_build_owner:
    import native_verification_worker
    target = Path(os.environ.get('CARGO_TARGET_DIR', root / 'target')).resolve()
    try:
        staged = native_verification_worker.stage(root, target, out, args.from_native_build_owner,
                    profile=args.profile, oracle=args.oracle, share_runtime_images=args.share_runtime_images)
    except (ValueError, KeyError, OSError, subprocess.SubprocessError) as error:
        parser.error(str(error))
    print('staged captured native-graph worker:', staged['worker']['path'])
    sys.exit(0)
assert root / 'target' in out.parents, 'Evidence and worker must live under target/'
assert not out.exists(), 'Use a fresh output directory'
assert not os.environ.get('CARGO_TARGET_DIR'), 'This tool selects the ordinary workspace target directory'


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def source():
    files = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '-z']).split(b'\0')
    return {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
            'tracked_patch_sha256': hashlib.sha256(subprocess.check_output(['git', 'diff', '--binary', 'HEAD', '--'])).hexdigest(),
            'untracked': {os.fsdecode(p): sha(root / os.fsdecode(p)) for p in files if p}}


for path in Path('/proc').glob('[0-9]*/exe'):
    if int(path.parent.name) == os.getpid():
        continue
    try:
        executable = Path(os.readlink(path))
    except OSError:
        continue
    assert executable.name not in ('cargo', 'rustc', 'rustdoc') and root / 'target' not in executable.parents, ('Wait for verification/builds to finish', path, executable)
assert shutil.disk_usage(root / 'target').free > 2 * 1024**3, 'Reserve two GiB for a real Cargo link'
before = source()
out.mkdir(parents=True)
receipt = {'complete': False, 'source_before': before, 'profile': args.profile,
           'oracle': args.oracle, 'shared_runtime_images': args.share_runtime_images,
           'driver_sha256': sha(__file__), 'steps': [],
           'policy': 'Private verification worker only. Actual Cargo-selected CLI, exact loaded-image identities, unchanged allocated sections, verified shared arrays, normal product restoration and frozen source. No source/profile/freshness/deadline/verdict changes.'}
env = dict(os.environ)
assert not env.get('RUSTC_WRAPPER'), 'Cannot replace an existing compiler wrapper'
env.update(RUSTC_WRAPPER=str(root / 'scripts/test-shared-runtime-images/test-rustc'),
           LILA_TEST_NO_INCREMENTAL='1', LILA_VERIFICATION_WORKER_PROFILE=args.profile,
           LILA_VERIFICATION_WORKER_LINK_RECEIPT=str(out / 'link.json'))
wrapper = root / 'scripts/test-shared-runtime-images/worker-cc'
if args.share_runtime_images:
    assert os.access(wrapper, os.X_OK), 'Private linker must be executable'
    receipt['linker_sha256'] = sha(wrapper)
    # Cargo does not watch an external linker's bytes. A stable metadata tag
    # invalidates only this leaf when the linker or shared-image producer changes.
    leaf_tag = hashlib.sha256((sha(wrapper) + sha(wrapper.with_name('shared_image.py'))).encode()).hexdigest()
sys.path.insert(0, str(wrapper.parent))
from strip_debug import strip_debug
receipt['strip_helper_sha256'] = sha(wrapper.with_name('strip_debug.py'))


def seal():
    receipt['source_after'] = source()
    receipt['source_unchanged'] = receipt['source_after'] == before
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    assert receipt['source_unchanged'], 'Source changed during staging'


def run(label, command):
    stdout, stderr = out / (label + '.stdout'), out / (label + '.stderr')
    started = time.monotonic()
    with stdout.open('xb') as a, stderr.open('xb') as b:
        code = subprocess.run(command, stdout=a, stderr=b, env=env).returncode
    row = {'label': label, 'command': command, 'exit_code': code,
           'seconds': round(time.monotonic() - started, 6),
           'stdout': str(stdout), 'stdout_sha256': sha(stdout),
           'stderr': str(stderr), 'stderr_sha256': sha(stderr)}
    receipt['steps'].append(row)
    seal()
    print(label, code, row['seconds'], flush=True)
    assert code == 0, 'See the retained stdout/stderr and receipt'
    return row


def selected(row):
    records = [json.loads(line) for line in Path(row['stdout']).read_text().splitlines()]
    assert any(m.get('reason') == 'build-finished' and m['success'] for m in records)
    candidates = [m for m in records if m.get('reason') == 'compiler-artifact'
                  and m['target']['name'] == 'lila' and m['target']['kind'] == ['bin']
                  and not m['profile']['test']]
    assert len(candidates) == 1
    return candidates[0]


flags = (['--release'] if args.profile == 'release' else [])
if args.oracle:
    flags += ['--features', 'spec-exec-oracle']
normal = ['cargo', 'build', '--locked', '--offline', '-p', 'lila-cli', '--bin', 'lila', *flags,
          '--message-format=json-render-diagnostics']
ordinary = selected(run('ordinary-build', normal))
canonical = root / 'target' / args.profile / 'lila'
assert Path(ordinary['executable']) == canonical
original_hash = sha(canonical)
receipt['ordinary_before'] = {'cargo_artifact': ordinary, 'sha256': original_hash,
                              'bytes': canonical.stat().st_size}
ordinary_identity = json.loads(Path(run('ordinary-identity', [str(canonical), '--jobs', '1', 'compiler-identity'])['stdout']).read_text())
assert ordinary_identity['executable_sha256'] == original_hash
assert ordinary_identity['source_revision'] == {'kind': 'git-commit', 'commit': before['head']}
image = out / 'lila-worker'
try:
    if args.share_runtime_images:
        private = selected(run('private-link', ['cargo', 'rustc', '--locked', '--offline', '-p', 'lila-cli', '--bin', 'lila', *flags,
                               '--message-format=json-render-diagnostics', '--', '-C', 'linker=' + str(wrapper), '-C', 'metadata=' + leaf_tag]))
        if not (out / 'link.json').exists():
            assert private['fresh'], 'A new link must produce its own receipt'
            cached = root / 'target/shared-test-runtime-images/worker-links' / (sha(private['executable']) + '.json')
            link = json.loads(cached.read_text())
            assert link['wrapper_sha256'] == sha(wrapper)
            assert link['identity']['builder_sha256'] == sha(wrapper.with_name('shared_image.py'))
            assert sha(link['output']) == sha(private['executable']) == link['executable_sha256']
            assert sha(link['shared_image']) == link['shared_image_sha256']
            (out / 'link.json').write_text(json.dumps(link, indent=2) + '\n')
            receipt['cached_link_receipt'] = {'path': str(cached), 'sha256': sha(cached)}
        link = json.loads((out / 'link.json').read_text())
        assert sha(private['executable']) == link['executable_sha256']
        receipt.update(private_cargo_artifact=private, link=link)
    with (canonical.parent / '.cargo-lock').open('a+b') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        shutil.copy2(canonical, image)
        assert sha(image) == sha(canonical)
finally:
    if args.share_runtime_images:
        selected(run('restore-ordinary-build', normal))
    receipt['ordinary_after'] = {'sha256': sha(canonical), 'bytes': canonical.stat().st_size}
    seal()
    assert sha(canonical) == original_hash, 'Ordinary Cargo product must be restored byte-for-byte'

receipt['strip'] = strip_debug(image)
identity = json.loads(Path(run('worker-identity', [str(image), '--jobs', '1', 'compiler-identity'])['stdout']).read_text())
assert identity['executable_sha256'] == sha(image)
assert {k: v for k, v in identity.items() if k != 'executable_sha256'} == {k: v for k, v in ordinary_identity.items() if k != 'executable_sha256'}
if args.share_runtime_images:
    assert sha(receipt['link']['shared_image']) == receipt['link']['shared_image_sha256']
receipt.update(complete=True, worker={'path': str(image), 'sha256': sha(image), 'bytes': image.stat().st_size, 'identity': identity})
seal()
print('staged verification worker:', image, flush=True)
