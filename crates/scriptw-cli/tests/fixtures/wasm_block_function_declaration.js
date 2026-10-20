let value = 0;

if (true) {
  value = nested(3);

  function nested(input) {
    return input + 4;
  }
}

value === 7;
