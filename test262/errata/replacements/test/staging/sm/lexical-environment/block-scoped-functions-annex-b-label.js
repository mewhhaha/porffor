// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: Labelled function declarations follow pinned lexical scope and outer-copy rules
esid: sec-static-semantics-lexicallyscopeddeclarations
info: |
    LexicallyScopedDeclarations of LabelledItem : FunctionDeclaration includes
    the function; LabelledItem : Statement contributes no lexical declarations.
    BlockDeclarationInstantiation initializes direct labelled functions before
    statement evaluation. Annex B outer-copy candidates are functions directly
    contained in block/case statement lists, excluding functions under labels.
    Top-level labelled chains use the separate TopLevelVarScopedDeclarations.
---*/

function expectSyntaxError(str) {
  assert.throws(SyntaxError, function() {
    eval(str);
  });

  assert.throws(SyntaxError, function() {
    eval('"use strict";' + str);
  });
}

function expectSloppyPass(str) {
  eval(str);

  assert.throws(SyntaxError, function() {
    eval('"use strict";' + str);
  });
}

expectSloppyPass(`l: function f1() {}`);
expectSloppyPass(`l0: l: function f1() {}`);
expectSloppyPass(`{ f1(); l: function f1() {} }`);
expectSloppyPass(`{
  assert.throws(ReferenceError, function() { f1(); });
  l0: l: function f1() {}
  assert.throws(ReferenceError, function() { f1(); });
}`);
expectSloppyPass(`{
  assert.sameValue(f1(), 42);
  l: function f1() { return 42; }
  assert.sameValue(f1(), 42);
}
assert.throws(ReferenceError, function() { f1(); });`);
expectSloppyPass(`eval("fe(); l: function fe() {}")`);
expectSyntaxError(`if (1) l: function f2() {}`);
expectSyntaxError(`if (1) {} else l: function f3() {}`);
expectSyntaxError(`do l: function f4() {} while (0)`);
expectSyntaxError(`while (0) l: function f5() {}`);
expectSyntaxError(`for (;;) l: function f6() {}`);
expectSloppyPass(`switch (1) { case 1: l: function f7() {} }`);
expectSloppyPass(`switch (1) {
  case 1: assert.sameValue(f8(), 'f8');
  case 2: l: function f8() { return 'f8'; }
    assert.sameValue(f8(), 'f8');
}
assert.throws(ReferenceError, function() { f8(); });`);
