var symA = Symbol("A");
var symB = Symbol("B");
var obj = {};

obj[symA] = 1;
Object.preventExtensions(obj);
obj[symA] = 2;
obj[symB] = 1;

obj[symA] === 2
  && delete obj[symA] === true
  && obj[symA] === undefined
  && obj[symB] === undefined;
