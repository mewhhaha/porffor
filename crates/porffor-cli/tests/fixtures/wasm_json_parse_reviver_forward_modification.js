let arrayCalls = 0;
let objectCalls = 0;

JSON.parse("[1,[]]", function (key, value) {
  arrayCalls = arrayCalls + 1;
  JSON.stringify(value);
  if (value === 1) {
    this[1].push("barf");
  }
  return this[key];
});

JSON.parse('{"p":1,"q":{}}', function (key, value) {
  objectCalls = objectCalls + 1;
  JSON.stringify(value);
  if (value === 1) {
    this.q.added = "barf";
  }
  return this[key];
});

arrayCalls === 4 && objectCalls === 4;
