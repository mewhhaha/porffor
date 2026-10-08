function tag(strings) { return strings; }
var constructor = Function('tag', 'return class { value = tag`context`; };')(tag);
var anotherConstructor = Function('tag', 'return class { value = tag`context`; };')(tag);
var first = new constructor();
var again = new constructor();
var other = new anotherConstructor();
var generator = Function('tag', 'return function*(){ yield tag`context`; yield tag`context`; };')(tag);
var generator1 = generator();
var generator2 = generator();
var firstSite = generator1.next().value;
var secondSite = generator1.next().value;
var sameFirstSite = generator2.next().value;
var sameSecondSite = generator2.next().value;
var staticFactory = Function('tag', 'return function(){ return class { static field = tag`static`; static { this.block = tag`block`; } }; };')(tag);
var separateStaticFactory = Function('tag', 'return function(){ return class { static field = tag`static`; static { this.block = tag`block`; } }; };')(tag);
var staticFirst = staticFactory();
var staticAgain = staticFactory();
var staticOther = separateStaticFactory();
var foreign = __lilaCreateRealm();
var foreignStaticFactory = foreign.global.Function('tag', 'return function(){ return class { static field = tag`static`; static { this.block = tag`block`; } }; };')(tag);
var foreignStaticFirst = foreignStaticFactory();
var foreignStaticAgain = foreignStaticFactory();
first.value === again.value
  && first.value !== other.value
  && firstSite !== secondSite
  && firstSite === sameFirstSite
  && secondSite === sameSecondSite
  && first.value[0] === 'context'
  && staticFirst.field === staticAgain.field
  && staticFirst.block === staticAgain.block
  && staticFirst.field !== staticOther.field
  && staticFirst.block !== staticOther.block
  && staticFirst.field !== staticFirst.block
  && foreignStaticFirst.field === foreignStaticAgain.field
  && foreignStaticFirst.block === foreignStaticAgain.block
  && staticFirst.field !== foreignStaticFirst.field
  && Object.getPrototypeOf(staticFirst.field) === Array.prototype
  && Object.getPrototypeOf(staticFirst.block) === Array.prototype
  && Object.getPrototypeOf(foreignStaticFirst.field) === foreign.global.Array.prototype
  && Object.getPrototypeOf(foreignStaticFirst.block) === foreign.global.Array.prototype;
