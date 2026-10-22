(function () {
  function check(value, expected, name) { if (value !== expected) throw new Error(name); }
  var first = {}, second = {}, key = Symbol('key');
  check(first === second, false, 'fresh ordinary objects');
  first.x = 17; first[key] = 19;
  check(second.x, undefined, 'empty headers do not share inserted data');
  check(second[key], undefined, 'empty headers do not share symbols');
  Object.defineProperty(second, 'x', {get: function () { return 23; }, configurable: true});
  check(first.x, 17, 'data survives other accessor publication');
  check(second.x, 23, 'accessor publication');
  delete first.x; first.y = 29; first.x = 31;
  check(Object.getOwnPropertyNames(first).join(','), 'y,x', 'delete reinsert order');
  check(second.y, undefined, 'other object remains independent');
  class Private {
    #value = 37;
    get() { return this.#value; }
    set(value) { this.#value = value; }
  }
  var one = new Private(), two = new Private();
  one.set(41);
  check(one.get(), 41, 'private storage write');
  check(two.get(), 37, 'private storage does not alias');
  function save() { return arguments; }
  var a = save(), b = save();
  check(a === b, false, 'fresh arguments objects');
  check(a.length, 0, 'empty arguments length');
  a[0] = 43;
  check(b[0], undefined, 'arguments storage does not alias');
  function rest(...args) { return args; }
  var r = rest(), s = rest();
  check(r === s, false, 'fresh rest arrays');
  r.push(47);
  check(s.length, 0, 'rest storage does not alias');
  var bound = save.bind(null), c = bound(), d = bound();
  check(c === d, false, 'bound empty arguments freshness');
  check(c.length + d.length, 0, 'bound empty arguments lengths');
  var lists = [];
  var proxy = new Proxy(save, {apply: function (target, receiver, args) {
    lists.push(args);
    var returned = Reflect.apply(target, receiver, args);
    args.push(53);
    return returned;
  }});
  var e = proxy(), f = proxy();
  check(lists[0] === lists[1], false, 'proxy receives fresh argument arrays');
  check(e.length + f.length, 0, 'proxy input mutation does not alter callee arguments');
  check(lists[0].length + lists[1].length, 2, 'proxy mutations remain local');
  var retained = [];
  for (var i = 0; i < 1024; i++) { var o = {}; o.index = i; retained.push(o); }
  var sum = 0;
  for (var i = 0; i < retained.length; i++) sum += retained[i].index;
  check(sum, 523776, 'growth and retained objects');
  print('shared-empty-storage:ok');
})();
