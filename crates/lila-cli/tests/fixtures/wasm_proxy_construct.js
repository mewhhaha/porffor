class Foo {
  constructor(arg) {
    this.arg = arg;
  }
}

var FooTarget = new Proxy(Foo, {});
var FooProxy = new Proxy(FooTarget, {
  construct: null
});

var foo = new FooProxy(1);

class Bar extends Foo {
  get isBar() {
    return true;
  }
}

var bar = Reflect.construct(FooProxy, [2], Bar);

foo instanceof Foo
  && foo.arg === 1
  && bar instanceof Bar
  && bar.arg === 2
  && bar.isBar === true;
