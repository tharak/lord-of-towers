"""Serve the release exactly under the GitHub Pages project prefix."""
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

class Handler(SimpleHTTPRequestHandler):
    def translate_path(self, path):
        clean = path.split('?', 1)[0].split('#', 1)[0]
        prefix = '/lord-of-towers/'
        if not clean.startswith(prefix):
            return str(Path('dist/__missing__').resolve())
        relative = clean[len(prefix):]
        root = Path('dist').resolve()
        result = (root / relative).resolve()
        return str(result if result.is_relative_to(root) else root / '__missing__')

ThreadingHTTPServer(('127.0.0.1', 4173), Handler).serve_forever()
