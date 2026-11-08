from pathlib import Path
import hashlib,json,re
out=Path(__file__).resolve().parent
log=(out/'core-final.txt').read_text()
sections=re.split(r'(?=     Running unittests src/lib.rs)',log)[1:]
names=['default','experimental-writes','controlled-writes','application-services','all-features']
assert len(sections)==len(names)
summary={}
for name,section in zip(names,sections):
    rows=re.findall(r'^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored',section,re.M)
    assert rows and all(row[0]=='ok' and int(row[2])==0 for row in rows)
    summary[name]={key:sum(int(row[index]) for row in rows) for key,index in [('passed',1),('failed',2),('ignored',3)]}
lab=json.loads((out/'core/lab-results.json').read_text())
assert lab['passed']==len(lab['cases'])==24 and lab['skipped']==0
summary['lab']={'passed':lab['passed'],'failed':0,'skipped':lab['skipped']}
assert json.loads((out/'core/environment.json').read_text())['rust_test_threads']=='4'
for path,digest in json.loads((out/'source-files-final.json').read_text()).items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest, path
(out/'matrix-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
