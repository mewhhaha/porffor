function tag(strings) { return strings; }
var outer = Function('tag', 'return async function(){ await 0; return tag`async`; };');
var first = outer(tag);
var sibling = outer(tag);
var separate = Function('tag', 'return async function(){ await 0; return tag`async`; };')(tag);
Promise.all([first(), first(), sibling(), separate()]).then(function(values) {
  if (values[0] !== values[1] || values[0] !== values[2] || values[0] === values[3])
    throw new Error('async template owner');
  if (!Object.isFrozen(values[0]) || !Object.isFrozen(values[0].raw))
    throw new Error('async frozen template arrays');
  print('async-template-ok');
}).catch(function(error) { print('async-error:' + error); });
