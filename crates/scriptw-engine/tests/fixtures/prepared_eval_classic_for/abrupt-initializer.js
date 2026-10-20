var marker = {};
var count = 37;
var trace = [];
var received;
function fail() { trace.push('initialize'); throw marker; }
try {
  eval("for (var count = fail(), later = trace.push('later');;) { trace.push('body'); break; }");
} catch (error) { received = error; }
if (received !== marker || count !== 37 || later !== undefined
    || trace.join(',') !== 'initialize') throw new Error('initializer throw identity');
var scope = {set count(value) { trace.push('set:' + value); throw marker; }};
received = undefined;
try {
  with (scope) {
    eval("for (var count = 7, later = trace.push('later');;) { trace.push('body'); break; }");
  }
} catch (error) { received = error; }
if (received !== marker || count !== 37 || later !== undefined
    || trace.join(',') !== 'initialize,set:7') throw new Error('publication throw identity');
print('ok');
true;
