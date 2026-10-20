function check(condition, message) {
  if (!condition) throw message;
}

var sum = 0;
function oneCase(name, runner, base) {
  var log = [];
  class TestIterator extends Iterator {
    next() {
      log.push("next");
      return { done: true };
    }
  }
  var iter = new TestIterator();
  Object.defineProperty(iter, "return", {
    get: function() {
      log.push("get: return");
      return function() {
        log.push("call: return");
        return {};
      };
    }
  });
  var threw = null;
  try {
    runner(iter);
  } catch (e) {
    threw = e;
  }
  check(threw instanceof TypeError, name + " throws TypeError");
  check(
    log.length === 2 && log[0] === "get: return" && log[1] === "call: return",
    name + " closes via GetMethod: " + log.join(",")
  );
  sum += base;
}

oneCase("every", function(it) { it.every(1); }, 1);
oneCase("some", function(it) { it.some(1); }, 2);
oneCase("find", function(it) { it.find(1); }, 4);
oneCase("reduce", function(it) { it.reduce(1); }, 8);
sum;
