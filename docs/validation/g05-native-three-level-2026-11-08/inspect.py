from oracle_common import *

label=sys.argv[1]
batches=sorted((json.loads(v) for v in records['batches'].values()),key=lambda b:b['created_at_ms']);latest=batches[-1]
summary={'batch':latest,'operations':[]}
for item in latest['items']:
 op=json.loads(records['operations'][item['operation']]);proposal=op['proposal'];summary['operations'].append({'id':item['operation'],'phase':op['phase'],'published':op['published'],'reconciled':op['reconciled'],'new_head':op['new_head'],'selected':proposal['request']['selected'],'patch':bytes(proposal['patch']).decode(),'message':bytes(proposal['message']).decode()})
if label=='prepared-first':
 assert current==original
 assert [i['worktree'] for i in latest['items']]==[f['entries'][f[k]] for k in ['leaf','middle','parent']]
 assert latest['items'][0]['dependencies']==[]
 assert latest['items'][1]['dependencies']==[f['entries'][f['leaf']]]
 assert latest['items'][2]['dependencies']==[f['entries'][f['middle']]]
 assert all(o['phase']=='proposed' and not o['published'] for o in summary['operations'])
assert snap(workspace)==current
(out/(label+'-records.json')).write_text(json.dumps(records,indent=2)+'\n')
(out/(label+'-summary.json')).write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
