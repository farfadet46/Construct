"""Référence JS : exécute runtime.js (version HTML) sur un projet avec un script de touches
et affiche les positions, pour comparer avec la version Rust.
usage : python3 jsref.py projet.json ticks "script" "Nom1|Nom2" """
import sys, json, asyncio
from playwright.async_api import async_playwright
RT = open('/home/claude/pf2/src/runtime.js').read()
async def main():
    proj, ticks, script, names = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
    P = open(proj).read()
    ev = {}
    for s in [x for x in script.split(',') if x]:
        t, r = s.split(':', 1); ev.setdefault(str(int(t)), []).append(r)
    async with async_playwright() as p:
        b = await p.chromium.launch(); pg = await b.new_page()
        await pg.set_content('<canvas id=c width=854 height=480></canvas><script>' + RT + '</script>')
        res = await pg.evaluate('''async ([P, ticks, ev, names]) => {
          Math.random = (function(){let s=42;return function(){s=(s*16807)%2147483647;return (s-1)/2147483646;}})();
          const rt = PF.Runtime(document.getElementById('c'), JSON.parse(P), {embedded:true}); rt.setPaused(true);
          for (let k=0;k<200 && !rt.layoutName();k++) await new Promise(r=>setTimeout(r,20));
          rt.setPaused(true);
          const key=(type,code)=>window.dispatchEvent(new KeyboardEvent(type,{code:code,key:code}));
          for (let t=0;t<ticks;t++){ (ev[t]||[]).forEach(e=>{ if(e[0]=='+')key('keydown',e.slice(1)); else if(e[0]=='-')key('keyup',e.slice(1)); }); rt.bench(1); await new Promise(r=>requestAnimationFrame(r)); }
          const d=rt.debug(); const out={layout:d.layout,globals:d.globals,count:d.count,errors:d.errors,pos:{}};
          names.split('|').filter(Boolean).forEach(n=>{out.pos[n]=d.find(n).filter(i=>!i.dead).map(i=>[+i.x.toFixed(2),+i.y.toFixed(2)]);});
          return out; }''', [P, ticks, ev, names])
        print(json.dumps(res, ensure_ascii=False)); await b.close()
asyncio.run(main())
