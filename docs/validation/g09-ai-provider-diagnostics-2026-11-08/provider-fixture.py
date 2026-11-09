"""Owned loopback-only synthetic provider; never opens a managed repository."""
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
from pathlib import Path
import json, os, sys, time
root=Path(sys.argv[1]);root.mkdir(mode=0o700,parents=True,exist_ok=True)
class Handler(BaseHTTPRequestHandler):
    def log_message(self,*args): pass
    def send(self,status,value,headers=None):
        data=json.dumps(value).encode();self.send_response(status)
        for name,value in (headers or {}).items(): self.send_header(name,value)
        self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(data)));self.end_headers()
        try:self.wfile.write(data)
        except (BrokenPipeError,ConnectionResetError):pass
    def record(self,body):
        record={'method':self.command,'path':self.path,'body':body,'authorization_present':bool(self.headers.get('Authorization'))}
        with (root/'requests.jsonl').open('a') as f:f.write(json.dumps(record)+'\n')
    def do_GET(self):
        self.record(None)
        if self.path=='/api/tags':self.send(200,{'models':[{'name':'fixture-model','size':12345}]})
        else:self.send(404,{'error':'unexpected route'})
    def do_POST(self):
        length=int(self.headers.get('Content-Length','0'))
        if not 0<=length<=2*1024*1024:self.send(413,{});return
        body=json.loads(self.rfile.read(length));self.record(body)
        if self.path!='/api/chat':self.send(404,{});return
        while (root/'hold').exists() and not (root/'release').exists():time.sleep(.01)
        self.send(200,{'model':body['model'],'done':True,'done_reason':'stop','message':{'role':'assistant','content':json.dumps({'subject':'fix: update synthetic example','body':'Describe only the fixed diagnostic diff.'})},'prompt_eval_count':42,'eval_count':12})
server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
(root/'server.json').write_text(json.dumps({'pid':os.getpid(),'endpoint':'http://127.0.0.1:'+str(server.server_port)+'/'})+'\n')
server.serve_forever()
