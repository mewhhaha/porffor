const instant = new Temporal.Instant(0n);
const method = Temporal.Instant.prototype.toLocaleString;
let trace = '';
const locales = {
  get length(){trace += 'length,'; return 1;},
  get 0(){trace += 'locale,'; return {toString(){trace += 'locale string,'; return 'en-US';}};}
};
const options = {
  get localeMatcher(){trace += 'matcher,'; return 'lookup';},
  get timeZone(){trace += 'zone,'; return {toString(){trace += 'zone string,'; return 'UTC';}};},
  get second(){trace += 'second,'; return '2-digit';},
  get fractionalSecondDigits(){trace += 'fraction,'; return {valueOf(){trace += 'fraction number,'; return 3;}};},
  get dateStyle(){trace += 'date style,';},
  get timeStyle(){trace += 'time style';}
};
method.call(instant, locales, options);
if (trace !== 'length,locale,locale string,matcher,zone,zone string,second,fraction,fraction number,date style,time style') throw trace;

for (const marker of [undefined, null, 'sentinel', 7, Symbol('sentinel'), 1n, {}, new TypeError('sentinel')]) {
  for (const phase of ['locale length', 'locale string', 'option getter', 'option conversion']) {
    let later = 0;
    let caught = false;
    const abruptLocales = phase === 'locale length' ? {get length(){throw marker;}} : phase === 'locale string' ? [{toString(){throw marker;}}] : 'en-US';
    const abruptOptions = {
      get localeMatcher(){if (phase === 'option getter') throw marker; if (phase === 'option conversion') return {toString(){throw marker;}}; later++; return 'lookup';},
      get timeZone(){later++; return 'UTC';}
    };
    try { method.call(instant, abruptLocales, abruptOptions); } catch (error) {
      if (!Object.is(error, marker)) throw 'abrupt identity';
      caught = true;
    }
    if (!caught || later !== 0) throw 'abrupt before later options';
  }
}
for (const primitive of [0, true, '', 1n, Symbol('options')]) {
  if (method.call(instant, 'en-US', primitive) !== new Intl.DateTimeFormat('en-US', primitive).format(instant)) throw 'primitive options';
}
let stored = 1;
method.call(instant, 'en-US', {get localeMatcher(){stored = {value:'changed'}; return 'lookup';}});
if (stored.value !== 'changed') throw 'locale callback mutation';
print('ok');
