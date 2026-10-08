function check(value, label) { if (!value) throw new Error(label); }
var parsers = [parseInt, Number.parseInt];
for (var p = 0; p < parsers.length; p++) {
  var parse = parsers[p], log = [], sentinel = {}, caught, didCatch = false;
  var radix = {get valueOf() { log.push('radix'); return function() { return 16; }; }};
  var input = {toString() { log.push('string'); throw sentinel; }};
  try { parse(input, radix); } catch (error) { didCatch = true; caught = error; }
  check(didCatch && caught === sentinel && log.join('|') === 'string',
        'string throw identity and skipped radix');
  log = []; didCatch = false; caught = sentinel;
  input = {toString() { log.push('string'); return '10'; }};
  radix = {valueOf() { log.push('radix'); throw undefined; }};
  try { parse(input, radix); } catch (error) { didCatch = true; caught = error; }
  check(didCatch && caught === undefined && log.join('|') === 'string|radix',
        'radix undefined throw identity');
  log = []; didCatch = false;
  radix = {valueOf() { log.push('radix'); return 16; }};
  try { parse(Symbol('input'), radix); } catch (error) { didCatch = error instanceof TypeError; }
  check(didCatch && log.length === 0, 'symbol input prevents radix conversion');
  didCatch = false;
  try { parse('10', 16n); } catch (error) { didCatch = error instanceof TypeError; }
  check(didCatch, 'BigInt radix retains ToNumber TypeError');
}
