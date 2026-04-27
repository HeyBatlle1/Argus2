#!/usr/bin/env python3
"""Argus workspace execution server.

Receives POST /exec from argus-daemon and runs the command in the
workspace container. Returns {stdout, stderr, exit_code} as JSON.
Only reachable on the internal Docker network — not exposed to the host.
"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import subprocess
import json
import sys


class ExecHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass  # suppress per-request access logs

    def do_POST(self):
        if self.path != '/exec':
            self.send_response(404)
            self.end_headers()
            return

        try:
            length = int(self.headers.get('Content-Length', 0))
            body = json.loads(self.rfile.read(length))
            command = body.get('command', '').strip()
        except Exception as e:
            self._respond(400, {'error': str(e)})
            return

        if not command:
            self._respond(400, {'error': 'empty command'})
            return

        try:
            result = subprocess.run(
                ['sh', '-c', command],
                capture_output=True,
                text=True,
                timeout=30,
                cwd='/workspace',
            )
            self._respond(200, {
                'stdout': result.stdout,
                'stderr': result.stderr,
                'exit_code': result.returncode,
            })
        except subprocess.TimeoutExpired:
            self._respond(200, {
                'stdout': '',
                'stderr': 'Command timed out after 30s',
                'exit_code': -1,
            })
        except Exception as e:
            self._respond(200, {
                'stdout': '',
                'stderr': str(e),
                'exit_code': -1,
            })

    def _respond(self, status, data):
        payload = json.dumps(data).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)


if __name__ == '__main__':
    server = HTTPServer(('0.0.0.0', 9001), ExecHandler)
    print('[+] Workspace exec server listening on :9001', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print('[*] Workspace exec server shutting down', flush=True)
        sys.exit(0)
