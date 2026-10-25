"""Persisted API integration tests. Use a dedicated local/demo database only.
API_BASE_URL=http://localhost:8081 python3 scripts/smoke.py
Set TEST_DB_CONTAINER=smartride-postgres for expiry/hash/database assertions.
"""
import json,os,urllib.request,urllib.error,subprocess,uuid
from concurrent.futures import ThreadPoolExecutor
BASE=os.environ.get('API_BASE_URL','http://localhost:8081')
def request(path,body=None,status=200):
 data=None if body is None else json.dumps(body).encode()
 req=urllib.request.Request(BASE+path,data=data,headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(req,timeout=15) as r: code=r.status; result=json.load(r)
 except urllib.error.HTTPError as e:code=e.code;result=json.load(e)
 assert code in (status if isinstance(status, tuple) else (status,)),(path,code,result,status)
 return result
config=request('/api/v1/demo');merchant=config['merchant']['id'];riders=request('/api/v1/riders');rider=riders[0]['id']
postcodes=config['postcodes'] if config['provider']=='mock' else [os.environ['TEST_POSTCODE']]
customer_address='12 Sample Street, Demo District, Test City'
for postcode in postcodes:
 a=request('/api/v1/addresses/resolve?postcode='+postcode);assert a['provider']==config['provider'] and a['provider_data']['verified']==(config['provider']=='nipost')
request('/api/v1/addresses/resolve?postcode=invalid',status=400)
def create():
 return request('/api/v1/deliveries',dict(merchant_id=merchant,recipient_name='Smoke Test Recipient',recipient_phone='+2340000000000',destination_postcode=postcodes[0],customer_address=customer_address,landmark='Sample landmark',parcel_description='Integration test'),201)['id']
def action(id,op,body=None,status=200):return request(f'/api/v1/deliveries/{id}/{op}',body or {},status)
def advance(id):
 action(id,'assign',{'rider_id':rider})
 for state in ['picked_up','out_for_delivery']:action(id,'status',{'rider_id':rider,'status':state})
def sql(query):
 return subprocess.check_output(['docker','exec',os.environ['TEST_DB_CONTAINER'],'psql','-U','smartride','-d','smartride','-Atc',query],text=True).strip()
id=create()
action(id,'status',{'rider_id':rider,'status':'delivered'},409)
action(id,'assign',{'rider_id':str(uuid.uuid4())},409)
advance(id)
action(id,'status',{'rider_id':rider,'status':'picked_up'},409)
otp=action(id,'otp')['demo_otp'];assert len(otp)==6 and otp.isdigit()
action(id,'otp',status=409)
action(id,'confirm',{'rider_id':str(uuid.uuid4()),'code':otp},409)
wrong='000000' if otp!='000000' else '111111'
action(id,'confirm',{'rider_id':rider,'code':wrong},400)
d=action(id,'confirm',{'rider_id':rider,'code':otp});assert d['status']=='delivered' and d['proof_confirmed']
assert d['customer_address']==customer_address and d['formatted_address']==customer_address and d['address_source']=='customer'
assert d['postcode_verified']==(config['provider']=='nipost') and d['landmark']=='Sample landmark'
action(id,'confirm',{'rider_id':rider,'code':otp},409)
action(id,'otp',status=409)
assert [e['event_type'] for e in d['events']]==['created','assigned','picked_up','out_for_delivery','delivered']
assert any(d['id']==x['id'] for x in request('/api/v1/riders/'+rider+'/deliveries'))
request('/api/v1/deliveries',dict(merchant_id=str(uuid.uuid4()),recipient_name='Demo',recipient_phone='000',destination_postcode=postcodes[0]),403)
request('/api/v1/deliveries/'+str(uuid.uuid4()),status=404)
id2=create();advance(id2);otp2=action(id2,'otp')['demo_otp'];wrong='000000' if otp2!='000000' else '111111'
for _ in range(5):action(id2,'confirm',{'rider_id':rider,'code':wrong},400)
action(id2,'confirm',{'rider_id':rider,'code':otp2},409);action(id2,'otp',status=409)
if os.environ.get('TEST_DB_CONTAINER'):
 assert sql(f"SELECT code_hash LIKE '$2%' AND consumed_at IS NOT NULL FROM delivery_otps WHERE delivery_id='{id}'")=='t'
 id3=create();advance(id3);otp3=action(id3,'otp')['demo_otp']
 sql(f"UPDATE delivery_otps SET expires_at=NOW()-INTERVAL '1 second' WHERE delivery_id='{id3}'")
 action(id3,'confirm',{'rider_id':rider,'code':otp3},409)
 assert sql('SELECT count(*) FROM _sqlx_migrations WHERE success')=='3'
 print('PASS: additive migrations, bcrypt-only storage, actual expired OTP rejection')
assert request('/api/v1/stats')['completed']>0
request('/health');request('/openapi.json')
print('PASS: postcode → create → assign → pickup → transit → OTP → delivered; timeline, replay, attempts, scope, health, docs')

# Concurrent verification must consume the OTP exactly once.
id4=create();advance(id4);otp4=action(id4,'otp')['demo_otp']
with ThreadPoolExecutor(max_workers=2) as pool:
 results=list(pool.map(lambda _: action(id4,'confirm',{'rider_id':rider,'code':otp4},(200,409)),range(2)))
assert sum(r.get('status')=='delivered' for r in results)==1
history=request('/api/v1/deliveries/'+id4)['events']
assert sum(e['event_type']=='delivered' for e in history)==1
new_rider=request('/api/v1/riders',{'name':'Demo Smoke Rider','phone':'+2340000000000'},201)
assert request('/api/v1/riders/'+new_rider['id']+'/deliveries')==[]
failed=create();action(failed,'assign',{'rider_id':rider});action(failed,'status',{'rider_id':rider,'status':'failed'})
action(failed,'status',{'rider_id':rider,'status':'picked_up'},409)
print('PASS: concurrent OTP consumption exactly once, rider registration, terminal failed state')
