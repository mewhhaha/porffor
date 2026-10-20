function check(condition, message) {
  if (!condition) throw message;
}

var iterator = [1, 2].values();
iterator["$ScriptwStaticGeneratorIterator"] = true;
[] = iterator;

var first = iterator.next();
check(!first.done && first.value === 1, "source property spoofed Array iterator completion");

var target = {
  "$ScriptwStaticGenerator.values": function() {
    return 17;
  },
};
check(
  target["$ScriptwStaticGenerator.values"]() === 17,
  "synthetic spelling was not an ordinary source property",
);
true;
