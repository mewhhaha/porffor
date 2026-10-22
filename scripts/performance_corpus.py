#!/usr/bin/env python3
"""Deterministic, checked input families for the ordinary performance runner.

These are JavaScript benchmark fixtures, never a compiler/runtime implementation.
All invocations include construction. Read-pass scaling estimates marginal
traversal cost; it is not a fabricated timer that excludes setup inside JS.
"""
import argparse
import json
from pathlib import Path


def workloads(rows=(256, 1024, 4096), widths=(4,), passes=(0, 1, 4)):
    entries = []

    def add(name, body, expected, family, **parameters):
        entries.append(dict(file=name + '.js', source='(function () {\n' + body + '\n})();\n',
                            stdout=str(expected) + '\n', work=family,
                            family=family, parameters=parameters))

    add('control-empty', 'print(0);', 0, 'fixed setup')
    add('control-loop', 'var i; for (i = 0; i < 10000; i++) {} print(i);', 10000, 'empty loop')
    add('control-scalar', 'var x = 19; print(x * x + 3);', 364, 'scalar completion')
    add('control-return', 'function value() { return 364; } print(value());', 364, 'scalar return')
    for count in rows:
        for width in widths:
            values = ', '.join('i + ' + str(j) for j in range(width))
            for read_passes in passes:
                setup = f'''var rows = [];
for (var i = 0; i < {count}; i++) rows.push({{name: 'row-' + i, values: [{values}]}});
var sum = rows.length + rows[rows.length - 1].values[{width - 1}];'''
                read = f'''
for (var pass = 0; pass < {read_passes}; pass++) {{
  for (var column = 0; column < {width}; column++) {{
    for (var i = 0; i < rows.length; i++) sum += rows[i].values[column];
  }}
}}
print(sum);'''
                expected = 2 * count + width - 2 + read_passes * (
                    width * count * (count - 1) // 2 + count * width * (width - 1) // 2)
                add(f'rows-n{count}-w{width}-p{read_passes}', setup + read, expected,
                    'retained rows; construction plus read-pass scaling',
                    rows=count, width=width, read_passes=read_passes,
                    boundary='whole invocation; construction always included')

    for fields in (2, 4, 8, 16):
        properties = ', '.join(f'k{j}: i + {j}' for j in range(fields))
        reads = ' + '.join(f'rows[i].k{j}' for j in range(fields))
        add(f'fields-{fields}', f'''var rows = [];
for (var i = 0; i < 512; i++) rows.push({{{properties}}});
var sum = 0;
for (var i = 0; i < rows.length; i++) sum += {reads};
print(sum);''', fields * 512 * 511 // 2 + 512 * fields * (fields - 1) // 2,
            'ordinary literal field count', fields=fields, rows=512)

    add('property-own', 'var o = {x: 7}; var s = 0; for (var i=0;i<1024;i++) s += o.x; print(s);',
        7168, 'ordinary fixed own key')
    add('property-dynamic', "var o={x:7,y:11}; var s=0; for(var i=0;i<1024;i++) s+=o[i%2?'x':'y']; print(s);",
        9216, 'ordinary changing key')
    add('property-inherited', 'var o=Object.create({x:7}); var s=0; for(var i=0;i<1024;i++) s+=o.x; print(s);',
        7168, 'prototype lookup')
    add('property-getter', 'var n=0; var o={get x(){n++;return 7;}}; var s=0; for(var i=0;i<1024;i++) s+=o.x; print(s+n);',
        8192, 'accessor lookup and effects')
    add('property-proxy', 'var n=0; var o=new Proxy({x:7},{get:function(t,k){n++;return t[k];}}); var s=0; for(var i=0;i<1024;i++) s+=o.x; print(s+n);',
        8192, 'proxy lookup and effects')
    add('call-zero', 'function f(){return 7;} var s=0; for(var i=0;i<1024;i++) s+=f(); print(s);',
        7168, 'zero-arity calls')
    add('call-two', 'function f(x,y){return x+y;} var s=0; for(var i=0;i<1024;i++) s+=f(i,1); print(s);',
        524800, 'two-arity calls')
    add('call-closure', 'function outer(){var n=0;return function(){return ++n;};} var f=outer(); var s=0; for(var i=0;i<1024;i++) s+=f(); print(s);',
        524800, 'captured mutable binding')
    add('call-indirect', 'var fs=[function(x){return x+1;},function(x){return x+2;}]; var s=0; for(var i=0;i<1024;i++) s+=fs[i%2](i); print(s);',
        525312, 'polymorphic calls')
    add('call-recursive', 'function f(n){return n<2?n:f(n-1)+f(n-2);} var s=0; for(var i=0;i<16;i++) s+=f(12); print(s);',
        2304, 'recursive calls')
    for count in (16, 64, 256):
        args = ','.join("'ab'" for _ in range(count))
        add(f'concat-{count}', f"var s=0; for(var i=0;i<16;i++) s+='x'.concat({args}).length; print(s);",
            16 * (1 + count * 2), 'multiargument string concat', arguments=count, repeats=16)
    for width in (1, 16, 64):
        add(f'search-{width}', f"var h='a'.repeat(1024)+'b'; var n='a'.repeat({width})+'b'; var s=0; for(var i=0;i<8;i++) s+=h.indexOf(n); print(s);",
            (1024 - width) * 8, 'repetitive string search', haystack=1025, needle=width+1)
    add('regexp-exec', "var r=/a+b/g; var s=0; for(var i=0;i<128;i++){r.lastIndex=0;var m=r.exec('xxaaaabyy');s+=m.index+m[0].length;} print(s);",
        896, 'regexp search and capture results')
    add('map-string', "var m=new Map();var keys=[];for(var i=0;i<256;i++){var k='key-'+i;keys.push(k);m.set(k,i);}var s=0;for(var p=0;p<4;p++)for(var i=0;i<256;i++)s+=m.get(keys[i]);print(s);",
        130560, 'reused string collection keys', keys=256, read_passes=4)
    add('set-churn', 'var s=new Set();for(var i=0;i<256;i++)s.add(i);for(var i=0;i<128;i++)s.delete(i);for(var i=0;i<128;i++)s.add(i);var n=0;s.forEach(function(v){n+=v;});print(n+s.size);',
        32896, 'collection deletion and iteration history')
    add('typed-copy', 'var a=new Uint32Array(1024);for(var i=0;i<a.length;i++)a[i]=i;var b=new Uint32Array(1024);b.set(a);var s=0;for(var i=0;i<b.length;i++)s+=b[i];print(s);',
        523776, 'compatible typed-array copy', elements=1024)
    add('typed-sort', 'var a=new Float64Array(256);for(var i=0;i<a.length;i++)a[i]=255-i;a.sort();print(a[0]+a[255]);',
        255, 'default typed-array numeric sort', elements=256)
    data = [{'x': i, 's': 'row-' + str(i)} for i in range(256)]
    text = json.dumps(data, separators=(',', ':'))
    add('json-parse', f'var a=JSON.parse({json.dumps(text)}); print(a.length+a[255].x+a[0].s.length);',
        516, 'JSON parse ordinary records', rows=256)
    add('json-stringify', 'var a=[];for(var i=0;i<256;i++)a.push({x:i,s:"row-"+i});print(JSON.stringify(a).length);',
        len(text), 'JSON stringify ordinary records', rows=256)
    add('intl-reuse', "var f=new Intl.NumberFormat('en-US');var s=0;for(var i=0;i<32;i++)s+=f.format(i).length;print(s);",
        sum(len(str(i)) for i in range(32)), 'reused Intl NumberFormat', values=32)
    names = [entry['file'] for entry in entries]
    if len(set(names)) != len(names):
        raise ValueError('duplicate generated workload')
    return entries


def write_corpus(destination, rows, widths, passes):
    entries = workloads(rows, widths, passes)
    destination.mkdir(parents=True, exist_ok=False)
    (destination / 'startup.js').write_text('print("scriptw-startup-ok");\n')
    for entry in entries:
        (destination / entry['file']).write_text(entry['source'])
    plan = dict(schema='scriptw-v8-workloads-v1',
                startup=dict(file='startup.js', stdout='scriptw-startup-ok\n'),
                execution=[{key: value for key, value in entry.items() if key != 'source'} for entry in entries])
    (destination / 'manifest.json').write_text(json.dumps(plan, indent=2) + '\n')
    return plan


def bounded_list(value, maximum):
    values = tuple(int(n) for n in value.split(','))
    if not values or len(set(values)) != len(values) or any(n < 0 or n > maximum for n in values):
        raise ValueError('distinct values within 0..' + str(maximum) + ' required')
    return values


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output-dir', type=Path, required=True)
    parser.add_argument('--rows', default='256,1024,4096')
    parser.add_argument('--widths', default='4')
    parser.add_argument('--passes', default='0,1,4')
    args = parser.parse_args()
    try:
        rows = bounded_list(args.rows, 16384)
        widths = bounded_list(args.widths, 16)
        passes = bounded_list(args.passes, 16)
        if 0 in rows or 0 in widths:
            raise ValueError('row and width counts must be positive')
        if len(rows) * len(widths) * len(passes) > 128:
            raise ValueError('at most 128 row/width/pass combinations per corpus')
        plan = write_corpus(args.output_dir, rows, widths, passes)
    except (OSError, ValueError) as error:
        parser.exit(1, f'performance-corpus: {error}\n')
    print(f"Wrote {len(plan['execution'])} checked workloads to {args.output_dir}")


if __name__ == '__main__':
    main()
