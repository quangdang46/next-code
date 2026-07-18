import subprocess, json
from pathlib import Path
base=Path("/Users/tranquangdang21/.herdr/worktrees/next-code")
out=[]
for name in ["orch-telemetry-purge-appcore","orch-telemetry-purge-tui","orch-telemetry-purge-cli-base"]:
    p=base/name
    out.append("==== "+name+" ====")
    st=subprocess.check_output(["/usr/bin/git","-C",str(p),"status","--short"], text=True)
    out.append(st if st.strip() else "(clean)")
    if "appcore" in name: roots=[p/"crates/next-code-app-core"]
    elif name.endswith("tui"): roots=[p/"crates/next-code-tui"]
    else: roots=[p/"src/cli", p/"crates/next-code-base/src/memory"]
    n=0; files=[]
    for r in roots:
      if not r.exists(): continue
      for f in r.rglob("*.rs"):
        t=f.read_text(errors="ignore"); c=t.count("crate::telemetry::")
        if c: files.append((str(f.relative_to(p)),c)); n+=c
    out.append("remaining="+str(n))
    for f,c in files: out.append("  %d %s"%(c,f))
for pane in ["w3R:p2","w3S:p2","w3T:p2"]:
    d=json.loads(subprocess.check_output(["herdr","pane","get",pane], text=True))["result"]["pane"]
    out.append("%s status=%s"%(pane,d.get("agent_status")))
Path("/Users/tranquangdang21/Projects/next-code/data/orchestrator/evidence/progress.txt").write_text("\n".join(out)+"\n")
print("\n".join(out))
