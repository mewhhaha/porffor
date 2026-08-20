function check(value, label) {
  if (!value) {
    throw "static exponentiation fixture failed: " + label;
  }
}

check(2 ** 16 === 65536, "power 16");
check(2 ** 31 === 2147483648, "power 31");
check(2 ** 32 + 1 === 4294967297, "power plus one");
check((3 ** 2) ** 2 === 81, "nested power");
check((-2) ** 3 === -8, "negative base odd");
check((-2) ** 2 === 4, "negative base even");

var parts = "undefined is not a function".split(undefined, 2 ** 32 + 1);
check(parts.length === 1, "split undefined limit length");
check(parts[0] === "undefined is not a function", "split undefined limit value");

true;
