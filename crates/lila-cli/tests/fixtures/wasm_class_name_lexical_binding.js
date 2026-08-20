let heritageProbe;
let heritageSet;
let assignmentRhsEvaluated = false;

function Base() {}

class Declaration extends (
  heritageProbe = () => Declaration,
  heritageSet = () => {
    Declaration = null;
  },
  Base
) {
  static field = Declaration;
  static #privateMethod() {
    return Declaration;
  }
  static privateCall() {
    return Declaration.#privateMethod();
  }
  static borrowed() {
    return Declaration;
  }
  static {
    this.block = () => Declaration;
  }
  self() {
    return Declaration;
  }
  assign() {
    Declaration = (assignmentRhsEvaluated = true, null);
  }
}

const originalDeclaration = Declaration;
if (heritageProbe() !== originalDeclaration) throw "heritage closure";
if (originalDeclaration.field !== originalDeclaration) throw "static field";
if (originalDeclaration.block() !== originalDeclaration) throw "static block";
if (originalDeclaration.privateCall() !== originalDeclaration) throw "private method";
class Borrower extends originalDeclaration {}
if (Borrower.borrowed() !== originalDeclaration) throw "inherited method";

Declaration = null;
if (new originalDeclaration().self() !== originalDeclaration) throw "declaration self";

let assignmentThrew = false;
try {
  new originalDeclaration().assign();
} catch (error) {
  assignmentThrew = error.name === "TypeError";
}
if (!assignmentRhsEvaluated || !assignmentThrew) throw "immutable assignment";

let heritageAssignmentThrew = false;
try {
  heritageSet();
} catch (error) {
  heritageAssignmentThrew = error.name === "TypeError";
}
if (!heritageAssignmentThrew) throw "immutable heritage closure";

let Outer = "outer";
const expression = class Outer {
  self() {
    return Outer;
  }
};
if (Outer !== "outer") throw "class expression leak";
if (new expression().self() !== expression) throw "class expression self";

var accessorSeen;
class Accessor {
  get value() {
    return Accessor;
  }
  set value(next) {
    accessorSeen = Accessor;
  }
}
const originalAccessor = Accessor;
const accessor = new Accessor();
if (accessor.value !== originalAccessor) throw "paired getter";
accessor.value = null;
if (accessorSeen !== originalAccessor) throw "captured setter";

let tdzThrew = false;
try {
  class Tdz extends Tdz {}
} catch (error) {
  tdzThrew = error.name === "ReferenceError";
}
if (!tdzThrew) throw "class heritage TDZ";

true;
