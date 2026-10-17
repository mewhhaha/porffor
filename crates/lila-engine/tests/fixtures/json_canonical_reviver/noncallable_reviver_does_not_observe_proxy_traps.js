var traps = 0;
var ignored = new Proxy({}, {
  get: function () { traps++; throw "unexpected reviver lookup"; },
  getPrototypeOf: function () { traps++; throw "unexpected reviver prototype"; }
});
var input = {[Symbol.toPrimitive]: function (hint) {
  print("coerce:" + hint);
  return '[{"value":1},null,true]';
}};
var result = JSON.parse(input, ignored);
print(result.length + ":" + result[0].value + ":" + result[1] + ":" + result[2] + ":" + traps);
