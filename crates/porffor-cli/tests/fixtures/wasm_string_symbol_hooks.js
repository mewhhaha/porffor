function IsHTMLDDA() {
  return null;
}

Object.defineProperty(IsHTMLDDA, "$IsHTMLDDA", {
  value: true,
  writable: false,
  enumerable: false,
  configurable: false
});

var $262 = {
  IsHTMLDDA: IsHTMLDDA
};

var total = 0;

function check(method, name, symbol, expectedArgs) {
  var target = $262.IsHTMLDDA;
  var gets = 0;
  Object.defineProperty(target, symbol, {
    get: function() {
      gets += 1;
      return function() {
        if (this !== target) throw name + " this";
        if (arguments.length !== expectedArgs) throw name + " argc";
        if (arguments[0] !== "") throw name + " arg0";
        return null;
      };
    },
    configurable: true
  });
  if (method.call("", target) !== null) throw name + " result";
  if (gets !== 1) throw name + " getter";
  total += gets;
}

check(String.prototype.match, "match", Symbol.match, 1);
check(String.prototype.matchAll, "matchAll", Symbol.matchAll, 1);
check(String.prototype.replace, "replace", Symbol.replace, 2);
check(String.prototype.replaceAll, "replaceAll", Symbol.replace, 2);
check(String.prototype.search, "search", Symbol.search, 1);
check(String.prototype.split, "split", Symbol.split, 2);

if (total !== 6) throw "total";

262;
