"""Nonce binding, failed protocols and real-child checked-result arrival."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from boundary_process import decode_ready_marker, measure_ready_process, validate_ready

OUTPUT = '42\n'
SHA = hashlib.sha256(OUTPUT.encode()).hexdigest()


class ReadyProtocolTests(unittest.TestCase):
    def test_marker_is_exactly_one_bounded_nonce_bound_line(self):
        nonce = 'a' * 32
        line = f'SWREADY1 {nonce} {SHA}\n'.encode()
        self.assertEqual(decode_ready_marker(line, nonce), dict(nonce=nonce, stdout_sha256=SHA))
        for raw in (b'', line[:-1], line + line, line + b'x', line.replace(b'aa', b'bb', 1),
                    line.replace(SHA.encode(), b'z' * 64)):
            with self.subTest(raw=raw[:30]), self.assertRaises(ValueError):
                decode_ready_marker(raw, nonce)

    def test_final_proof_and_exact_output_hash_bind_the_ready_event(self):
        accounting = dict(result_ready=dict(status='valid', nonce='a' * 32, stdout_sha256=SHA),
            result_ready_elapsed_ns=10, elapsed_ns=20, exit_code=0, timed_out=False)
        child = dict(result_ready=dict(nonce='a' * 32, stdout_sha256=SHA))
        validate_ready(accounting, child, SHA)
        for changed in [dict(result_ready=None), dict(result_ready=dict(nonce='b' * 32, stdout_sha256=SHA)),
                        dict(result_ready=dict(nonce='a' * 32, stdout_sha256='0' * 64))]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                validate_ready(accounting, changed, SHA)
        for changed in [dict(result_ready=dict(status='missing')), dict(exit_code=1),
                        dict(timed_out=True), dict(result_ready_elapsed_ns=21)]:
            with self.subTest(changed=changed), self.assertRaises(ValueError):
                validate_ready(dict(accounting, **changed), child, SHA)

    def test_real_child_ready_precedes_post_result_work_and_failed_protocols_never_score(self):
        time_binary = os.environ.get('SCRIPTW_GNU_TIME') or shutil.which('gtime') or shutil.which('time')
        if time_binary is None and Path('/tmp/scriptw-performance-tools/bin/time').is_file():
            time_binary = '/tmp/scriptw-performance-tools/bin/time'
        self.assertIsNotNone(time_binary, 'GNU time is required for actual-child ready/RSS controls')
        program = r'''
import json,os,sys,time
mode=sys.argv[1]
nonce=os.environ['SCRIPTW_BOUNDARY_READY_NONCE']
fd=int(os.environ['SCRIPTW_BOUNDARY_READY_FD'])
sha=sys.argv[2]
proof=dict(nonce=nonce,stdout_sha256=sha)
line=f'SWREADY1 {nonce} {sha}'.encode()+bytes([10])
if mode=='valid':
 os.write(fd,line)
 time.sleep(.2)
elif mode=='duplicate':os.write(fd,line+line)
elif mode=='truncated':os.write(fd,line[:-1])
elif mode=='wrong_nonce':os.write(fd,line.replace(nonce.encode(),b'f'*32))
elif mode=='wrong_hash':
 proof['stdout_sha256']='0'*64
 os.write(fd,line.replace(sha.encode(),b'0'*64))
elif mode=='timeout':
 os.write(fd,line)
 time.sleep(10)
print(json.dumps(dict(result_ready=proof)))
'''
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for mode in ('valid', 'missing', 'duplicate', 'truncated', 'wrong_nonce', 'wrong_hash', 'timeout'):
                with self.subTest(mode=mode):
                    stdout, stderr = root / (mode + '.stdout'), root / (mode + '.stderr')
                    with stdout.open('wb') as out, stderr.open('wb') as err:
                        accounting = measure_ready_process([sys.executable, '-c', program, mode, SHA],
                            cwd=root, env=os.environ, stdout=out, stderr=err,
                            timeout=.2 if mode == 'timeout' else 5, time_binary=time_binary)
                    if mode == 'valid':
                        child = json.loads(stdout.read_text())
                        validate_ready(accounting, child, SHA)
                        self.assertGreater(accounting['elapsed_ns'] - accounting['result_ready_elapsed_ns'], 50_000_000)
                        self.assertGreater(accounting['peak_rss_bytes'], 0)
                        self.assertEqual(accounting['memory_scope'], 'runtime-child-lifetime-high-water-rss;excludes-python-launcher')
                    else:
                        child = json.loads(stdout.read_text()) if stdout.read_text() else dict(
                            result_ready=dict(nonce=accounting['result_ready']['nonce'], stdout_sha256=SHA))
                        with self.assertRaises(ValueError):
                            validate_ready(accounting, child, SHA)
                        if mode == 'timeout': self.assertTrue(accounting['timed_out'])
