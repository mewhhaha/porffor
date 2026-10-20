var state = { value: 1 };

function inspect() {}

var argumentsList = new Proxy([], {
  get: function(target, key, receiver) {
    state.value = "s";
    return Reflect.get(target, key, receiver);
  }
});

inspect.apply(null, argumentsList);
state.value + 1 === "s1";
