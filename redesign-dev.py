"""Local frontend development host using the already-running GoXLR service.
No device is opened and no command is sent at startup.
"""
import argparse
import http.client
import json
import mimetypes
import re
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parent / 'daemon' / 'web-content'
READ_ROUTES = re.compile(r'^/api/(get-devices|diagnostics(?:/log)?|button-actions/[^/]+|application-controls/[^/]+)$')
WRITE_ROUTES = re.compile(r'^/api/(command|button-actions/[^/]+(?:/test)?|application-controls/[^/]+)$')
DAEMON_COMMANDS = {'OpenPath', 'SetLogLevel', 'SetFirmwareSource', 'SetShowTrayIcon',
                   'SetLocale', 'SetTTSEnabled', 'SetAutoStartEnabled', 'SetAllowNetworkAccess',
                   'SetUiLaunchOnLoad', 'SetActivatorPath', 'RecoverDefaults',
                   'SetSampleGainPct', 'ApplySampleChange', 'HandleMacOSAggregates'}

def permitted_packet(packet):
    if not isinstance(packet, dict) or len(packet) != 1:
        return False
    if 'GetMicLevel' in packet:
        return isinstance(packet['GetMicLevel'], str)
    if 'Command' in packet:
        return isinstance(packet['Command'], list) and len(packet['Command']) == 2
    if 'Daemon' in packet:
        value = packet['Daemon']
        return (isinstance(value, str) and value in DAEMON_COMMANDS or
                isinstance(value, dict) and len(value) == 1 and next(iter(value)) in DAEMON_COMMANDS)
    if 'RunFirmwareUpdate' in packet:
        value = packet['RunFirmwareUpdate']
        return isinstance(value, list) and len(value) == 3 and value[2] is False
    return next(iter(packet)) in {'ContinueFirmwareUpdate', 'ClearFirmwareState'} and isinstance(next(iter(packet.values())), str)

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def respond(self, status, body, content_type='text/plain; charset=utf-8'):
        self.send_response(status)
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Cache-Control', 'no-store')
        self.send_header('X-Content-Type-Options', 'nosniff')
        self.end_headers()
        self.wfile.write(body)

    def handle_request(self):
        port = self.server.server_port
        allowed_hosts = {f'localhost:{port}', f'127.0.0.1:{port}'}
        if self.headers.get('Host') not in allowed_hosts:
            return self.respond(403, b'Local host required')
        origin = self.headers.get('Origin')
        if origin and origin not in {f'http://{host}' for host in allowed_hosts}:
            return self.respond(403, b'Same origin required')
        path = urlsplit(self.path).path
        # The incumbent frontend needs its native websocket endpoint. Keep it
        # on the service origin instead of pretending to proxy websockets.
        if self.command == 'GET' and path == '/index.html':
            self.send_response(302)
            self.send_header('Location', f'http://localhost:{self.server.backend_port}/index.html')
            self.send_header('Content-Length', '0')
            self.end_headers()
            return
        if path.startswith('/api/'):
            route = READ_ROUTES if self.command == 'GET' else WRITE_ROUTES
            if not route.fullmatch(path):
                return self.respond(404, b'API route not available in this development host')
            size = int(self.headers.get('Content-Length', '0'))
            if size < 0 or size > 2_000_000:
                return self.respond(413, b'Request too large')
            body = self.rfile.read(size) if size else None
            if self.command == 'POST' and path == '/api/command':
                try:
                    packet = json.loads(body)
                    if not permitted_packet(packet):
                        raise ValueError()
                except (ValueError, TypeError):
                    return self.respond(400, b'Command not supported by this development host')
            conn = http.client.HTTPConnection('localhost', self.server.backend_port, timeout=10)
            try:
                conn.request(self.command, path, body=body, headers={'Content-Type':'application/json'})
                response = conn.getresponse()
                self.respond(response.status, response.read(), response.getheader('Content-Type', 'application/json'))
            except OSError:
                self.respond(503, b'O servico GoXLR nao respondeu. Confirma que esta aberto.')
            finally:
                conn.close()
            return
        if self.command != 'GET':
            return self.respond(405, b'Method not allowed')
        target = (ROOT / unquote(path).lstrip('/')).resolve() if path != '/' else ROOT / 'redesign.html'
        if not target.is_relative_to(ROOT.resolve()) or not target.is_file():
            return self.respond(404, b'Not found')
        self.respond(200, target.read_bytes(), mimetypes.guess_type(target)[0] or 'application/octet-stream')

    do_GET = handle_request
    do_POST = handle_request

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--port', type=int, default=8767)
    parser.add_argument('--backend-port', type=int, default=14564)
    args = parser.parse_args()
    server = ThreadingHTTPServer(('127.0.0.1', args.port), Handler)
    server.backend_port = args.backend_port
    print(f'GoXLR redesign: http://localhost:{args.port}/redesign.html', flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
