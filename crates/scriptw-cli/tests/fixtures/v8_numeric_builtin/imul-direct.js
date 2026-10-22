(function workload() {
  var state = 123456789;
    for (var i = 0; i < 50000; ++i) {
    state = (Math.imul(state, 1664525) + 1013904223) >>> 0;
  }
  print(String(state));
})();
