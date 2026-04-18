let x = 0;
outer: while (x < 3) {
  x += 1;
  switch (x) {
    case 1:
      continue outer;
    case 2:
      debugger;
      break outer;
    default:
      x = 9;
  }
}
x;
