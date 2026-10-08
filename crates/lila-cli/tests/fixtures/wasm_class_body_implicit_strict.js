// ClassBody is unconditionally strict: implicit-global assignment inside a
// constructor, method, getter or setter throws ReferenceError even when the
// surrounding script is sloppy (this file has no "use strict" directive).
var results = [];

class StrictBody {
  constructor() {
    try {
      undeclaredCtor = 1;
      results.push("ctor-nothrow");
    } catch (error) {
      results.push(error instanceof ReferenceError ? "ctor-ref" : "ctor-other");
    }
  }

  method() {
    try {
      undeclaredMethod = 1;
      results.push("method-nothrow");
    } catch (error) {
      results.push(error instanceof ReferenceError ? "method-ref" : "method-other");
    }
  }

  static smethod() {
    try {
      undeclaredStatic = 1;
      results.push("smethod-nothrow");
    } catch (error) {
      results.push(error instanceof ReferenceError ? "smethod-ref" : "smethod-other");
    }
  }

  get value() {
    try {
      undeclaredGetter = 1;
      results.push("get-nothrow");
    } catch (error) {
      results.push(error instanceof ReferenceError ? "get-ref" : "get-other");
    }
    return 0;
  }

  set value(next) {
    try {
      undeclaredSetter = 1;
      results.push("set-nothrow");
    } catch (error) {
      results.push(error instanceof ReferenceError ? "set-ref" : "set-other");
    }
  }
}

var instance = new StrictBody();
instance.method();
StrictBody.smethod();
instance.value;
instance.value = 1;

results.join(",") === "ctor-ref,method-ref,smethod-ref,get-ref,set-ref";
