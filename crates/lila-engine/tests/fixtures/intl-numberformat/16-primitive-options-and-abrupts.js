function check(value, message) { if (!value) throw new Error(message); }
const other = __lilaCreateRealm().global;
let observed = 0;
Object.defineProperty(other.Number.prototype, 'localeMatcher', {configurable:true,get(){observed++;return 'lookup';}});
check(other.Intl.NumberFormat.supportedLocalesOf('en-US',1).join() === 'en-US' && observed === 1, 'filter primitive box Realm');
check(new other.Intl.NumberFormat('en-US',1).format(1) === '1' && observed === 2, 'constructor primitive box Realm');
for (const value of [true,'',1n,Symbol('option')]) check(new Intl.NumberFormat('en-US',value).format(1) === '1', 'primitive option');
let later = 0;
try { new other.Intl.NumberFormat(null,{get style(){later++;}}); throw 'missing error'; }
catch (error) { check(error.constructor === other.TypeError && later === 0, 'canonical locales error Realm/order'); }
const marker = {};
try { new Intl.NumberFormat('en-US',{get roundingMode(){throw marker;},get compactDisplay(){later++;}}); throw 'missing option throw'; }
catch (error) { check(error === marker && later === 0, 'option abrupt identity'); }
const format = new Intl.NumberFormat('en-US').format;
let count = 0, reached = false;
try { format({[Symbol.toPrimitive](hint){check(hint === 'number','hint');count++;throw undefined;}}); reached = true; }
catch (error) { check(error === undefined,'undefined rejection identity'); }
check(!reached && count === 1, 'one abrupt primitive conversion');
const invalid = ['\ud800','\udc00','1_0','\u0000','0x','Infinityx'];
for (const input of invalid) check(format(input) === 'NaN', 'invalid String numeric syntax');
check(format('\u00a0\u2028 12 \u3000') === '12', 'UTF16 numeric whitespace');
const raw = new Intl.NumberFormat('en-US', {useGrouping:false});
check(raw.format(Object(9007199254740993n)) === '9007199254740993', 'boxed BigInt');
let receiverChecked = 0;
try { Number.prototype.toLocaleString.call({}, {get length(){receiverChecked++;}}); throw 'missing receiver throw'; }
catch (error) { check(error instanceof TypeError && receiverChecked === 0,'Number receiver before locale'); }
try { BigInt.prototype.toLocaleString.call(1, {get length(){receiverChecked++;}}); throw 'missing receiver throw'; }
catch (error) { check(error instanceof TypeError && receiverChecked === 0,'BigInt receiver before locale'); }
print('ok primitive options and abrupts');
