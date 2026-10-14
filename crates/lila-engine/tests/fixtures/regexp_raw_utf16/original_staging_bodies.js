// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Provides both:

    - An error class to avoid false positives when testing for thrown exceptions
    - A function to explicitly throw an exception using the Test262Error class
defines: [Test262Error, $DONOTEVALUATE]
---*/


function Test262Error(message) {
  if (!(this instanceof Test262Error)) return new Test262Error(message);
  this.message = message || "";
}

Test262Error.prototype.toString = function () {
  return "Test262Error: " + this.message;
};

Test262Error.thrower = function (message) {
  throw new Test262Error(message);
};

function $DONOTEVALUATE() {
  throw "Test262: This statement should not be evaluated.";
}

// Copyright (C) 2017 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Collection of assertion functions used throughout test262
defines:
  - assert
  - compareArray
  - formatIdentityFreeValue
  - formatSimpleValue
  - isNegativeZero
  - isPrimitive
---*/


function isNegativeZero(value) {
  return value === 0 && 1 / value === -Infinity;
}

function isPrimitive(value) {
  return !value || (typeof value !== 'object' && typeof value !== 'function');
}

function formatIdentityFreeValue(value) {
  switch (value === null ? 'null' : typeof value) {
    case 'string':
      return typeof JSON !== "undefined" ? JSON.stringify(value) : '"' + value + '"';
    case 'bigint':
      return String(value) + "n";
    case 'number':
      if (isNegativeZero(value)) return '-0';
      // falls through
    case 'boolean':
    case 'undefined':
    case 'null':
      return String(value);
  }
}

function formatSimpleValue(value) {
  var basic = formatIdentityFreeValue(value);
  if (basic) return basic;
  try {
    return String(value);
  } catch (err) {
    if (err.name === 'TypeError') {
      return Object.prototype.toString.call(value);
    }
    throw err;
  }
}

function assert(mustBeTrue, message) {
  if (mustBeTrue === true) {
    return;
  }

  if (message === undefined) {
    message = 'Expected true but got ' + assert._toString(mustBeTrue);
  }
  throw new Test262Error(message);
}

assert._isSameValue = function (a, b) {
  if (a === b) {
    // Handle +/-0 vs. -/+0
    return a !== 0 || 1 / a === 1 / b;
  }

  // Handle NaN vs. NaN
  return a !== a && b !== b;
};

assert.sameValue = function (actual, expected, message) {
  try {
    if (assert._isSameValue(actual, expected)) {
      return;
    }
  } catch (error) {
    throw new Test262Error(message + ' (_isSameValue operation threw) ' + error);
    return;
  }

  if (message === undefined) {
    message = '';
  } else {
    message += ' ';
  }

  message += 'Expected SameValue(«' + assert._toString(actual) + '», «' + assert._toString(expected) + '») to be true';

  throw new Test262Error(message);
};

assert.notSameValue = function (actual, unexpected, message) {
  if (!assert._isSameValue(actual, unexpected)) {
    return;
  }

  if (message === undefined) {
    message = '';
  } else {
    message += ' ';
  }

  message += 'Expected SameValue(«' + assert._toString(actual) + '», «' + assert._toString(unexpected) + '») to be false';

  throw new Test262Error(message);
};

assert.throws = function (expectedErrorConstructor, func, message) {
  var expectedName, actualName;
  if (typeof func !== "function") {
    throw new Test262Error('assert.throws requires two arguments: the error constructor ' +
      'and a function to run');
    return;
  }
  if (message === undefined) {
    message = '';
  } else {
    message += ' ';
  }

  try {
    func();
  } catch (thrown) {
    if (typeof thrown !== 'object' || thrown === null) {
      message += 'Thrown value was not an object!';
      throw new Test262Error(message);
    } else if (thrown.constructor !== expectedErrorConstructor) {
      expectedName = expectedErrorConstructor.name;
      actualName = thrown.constructor.name;
      if (expectedName === actualName) {
        message += 'Expected a ' + expectedName + ' but got a different error constructor with the same name';
      } else {
        message += 'Expected a ' + expectedName + ' but got a ' + actualName;
      }
      throw new Test262Error(message);
    }
    return;
  }

  message += 'Expected a ' + expectedErrorConstructor.name + ' to be thrown but no exception was thrown at all';
  throw new Test262Error(message);
};

