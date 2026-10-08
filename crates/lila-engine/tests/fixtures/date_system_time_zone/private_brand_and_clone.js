function check(value, label) { if (!value) throw label; }
function typeError(body, label) {
  var caught;
  try { body(); } catch (error) { caught = error; }
  check(caught !== undefined && Object.getPrototypeOf(caught) === TypeError.prototype, label);
}
var date = new Date(1234);
check(Object.getOwnPropertyNames(date).length === 0, 'Date has no public slot key');
var reads = 0;
Object.defineProperty(date, '$DateValue', {get() { reads++; throw 'public slot read'; }});
check(date.getTime() === 1234 && date.setTime(5678) === 5678, 'private slot ignores public accessor');
check(date.getTime() === 5678 && reads === 0, 'public slot accessor remains unobserved');
var fake = {$DateValue:1234};
Object.defineProperty(fake, 'valueOf', {get() { reads++; throw 'fake coercion'; }});
for (var method of ['getTime', 'valueOf', 'getFullYear', 'setTime', 'toISOString']) {
  typeError(function() { Date.prototype[method].call(fake, 0); }, 'real brand ' + method);
}
typeError(function() { Date.prototype.getTime.call(Date.prototype); }, 'prototype lacks DateValue');
typeError(function() { Date.prototype.getTime.call(new Proxy(new Date(0), {})); }, 'Proxy lacks DateValue');
var hooks = 0;
for (var epoch of [5678, NaN]) {
  var source = new Date(epoch);
  Object.setPrototypeOf(source, null);
  for (var key of [Symbol.toPrimitive, 'valueOf', 'toString', 'getTime', '$DateValue']) {
    Object.defineProperty(source, key, {get() { hooks++; throw 'clone hook'; }});
  }
  var clone = new Date(source);
  check(Object.getPrototypeOf(clone) === Date.prototype, 'clone gets constructor prototype');
  check(Object.is(clone.getTime(), epoch), 'clone preserves exact DateValue');
}
check(hooks === 0 && reads === 0, 'brand and clone do not coerce');
var generic = {valueOf() { return 0; }, toISOString() { return 'generic'; }};
check(Date.prototype.toJSON.call(generic) === 'generic', 'toJSON remains generic');
check(Date.prototype[Symbol.toPrimitive].call({toString() { return 'generic'; }}, 'string') === 'generic', 'primitive conversion remains generic');
print('ok');
262;
