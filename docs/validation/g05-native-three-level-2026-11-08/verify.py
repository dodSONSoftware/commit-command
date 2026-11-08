from oracle_common import *

label=sys.argv[1]
keys=['leaf','middle','parent'];stage=keys.index(label) if label in keys else 2
for index,key in enumerate(keys):
 old=f['heads'][f[key]]
 assert git(key,'rev-list','--count',old+'..HEAD').strip()==(b'1' if index<=stage else b'0'),key
 if index<=stage:
  assert git(key,'show','HEAD:file.txt')==b'Reviewed native three-level content\n'
  assert git(key,'log','-1','--format=%B')==b'G05 native three-level reviewed commit\n\nRequire fresh approval at each changed parent.\n\n'
  assert git(key,'status','--porcelain')==b'',key
 else:
  assert git(key,'rev-parse','HEAD').decode().strip()==old
allowed='parent/.git/modules/middle/modules/leaf/' if stage==0 else 'parent/.git/modules/middle/' if stage==1 else 'parent/.git/'
assert {k:v for k,v in current.items() if not k.startswith(allowed)}=={k:v for k,v in original.items() if not k.startswith(allowed)}
if stage>=1:
 previous_files=json.loads((out/'leaf-files.json').read_text())
 prefix='parent/.git/modules/middle/modules/leaf/'
 assert {k:v for k,v in current.items() if k.startswith(prefix)}=={k:v for k,v in previous_files.items() if k.startswith(prefix)}
 assert git('middle','rev-parse','HEAD:leaf')==git('leaf','rev-parse','HEAD')
if stage>=2:
 previous_files=json.loads((out/'middle-files.json').read_text());prefix='parent/.git/modules/'
 assert {k:v for k,v in current.items() if k.startswith(prefix)}=={k:v for k,v in previous_files.items() if k.startswith(prefix)}
 assert git('parent','rev-parse','HEAD:middle')==git('middle','rev-parse','HEAD')
batches=sorted((json.loads(v) for v in records['batches'].values()),key=lambda b:b['created_at_ms'])
cancelled=[b for b in batches if b['id']=='258bbae2-f235-4b80-88ac-0085901dd931']
if stage>=1:
 assert len(cancelled)==1 and cancelled[0]['phase']=='cancelled'
 for item in cancelled[0]['items']:
  op=json.loads(records['operations'][item['operation']])
  assert op['phase']=='cancelled_before_publication' and not op['published']
batches=[b for b in batches if b['id']!='258bbae2-f235-4b80-88ac-0085901dd931']
assert len(batches)==stage+1
for i,batch in enumerate(batches):
 assert batch['phase']==('complete' if i==2 else 'needs_reauthorization')
 assert [item['worktree'] for item in batch['items']]==[f['entries'][f[k]] for k in keys[i:]]
 for j,item in enumerate(batch['items']):
  op=json.loads(records['operations'][item['operation']])
  assert op['phase']==('complete' if j==0 else 'cancelled_before_publication')
  assert op['published']==(j==0)
  if j==0:assert op['reconciled']
if stage>0:
 previous_records=json.loads((out/(keys[stage-1]+'-records.json')).read_text())
 assert all(records[kind][k]==v for kind in previous_records for k,v in previous_records[kind].items())
assert snap(workspace)==current
proof={'stage':label,'heads':{key:git(key,'rev-parse','HEAD').decode().strip() for key in keys},'exact_one_commit_each_completed':True,'ancestor_metadata_unchanged_until_approved':True,'descendant_checkout_and_actual_metadata_preserved':True,'working_files_modes_preserved':True,'control_exact_untrusted':True,'exact_three_trusts':True,'batch_phases':[b['phase'] for b in batches],'integrity':'ok'}
if label=='restart':
 assert current==json.loads((out/'parent-files.json').read_text());assert records==json.loads((out/'parent-records.json').read_text());proof['restart_exact_files_records_no_replay']=True
else:
 (out/(label+'-files.json')).write_text(json.dumps(current,indent=2)+'\n');(out/(label+'-records.json')).write_text(json.dumps(records,indent=2)+'\n')
(out/('verify-'+label+'.json')).write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof,indent=2))
