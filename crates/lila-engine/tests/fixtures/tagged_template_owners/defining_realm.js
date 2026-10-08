function tag(strings) { return strings; }
var foreign = __lilaCreateRealm();
var localBody = Function('tag', 'return tag`realm`;');
var foreignBody = foreign.global.Function('tag', 'return tag`realm`;');
var anotherForeignBody = foreign.global.Function('tag', 'return tag`realm`;');
var local = localBody(tag);
var other = foreignBody(tag);
var otherAgain = foreignBody(tag);
var separate = anotherForeignBody(tag);
function runForeignScript() { return foreign.evalScript('(function(s){return s;})`realm`;'); }
var script1 = runForeignScript();
var script2 = runForeignScript();
var foreignClosure = foreign.evalScript('(function(){ return (function(s){return s;})`realm`; })');
var closureValue = foreignClosure();
local !== other
  && other === otherAgain
  && other !== separate
  && script1 !== script2
  && closureValue === foreignClosure()
  && Object.getPrototypeOf(local) === Array.prototype
  && Object.getPrototypeOf(other) === foreign.global.Array.prototype
  && Object.getPrototypeOf(other.raw) === foreign.global.Array.prototype
  && Object.getPrototypeOf(script1) === foreign.global.Array.prototype
  && Object.getPrototypeOf(closureValue) === foreign.global.Array.prototype
  && Object.isFrozen(other)
  && Object.isFrozen(other.raw);
