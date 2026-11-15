from pathlib import Path
import sys,json,hashlib,datetime,re
root=Path(__file__).resolve().parent;step=sys.argv[1]
assert re.fullmatch(r'[A-Za-z0-9-]+',step)
pid=json.loads((root/'INITIAL-PID.json').read_text())['actor_pid'];rows=[]
paths=list(root.glob('*.weftforge*'));recovery=Path.home()/'Library/Application Support/WeftForge/Recovery'
if recovery.is_dir():paths+=list(recovery.glob(f'*-{pid}-*'))
for p in sorted(paths):
 if not p.is_file():continue
 raw=p.read_bytes();rev=None
 try:rev=json.loads(raw)['project']['revision']
 except Exception:pass
 rows.append({'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':oct(p.stat().st_mode&0o777),'revision':rev,'originating_step':step})
out={'step':step,'actor_pid':pid,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'files':rows}
with (root/f'sidecars-{step}.json').open('x') as f:json.dump(out,f,indent=2);f.write('\n')
print(json.dumps({'step':step,'files':[{'name':Path(row['path']).name,'revision':row['revision'],'bytes':row['bytes']} for row in rows]}))
