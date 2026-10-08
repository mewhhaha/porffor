function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var foreign=__lilaCreateRealm().global, Other=foreign.Intl.DisplayNames;
var foreignTypeError=foreign.TypeError, foreignRangeError=foreign.RangeError;
var local=new Intl.DisplayNames('en-US',{type:'region'}), remote=new Other('en-US',{type:'region'});
same(Object.getPrototypeOf(remote),Other.prototype,'foreign family prototype');same(Object.getPrototypeOf(Other),foreign.Function.prototype,'foreign constructor');
same(Object.getPrototypeOf(Other.prototype.resolvedOptions.call(local)),foreign.Object.prototype,'resolved called-function Realm');
same(Object.getPrototypeOf(Intl.DisplayNames.prototype.resolvedOptions.call(remote)),Object.prototype,'borrowed local result Realm');
same(Object.getPrototypeOf(Other.supportedLocalesOf.call({},['en-US'])),foreign.Array.prototype,'static called-function Realm');
var boxed=0;Object.defineProperty(foreign.Number.prototype,'localeMatcher',{configurable:true,get(){boxed++;same(Object.getPrototypeOf(this),foreign.Number.prototype,'foreign primitive boxing');return 'lookup';}});
same(Other.supportedLocalesOf('en-US',7)[0],'en-US','foreign boxed options');same(boxed,1,'one foreign boxing Get');
foreign.TypeError=function PublicReplacement(){throw new Error('public constructor observed');};foreign.RangeError=function PublicReplacement(){throw new Error('public constructor observed');};foreign.Intl.DisplayNames=function PublicReplacement(){throw new Error('public DisplayNames observed');};
throws(foreignTypeError,function(){Other('en-US',{type:'region'});},'primordial requires new');
throws(foreignTypeError,function(){new Other('en-US',1);},'foreign strict options');
throws(foreignTypeError,function(){new Other('en-US',{});},'foreign required type');
throws(foreignRangeError,function(){new Other('en-US',{type:'wrong'});},'foreign invalid type');
var gets=0;throws(foreignTypeError,function(){Other.prototype.of.call({},new Proxy({},{get(){gets++;throw new Error('brand');}}));},'foreign brand');same(gets,0,'brand before reads');
throws(foreignRangeError,function(){Other.prototype.of.call(local,'USA');},'native rejection called-function Realm');
throws(RangeError,function(){Intl.DisplayNames.prototype.of.call(remote,'USA');},'local borrowed native error');
var code={toString(){var nested=Intl.DisplayNames.prototype.resolvedOptions.call(remote);same(Object.getPrototypeOf(nested),Object.prototype,'nested local method Realm');return 'USA';}};
throws(foreignRangeError,function(){Other.prototype.of.call(local,code);},'native error after nested Realm switch');
throws(foreignTypeError,function(){Other.prototype.of.call(local,Symbol('code'));},'ToString error called-function Realm');
print('ok called_function_realms');
262;
