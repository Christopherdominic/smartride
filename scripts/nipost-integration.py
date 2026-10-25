"""L1 integration against a local HTTP fixture and disposable PostgreSQL.
Never uses a real NIPOST key or calls NIPOST. Run after `cargo build` in backend/.
VERIFY_BROWSER=1 additionally tests the production frontend running on localhost:3000.
"""
from http.server import ThreadingHTTPServer,BaseHTTPRequestHandler
from threading import Thread
from pathlib import Path
from urllib.parse import urlsplit,parse_qs
import subprocess,os,secrets,tempfile,json,time,uuid,urllib.request,urllib.error
class Upstream(BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_GET(self):
  assert self.headers.get('X-API-Key')=='nipost_test_fixture'
  url=urlsplit(self.path);q=parse_qs(url.query);status=200
  if url.path=='/v1/search/autocomplete':value={'data':{'segment':'unit','suggestions':[{'code':'27','label':'27'}]}}
  else:
   assert url.path=='/v1/lookup' and q.get('level')==['1']
   code=q['code'][0]
   if code=='FC-02-A01-KP-27':status=503;value={'error':{'code':'unavailable','message':'fixture service failure'}}
   elif code=='FC-03-A01-KP-27':status=429;value={'error':{'code':'rate_limited','message':'fixture throttle'}}
   else:value={'data':{'postcode':code,'valid':code=='FC-01-A01-KP-27'}}
  data=json.dumps(value).encode();self.send_response(status);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(data)));self.end_headers();self.wfile.write(data)
def docker(*args):return subprocess.check_output(['docker',*args],text=True,stderr=subprocess.PIPE).strip()
def api(path,body=None,expected=200):
 req=urllib.request.Request('http://localhost:8081/api/v1'+path,data=json.dumps(body).encode() if body is not None else None,headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(req,timeout=15) as response:code=response.status;data=json.load(response)
 except urllib.error.HTTPError as e:code=e.code;data=json.load(e)
 assert code==expected,(path,code,data);return data
name='smartride-l1-check-'+uuid.uuid4().hex[:8];backend=None
server=ThreadingHTTPServer(('127.0.0.1',0),Upstream);Thread(target=server.serve_forever,daemon=True).start()
try:
 with tempfile.TemporaryDirectory(prefix='smartride-l1-') as directory:
  password=secrets.token_hex(20);p=Path(directory,'db.env');p.write_text(f'POSTGRES_USER=smartride\nPOSTGRES_DB=smartride\nPOSTGRES_PASSWORD={password}\n');p.chmod(0o600)
  docker('run','-d','--name',name,'--env-file',str(p),'--tmpfs','/var/lib/postgresql/data','-p','127.0.0.1::5432','postgres:16-alpine')
  for _ in range(100):
   if subprocess.run(['docker','exec',name,'pg_isready','-h','127.0.0.1','-U','smartride','-d','smartride'],capture_output=True).returncode==0:break
   time.sleep(.2)
  port=json.loads(docker('inspect',name))[0]['NetworkSettings']['Ports']['5432/tcp'][0]['HostPort']
  env=dict(os.environ,DATABASE_URL=f'postgres://smartride:{password}@localhost:{port}/smartride',APP_MODE='demo',POSTCODE_PROVIDER='nipost',NIPOST_API_KEY='nipost_test_fixture',NIPOST_API_BASE_URL=f'http://127.0.0.1:{server.server_port}',PORT='8081',FRONTEND_ORIGIN='http://localhost:3000')
  with open('/tmp/smartride-l1-backend.log','w') as log:
   backend=subprocess.Popen(['backend/target/debug/backend'],env=env,stdout=log,stderr=log)
   for _ in range(100):
    try:
     if api('/demo')['provider']=='nipost':break
    except Exception:time.sleep(.2)
   else:raise RuntimeError('Fixture API did not start; inspect /tmp/smartride-l1-backend.log')
   assert api('/addresses/resolve?postcode=FC-01-A01-KP-27')['formatted_address'] is None
   assert api('/addresses/autocomplete?q=FC01A01KP')['suggestions'][0]['code']=='27'
   api('/addresses/autocomplete?q=x',expected=400)
   api('/addresses/resolve?postcode=FC-99-Z99-ZZ-99',expected=404)
   api('/addresses/resolve?postcode=FC-00-A01-KP-27',expected=400)
   api('/addresses/resolve?postcode=FC-02-A01-KP-27',expected=502)
   api('/addresses/resolve?postcode=FC-03-A01-KP-27',expected=429)
   config=api('/demo');body={'merchant_id':config['merchant']['id'],'recipient_name':'Fixture Recipient','recipient_phone':'+2340000000000','destination_postcode':'FC-01-A01-KP-27'}
   api('/deliveries',body,400)
   body.update(customer_address='12 Sample Street, Demo City',landmark='Sample landmark',destination_postcode='FC-99-Z99-ZZ-99',postcode_verified=True)
   api('/deliveries',body,404)
   test_env=dict(env,API_BASE_URL='http://localhost:8081',TEST_DB_CONTAINER=name,TEST_POSTCODE='FC-01-A01-KP-27')
   subprocess.run(['python3','scripts/smoke.py'],env=test_env,check=True)
   if os.environ.get('VERIFY_BROWSER')=='1':subprocess.run(['node','scripts/browser-smoke.mjs'],env=test_env,check=True)
   print('PASS: Reqwest L1 fixture → fresh SQL migrations → customer-address delivery → rider → OTP; unavailable/invalid/throttled lookups fail closed')
finally:
 if backend is not None:backend.terminate();backend.wait(timeout=10)
 server.shutdown();server.server_close()
 subprocess.run(['docker','rm','-f',name],capture_output=True)
