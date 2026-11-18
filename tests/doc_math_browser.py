#!/usr/bin/env python3
"""Explicit real-browser G4 witness; requires exported real Doc8 fragments.

MIZAR_DOC_MATH_WITNESS_DIR=/tmp/g4-witness CARGO_BUILD_JOBS=4 cargo test -p mizar-test doc13_g4_real_doc8_transport_and_browser_witnesses
python3 tests/doc_math_browser.py /tmp/g4-witness --chrome /usr/bin/google-chrome
"""
import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile
import threading
from urllib.parse import urlsplit

PIN = "151.0.7922.75"
ASSETS = Path(__file__).resolve().parents[1] / "crates/mizar-doc/assets/katex"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("witnesses", type=Path)
    parser.add_argument("--chrome", default="/usr/bin/google-chrome")
    args = parser.parse_args()
    assert platform.system() == "Linux" and platform.machine() == "x86_64"
    version = subprocess.check_output([args.chrome, "--version"], text=True).strip()
    assert PIN in version, f"requires Chrome {PIN}, got {version}"
    cases = sorted(args.witnesses.glob("*/fragments.html"))
    assert len(cases) >= 10, "export the Rust Doc8 witness first"
    fragments, expected = [], []
    for case in cases:
        fragments.append(case.read_text())
        for payload in sorted(case.parent.glob("*.tex"), key=lambda p: int(p.stem)):
            expected.append({"tex": payload.read_bytes().decode(), "state": payload.with_suffix(".state").read_text(), "case": case.parent.name})
    expected_json = json.dumps(expected, ensure_ascii=True).replace("<", "\\u003c")
    assertions = r"""
const expected = EXPECTED;
const absent = ABSENT;
const refuse = REFUSE;
(async () => {
 try {
  const nodes = [...document.querySelectorAll('.mizar-math')];
  if (nodes.length !== expected.length) throw Error('occurrence count');
  if (!absent) {
   for (let i=0; i<200 && nodes.some(n => !n.dataset.state); i++) await new Promise(r=>setTimeout(r,50));
  }
  await document.fonts.ready;
  if (!absent && !refuse && ![...document.fonts].some(f => f.family.includes('KaTeX') && f.status === 'loaded')) throw Error('KaTeX font was not loaded');
  const assert = (ok, msg) => { if (!ok) throw Error(msg); };
  assert(navigator.language === 'en-US', 'locale');
  assert(Intl.DateTimeFormat().resolvedOptions().timeZone === 'UTC', 'timezone');
  assert(!globalThis.payloadRan, 'payload executed');
  for (let i=0; i<nodes.length; i++) {
   const n=nodes[i], e=expected[i];
   const raw = Uint8Array.from(n.dataset.mizarTex.match(/../g)||[], x=>parseInt(x,16));
   assert(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(raw)===e.tex, 'transport '+e.case);
   if (absent) {
    assert(!n.dataset.state && n.querySelector('[role=status]').textContent.includes('pending'), 'pending fallback');
    const visible=e.tex.replace(/[\u0000-\u0008\u000b-\u001f\u007f-\u009f]/g, c=>'\\u{'+c.codePointAt(0).toString(16)+'}');
    assert(n.querySelector('code').textContent === visible, 'literal fallback '+e.case);
   } else if (refuse || e.state === 'error') {
    assert(n.dataset.state === 'error', 'error state '+e.case);
    assert(n.querySelector('code').textContent === e.tex, 'exact fallback '+e.case);
    assert(n.querySelector('[role=status]').textContent === 'Mathematics unavailable', 'accessible fallback');
   } else {
    assert(n.dataset.state === 'rendered', 'success '+e.case);
    assert(n.querySelector('.katex'), 'actual KaTeX '+e.case);
    const annotation=n.querySelector('annotation[encoding="application/x-tex"]');
    assert(annotation && annotation.textContent === e.tex, 'MathML payload '+e.case);
   }
   assert(!n.querySelector('script,img,iframe,a,.injected'), 'active trusted HTML '+e.case);
   if (e.case === 'size' && !absent && !refuse) {
    const styles=[...n.querySelectorAll('[style]')].map(x=>x.getAttribute('style')).join(';');
    assert(!styles.includes('100em') && styles.includes('10em'), 'maxSize clipping');
   }
  }
  for (const entry of performance.getEntriesByType('resource')) {
   const u = new URL(entry.name, location.href);
   assert(u.protocol === 'blob:' || u.origin === location.origin, 'foreign request '+u.href);
  }
  const result=document.createElement('pre'); result.id='g4-result';
  result.textContent=JSON.stringify({ok:true,dom:nodes.map(n=>n.outerHTML)});
  document.body.append(result);
 } catch(e) {
  const result=document.createElement('pre'); result.id='g4-result';
  result.textContent=JSON.stringify({ok:false,error:String(e)}); document.body.append(result);
 }
})();
"""
    requests = []
    failures = {
        "missing-js": ("katex.min.js", "missing"),
        "corrupt-js": ("katex.min.js", "corrupt"),
        "mixed-js": ("katex.min.js", "mixed"),
        "missing-css": ("katex.min.css", "missing"),
        "corrupt-css": ("katex.min.css", "corrupt"),
        "missing-font": ("fonts/KaTeX_Main-Regular.woff2", "missing"),
        "corrupt-font": ("fonts/KaTeX_Main-Regular.woff2", "corrupt"),
        "missing-config": ("config.json", "missing"),
        "corrupt-config": ("config.json", "corrupt"),
        "mixed-config": ("config.json", "mixed"),
    }

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            path = urlsplit(self.path).path
            requests.append(path)
            scenario = path.split('/')[1]
            if path.endswith('.html'):
                body = "<!doctype html><html lang=en-US><meta charset=utf-8><body>" + '\n'.join(fragments)
                if scenario != "absent":
                    body += f'<script src="/{scenario}/assets/katex/client.js"></script>'
                script = assertions.replace("EXPECTED", expected_json).replace("ABSENT", str(scenario == "absent").lower()).replace("REFUSE", str(scenario in failures).lower())
                body += '<script>' + script + '</script></body></html>'
                data, content_type = body.encode(), 'text/html; charset=utf-8'
            elif '/assets/katex/' in path:
                asset = path.split('/assets/katex/', 1)[1]
                file = ASSETS / asset
                if not file.is_file() or (scenario in failures and failures[scenario] == (asset, "missing")):
                    self.send_error(404)
                    return
                data = file.read_bytes()
                if scenario in failures and failures[scenario][0] == asset:
                    mode = failures[scenario][1]
                    data = b'altered release bytes' if mode == 'mixed' else bytes([data[0] ^ 1]) + data[1:]
                content_type = 'application/json' if asset.endswith('.json') else 'application/javascript' if asset.endswith('.js') else 'text/css' if asset.endswith('.css') else 'application/octet-stream'
            else:
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header('Content-Type', content_type)
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    snapshots = []
    import html
    try:
        for scenario in ["good", "good", "absent", *failures]:
            with tempfile.TemporaryDirectory(prefix='g4-chrome-') as profile:
                url = f'http://127.0.0.1:{server.server_port}/{scenario}/nested/module/page.html'
                command = [args.chrome, '--headless=new', '--disable-gpu', '--disable-dev-shm-usage', '--disable-background-networking', '--lang=en-US', '--no-proxy-server', f'--user-data-dir={profile}', '--virtual-time-budget=20000', '--timeout=30000', '--dump-dom', url]
                result = subprocess.run(command, capture_output=True, text=True, timeout=60, env={**os.environ, 'TZ': 'UTC', 'LANG': 'en_US.UTF-8'})
                assert result.returncode == 0, result.stderr
                match = re.search(r'<pre id="g4-result">(.*?)</pre>', result.stdout, re.S)
                assert match, f'{scenario}: browser did not finish\n{result.stderr[-1000:]}'
                report = json.loads(html.unescape(match[1]))
                assert report['ok'], f'{scenario}: {report}'
                if scenario == 'good':
                    snapshots.append(report['dom'])
                print(f'{scenario}: passed ({len(expected)} occurrences)', flush=True)
        assert snapshots[0] == snapshots[1], 'serialized terminal DOM differs'
        assert any('/good/assets/katex/fonts/' in p for p in requests), 'font loading absent'
        print(f'reproducibility: identical terminal DOM; {version}; en-US UTC fonts ready')
        print('asset bytes SHA256:', hashlib.sha256(b''.join(p.read_bytes() for p in sorted(ASSETS.rglob('*')) if p.is_file())).hexdigest())
    finally:
        server.shutdown()


if __name__ == '__main__':
    main()
