// Standalone diagnostic; output is an observation, not a Test262 pass.
(function () {
  var before = typeof arguments;
  { function arguments() {} }
  print(JSON.stringify([before, typeof arguments]));
})();
