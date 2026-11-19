#!/usr/bin/env python3
"""Real DOM supplied-index witnesses: python3 tests/doc_search_browser.py --chrome /usr/bin/google-chrome."""
import argparse
import copy
import hashlib
import html
import http.server
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import tempfile
import threading
from urllib.parse import quote, urlsplit

PIN = "151.0.7922.75"
ASSETS = Path(__file__).resolve().parents[1] / "crates/mizar-doc/assets"
KINDS = "predicate functor mode attribute structure selector registration theorem lemma algorithm scheme template synonym antonym redefinition builtin".split()


def encode(value):
    return quote(value, safe="-._~")


def record(fqn="Zeta.Alpha", summary="Beta É Ω e\u0301", signature="GAMMA β", origin="first", kind="theorem"):
    return {"identity": ["p", "1.2", None, ["m"], "2026", origin, "a" * 64],
            "fqn": fqn, "kind": kind, "summary": summary, "signature": signature,
            "href": "items/item.html#" + encode(fqn)}


def wire(items):
    return {"format_version": 1, "package_version": "1.2", "items": items}


def payload(value):
    return (json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n").encode()


def fixtures():
    items = [record(), record(summary="second", signature="second", origin="second"),
             record(fqn="alpha", summary=None, signature="", origin="third"),
             record(fqn="", summary="", signature="", origin="fourth")]
    items[3]["identity"][2] = "locked"
    items[3]["identity"][1] = "origin-7"
    hostile = record(fqn='<svg onload="globalThis.payloadRan=1">é%#',
                     summary='<script>globalThis.payloadRan=1</script>',
                     signature='<img src=x onerror="globalThis.payloadRan=1">')
    hostile["href"] = "items/" + encode('é%#<>" &.html') + "#" + encode(hostile["fqn"])
    cases = {"good": payload(wire(items)),
             "pretty": json.dumps(wire(items), ensure_ascii=False, indent=2, sort_keys=True).encode(),
             "empty": payload(wire([])),
             "hostile": payload(wire([hostile])),
             "kinds": payload(wire([record(fqn=k, kind=k, origin=k) for k in KINDS]))}
    invalid = {"utf8": b"\xff", "json": b'{"items":', "bom": b"\xef\xbb\xbf" + cases["good"]}
    invalid.update({"root-" + str(i): payload(value) for i, value in enumerate([None, [], "", 1])})
    base = wire([record()])
    for field in base:
        value = copy.deepcopy(base)
        del value[field]
        invalid["missing-envelope-" + field] = payload(value)
    for field in base["items"][0]:
        value = copy.deepcopy(base)
        del value["items"][0][field]
        invalid["missing-record-" + field] = payload(value)
    substitutions = [("format_version", x) for x in [0, 2, 1.5, "1", True, None]]
    substitutions += [("package_version", x) for x in [None, 12, [], {}]]
    substitutions += [("items", x) for x in [None, {}, "", [None], [True]]]
    substitutions += [("extra", 1)]
    for i, (field, replacement) in enumerate(substitutions):
        value = copy.deepcopy(base)
        value[field] = replacement
        invalid[f"envelope-{i}"] = payload(value)
    changes = [("extra", 1), ("fqn", None), ("kind", "unknown"), ("kind", "<img>"),
               ("kind", 1), ("summary", {}), ("summary", False), ("signature", None), ("href", None)]
    changes += [("identity", x) for x in [None, {}, [], ["p"] * 6, ["p"] * 8]]
    hrefs = ["/item.html#Zeta.Alpha", "//foreign.invalid/a#Zeta.Alpha", "https://foreign.invalid/a#Zeta.Alpha",
             "javascript:alert(1)", "../item.html#Zeta.Alpha", "items/../item.html#Zeta.Alpha",
             "items/%2E%2E/item.html#Zeta.Alpha", "items/%2F.html#Zeta.Alpha", "items/a\\b.html#Zeta.Alpha",
             "items/%GG.html#Zeta.Alpha", "items/%c3%a9.html#Zeta.Alpha", "items/é.html#Zeta.Alpha",
             "items/item.html?x=1#Zeta.Alpha", "items/item.html#Alpha", "items/item.html", ""]
    changes += [("href", x) for x in hrefs]
    for i, (field, replacement) in enumerate(changes):
        value = copy.deepcopy(base)
        value["items"][0][field] = replacement
        invalid[f"record-{i}"] = payload(value)
    tuple_changes = [(0, None), (1, None), (2, 1), (3, "m"), (3, [None]), (4, None), (5, 1),
                     (6, "A" * 64), (6, "a" * 63), (6, "g" * 64), (6, None)]
    for i, (position, replacement) in enumerate(tuple_changes):
        value = copy.deepcopy(base)
        value["items"][0]["identity"][position] = replacement
        invalid[f"identity-{i}"] = payload(value)
    raw = payload(base)
    # Preserve exact non-integral values that JSON.parse rounds to version 1.
    # Do not construct these tokens through Python floating-point values.
    version_tokens = [b"1.0000000000000001", b"0.99999999999999999",
                      b"10000000000000001e-16", b"99999999999999999e-17"]
    reordered = payload({"items": items, "package_version": "1.2", "format_version": 1})
    cases["escaped-version"] = reordered.replace(b'"format_version":1', b'"format_vers\\u0069on" \t:\r\n1')
    for i, token in enumerate(version_tokens):
        invalid[f"rounded-version-{i}"] = cases["good"].replace(b'"format_version":1', b'"format_version":' + token)
        invalid[f"escaped-rounded-version-{i}"] = reordered.replace(
            b'"format_version":1', b'"format_vers\\u0069on" \t:\r\n' + token)
    invalid["duplicate-envelope"] = raw.replace(b'"format_version":1', b'"format_version":1,"format_version":1')
    invalid["escaped-duplicate-envelope"] = raw.replace(b'"items":', b'"it\\u0065ms":[],"items":')
    invalid["duplicate-record"] = raw.replace(b'"fqn":', b'"fqn":"ignored","fqn":')
    invalid["escaped-duplicate-record"] = raw.replace(b'"fqn":', b'"f\\u0071n":"ignored","fqn":')
    invalid["nested-duplicate"] = raw.replace(b'["p","1.2",null,', b'[{"x":1,"\\u0078":2},"1.2",null,')
    invalid["bad-later-record"] = payload(wire([record(), {**record(origin="later"), "kind": "bad"}]))
    cases.update({"bad-" + name: value for name, value in invalid.items()})
    return cases, items, hostile


ASSERTIONS = r"""
(async () => {
 const assert = (ok, message) => { if (!ok) throw Error(message); };
 const section = name => document.getElementById(name);
 const rows = name => [...section(name).querySelectorAll('.mizar-search-results > li')];
 const status = name => section(name).querySelector('.mizar-search-status').textContent;
 const query = name => section(name).querySelector('.mizar-search-query');
 const search = (name, text) => { query(name).value=text; query(name).dispatchEvent(new Event('input',{bubbles:true})); };
 try {
  const expected = EXPECTED;
  const bad = BAD;
  for (let i=0; i<200 && [...document.querySelectorAll('.mizar-search-status')].some(n=>!n.textContent || /loading/i.test(n.textContent)); i++) await new Promise(r=>setTimeout(r,25));
  for (const name of ['good','empty','hostile','kinds','neighbor','nested-neighbor']) assert(!/unavailable|loading/i.test(status(name)), 'success '+name+': '+status(name));
  for (const name of bad) {
   assert(/unavailable/i.test(status(name)), 'unavailable '+name+': '+status(name));
   assert(rows(name).length===0, 'partial records '+name);
   search(name,'Zeta');
   assert(/unavailable/i.test(status(name)) && rows(name).length===0,'refusal after query '+name);
  }
  for (const name of OVERLAPS) {
   const state=section(name).querySelector('.mizar-search-status');
   assert(/unavailable/i.test(state.textContent),'overlap unavailable '+name);
   assert(rows(name).length===0,'overlap rendered results '+name);
   search(name,'Zeta');
   assert(/unavailable/i.test(state.textContent) && rows(name).length===0,'overlap refusal after query '+name);
  }
  assert(rows('empty').length===0, 'complete-empty');
  assert(rows('kinds').length===16, 'G1 vocabulary');
  const checkRows = (name, items) => {
   const actual=rows(name); assert(actual.length===items.length,'row count '+name);
   for (let i=0; i<actual.length; i++) {
    const row=actual[i], e=items[i];
    assert(row.tagName==='LI' && row.children.length===4, 'li shape');
    const [a,k,s,g]=row.children;
    assert(a.tagName==='A' && k.tagName==='SPAN' && k.className==='mizar-kind' && s.tagName==='SPAN' && s.className==='mizar-summary' && g.tagName==='SPAN' && g.className==='mizar-signature','literal children');
    assert(a.textContent===e.fqn && k.textContent===e.kind && s.textContent===(e.summary??'') && g.textContent===e.signature,'exact display/order');
    assert(!row.querySelector('script,img,svg,iframe,object'), 'active display markup');
    assert(a.children.length===0 && k.children.length===0 && s.children.length===0 && g.children.length===0,'text-only fields');
    const u=new URL(a.href);
    assert(u.origin===location.origin && u.pathname==='/version/'+e.href.split('#')[0] && a.getAttribute('href').endsWith('#'+e.href.split('#')[1]), 'href rebasing '+a.href);
   }
  };
  checkRows('good',expected.items);
  checkRows('pretty',expected.items);
  checkRows('escaped-version',expected.items);
  assert(query('good').labels.length===1 && query('good').labels[0].textContent.includes('Search'),'accessible label');
  assert(section('good').querySelector('.mizar-search-status').getAttribute('role')==='status','accessible status');
  const queries=[['ALPHA\tTHEOREM \nBETA \rGAMMA\v\f',[0]],['',[0,1,2,3]],['   \t\r\n\v\f',[0,1,2,3]],['second',[1]],['é',[]],['É',[0]],['Ω',[0]],['ω',[]],['e\u0301',[0]],['é beta',[]],['alpha\u00a0beta',[]],['absent',[]]];
  for (const [text,indices] of queries) { search('good',text); checkRows('good',indices.map(i=>expected.items[i])); }
  assert(status('good')!==status('empty') && !/unavailable/i.test(status('good')),'zero versus empty/unavailable');
  search('good',''); checkRows('good',expected.items);
  checkRows('hostile',[expected.hostile]);
  assert(!globalThis.payloadRan,'hostile display executed');
  const a=rows('hostile')[0].firstElementChild;
  const response=await fetch(a.href); assert(response.ok,'local destination');
  const target=new DOMParser().parseFromString(await response.text(),'text/html');
  assert(target.getElementById(expected.hostile.fqn),'full exact FQN target id');
  assert(decodeURIComponent(new URL(a.href).hash.slice(1))===expected.hostile.fqn,'whole encoded fragment');
  query('good').focus();
  const focus=getComputedStyle(query('good'));
  assert(focus.outlineStyle!=='none' && parseFloat(focus.outlineWidth)>0,'visible focus');
  assert(getComputedStyle(rows('good')[0].querySelector('.mizar-summary')).display==='block','local summary presentation');
  assert(getComputedStyle(rows('good')[0].querySelector('.mizar-signature')).whiteSpace==='pre-wrap','local signature presentation');
  assert([...document.styleSheets].some(s=>s.href && new URL(s.href).pathname==='/assets/style.css'),'local stylesheet');
  assert(rows('neighbor').length===expected.items.length,'neighbor independence');
  search('neighbor','second'); checkRows('neighbor',[expected.items[1]]);
  search('neighbor',''); checkRows('neighbor',expected.items);
  assert(rows('nested-neighbor').length===expected.items.length,'nested neighbor independence');
  for (const sink of ['results','status']) for (const input of ['complete','missing']) {
   const outer='isolation-'+sink+'-'+input, inner='nested-'+sink+'-'+input;
   assert(section(inner)?.isConnected,'nested control removed '+outer);
   checkRows(inner,expected.items);
   search(inner,'second'); checkRows(inner,[expected.items[1]]);
   search(inner,''); checkRows(inner,expected.items);
   if (input==='complete') search(outer,'Zeta');
   assert(section(inner).isConnected,'outer query removed nested control '+outer);
   checkRows(inner,expected.items);
   if (sink==='results') assert(/unavailable/i.test(status(outer)),'unsafe results status '+outer);
   else assert(section(outer).querySelector(':scope > .mizar-search-results').children.length===0,'unsafe status rendered '+outer);
  }
  for (const malformed of document.querySelectorAll('[id^="malformed-"]')) {
   const list=malformed.querySelector('.mizar-search-results');
   if (list) assert(list.children.length===0,'malformed control rendered results');
   const state=malformed.querySelector('.mizar-search-status');
   if (state) assert(/unavailable/i.test(state.textContent),'malformed control status');
  }
  assert(navigator.language==='en-US' && Intl.DateTimeFormat().resolvedOptions().timeZone==='UTC','browser locale/timezone');
  for (const entry of performance.getEntriesByType('resource')) assert(new URL(entry.name).origin===location.origin,'foreign request '+entry.name);
  const result=document.createElement('pre'); result.id='search-result'; result.textContent=JSON.stringify({ok:true,dom:section('good').outerHTML}); document.body.append(result);
 } catch(e) {
  const result=document.createElement('pre'); result.id='search-result'; result.textContent=JSON.stringify({ok:false,error:String(e)}); document.body.append(result);
 }
})();
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--chrome", default="/usr/bin/google-chrome")
    args = parser.parse_args()
    assert platform.system() == "Linux" and platform.machine() == "x86_64"
    version = subprocess.check_output([args.chrome, "--version"], text=True).strip()
    assert PIN in version, f"requires Chrome {PIN}, got {version}"
    manifest = json.loads((ASSETS / "search-manifest.json").read_bytes())
    assert manifest["format_version"] == 1 and len(manifest["assets"]) == 2
    assert [entry["path"] for entry in manifest["assets"]] == ["assets/style.css", "assets/search.js"]
    for entry in manifest["assets"]:
        assert hashlib.sha256((ASSETS / Path(entry["path"]).name).read_bytes()).hexdigest() == entry["sha256"]
    cases, items, hostile = fixtures()
    overlaps = ["overlap-shared", "overlap-status-ancestor", "overlap-results-ancestor"]
    failed = [name for name in cases if name.startswith("bad-")] + ["missing", "http-failed", "redirect", "foreign"]
    requests = []

    def shell(name, index=None):
        index = index if index is not None else "../../version/" + name + ".json"
        return (f'<section id="{name}" class="mizar-search" data-mizar-index="{html.escape(index, quote=True)}">'
                '<label>Search <input type="search" class="mizar-search-query"></label>'
                '<p class="mizar-search-status" role="status"></p><ul class="mizar-search-results"></ul></section>')

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_GET(self):
            path = urlsplit(self.path).path
            requests.append(path)
            if path == "/nested/module/page.html":
                controls = [shell(name) for name in cases]
                controls += [shell(name) for name in ["missing", "http-failed", "redirect"]]
                controls += [shell("foreign", f"http://localhost:{self.server.server_port}/version/good.json")]
                # Missing each scoped hook, duplicate hooks, wrong input type and no index must not break a neighbor.
                good = shell("malformed", "../../version/good.json")
                malformed = [good.replace(f'class="{hook}"', '') for hook in ["mizar-search-query", "mizar-search-status", "mizar-search-results"]]
                malformed += [good.replace('<label>', '<input class="mizar-search-query"><label>'),
                              good.replace('type="search"', 'type="button"'), good.replace('data-mizar-index="../../version/good.json"', '')]
                controls += [control.replace('id="malformed"', f'id="malformed-{i}"') for i, control in enumerate(malformed)]
                controls += [shell("neighbor", "../../version/good.json")]
                sinks = '<p class="mizar-search-status" role="status"></p><ul class="mizar-search-results"></ul>'
                for name, replacement in zip(overlaps, [
                    '<ul class="mizar-search-status mizar-search-results" role="status"></ul>',
                    '<div class="mizar-search-status" role="status"><ul class="mizar-search-results"></ul></div>',
                    '<ul class="mizar-search-results"><li><p class="mizar-search-status" role="status"></p></li></ul>',
                ]):
                    controls.append(shell(name).replace(sinks, replacement))
                nested = shell("malformed-nested", "../../version/good.json").replace('<label>Search <input type="search" class="mizar-search-query"></label>', '')
                controls += [nested[:-10] + shell("nested-neighbor", "../../version/good.json") + '</section>']
                for sink in ["results", "status"]:
                    for input_state in ["complete", "missing"]:
                        name = sink + "-" + input_state
                        outer = shell("isolation-" + name, "../../version/good.json")
                        if input_state == "missing":
                            outer = outer.replace('<label>Search <input type="search" class="mizar-search-query"></label>', '')
                        inner = shell("nested-" + name, "../../version/good.json")
                        if sink == "results":
                            outer = outer.replace('<ul class="mizar-search-results"></ul>', '<ul class="mizar-search-results"><li>' + inner + '</li></ul>')
                        else:
                            outer = outer.replace('<p class="mizar-search-status" role="status"></p>', '<div class="mizar-search-status" role="status">' + inner + '</div>')
                        controls.append(outer)
                script = ASSERTIONS.replace("EXPECTED", json.dumps({"items": items, "hostile": hostile}, ensure_ascii=True).replace("<", "\\u003c")).replace("BAD", json.dumps(failed)).replace("OVERLAPS", json.dumps(overlaps))
                data = ('<!doctype html><html lang=en-US><meta charset=utf-8><link rel=stylesheet href=/assets/style.css>'
                        + ''.join(controls) + '<script src=/assets/search.js></script><script>' + script + '</script></html>').encode()
                content_type = "text/html; charset=utf-8"
            elif path == "/version/redirect.json":
                self.send_response(302)
                self.send_header("Location", "/version/good.json")
                self.end_headers()
                return
            elif path == "/version/http-failed.json":
                self.send_error(503)
                return
            elif path.startswith("/version/") and path.endswith(".json") and (path[9:-5] in cases or path[9:-5] in overlaps):
                data, content_type = cases["good" if path[9:-5] in overlaps else path[9:-5]], "application/json"
            elif path.startswith("/assets/") and path[8:] in ["style.css", "search.js"]:
                data = (ASSETS / path[8:]).read_bytes()
                content_type = "text/css" if path.endswith(".css") else "application/javascript"
            elif path == "/version/" + hostile["href"].split("#")[0]:
                data = ('<!doctype html><p id="' + html.escape(hostile["fqn"], quote=True) + '">local destination</p>').encode()
                content_type = "text/html; charset=utf-8"
            else:
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    snapshots = []
    try:
        for _ in range(2):
            with tempfile.TemporaryDirectory(prefix="search-chrome-") as profile:
                command = [args.chrome, "--headless=new", "--disable-gpu", "--disable-dev-shm-usage", "--disable-background-networking", "--lang=en-US", "--no-proxy-server", f"--user-data-dir={profile}", "--virtual-time-budget=20000", "--timeout=30000", "--dump-dom", f"http://127.0.0.1:{server.server_port}/nested/module/page.html"]
                result = subprocess.run(command, capture_output=True, text=True, timeout=60,
                                        env={**os.environ, "TZ": "UTC", "LANG": "en_US.UTF-8"})
                assert result.returncode == 0, result.stderr
                match = re.search(r'<pre id="search-result">(.*?)</pre>', result.stdout, re.S)
                assert match, "browser did not finish\n" + result.stderr[-1000:]
                report = json.loads(html.unescape(match[1]))
                assert report["ok"], report
                snapshots.append(report["dom"])
        assert snapshots[0] == snapshots[1], "terminal result DOM differs"
        assert "/version/http-failed.json" in requests and "/version/redirect.json" in requests
        assert all("/version/" + name + ".json" not in requests for name in overlaps), "overlapping control loaded index"
        print(f"six search witness families passed; {len(cases)} supplied indexes; {version}; repeated DOM identical")
    finally:
        server.shutdown()


if __name__ == "__main__":
    main()
