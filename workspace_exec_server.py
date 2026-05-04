#!/usr/bin/env python3
"""Argus workspace execution server.

Endpoints:
  POST /exec  — run a shell command (existing)
  POST /run   — run a code snippet in a specific language (new)

Also starts a background HTTP file server on :8081 serving /workspace/public/
so agent-written HTML pages are immediately browsable.
"""
from http.server import HTTPServer, SimpleHTTPRequestHandler, BaseHTTPRequestHandler
import subprocess
import threading
import hashlib
import datetime
import tempfile
import json
import os
import sys

EXEC_TOKEN = os.environ.get('WORKSPACE_EXEC_TOKEN', '')
LOG_PATH    = '/workspace/exec_audit.log'
PUBLIC_DIR  = '/workspace/public'
FILES_PORT  = 8081

LANGUAGE_RUNNERS = {
    'python':     ('python3', '.py'),
    'python3':    ('python3', '.py'),
    'javascript': ('node',    '.js'),
    'js':         ('node',    '.js'),
    'node':       ('node',    '.js'),
    'typescript': ('npx ts-node --skip-project', '.ts'),
    'ts':         ('npx ts-node --skip-project', '.ts'),
    'bash':       ('bash',    '.sh'),
    'sh':         ('sh',      '.sh'),
    'ruby':       ('ruby',    '.rb'),
    'go':         ('go run',  '.go'),
    'rust':       ('rustc -o /tmp/argus_run_out && /tmp/argus_run_out', '.rs'),
}


def log_execution(command, exit_code, stdout, stderr):
    entry = {
        'timestamp': datetime.datetime.utcnow().isoformat(),
        'command': command[:500],
        'exit_code': exit_code,
        'stdout_hash': hashlib.sha256(stdout.encode()).hexdigest(),
        'stderr_hash': hashlib.sha256(stderr.encode()).hexdigest(),
    }
    try:
        with open(LOG_PATH, 'a') as f:
            f.write(json.dumps(entry) + '\n')
    except Exception as e:
        print(f'[exec_audit] log failed: {e}', flush=True)


def run_command(cmd, cwd='/workspace', timeout=60):
    try:
        result = subprocess.run(
            ['sh', '-c', cmd],
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=cwd,
        )
        log_execution(cmd, result.returncode, result.stdout, result.stderr)
        return {
            'stdout': result.stdout,
            'stderr': result.stderr,
            'exit_code': result.returncode,
        }
    except subprocess.TimeoutExpired:
        log_execution(cmd, -1, '', f'Timed out after {timeout}s')
        return {'stdout': '', 'stderr': f'Timed out after {timeout}s', 'exit_code': -1}
    except Exception as e:
        log_execution(cmd, -1, '', str(e))
        return {'stdout': '', 'stderr': str(e), 'exit_code': -1}


class ExecHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass  # suppress access logs

    def _auth(self):
        auth = self.headers.get('X-Argus-Auth', '')
        if not EXEC_TOKEN or auth != EXEC_TOKEN:
            self.send_response(403)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"error": "forbidden"}')
            return False
        return True

    def _read_body(self):
        length = int(self.headers.get('Content-Length', 0))
        return json.loads(self.rfile.read(length))

    def _respond(self, status, data):
        payload = json.dumps(data).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_POST(self):
        if self.path == '/exec':
            self._handle_exec()
        elif self.path == '/run':
            self._handle_run()
        else:
            self.send_response(404)
            self.end_headers()

    def _handle_exec(self):
        if not self._auth():
            return
        try:
            body = self._read_body()
            command = body.get('command', '').strip()
        except Exception as e:
            self._respond(400, {'error': str(e)})
            return

        if not command:
            self._respond(400, {'error': 'empty command'})
            return

        self._respond(200, run_command(command))

    def _handle_run(self):
        if not self._auth():
            return
        try:
            body    = self._read_body()
            lang    = body.get('language', 'python').lower().strip()
            code    = body.get('code', '').strip()
            timeout = int(body.get('timeout', 30))
        except Exception as e:
            self._respond(400, {'error': str(e)})
            return

        if not code:
            self._respond(400, {'error': 'empty code'})
            return

        runner = LANGUAGE_RUNNERS.get(lang)
        if not runner:
            self._respond(400, {'error': f'unsupported language: {lang}. Supported: {list(LANGUAGE_RUNNERS.keys())}'})
            return

        interpreter, ext = runner

        # Write code to temp file — avoids all shell quoting nightmares
        try:
            with tempfile.NamedTemporaryFile(
                suffix=ext, prefix='argus_run_', dir='/tmp',
                mode='w', delete=False
            ) as f:
                f.write(code)
                tmp_path = f.name
        except Exception as e:
            self._respond(500, {'error': f'Could not write temp file: {e}'})
            return

        cmd = f'{interpreter} {tmp_path}'
        result = run_command(cmd, timeout=timeout)

        # Clean up temp file
        try:
            os.unlink(tmp_path)
        except Exception:
            pass

        self._respond(200, result)


def start_file_server():
    """Serve /workspace/public on port 8081 — agent-created HTML pages."""
    os.makedirs(PUBLIC_DIR, exist_ok=True)

    # Write a default index so the browser doesn't 404 on first visit
    index = os.path.join(PUBLIC_DIR, 'index.html')
    if not os.path.exists(index):
        with open(index, 'w') as f:
            f.write("""<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Argus Pages</title>
<style>body{background:#0a0a0f;color:#c0c0e0;font-family:monospace;padding:2rem}</style>
</head>
<body><h1>◈ Argus Pages</h1>
<p>Files written by the agent to <code>/workspace/public/</code> appear here.</p>
</body></html>""")

    class QuietHandler(SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=PUBLIC_DIR, **kwargs)
        def log_message(self, format, *args):
            pass

    server = HTTPServer(('0.0.0.0', FILES_PORT), QuietHandler)
    print(f'[+] Argus file server listening on :{FILES_PORT} → {PUBLIC_DIR}', flush=True)
    server.serve_forever()


if __name__ == '__main__':
    # Start static file server in background thread
    t = threading.Thread(target=start_file_server, daemon=True)
    t.start()

    # Start exec server in foreground
    server = HTTPServer(('0.0.0.0', 9001), ExecHandler)
    print('[+] Workspace exec server listening on :9001', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print('[*] Workspace exec server shutting down', flush=True)
        sys.exit(0)
