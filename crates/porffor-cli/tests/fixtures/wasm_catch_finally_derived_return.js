class CatchReturn extends class {} {
  constructor() {
    try {
      throw null;
    } catch (error) {
      return;
    } finally {
      super();
    }
  }
}

class TryReturn extends class {} {
  constructor() {
    try {
      return;
    } finally {
      super();
    }
  }
}

class CatchNormal extends class {} {
  constructor() {
    try {
      throw null;
    } catch (error) {
    } finally {
      super();
    }
  }
}

class ObjectReturn extends class {} {
  constructor() {
    try {
      throw null;
    } catch (error) {
      return { override: true };
    } finally {
      super();
    }
  }
}

class FinallyOverridesReturn extends class {} {
  constructor() {
    try {
      throw null;
    } catch (error) {
      return;
    } finally {
      throw "finally override";
    }
  }
}

var catchReturn = new CatchReturn();
var tryReturn = new TryReturn();
var catchNormal = new CatchNormal();
var objectReturn = new ObjectReturn();
var finallyOverrides = false;
try {
  new FinallyOverridesReturn();
} catch (error) {
  finallyOverrides = error === "finally override";
}

if (typeof catchReturn !== "object") throw "catch return";
if (typeof tryReturn !== "object") throw "try return";
if (typeof catchNormal !== "object") throw "catch normal";
if (objectReturn.override !== true) throw "object return";
if (!finallyOverrides) throw "finally override";

true;
