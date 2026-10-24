"""Verify the built image with a new, disposable PostgreSQL database.
Requires Docker and the image smartride-api:free-hosting. No existing database is touched.
Run: python3 scripts/verify-container.py
"""
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time
import urllib.request
import uuid

IMAGE = os.environ.get('VERIFY_IMAGE', 'smartride-api:free-hosting')
suffix = uuid.uuid4().hex[:8]
network = f'smartride-check-{suffix}'
database = f'{network}-db'
demo = f'{network}-demo'
production = f'{network}-production'
containers = []

def docker(*args):
    result = subprocess.run(['docker', *args], check=True, text=True, capture_output=True)
    return result.stdout.strip()

def ready(port):
    for _ in range(120):
        try:
            with urllib.request.urlopen(f'http://localhost:{port}/health', timeout=2) as response:
                if json.load(response)['status'] == 'ok':
                    return
        except (OSError, ValueError):
            pass
        time.sleep(.5)
    raise RuntimeError(f'Container API did not become healthy on {port}')

try:
    docker('network', 'create', network)
    with tempfile.TemporaryDirectory(prefix='smartride-check-') as directory:
        password = secrets.token_hex(24)
        db_env = Path(directory, 'db.env')
        db_env.write_text(f'POSTGRES_USER=smartride\nPOSTGRES_DB=smartride\nPOSTGRES_PASSWORD={password}\n')
        db_env.chmod(0o600)
        api_env = Path(directory, 'api.env')
        api_env.write_text(f'DATABASE_URL=postgres://smartride:{password}@{database}:5432/smartride\nPORT=10000\nPOSTCODE_PROVIDER=mock\nFRONTEND_ORIGIN=http://localhost:3000\n')
        api_env.chmod(0o600)
        containers.append(database)
        docker('run', '-d', '--name', database, '--network', network, '--env-file', str(db_env), '--tmpfs', '/var/lib/postgresql/data', 'postgres:16-alpine')
        for _ in range(120):
            result = subprocess.run(['docker', 'exec', database, 'pg_isready', '-U', 'smartride'], capture_output=True)
            if result.returncode == 0:
                break
            time.sleep(.5)
        for name, mode, port in [(demo, 'demo', 8081), (production, 'production', 8082)]:
            containers.append(name)
            docker('run', '-d', '--name', name, '--network', network, '--env-file', str(api_env), '-e', f'APP_MODE={mode}', '-p', f'127.0.0.1:{port}:10000', IMAGE)
            ready(port)
        assert docker('exec', demo, 'id', '-u') == '10001'
        image_env = json.loads(docker('image', 'inspect', IMAGE, '--format', '{{json .Config.Env}}'))
        assert not any(item.startswith('DATABASE_URL=') for item in image_env)
        sql = ['exec', database, 'psql', '-U', 'smartride', '-d', 'smartride', '-Atc']
        assert docker(*sql, 'SELECT count(*) FROM _sqlx_migrations WHERE success') == '2'
        assert docker(*sql, 'SELECT count(*) FROM merchants WHERE is_demo') == '1'
        assert docker(*sql, 'SELECT count(*) FROM riders') == '3'
        assert docker(*sql, 'SELECT count(*) FROM deliveries') == '5'
        docker('restart', demo)
        ready(8081)
        assert docker(*sql, 'SELECT count(*) FROM deliveries') == '5'
        env = dict(os.environ, API_BASE_URL='http://localhost:8081', TEST_DB_CONTAINER=database)
        subprocess.run(['python3', 'scripts/smoke.py'], env=env, check=True)
        if os.environ.get('VERIFY_BROWSER') == '1':
            subprocess.run(['node', 'scripts/browser-smoke.mjs'], env=env, check=True)
        subprocess.run(['python3', 'scripts/security-smoke.py'], env=env, check=True)
        docker('exec', demo, 'smartride', '--healthcheck')
        print('PASS: production image, non-root user, custom PORT, fresh migrations, idempotent seed, persisted workflow and production security')
finally:
    for name in reversed(containers):
        subprocess.run(['docker', 'rm', '-f', name], capture_output=True)
    subprocess.run(['docker', 'network', 'rm', network], capture_output=True)
