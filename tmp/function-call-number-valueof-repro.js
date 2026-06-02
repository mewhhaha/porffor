function f() {
  return this.valueOf();
}

f.call(101);
