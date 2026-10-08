function tag(strings) { return strings; }
function makeBody() { return Function('tag', 'return function(){ return tag`life`; };'); }
var firstBody = makeBody();
var secondBody = makeBody();
var firstClosure = firstBody(tag);
var siblingClosure = firstBody(tag);
var secondClosure = secondBody(tag);
var first = firstClosure();
function makeEvalClosure() { return eval('(function(){ return tag`life`; })'); }
var evalClosure1 = makeEvalClosure();
var evalClosure2 = makeEvalClosure();
var evalFirst = evalClosure1();
function direct() { return eval('tag`life`;'); }
function indirect() { return (0, eval)('(function(s){return s;})`life`;'); }
var direct1 = direct();
var direct2 = direct();
var indirect1 = indirect();
var indirect2 = indirect();
first === firstClosure()
  && first === siblingClosure()
  && first !== secondClosure()
  && evalFirst === evalClosure1()
  && evalFirst !== evalClosure2()
  && direct1 !== direct2
  && indirect1 !== indirect2
  && direct1 !== indirect1
  && first[0] === 'life'
  && evalFirst[0] === 'life';
