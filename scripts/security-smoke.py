"""Run after other tests. Exhausts the demo instance's one-minute rate limit.
Requires demo API on 8081 and production-mode API on 8082 by default.
"""
import json
import os
import urllib.request
import urllib.error
DEMO = os.environ.get('API_BASE_URL', 'http://localhost:8081')
PROD = os.environ.get('PRODUCTION_API_BASE_URL', 'http://localhost:8082')
def call(base, path, data=None, origin=None):
    headers = {'Content-Type': 'application/json'}
    if origin:
        headers['Origin'] = origin
    request = urllib.request.Request(base+path, data=data, headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=10) as response:
            return response.status, response.headers, json.load(response)
    except urllib.error.HTTPError as error:
        return error.code, error.headers, json.load(error)
for path, expected in [('/health',200),('/api/v1/demo',403),('/api/v1/deliveries',403),('/openapi.json',200)]:
    code, _, body = call(PROD, path)
    assert code == expected, (path, code)
    if code == 403:
        assert body['error']
assert call(PROD, '/api/v1/deliveries', b'{}')[0] == 403
assert call(DEMO, '/api/v1/deliveries', b'{}')[0] == 400
_, headers, _ = call(DEMO, '/api/v1/demo', origin='http://localhost:3000')
assert headers.get('Access-Control-Allow-Origin') == 'http://localhost:3000'
_, headers, _ = call(DEMO, '/api/v1/demo', origin='https://untrusted.example')
assert headers.get('Access-Control-Allow-Origin') != 'https://untrusted.example'
for _ in range(241):
    code, _, body = call(DEMO, '/api/v1/demo')
    if code == 429:
        assert body['error']
        break
else:
    raise AssertionError('rate limiter did not reject excess requests')
assert call(DEMO, '/health')[0] == 200
print('PASS: production denies reads/writes, JSON validation, CORS origin restrictions, rate limiting, health remains available')