assert.compareArray = function (actual, expected, message) {
  message = message === undefined ? '' : message;

  if (typeof message === 'symbol') {
    message = message.toString();
  }

  if (isPrimitive(actual)) {
    assert(false, "Actual argument [" + actual + "] shouldn't be primitive. " + String(message));
  } else if (isPrimitive(expected)) {
    assert(false, "Expected argument [" + expected + "] shouldn't be primitive. " + String(message));
  }
  var result = compareArray(actual, expected);
  if (result) return;

  var format = compareArray.format;
  assert(false, "Actual " + format(actual) + " and expected " + format(expected) + " should have the same contents. " + String(message));
};

function compareArray(a, b) {
  if (b.length !== a.length) {
    return false;
  }
  for (var i = 0; i < a.length; i++) {
    if (!assert._isSameValue(b[i], a[i])) {
      return false;
    }
  }
  return true;
}

compareArray.format = function (arrayLike) {
  return "[" + Array.prototype.map.call(arrayLike, String).join(", ") + "]";
};

assert._formatIdentityFreeValue = formatIdentityFreeValue;

assert._toString = formatSimpleValue;

// Copyright (C) 2017 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Deprecated now that compareArray is defined in assert.js.
allow_unused: true
---*/

// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- raw unicode.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== standalone ====

assert.compareArray(eval(`/\uD83D\uDC38/u`).exec("\u{1F438}"),
              ["\u{1F438}"]);

// no unicode flag
assert.compareArray(eval(`/\uD83D\uDC38/`).exec("\u{1F438}"),
              ["\u{1F438}"]);

// escaped (lead)
assert.sameValue(eval(`/\\uD83D\uDC38/u`).exec("\u{1F438}"),
         null);
assert.sameValue(eval(`/\\u{D83D}\uDC38/u`).exec("\u{1F438}"),
         null);

// escaped (trail)
assert.sameValue(eval(`/\uD83D\\uDC38/u`).exec("\u{1F438}"),
         null);
assert.sameValue(eval(`/\uD83D\\u{DC38}/u`).exec("\u{1F438}"),
         null);

// escaped (lead), no unicode flag
assert.compareArray(eval(`/\\uD83D\uDC38/`).exec("\u{1F438}"),
              ["\u{1F438}"]);

// escaped (trail), no unicode flag
assert.compareArray(eval(`/\uD83D\\uDC38/`).exec("\u{1F438}"),
              ["\u{1F438}"]);

// ==== RegExp constructor ====

assert.compareArray(new RegExp("\uD83D\uDC38", "u").exec("\u{1F438}"),
              ["\u{1F438}"]);

// no unicode flag
assert.compareArray(new RegExp("\uD83D\uDC38", "").exec("\u{1F438}"),
              ["\u{1F438}"]);

// escaped(lead)
assert.sameValue(new RegExp("\\uD83D\uDC38", "u").exec("\u{1F438}"),
         null);
assert.sameValue(new RegExp("\\u{D83D}\uDC38", "u").exec("\u{1F438}"),
         null);

// escaped(trail)
assert.sameValue(new RegExp("\uD83D\\uDC38", "u").exec("\u{1F438}"),
         null);
assert.sameValue(new RegExp("\uD83D\\u{DC38}", "u").exec("\u{1F438}"),
         null);

// escaped(lead), no unicode flag
assert.compareArray(new RegExp("\\uD83D\uDC38", "").exec("\u{1F438}"),
              ["\u{1F438}"]);

// escaped(trail), no unicode flag
assert.compareArray(new RegExp("\uD83D\\uDC38", "").exec("\u{1F438}"),
              ["\u{1F438}"]);

// ==== ? ====

