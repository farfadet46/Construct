#!/bin/bash
# compare JS (référence) et Rust sur un projet : parity.sh projet.json ticks "script" "Noms"
cd "$(dirname "$0")/.."
J=$(python3 tools/jsref.py "$1" "$2" "$3" "$4")
R=$(PF_LCG=1 PF_RENDER=1 ./target/release/examples/play "$1" "$2" "$3" "" "$4")
python3 - "$J" "$R" <<'PY'
import json,sys
j=json.loads(sys.argv[1]); r=sys.argv[2].splitlines()
rl=r[0]; rg=r[1].strip(); rp={}
for l in r[2:]:
    if ' x=' in l:
        n,rest=l.split(' x='); x,y=rest.split(' y='); rp.setdefault(n,[]).append([round(float(x),2),round(float(y),2)])
jg=' '.join(f"{k}={v if not isinstance(v,float) or v!=int(v) else int(v)}" for k,v in j['globals'].items())
ok=True
def chk(a,b,what):
    global ok
    if a!=b: ok=False; print('  DIFF',what,'\n    js  :',a,'\n    rust:',b)
chk('layout='+j['layout'], rl.split(' tick')[0], 'layout')
chk(j['count'], int(rl.split('count=')[1].split()[0]), 'count')
chk(jg, rg, 'globals')
for n,v in j['pos'].items(): chk(v, rp.get(n,[]), 'pos '+n)
if j['errors']: print('  js errors', j['errors'])
print('  OK' if ok else '  ÉCART')
PY
