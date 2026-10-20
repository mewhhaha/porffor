class DefaultArray extends Array {}

class ExplicitZero extends Array {
  constructor() { super(); }
}

class ExplicitOne extends Array {
  constructor(value) { super(value); }
}

class ExplicitMany extends Array {
  constructor(first, second) { super(first, second); }
}

class NormalExpression extends Array {
  constructor() { super(); ({ fake: true }); }
}

class ExplicitObjectReturn extends Array {
  constructor() { return { sentinel: true }; }
}

class ExplicitUndefinedReturn extends Array {
  constructor() { super(); return undefined; }
}

class ExplicitPrimitiveReturn extends Array {
  constructor() { super(); return 1; }
}




class DeeperArray extends DefaultArray {}

function objectPrototypeNewTarget() {}
var objectPrototype = { marker: true };
objectPrototypeNewTarget.prototype = objectPrototype;

function primitivePrototypeNewTarget() {}
primitivePrototypeNewTarget.prototype = 1;

var defaultZero = new DefaultArray();
var defaultOne = new DefaultArray(3);
var defaultMany = new DefaultArray(1, 2);
var explicitZero = new ExplicitZero();
var explicitOne = new ExplicitOne(3);
var explicitMany = new ExplicitMany(1, 2);
var normalExpression = new NormalExpression();
var explicitObjectReturn = new ExplicitObjectReturn();
var explicitUndefinedReturn = new ExplicitUndefinedReturn();
var primitiveReturnThrows = false;
try {
  new ExplicitPrimitiveReturn();
} catch (error) {
  primitiveReturnThrows = error instanceof TypeError;
}
var deeper = new DeeperArray(1, 2);
var objectPrototypeArray = Reflect.construct(Array, [1, 2], objectPrototypeNewTarget);
var primitivePrototypeArray = Reflect.construct(Array, [1, 2], primitivePrototypeNewTarget);

defaultZero.length === 0 &&
  defaultOne.length === 3 &&
  defaultMany.length === 2 && defaultMany[0] === 1 && defaultMany[1] === 2 &&
  explicitZero.length === 0 &&
  explicitOne.length === 3 &&
  explicitMany.length === 2 && explicitMany[0] === 1 && explicitMany[1] === 2 &&
  Array.isArray(normalExpression) && normalExpression instanceof NormalExpression &&
  explicitObjectReturn.sentinel === true && !Array.isArray(explicitObjectReturn) &&
  Array.isArray(explicitUndefinedReturn) && explicitUndefinedReturn instanceof ExplicitUndefinedReturn &&
  primitiveReturnThrows &&
  Object.getPrototypeOf(defaultMany) === DefaultArray.prototype && defaultMany instanceof DefaultArray &&
  Object.getPrototypeOf(explicitMany) === ExplicitMany.prototype && explicitMany instanceof ExplicitMany &&
  Object.getPrototypeOf(deeper) === DeeperArray.prototype && deeper instanceof DeeperArray && deeper instanceof DefaultArray &&
  Object.getPrototypeOf(objectPrototypeArray) === objectPrototype && objectPrototypeArray instanceof objectPrototypeNewTarget &&
  Object.getPrototypeOf(primitivePrototypeArray) === Array.prototype && primitivePrototypeArray instanceof Array;
