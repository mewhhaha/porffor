(function workload() {
  var state = 123456789;
  var box = { valueOf: function () { return state; } };
  for (var i = 0; i < 5000; ++i) {
    state = (Math.imul(box, 1664525) + 1013904223) >>> 0;
  }
  print(String(state));
})();
