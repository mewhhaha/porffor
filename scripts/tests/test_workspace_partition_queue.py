"""Actual driver protocol with tiny fake libtests; no Cargo or JavaScript engine."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

DRIVER=Path(__file__).resolve().parents[1]/'workspace_test_partitions.py'
ROOT=DRIVER.parent.parent
FAKE=r'''#!/usr/bin/env python3
import json,os,sys,time
from pathlib import Path
all_names=['case::a','case::b','case::c','case::d']
names=[x for x in sys.argv[1:] if x in all_names] or all_names
if '--list' in sys.argv:
    for name in names: print(name+': test')
    raise SystemExit(0)
assert '--test-threads=1' in sys.argv and '--include-ignored' in sys.argv
assert len(os.sched_getaffinity(0))==1
journal=Path(os.environ['SCRIPTW_QUEUE_TEST_JOURNAL'])
def record(event,name):
    with journal.open('a') as f:
        f.write(json.dumps(dict(event=event,name=name,pid=os.getpid(),cpu=list(os.sched_getaffinity(0))[0],time=time.monotonic_ns()))+'\n')
failed=0
for name in names:
    record('start',name)
    time.sleep(4 if name=='case::a' else .05)
    failure=name=='case::b'
    print('test '+name+' ... '+('FAILED' if failure else 'ok'),flush=True)
    failed+=int(failure)
    record('end',name)
print('test result: '+('FAILED' if failed else 'ok')+'. '+str(len(names)-failed)+' passed; '+str(failed)+' failed; 0 ignored; 0 measured; 0 filtered out;',flush=True)
raise SystemExit(101 if failed else 0)
'''

class BoundedPartitionQueueTests(unittest.TestCase):
    @unittest.skipUnless(hasattr(os,'sched_getaffinity') and len(os.sched_getaffinity(0))>=2,'two admitted Linux CPUs')
    def test_actual_driver_replenishes_free_cpu_and_keeps_all_outcomes_after_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            tmp=Path(directory);exe=tmp/'fake-libtest';exe.write_text(FAKE);exe.chmod(0o755)
            # The fake libtest needs no memory budget. Keep this scheduling test
            # portable on uncapped Linux CI; real cloud admission is covered by
            # test_limited_verification, while the stub checks exact launch flags.
            scripts=tmp/'scripts';scripts.mkdir()
            for helper in (ROOT/'scripts').iterdir():
                if helper.name!='limited_verification.py':
                    (scripts/helper.name).symlink_to(helper.resolve(), target_is_directory=helper.is_dir())
            (scripts/'limited_verification.py').write_text(
                "import os,sys\n"
                "assert sys.argv[1:3]==['--cloud','--']\n"
                f"assert sys.argv[3]=={str(exe)!r}\n"
                "os.execv(sys.argv[3],sys.argv[3:])\n")
            inventory=tmp/'inventory.log';inventory.write_text(''.join('case::'+c+': test\n' for c in 'abcd'))
            journal=tmp/'events.jsonl';output=tmp/'proof'
            cpus=sorted(os.sched_getaffinity(0))[:2]
            command=[sys.executable,'-B',str(DRIVER),'--executable',str(exe),
                     '--inventory',str(inventory),'--output',str(output),'--cpus',*map(str,cpus),
                     '--include-ignored','--partition-count','4']
            result=subprocess.run(command,cwd=tmp,env=dict(os.environ,SCRIPTW_QUEUE_TEST_JOURNAL=str(journal)),capture_output=True,text=True,timeout=40)
            self.assertEqual(result.returncode,1,result.stdout+result.stderr)
            receipt=json.loads((output/'receipt.json').read_text())
            self.assertTrue(receipt['inventory_matches'])
            self.assertEqual(receipt['counts'],dict(passed=3,failed=1,ignored=0))
            self.assertEqual(receipt['partition_count'],4)
            self.assertEqual(receipt['maximum_native_processes'],2)
            self.assertEqual([r['state'] for r in receipt['partitions']],['passed','failed','passed','passed'])
            events=[json.loads(x) for x in journal.read_text().splitlines()]
            active={};peak=0
            for event in events:
                if event['event']=='start':
                    self.assertNotIn(event['cpu'],active)
                    active[event['cpu']]=event['pid'];peak=max(peak,len(active))
                else:
                    self.assertEqual(active.pop(event['cpu']),event['pid'])
            self.assertFalse(active);self.assertEqual(peak,2)
            starts={r['name']:r['time'] for r in events if r['event']=='start'}
            ends={r['name']:r['time'] for r in events if r['event']=='end'}
            self.assertEqual(set(starts),{'case::'+c for c in 'abcd'})
            self.assertLess(starts['case::d'],ends['case::a'],'CPU 1 must replenish while CPU 0 remains occupied')
            self.assertLess(ends['case::a'],starts['case::c'],'sequential chunks cannot share a live CPU')
            self.assertLess(ends['case::b'],starts['case::d'])

if __name__=='__main__':unittest.main()
