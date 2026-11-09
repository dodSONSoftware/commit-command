"""Owned native fixture. Persist only request metadata; never source or credentials."""
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
from pathlib import Path
import hashlib,json,sys
root=Path(sys.argv[1]); root.mkdir(parents=True,exist_ok=False)
class Handler(BaseHTTPRequestHandler):
 def log_message(self,*args): pass
 def do_POST(self):
  count=int(self.headers.get('Content-Length','0'))
  if self.path!='/api/chat' or count>2*1024*1024: self.send_error(400); return
  body=self.rfile.read(count)
  request=json.loads(body); user=json.loads(request['messages'][1]['content']); context=json.loads(user['context'])
  ids=[c['id'] for c in context['chunks']]
  candidate={'message':{'subject':'fix: preserve selected fixture changes','body':'Describe the explicitly selected fixture content.'},'rationale':'Summarize only the reviewed fixture selection.','summary':'Selected fixture text differs from HEAD.','concerns':[],'covered_chunks':ids}
  response=json.dumps({'model':request['model'],'done':True,'message':{'role':'assistant','content':json.dumps({'candidates':[candidate]})},'prompt_eval_count':42,'eval_count':12}).encode()
  metadata={'method':'POST','path':self.path,'body_bytes':len(body),'body_sha256':hashlib.sha256(body).hexdigest(),'context_sha256':hashlib.sha256(user['context'].encode()).hexdigest(),'model':request['model'],'chunks':len(ids)}
  with (root/'requests.jsonl').open('a') as f: f.write(json.dumps(metadata)+'\n')
  self.send_response(200); self.send_header('Content-Type','application/json'); self.send_header('Content-Length',str(len(response))); self.end_headers(); self.wfile.write(response)
server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
(root/'endpoint').write_text('http://127.0.0.1:'+str(server.server_port)+'/')
server.serve_forever()
