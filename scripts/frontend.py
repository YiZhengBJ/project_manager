#!/usr/bin/env python3
"""Dependency-free frontend build and local development server."""
import functools
import http.server
import pathlib
import shutil
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]


def build():
    output = ROOT / 'dist'
    output.mkdir(exist_ok=True)
    for name in ['index.html', 'styles.css', 'app.js', 'desktop.js']:
        shutil.copy2(ROOT / 'frontend' / name, output / name)
    print('Frontend built: ' + str(output), flush=True)


if __name__ == '__main__':
    if len(sys.argv) != 2 or sys.argv[1] not in ['build', 'dev']:
        raise SystemExit('Usage: python3 scripts/frontend.py build|dev')
    if sys.argv[1] == 'build':
        build()
    else:
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(ROOT / 'frontend'))
        http.server.ThreadingHTTPServer.allow_reuse_address = True
        with http.server.ThreadingHTTPServer(('127.0.0.1', 1420), handler) as server:
            print('Frontend: http://127.0.0.1:1420', flush=True)
            try:
                server.serve_forever()
            except KeyboardInterrupt:
                pass
