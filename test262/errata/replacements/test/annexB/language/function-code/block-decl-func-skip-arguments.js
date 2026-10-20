// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Annex B copies an eligible block function to the implicit arguments binding
esid: sec-web-compat-functiondeclarationinstantiation
flags: [noStrict]
info: |
    FunctionDeclarationInstantiation leaves paramNames as the BoundNames of
    formals. An implicit arguments binding is added to paramBindings instead.
    Annex B tests paramNames, so an implicit arguments binding does not prevent
    copying a block function named arguments to the enclosing variable environment.
    An explicit formal named arguments does prevent that copy.
---*/

// Simple parameters
(function() {
  assert.sameValue(arguments.toString(), "[object Arguments]");
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(typeof arguments, "function");
  assert.sameValue(arguments(), undefined);
}());

// Single named parameter
(function(x) {
  assert.sameValue(arguments.toString(), "[object Arguments]");
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(typeof arguments, "function");
  assert.sameValue(arguments(), undefined);
}());

// Non-simple parameters
(function(..._) {
  assert.sameValue(arguments.toString(), "[object Arguments]");
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(typeof arguments, "function");
  assert.sameValue(arguments(), undefined);
}());

// An explicit formal named arguments prevents the outer Annex B copy.
(function(arguments) {
  assert.sameValue(arguments, 42);
  {
    assert.sameValue(arguments(), undefined);
    function arguments() {}
    assert.sameValue(arguments(), undefined);
  }
  assert.sameValue(arguments, 42);
}(42));
