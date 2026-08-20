function declaration() {
  return declaration;
}

let originalDeclaration = declaration;
declaration.marker = 17;
if (originalDeclaration() !== originalDeclaration) throw "declaration lost its identity";
if (originalDeclaration.marker !== 17) throw "declaration lost its property";

function rebound() {
  return rebound;
}

let heldRebound = rebound;
rebound = function replacement() {
  return "replacement";
};
if (heldRebound() !== rebound) throw "declaration did not observe reassignment";

function outerHelper() {
  return 3;
}

function recursiveDeclaration(count) {
  if (count === 0) return outerHelper();
  return recursiveDeclaration(count - 1);
}

if (recursiveDeclaration(2) !== 3) throw "recursive declaration lost its outer helper";

let privateName = "outer";
let namedExpression = function privateName() {
  return privateName;
};
if (namedExpression() !== namedExpression) throw "named expression lost its identity";
namedExpression.marker = 23;
if (namedExpression().marker !== 23) throw "named expression lost its property";
if (privateName !== "outer") throw "named expression did not shadow the outer name";

let inferredName = function () {
  return inferredName;
};
let heldInferredName = inferredName;
inferredName = "reassigned outer binding";
if (heldInferredName() !== inferredName) throw "inferred name became a private binding";

function captureOuter(value) {
  return function namedCapture() {
    return value;
  };
}

if (captureOuter(11)() !== 11) throw "named expression lost its outer capture";

let nestedSelf = function namedSelf() {
  return function nestedClosure() {
    return namedSelf;
  };
};
if (nestedSelf()() !== nestedSelf) throw "nested closure lost named expression identity";

function createNamedExpression() {
  return function createdExpression() {
    return createdExpression;
  };
}

let firstCreatedExpression = createNamedExpression();
let secondCreatedExpression = createNamedExpression();
if (firstCreatedExpression === secondCreatedExpression) {
  throw "named expression identity leaked across evaluations";
}
if (
  firstCreatedExpression() !== firstCreatedExpression ||
  secondCreatedExpression() !== secondCreatedExpression
) {
  throw "named expression lost per-evaluation identity";
}

true;
