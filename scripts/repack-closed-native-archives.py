#!/usr/bin/env python3
"""Repack closed native outputs while proving exact source frames restorable.

One shared XZ window stores the decoded files. Before removing any old
archive, decode the entire new stream, check every original byte, and reproduce
each old compressed frame with the retained encoder and verify its exact hash.
Use --restore SOURCE_ARCHIVE --output NEW_PATH with --receipt NEW_RECEIPT to
recover one historical frame without restoring multi-gigabyte originals.
"""
import argparse,hashlib,json,os,shutil,subprocess,threading
from pathlib import Path

def ref(path):
 path=Path(path)
 with path.open('rb') as stream:digest=hashlib.file_digest(stream,'sha256').hexdigest()
 return dict(path=str(path),bytes=path.stat().st_size,sha256=digest)
def save(path,value):
 pending=path.with_suffix('.pending');pending.write_text(json.dumps(value,indent=2)+'\n');pending.replace(path)
def read(path):return json.loads(Path(path).read_text())
def identity(path):
 p=Path(path);s=p.stat();return (str(p.resolve()),s.st_dev,s.st_ino,s.st_mode,s.st_size,s.st_mtime_ns,s.st_ctime_ns)
def collect(value,paths):
 if isinstance(value,dict):
  if isinstance(value.get('path'),str) and Path(value['path']).is_file():paths.add(Path(value['path']).resolve())
  for item in value.values():collect(item,paths)
 elif isinstance(value,list):
  for item in value:collect(item,paths)
def inactive(paths):
 inodes={(p.stat().st_dev,p.stat().st_ino) for p in paths}
 for process in Path('/proc').iterdir():
  if not process.name.isdigit() or int(process.name)==os.getpid():continue
  try:
   for fd in (process/'fd').iterdir():
    try:s=fd.stat();assert (s.st_dev,s.st_ino) not in inodes,('active fd',str(fd))
    except (FileNotFoundError,PermissionError,ProcessLookupError):pass
   for line in (process/'maps').read_text().splitlines():
    fields=line.split(maxsplit=5)
    if len(fields)>=5:
     major,minor=fields[3].split(':');assert (os.makedev(int(major,16),int(minor,16)),int(fields[4])) not in inodes,('active map',str(process))
  except (FileNotFoundError,PermissionError,ProcessLookupError):pass
def exact_record(stream,row):
 descriptor=os.memfd_create('closed-native-restoration',os.MFD_CLOEXEC|os.MFD_ALLOW_SEALING)
 digest=hashlib.sha256();remaining=row['original']['bytes']
 try:
  with os.fdopen(os.dup(descriptor),'wb') as out:
   while remaining:
    chunk=stream.read(min(remaining,1024**2));assert chunk,'truncated shared record';digest.update(chunk);out.write(chunk);remaining-=len(chunk)
  assert digest.hexdigest()==row['original']['sha256'],'decoded original changed'
  import fcntl
  fcntl.fcntl(descriptor,1033,15);assert fcntl.fcntl(descriptor,1034)&15==15
  os.lseek(descriptor,0,os.SEEK_SET)
  return descriptor
 except BaseException:os.close(descriptor);raise
