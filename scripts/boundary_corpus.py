"""Exact-output fixtures for fresh precompiled and isolated same-Realm calls.

Separate cohort from performance_corpus: no historical ordinary timing changes.
"""
import base64
import gzip
import hashlib
import json
from pathlib import Path

GUEST_WARMUPS = 10


def call_program(prefix, samples, expected):
    """One initialization and separately observed calls; output outside spans."""
    source = prefix + f'''var result;
for (var warmup = 0; warmup < {GUEST_WARMUPS}; warmup++) {{
  result = batch(warmup % 8);
  print('warmup:' + warmup + ':' + result);
}}
print('scriptw:call:ready');
for (var sample = 0; sample < {samples}; sample++) {{
  print('scriptw:call:start:' + sample);
  result = batch(sample % 8);
  print('scriptw:call:end:' + sample);
  print('sample:' + sample + ':' + result);
}}
}})();
'''
    lines = [f'warmup:{i}:{expected(i % 8, i)}' for i in range(GUEST_WARMUPS)]
    lines.append('scriptw:call:ready')
    for i in range(samples):
        lines.extend((f'scriptw:call:start:{i}', f'scriptw:call:end:{i}',
                      f'sample:{i}:{expected(i % 8, GUEST_WARMUPS + i)}'))
    return source, '\n'.join(lines) + '\n'


