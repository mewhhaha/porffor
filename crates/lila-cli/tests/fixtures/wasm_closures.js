function outer() {
  let x = 1;

  function inner() {
    return x + 1;
  }

  return inner();
}

let f = function (x) {
  return x + 1;
};

outer() + f(2);
