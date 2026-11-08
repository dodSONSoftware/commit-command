"""Rebuild isolated test signer from pinned, SHA256-verified upstream archives."""
from pathlib import Path
import hashlib,json,os,subprocess,urllib.request
OUT=Path(__file__).resolve().parent
ROOT=Path('/private/tmp/gitfleet-m2-tools')
PREFIX=ROOT/'install'
manifest=Path('docs/validation/m2/gnupg-source-checksums.json')
packages=json.loads(manifest.read_text())
ROOT.mkdir(parents=True,exist_ok=True)
env=dict(os.environ,PATH=str(PREFIX/'bin')+':'+os.environ['PATH'],PKG_CONFIG_PATH=str(PREFIX/'lib/pkgconfig'),CPPFLAGS='-I'+str(PREFIX/'include'),LDFLAGS='-L'+str(PREFIX/'lib'))
results=[]
for name,version,digest in packages:
 archive=ROOT/(name+'-'+version+'.tar.bz2')
 url='https://gnupg.org/ftp/gcrypt/'+name+'/'+archive.name
 if not archive.exists():
  with urllib.request.urlopen(url,timeout=60) as response: archive.write_bytes(response.read())
 assert hashlib.sha256(archive.read_bytes()).hexdigest()==digest,archive
 results.append({'name':name,'version':version,'url':url,'sha256':digest,'verified':True})
(OUT/'verified-sources.json').write_text(json.dumps(results,indent=2)+'\n')
for name,version,_ in packages:
 source=ROOT/(name+'-'+version)
 if not source.exists():subprocess.run(['tar','-xjf',str(ROOT/(source.name+'.tar.bz2')),'-C',str(ROOT)],check=True)
 args=['./configure','--prefix='+str(PREFIX),'--disable-nls','--disable-doc']
 if name!='gnupg':args+=['--disable-shared','--enable-static']
 if name=='libgcrypt':args+=['--disable-asm']
 if name=='gnupg':args+=['--disable-ldap','--disable-gnutls','--disable-scdaemon','--disable-gpgsm','--disable-dirmngr','--disable-g13','--disable-wks-tools']
 print('BUILD',name,flush=True)
 for command in [args,['make','-j4'],['make','install']]:subprocess.run(command,cwd=source,env=env,check=True)
print(subprocess.check_output([str(PREFIX/'bin/gpg'),'--version'],text=True))
