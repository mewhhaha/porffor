function fn(object) {
  let callbacks = [];
  for (let key in object) {
    callbacks.push(function () {
      return key;
    });
  }

  let index = 0;
  for (let expected in object) {
    if (expected !== callbacks[index]()) {
      throw expected;
    }
    index++;
  }
}

fn({ a: 1, b: 2, c: 3 });
true;
