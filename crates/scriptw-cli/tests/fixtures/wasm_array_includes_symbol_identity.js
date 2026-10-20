var a = Symbol("1");
var b = Symbol("1");

var storedHit = [a].includes(a);
var separateMiss = [a].includes(b);
var immediateMiss = [Symbol("1")].includes(Symbol("1"));

storedHit === true && separateMiss === false && immediateMiss === false;
