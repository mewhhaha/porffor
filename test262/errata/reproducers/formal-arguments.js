// An explicit formal named arguments prevents the Annex B outer copy.
(function (arguments) {
  var before = arguments;
  { function arguments() {} }
  print(JSON.stringify([before, arguments]));
})(42);
