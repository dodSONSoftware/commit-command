"""Independent read-only Git/journal/preservation oracle for marked crash fixtures."""
from pathlib import Path
import hashlib,json,sqlite3,subprocess,sys
folder=Path(__file__).resolve().parent
which,stage=sys.argv[1:]
f=json.loads((folder/(which+'-fixture.json')).read_text());workspace=Path(f['workspace_root']);repo=workspace/'engine'
assert (Path(f['root'])/'.gitfleet-m3-fixture').is_file() and (repo/'.gitfleet-m2-fixture').is_file()
def git(*args):return subprocess.check_output(['git','--no-optional-locks','-c','core.fsmonitor=false','-C',str(repo),*args])
current={str(p.relative_to(workspace)):{'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'mode':p.stat().st_mode&0o777} for p in sorted(workspace.rglob('*')) if p.is_file() and not p.is_symlink()}
baseline=json.loads((folder/(which+'-baseline.json')).read_text());preserved=lambda s:{k:v for k,v in s.items() if not k.startswith('engine/.git/')}
assert preserved(current)==preserved(baseline)
assert git('ls-files','--stage','-z').hex()==f['index_entries_before']
head=git('rev-parse','HEAD').decode().strip();count=int(git('rev-list','--count','HEAD'))
if which=='before':assert head==f['head_before'] and count==2
else:
 assert head!=f['head_before'] and count==3
 assert git('rev-parse','HEAD^').decode().strip()==f['head_before']
 assert git('cat-file','commit','HEAD').split(b'\n\n',1)[1]==f['expected_message'].encode()
 assert hashlib.sha256(git('show','HEAD:README.md')).hexdigest()==f['expected_blob_sha256']
c=sqlite3.connect('file:'+f['data_directory']+'/state.db?mode=ro',uri=True);ops=[json.loads(p) for(p,) in c.execute('select payload from write_operations')];assert len(ops)==1
op=ops[0]
if stage in ('crashed', 'restarted'):
 assert op['phase']==('publication_pending' if which=='before' else 'reference_updated') and op['reason'] is None
if stage=='recovered':
 assert op['phase']==('cancelled_before_publication' if which=='before' else 'complete')
 assert op['published']==(which=='after')
 if which=='after':assert op['reconciled'] is True
 assert not (repo/'.git/index.lock').exists() and not (repo/'.git/HEAD.lock').exists()

result={'case':which,'stage':stage,'head':head,'commit_count':count,'phase':op['phase'],'journal_published':op['published'],'actual_ref_advanced':head!=f['head_before'],'index_entries_preserved':True,'working_files_and_other_repositories_preserved':True,'operation_id':op['proposal']['operation_id'],'reason':op['reason']}
(folder/(which+'-'+stage+'-verdict.json')).write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
