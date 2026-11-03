/* Disposable macOS Keychain fixture only. No default search-list/store changes. */
#include <Security/Security.h>
#include <CoreFoundation/CoreFoundation.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#ifndef FIXTURE_INTERACTIVE
#define FIXTURE_INTERACTIVE 0
#endif
#ifndef FIXTURE_ROOT
#error Fixture root required
#endif
static void location(char *out,const char *name){snprintf(out,4096,"%s/%s",FIXTURE_ROOT,name);}
static char *secret(const char *name,size_t *len){char path[4096];location(path,name);FILE*f=fopen(path,"rb");if(!f)return NULL;char*b=calloc(8193,1);*len=fread(b,1,8192,f);if(fgetc(f)!=EOF){fclose(f);free(b);return NULL;}fclose(f);return b;}
static void status(OSStatus s){char p[4096];location(p,"last-status");FILE*f=fopen(p,"w");if(f){fprintf(f,"%d",(int)s);fclose(f);}}
int main(int argc,char **argv){
 if(argc!=2)return 2;SecKeychainSetUserInteractionAllowed(FIXTURE_INTERACTIVE);
 if(!strcmp(argv[1],"searchlist")){CFArrayRef list=NULL;OSStatus s=SecKeychainCopySearchList(&list);if(s)return 1;for(CFIndex i=0;i<CFArrayGetCount(list);i++){char p[4096];UInt32 n=sizeof p; if(SecKeychainGetPath((SecKeychainRef)CFArrayGetValueAtIndex(list,i),&n,p)){CFRelease(list);return 1;}printf("%u:",n);fwrite(p,1,n,stdout);putchar('\n');}CFRelease(list);return 0;}
 char path[4096];location(path,"fixture.keychain");SecKeychainRef kc=NULL;OSStatus s;
 if(!strcmp(argv[1],"seed")){size_t n;char *pass=secret("keychain-password",&n);if(!pass)return 2;s=SecKeychainCreate(path,(UInt32)n,pass,false,NULL,&kc);memset(pass,0,n);free(pass);if(s){status(s);return 1;}}
 else {s=SecKeychainOpen(path,&kc);if(s){status(s);return !strcmp(argv[1],"delete")?0:1;}}
 if(!strcmp(argv[1],"lock")){s=SecKeychainLock(kc);}
 else if(!strcmp(argv[1],"unlock")){size_t n;char *pass=secret("keychain-password",&n);if(!pass)return 2;s=SecKeychainUnlock(kc,(UInt32)n,pass,true);memset(pass,0,n);free(pass);}
 else if(!strcmp(argv[1],"delete")){s=SecKeychainDelete(kc);}
 else if(!strcmp(argv[1],"get")||!strcmp(argv[1],"seed")){
  char host[4096]="",account[512]="",repo[4096]="",*line=NULL;size_t capacity=0;
  while(getline(&line,&capacity,stdin)>0){line[strcspn(line,"\r\n")]=0;if(!*line)break;if(!strncmp(line,"host=",5))snprintf(host,sizeof host,"%s",line+5);else if(!strncmp(line,"username=",9))snprintf(account,sizeof account,"%s",line+9);else if(!strncmp(line,"path=",5))snprintf(repo,sizeof repo,"%s",line+5);}free(line);
  char service[8192];snprintf(service,sizeof service,"https://%s/%s",host,repo);
  if(!strcmp(argv[1],"seed")){size_t n;char*pass=secret("credential-password",&n);if(!pass)return 2;s=SecKeychainAddGenericPassword(kc,(UInt32)strlen(service),service,(UInt32)strlen(account),account,(UInt32)n,pass,NULL);memset(pass,0,n);free(pass);}
  else {UInt32 n=0;void *pass=NULL;s=SecKeychainFindGenericPassword(kc,(UInt32)strlen(service),service,(UInt32)strlen(account),account,&n,&pass,NULL);if(!s){char hold[4096];location(hold,"hold");if(access(hold,F_OK)==0){char mark[4096];location(mark,"started");FILE*f=fopen(mark,"w");if(f){fprintf(f,"%d",getpid());fclose(f);}sleep(60);}fputs("password=",stdout);fwrite(pass,1,n,stdout);putchar('\n');SecKeychainItemFreeContent(NULL,pass);}}
 }else{s=errSecParam;}
 status(s);CFRelease(kc);return s?1:0;
}
