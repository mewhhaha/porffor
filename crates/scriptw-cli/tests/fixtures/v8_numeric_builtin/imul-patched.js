(function workload() {
  var state = 123456789;
  var owner = { imul: function (a, b) { return (a * b) | 0; } };
  for (var i = 0; i < 50000; ++i) {
    state = (owner.imul(state, 1664525) + 1013904223) >>> 0;
  }
  print(String(state));
})();