assert.compareArray(eval(`/\uD83D\uDC38?/u`).exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(eval(`/\uD83D\uDC38?/u`).exec(""),
              [""]);

assert.compareArray(eval(`/\uD83D\uDC38?/u`).exec("\uD83D"),
              [""]);

// no unicode flag
assert.compareArray(eval(`/\uD83D\uDC38?/`).exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.sameValue(eval(`/\uD83D\uDC38?/`).exec(""),
         null);

assert.compareArray(eval(`/\uD83D\uDC38?/`).exec("\uD83D"),
              ["\uD83D"]);

// escaped (lead)
assert.sameValue(eval(`/\\uD83D\uDC38?/u`).exec("\u{1F438}"),
         null);
assert.sameValue(eval(`/\\uD83D\uDC38?/u`).exec(""),
         null);

assert.compareArray(eval(`/\\uD83D\uDC38?/u`).exec("\uD83D"),
              ["\uD83D"]);

// escaped (trail)
assert.sameValue(eval(`/\uD83D\\uDC38?/u`).exec("\u{1F438}"),
         null);
assert.sameValue(eval(`/\uD83D\\uDC38?/u`).exec(""),
         null);

assert.compareArray(eval(`/\uD83D\\uDC38?/u`).exec("\uD83D"),
              ["\uD83D"]);

// escaped (lead), no unicode flag
assert.compareArray(eval(`/\\uD83D\uDC38?/`).exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.sameValue(eval(`/\\uD83D\uDC38?/`).exec(""),
         null);

assert.compareArray(eval(`/\\uD83D\uDC38?/`).exec("\uD83D"),
              ["\uD83D"]);

// escaped (trail), no unicode flag
assert.compareArray(eval(`/\uD83D\\uDC38?/`).exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.sameValue(eval(`/\uD83D\\uDC38?/`).exec(""),
         null);

assert.compareArray(eval(`/\uD83D\\uDC38?/`).exec("\uD83D"),
              ["\uD83D"]);

// ==== RegExp constructor, ? ====

assert.compareArray(new RegExp("\uD83D\uDC38?", "u").exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(new RegExp("\uD83D\uDC38?", "u").exec(""),
              [""]);

assert.compareArray(new RegExp("\uD83D\uDC38?", "u").exec("\uD83D"),
              [""]);

// no unicode flag
assert.compareArray(new RegExp("\uD83D\uDC38?", "").exec("\u{1F438}"),
              ["\u{1F438}"]);
assert.sameValue(new RegExp("\uD83D\uDC38?", "").exec(""),
         null);

assert.compareArray(new RegExp("\uD83D\uDC38?", "").exec("\uD83D"),
              ["\uD83D"]);

// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- raw unicode.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== standalone ====

assert.compareArray(eval(`/[\uD83D\uDC38]/u`).exec("\u{1F438}"),
              ["\u{1F438}"]);

// no unicode flag
assert.compareArray(eval(`/[\uD83D\uDC38]/`).exec("\u{1F438}"),
              ["\uD83D"]);

// escaped (lead)
assert.sameValue(eval(`/[\\uD83D\uDC38]/u`).exec("\u{1F438}"),
         null);
assert.sameValue(eval(`/[\\u{D83D}\uDC38]/u`).exec("\u{1F438}"),
         null);

// escaped (trail)
assert.sameValue(eval(`/[\uD83D\\uDC38]/u`).exec("\u{1F438}"),
         null);
assert.sameValue(eval(`/[\uD83D\\u{DC38}]/u`).exec("\u{1F438}"),
         null);

// escaped (lead), no unicode flag
assert.compareArray(eval(`/[\\uD83D\uDC38]/`).exec("\u{1F438}"),
              ["\uD83D"]);

// escaped (trail), no unicode flag
assert.compareArray(eval(`/[\uD83D\\uDC38]/`).exec("\u{1F438}"),
              ["\uD83D"]);

// ==== RegExp constructor ====

assert.compareArray(new RegExp("[\uD83D\uDC38]", "u").exec("\u{1F438}"),
              ["\u{1F438}"]);

// no unicode flag
assert.compareArray(new RegExp("[\uD83D\uDC38]", "").exec("\u{1F438}"),
              ["\uD83D"]);

// escaped(lead)
assert.sameValue(new RegExp("[\\uD83D\uDC38]", "u").exec("\u{1F438}"),
         null);
assert.sameValue(new RegExp("[\\u{D83D}\uDC38]", "u").exec("\u{1F438}"),
         null);

// escaped(trail)
assert.sameValue(new RegExp("[\uD83D\\uDC38]", "u").exec("\u{1F438}"),
         null);
assert.sameValue(new RegExp("[\uD83D\\u{DC38}]", "u").exec("\u{1F438}"),
         null);

// escaped(lead), no unicode flag
assert.compareArray(new RegExp("[\\uD83D\uDC38]", "").exec("\u{1F438}"),
              ["\uD83D"]);

// escaped(trail), no unicode flag
assert.compareArray(new RegExp("[\uD83D\\uDC38]", "").exec("\u{1F438}"),
              ["\uD83D"]);

print('raw-regexp-ok'); true;