def workloads(samples=7):
    if type(samples) is not int or not 7 <= samples <= 100:
        raise ValueError('call samples must be 7..100')
    result = []

    def add(name, setup, body, expected, partition='tuning'):
        prefix = '(function () {\n' + setup + '\nfunction batch(seed) {\n' + body + '\n}\n'
        fresh = prefix + 'print(batch(0));\n})();\n'
        source, stdout = call_program(prefix, samples, expected)
        result.append(dict(name=name, partition=partition,
            fresh_source=fresh, fresh_stdout=str(expected(0, 0)) + '\n',
            calls_source=source, calls_stdout=stdout,
            guest_warmups=GUEST_WARMUPS, call_samples=samples,
            boundary='one setup; one compiled batch(seed) call per observed span; exact numeric results checked outside spans'))

    add('calls', 'function step(x) { return x + 1; }',
        'var s=0;for(var i=0;i<1024;i++)s+=step(i+seed);return s;',
        lambda seed, call: 1024 * 1025 // 2 + 1024 * seed)
    add('closures', 'var next=(function(){var n=0;return function(){return ++n;};})();',
        'var s=0;for(var i=0;i<512;i++)s+=next();return s;',
        lambda seed, call: 512 * (1024 * call + 513) // 2)
    add('objects', '',
        'var a=[];for(var i=0;i<512;i++)a.push({x:i+seed,y:2*i+1});'
        'var s=0;for(var i=0;i<a.length;i++)s+=a[i].x+a[i].y;return s;',
        lambda seed, call: 3 * 512 * 511 // 2 + 512 + 512 * seed)
    add('arrays', '',
        'var a=[];for(var i=0;i<2048;i++)a.push(i+seed);'
        'var s=0;for(var i=0;i<a.length;i++)s+=a[i];return s;',
        lambda seed, call: 2048 * 2047 // 2 + 2048 * seed)
    add('strings', '',
        "var n=0;for(var i=0;i<128;i++){var s='a'.repeat(64)+seed;"
        "n+=s.slice(1).length+s.indexOf(String(seed));}return n;",
        lambda seed, call: 128 * 128)
    add('map', '',
        "var m=new Map();for(var i=0;i<512;i++)m.set('k'+i,i+seed);"
        "var s=0;for(var i=0;i<512;i++)s+=m.get('k'+i);return s;",
        lambda seed, call: 512 * 511 // 2 + 512 * seed)
    add('json', '',
        'var a=[];for(var i=0;i<256;i++)a.push({x:i+seed,y:i*2});'
        'var b=JSON.parse(JSON.stringify(a));var s=0;'
        'for(var i=0;i<b.length;i++)s+=b[i].x+b[i].y;return s;',
        lambda seed, call: 3 * 256 * 255 // 2 + 256 * seed)
    add('polymorphic-calls', 'var fs=[function(x){return x+1;},function(x){return x+2;}];',
        'var s=0;for(var i=0;i<768;i++)s+=fs[i%2](i+seed);return s;',
        lambda seed, call: 768 * 767 // 2 + 1152 + 768 * seed, 'holdout')
    add('array-holes', '',
        'var a=[];for(var i=0;i<768;i++)a[i*2]=i+seed;'
        'var s=0;for(var i=0;i<a.length;i++)if(i in a)s+=a[i];return s;',
        lambda seed, call: 768 * 767 // 2 + 768 * seed, 'holdout')
    add('object-mutation', '',
        'var a={x:seed,y:7};var s=0;for(var i=0;i<512;i++){'
        'a.x=i+seed;delete a.y;a.y=i+1;s+=a.x+a.y;}return s;',
        lambda seed, call: 512 * 512 + 512 * seed, 'holdout')
    return result


def supported_react_fixture(repository, samples=7):
    """Restore the exact previously supported synchronous probe, offline.

    No streaming/browser/polyfill support is inferred. Fresh uses the exact
    historical bundle. Calls wrap exact upstream production CommonJS bodies
    with ordinary source-level module factories; React initializes only once.
    """
    evidence = Path(repository) / 'docs/rust-rewrite/react-ssr-20261020.json'
    receipt = json.loads(evidence.read_text())

    def asset(name, expected_sha, run=2):
        path = f'repo:target/react-ssr-compatibility-20261020-run{run}/' + name
        matches = [a for a in receipt['assets'] if a['path'] == path]
        if len(matches) != 1:
            raise ValueError('missing unique supported React asset: ' + name)
        item = matches[0]
        if item['encoding'] != 'gzip+base64':
            raise ValueError('unsupported React asset encoding')
        compressed = base64.b64decode(item['data'], validate=True)
        if hashlib.sha256(compressed).hexdigest() != item['gzip_sha256']:
            raise ValueError('React compressed asset checksum mismatch')
        raw = gzip.decompress(compressed)
        if len(raw) != item['bytes'] or hashlib.sha256(raw).hexdigest() != expected_sha or item['sha256'] != expected_sha:
            raise ValueError('React raw asset checksum/size mismatch')
        return raw.decode('utf8')

    if type(samples) is not int or not 7 <= samples <= 100:
        raise ValueError('call samples must be 7..100')
    fresh_source = asset('synchronous-legacy.js', '5757c0b1ea427c86f16c5fce51cacda52368a4cfa13fb8a52bbc9441d17f7dfa')
    fresh_stdout = asset('synchronous-legacy-node-reference.stdout', '40651e125813a2dde6688e9bc81ebe8b97594b61a3c0adef7278b99e3022b421')
    upstream = {
        'react/cjs/react.production.js': '0b5529981841afa1b58c76ba85c29bb681ff5f07cd8cde6a10b967d3915b8e1c',
        'react-dom/cjs/react-dom.production.js': '7a05d7f7bb2b02aae84018a88d4815a2181eef48d2cb8172d403eaa2b131eefd',
        'react-dom/cjs/react-dom-server-legacy.browser.production.js': 'a470866a7a27a7f4fa69ef04e9bfdea0514fc9defdcc48bda43516b9ae2dfadd',
        'react/LICENSE': 'da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93',
        'react-dom/LICENSE': 'da6d3703ed11cbe42bd212c725957c98da23cbff1998c05fa4b3d976d1a58e93',
    }
    restored = {name: asset('node_modules/' + name, sha, run=1) for name, sha in upstream.items()}
    entry = asset('entry.js', '0f2918bc9888f22ab147edfa83b969c0abf05f51c3967d3db5d0b3f557395aac', run=1)
    imports = "import React from 'react';\nimport {renderToString} from 'react-dom/server.browser';\n"
    render_loop = "for (const [name, tree] of cases) print(JSON.stringify({react:React.version, name, html:renderToString(tree)}));\n"
    if not entry.startswith(imports) or entry.count(render_loop) != 1 or not entry.endswith(render_loop):
        raise ValueError('pinned React entry structure changed')

    def factory(binding, source, dependencies):
        # These are regular JS functions over exact retained upstream bodies,
        # not a source evaluator or a product module/compiler implementation.
        dispatch = ''.join(f"if(name==={json.dumps(name)})return {value};" for name, value in dependencies)
        return (f'var {binding}=(function(){{var exports={{}};function require(name){{' + dispatch +
                "throw new Error('unexpected pinned fixture dependency: '+name);}" +
                '\n(function(exports,require){\n' + source + '\n})(exports,require);return exports;})();\n')

    prefix = '(function(){\n'
    prefix += factory('React', restored['react/cjs/react.production.js'], [])
    prefix += factory('ReactDOM', restored['react-dom/cjs/react-dom.production.js'], [('react', 'React')])
    prefix += factory('ReactServer', restored['react-dom/cjs/react-dom-server-legacy.browser.production.js'],
                      [('react', 'React'), ('react-dom', 'ReactDOM')])
    prefix += 'var renderToString=ReactServer.renderToString;\n'
    prefix += entry[len(imports):-len(render_loop)]
    prefix += '''function batch(seed) {
var rendered=[];
for(var i=0;i<cases.length;i++)rendered.push({react:React.version,name:cases[i][0],html:renderToString(cases[i][1])});
return JSON.stringify(rendered);
}
'''
    expected = json.dumps([json.loads(line) for line in fresh_stdout.splitlines()], separators=(',', ':'), ensure_ascii=False)
    calls_source, calls_stdout = call_program(prefix, samples, lambda seed, call: expected)
    return dict(name='react-ssr-synchronous', partition='holdout',
        fresh_source=fresh_source, fresh_stdout=fresh_stdout,
        calls_source=calls_source, calls_stdout=calls_stdout,
        guest_warmups=GUEST_WARMUPS, call_samples=samples,
        restored_assets=dict(restored, **{'entry.js': entry}), upstream_sha256=upstream,
        source_evidence=str(evidence.relative_to(repository)),
        source_evidence_sha256=hashlib.sha256(evidence.read_bytes()).hexdigest(),
        supported_scope='React 19.3.1 + ReactDOM 19.3.1 unmodified synchronous production renderer alias; exact three HTML outputs. Fresh includes initialization; calls initialize exact upstream modules/trees once then render the three cases per batch. No streaming/public browser entry claim.')
