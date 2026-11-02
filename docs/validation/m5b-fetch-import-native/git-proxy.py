#!/usr/bin/python3
"""Disposable transparent Git proxy: hold only this fixture's prepared commit."""
import os,sys,subprocess,threading,time,json
from pathlib import Path
REAL='/opt/homebrew/bin/git'
ROOT=Path('/private/tmp/gitfleet-fetch-native-vthb_q8k')
args=sys.argv[1:]
if not ('update-ref' in args and '--stdin' in args and os.environ.get('GIT_COMMON_DIR')==str(ROOT/'workspace/repository/.git') and (ROOT/'hold-publication').is_file()):
    os.execv(REAL,[REAL,*args])
child=subprocess.Popen([REAL,*args],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
def output():
    for line in iter(child.stdout.readline,b''):
        sys.stdout.buffer.write(line);sys.stdout.buffer.flush()
thread=threading.Thread(target=output);thread.start()
for line in sys.stdin.buffer:
    if line==b'commit\n':
        (ROOT/'publication-held.json').write_text(json.dumps({'wrapper_pid':os.getpid(),'git_pid':child.pid,'process_group':os.getpgrp(),'held':'commit command after durable reference-prepared receipt; not forwarded'}))
        while (ROOT/'hold-publication').is_file():time.sleep(.01)
    child.stdin.write(line);child.stdin.flush()
child.stdin.close();status=child.wait();thread.join();sys.exit(status)
