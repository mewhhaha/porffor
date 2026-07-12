let observedNewTarget;
function A() {
  observedNewTarget = new.target;
  return {};
}

let B = A.bind(null);
function N() {}
let D = N.bind(null);
let C = B.bind(null);

Reflect.construct(B, [], D);
let throughUnrelatedBound = observedNewTarget === D && observedNewTarget !== N;
Reflect.construct(C, [], D);
let throughNestedUnrelatedBound = observedNewTarget === D && observedNewTarget !== N;

if (!(throughUnrelatedBound && throughNestedUnrelatedBound)) {
  throw "bound newTarget identity";
}

true;
