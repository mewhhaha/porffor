if (typeof unselected !== "undefined") throw "unselected Annex B binding was not undefined";
if (false) {
  function unselected() {
    return 1;
  }
}

if (true) {
  function selected() {
    return 7;
  }
}
if (selected() !== 7) throw "selected Annex B binding was not updated";
if (globalThis.selected !== selected) throw "Annex B global was not mirrored";

var initialF, currentF;
if (true) function f() {
  initialF = f;
  f = 123;
  currentF = f;
  return "decl";
}
f();
if (initialF() !== "decl") throw "inner Annex B binding was not initialized to the function";
if (currentF !== 123) throw "inner Annex B binding was not mutable";
if (f() !== "decl") throw "outer Annex B binding did not retain the function";

function functionOwner(flag) {
  var before = typeof local;
  if (flag) {
    function local() {
      return 9;
    }
  }
  return before === "undefined" && local() === 9;
}

function catchOwner() {
  try {
    throw 1;
  } catch (shadow) {
    {
      function shadow() {
        return 11;
      }
    }
    if (shadow !== 1) throw "Annex B copy overwrote catch binding";
  }
  return shadow() === 11;
}

function duplicateOwner() {
  {
    function duplicate() {
      return 1;
    }
    function duplicate() {
      return 2;
    }
  }
  return duplicate() === 2;
}

function switchOwner(value) {
  switch (value) {
    case 0:
      function shared() {
        return 3;
      }
      break;
    default:
      function shared() {
        return 4;
      }
  }
  return shared() === 4;
}

if (!functionOwner(true)) throw "function owner Annex B binding failed";
if (!catchOwner()) throw "catch shadow Annex B binding failed";
if (!duplicateOwner()) throw "duplicate Annex B declaration binding failed";
if (!switchOwner(0)) throw "switch CaseBlock Annex B binding failed";

function blockOwnerNestedCandidateBlocked() {
  {
    function blockOwnerValue() {
      return 1;
    }
    {
      function blockOwnerValue() {
        return 2;
      }
    }
  }
  if (blockOwnerValue() !== 1) throw "nested Block Annex B candidate overwrote direct Block owner";
  return true;
}

function blockOwnerIfCandidateBlocked() {
  {
    function blockOwnerIfValue() {
      return 1;
    }
    if (true) function blockOwnerIfValue() {
      return 2;
    }
  }
  if (blockOwnerIfValue() !== 1) throw "nested if Annex B candidate overwrote direct Block owner";
  return true;
}

function functionBodyNestedCandidateEligible() {
  function functionBodyValue() {
    return 1;
  }
  {
    function functionBodyValue() {
      return 2;
    }
  }
  if (functionBodyValue() !== 2) throw "nested Block Annex B candidate did not update FunctionBody owner";
  return true;
}

function switchCaseBlockNestedCandidateBlocked() {
  switch (0) {
    case 0:
      function switchCaseBlockValue() {
        return 1;
      }
      {
        function switchCaseBlockValue() {
          return 2;
        }
      }
      break;
  }
  if (switchCaseBlockValue() !== 1) throw "nested case Block Annex B candidate overwrote CaseBlock owner";
  return true;
}

if (!blockOwnerNestedCandidateBlocked()) throw "nested Block candidate guard failed";
if (!blockOwnerIfCandidateBlocked()) throw "nested if candidate guard failed";
if (!functionBodyNestedCandidateEligible()) throw "FunctionBody candidate eligibility failed";
if (!switchCaseBlockNestedCandidateBlocked()) throw "switch CaseBlock candidate guard failed";
true;