def frame(encoder,descriptor,out=None):
 child=subprocess.Popen([str(encoder),'-q','-T1','-3','--force','-c','/proc/self/fd/'+str(descriptor)],pass_fds=(descriptor,),stdout=subprocess.PIPE)
 digest=hashlib.sha256();size=0
 try:
  while chunk:=child.stdout.read(1024**2):
   digest.update(chunk);size+=len(chunk)
   if out is not None:out.write(chunk)
  assert child.wait()==0
 finally:child.stdout.close()
 return size,digest.hexdigest()
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--receipt',type=Path,required=True);parser.add_argument('--dest',type=Path);parser.add_argument('--protect',type=Path,action='append',default=[]);parser.add_argument('--max-output-bytes',type=int,default=200*1024**2);parser.add_argument('--restore');parser.add_argument('--output',type=Path)
 args=parser.parse_args();os.sched_setaffinity(0,{min(os.sched_getaffinity(0))})
 if args.restore:
  r=read(args.receipt);assert r['complete'] and r['passed'];assert ref(r['bundle']['path'])==r['bundle'] and ref(r['encoder']['path'])==r['encoder'] and ref(r['shared_codec']['path'])==r['shared_codec']
  assert args.output and not args.output.exists()
  row=next(x for x in r['records'] if x['archive']['path']==args.restore)
  child=subprocess.Popen([r['shared_codec']['path'],'-q','-d','-c',r['bundle']['path']],stdout=subprocess.PIPE)
  try:
   remaining=row['offset']
   while remaining:
    chunk=child.stdout.read(min(remaining,1024**2));assert chunk;remaining-=len(chunk)
   fd=exact_record(child.stdout,row)
   try:
    with args.output.open('xb') as out:size,digest=frame(r['encoder']['path'],fd,out)
    assert (size,digest)==(row['archive']['bytes'],row['archive']['sha256'])
   finally:os.close(fd)
  finally:child.stdout.close();child.terminate();child.wait()
  print(json.dumps(ref(args.output)));return
 assert args.dest and not args.dest.exists()
 prior=read(args.receipt);assert prior['complete'] and prior['passed'] and prior['active_inputs_excluded']
 rows=prior['files'];assert rows and all(x['decoded_bytes_verified'] and x['original_removed'] for x in rows)
 protected=set()
 for p in args.protect:collect(read(p),protected)
 archives=[Path(x['archive']['path']) for x in rows];assert len(archives)==len(set(archives))
 protected_inodes={(p.stat().st_dev,p.stat().st_ino) for p in protected}
 for row,path in zip(rows,archives):
  assert path.is_file() and not path.is_symlink() and path.resolve() not in protected and ref(path)==row['archive'];s=path.stat();assert (s.st_dev,s.st_ino) not in protected_inodes
 baseline={str(p):identity(p) for p in archives};inactive(archives)
 D=args.dest;D.mkdir();encoder=D/'zstd';shutil.copyfile(shutil.which('zstd'),encoder);encoder.chmod(0o500)
 shared_codec=D/'xz';shutil.copyfile(shutil.which('xz'),shared_codec);shared_codec.chmod(0o500)
 state=dict(schema='shared-native-history-window-v1',complete=False,passed=False,source_receipt=ref(args.receipt),driver=ref(__file__),encoder=ref(encoder),shared_codec=ref(shared_codec),encoder_version=subprocess.check_output([str(encoder),'--version'],text=True).strip(),protected_receipts=[ref(p) for p in args.protect],protected_paths=len(protected),all_source_archives_initially_inactive=True,records=[],source_archive_bytes=sum(x['archive']['bytes'] for x in rows),decoded_bytes=sum(x['original']['bytes'] for x in rows))
 receipt=D/'receipt.json';save(receipt,state);bundle=D/'decoded-files.xz';errors=[];sizes=[]
 compressor=subprocess.Popen([str(shared_codec),'-q','-T1','--lzma2=dict=256MiB,mode=fast,mf=hc3,nice=32','-c'],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
 def drain():
  try:
   size=0
   with bundle.open('xb') as out:
    while chunk:=compressor.stdout.read(1024**2):
     size+=len(chunk);assert size<=args.max_output_bytes,'shared candidate exceeds bounded storage allowance';out.write(chunk)
   sizes.append(size)
  except BaseException as error:errors.append(repr(error));compressor.terminate()
 drainer=threading.Thread(target=drain);drainer.start();offset=0
 try:
  for row in rows:
   decoder=subprocess.Popen([str(encoder),'-q','-d','-c',row['archive']['path']],stdout=subprocess.PIPE);digest=hashlib.sha256();size=0
   try:
    while chunk:=decoder.stdout.read(1024**2):digest.update(chunk);size+=len(chunk);compressor.stdin.write(chunk)
    assert decoder.wait()==0 and (size,digest.hexdigest())==(row['original']['bytes'],row['original']['sha256'])
   finally:
    decoder.stdout.close()
    if decoder.poll() is None:decoder.terminate();decoder.wait()
   state['records'].append(dict(**row,offset=offset,source_frame_reproduction_verified=False,source_archive_removed=False));offset+=size
  compressor.stdin.close();assert compressor.wait()==0;drainer.join();assert not errors,errors
  state['bundle']=ref(bundle);save(receipt,state)
  decoder=subprocess.Popen([str(shared_codec),'-q','-d','-c',str(bundle)],stdout=subprocess.PIPE)
  try:
   for row in state['records']:
    fd=exact_record(decoder.stdout,row)
    try:assert frame(encoder,fd)==(row['archive']['bytes'],row['archive']['sha256']),'original compressed frame is not byte-restorable'
    finally:os.close(fd)
    row['source_frame_reproduction_verified']=True;save(receipt,state)
   assert decoder.stdout.read(1)==b'' and decoder.wait()==0
  finally:
   decoder.stdout.close()
   if decoder.poll() is None:decoder.terminate();decoder.wait()
  assert state['source_archive_bytes']>state['bundle']['bytes']+state['encoder']['bytes']+state['shared_codec']['bytes']
  for p in archives:assert identity(p)==baseline[str(p)]
  for row in rows:assert ref(row['archive']['path'])==row['archive']
  inactive(archives);state['all_bytes_verified_before_any_unlink']=True;save(receipt,state)
  for row in state['records']:
   path=Path(row['archive']['path']);assert identity(path)==baseline[str(path)];path.unlink();row['source_archive_removed']=True;save(receipt,state)
  state.update(complete=True,passed=True,freed_bytes=state['source_archive_bytes']-state['bundle']['bytes']-state['encoder']['bytes']-state['shared_codec']['bytes']);save(receipt,state);print(json.dumps({k:state[k] for k in ['complete','passed','freed_bytes','source_archive_bytes','decoded_bytes']}))
 except BaseException as error:
  if compressor.poll() is None:compressor.terminate();compressor.wait()
  drainer.join();state.update(complete=True,passed=False,error=repr(error),compression_errors=errors);save(receipt,state)
  if not state.get('bundle') and bundle.exists():state['discarded_incomplete_generated_candidate']=ref(bundle);bundle.unlink();save(receipt,state)
  raise
if __name__=='__main__':main()
