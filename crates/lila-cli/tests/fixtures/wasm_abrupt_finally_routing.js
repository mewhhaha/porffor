var trace = "";
var tdzFinalized = false;
var returnFinalized = false;

function triggerTdz() {
  try {
    {
      x;
      let x = 1;
    }
  } finally {
    tdzFinalized = true;
  }
}

try {
  triggerTdz();
} catch (error) {
  if (!(error instanceof ReferenceError)) throw "TDZ throw value replaced";
}
if (!tdzFinalized) throw "TDZ finalizer skipped";

var calledThrow = {};
function throwArgument(value) {
  throw value;
}

try {
  try {
    throwArgument(calledThrow);
  } finally {
    trace += "E";
  }
} catch (error) {
  if (error !== calledThrow) throw "called throw value replaced";
}

var outerThrow = {};
try {
  try {
    throw outerThrow;
  } finally {
    trace += "F";
    try {
      throw "inner";
    } catch (error) {
      if (error !== "inner") throw "inner catch value replaced";
      trace += "C";
    }
  }
} catch (error) {
  if (error !== outerThrow) throw "outer throw value replaced";
}

var catchThrow = {};
try {
  try {
    throw "caught";
  } catch (error) {
    if (error !== "caught") throw "catch value replaced";
    trace += "C";
    try {
      throw catchThrow;
    } finally {
      trace += "F";
    }
  }
} catch (error) {
  if (error !== catchThrow) throw "catch finalizer throw value replaced";
}

function returnThroughFinally() {
  try {
    return 7;
  } finally {
    returnFinalized = true;
  }
}

if (returnThroughFinally() !== 7) throw "return value replaced";
if (!returnFinalized) throw "return finalizer skipped";

while (true) {
  try {
    break;
  } finally {
    trace += "B";
  }
}

for (var i = 0; i < 1; i += 1) {
  try {
    continue;
  } finally {
    trace += "N";
  }
}

try {
  while (true) {
    break;
  }
  trace += "I";
} finally {
  trace += "O";
}

if (trace !== "EFCCFBNIO") throw trace;
true;
