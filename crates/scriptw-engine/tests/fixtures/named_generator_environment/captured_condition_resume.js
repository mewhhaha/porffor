var flip;
function* activation(parameter) {
 let local = parameter;
 flip = function() { local = 0; };
 if (local) { yield local; local += arguments[0]; }
 return local;
}
var iterator = activation(1);
var first = iterator.next();
print('first=' + first.value + ',done=' + first.done);
flip();
var second = iterator.next();
print('second=' + second.value + ',done=' + second.done);
second.value === 1 && second.done;
