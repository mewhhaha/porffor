function check(condition, label) { if (!condition) throw label; }

// NamedEvaluation establishes a display name before static initialization;
// it leaves reads and later closures on the original outer variable cell.
function outerName() {
  var Label = 'outer-before';
  Label = class {
    static seen = Label;
    static display = this.name;
    static read() { return Label; }
  };
  var Constructor = Label;
  check(Constructor.seen === 'outer-before', 'inferred-static-outer-value');
  check(Constructor.display === 'Label', 'inferred-name-before-static');
  check(Constructor.read() === Constructor, 'inferred-method-outer-cell');
  Label = 'outer-after';
  check(Constructor.read() === 'outer-after', 'inferred-method-live-outer-cell');
  var descriptor = Object.getOwnPropertyDescriptor(Constructor, 'name');
  check(descriptor.value === 'Label' && !descriptor.writable &&
        !descriptor.enumerable && descriptor.configurable, 'inferred-name-descriptor');
}
outerName();

var holder = { noOuterBinding: class { static display = this.name; } };
check(holder.noOuterBinding.name === 'noOuterBinding' &&
      holder.noOuterBinding.display === 'noOuterBinding', 'property-label-without-source-binding');

// Explicit source names keep their own immutable cell, including methods
// invoked after the enclosing variable has changed.
function innerName() {
  var Inner = 'outside';
  var Label = class Inner {
    static seen = Inner;
    static read() { return Inner; }
  };
  var Constructor = Label;
  check(Constructor.name === 'Inner' && Constructor.seen === Constructor,
        'explicit-name-before-static');
  Inner = 'changed-outside'; Label = null;
  gc();
  check(Constructor.read() === Constructor, 'explicit-inner-cell-retained');
  class Declared { static read() { return Declared; } }
  var declaration = Declared;
  Declared = null;
  check(declaration.read() === declaration, 'class-declaration-inner-cell');
}
innerName();

// A SingleName anonymous default must observe the original lexical TDZ,
// before any later property default is evaluated or suspended.
var laterEffects = 0;
function* lexicalDefault() {
  const { f = class { static seen = typeof f; }, x = (laterEffects++, yield 'later') } = {};
  return f;
}
var iterator = lexicalDefault();
try { iterator.next(); throw 'missing-inferred-class-tdz'; }
catch (error) { check(error instanceof ReferenceError, 'inferred-class-outer-tdz'); }
check(laterEffects === 0 && iterator.next().done, 'tdz-prevents-later-default');

// Assignment defaults retain the old outer value through class evaluation.
var f = 'assignment-before', x;
function* assignmentDefault() {
  return ({ f = class { static seen = f; static read() { return f; } }, x = yield 'default' } = {});
}
iterator = assignmentDefault();
check(iterator.next().value === 'default', 'assignment-default-suspends-after-class');
var Assigned = f;
check(Assigned.name === 'f' && Assigned.seen === 'assignment-before', 'assignment-default-outer-value');
gc(); check(iterator.next(7).done && x === 7, 'assignment-default-resume');
f = 'assignment-after';
check(Assigned.read() === 'assignment-after', 'assignment-default-live-outer-cell');

// Heritage suspension retains the outer activation cell without installing
// an inferred class name environment. A later static read is still in TDZ.
function* heritageTdz() {
  let Inferred = class extends (yield 'parent') { static seen = typeof Inferred; };
  return Inferred;
}
iterator = heritageTdz();
check(iterator.next().value === 'parent', 'inferred-heritage-suspends');
gc();
try { iterator.next(function Base() {}); throw 'missing-resumed-class-tdz'; }
catch (error) { check(error instanceof ReferenceError, 'inferred-heritage-outer-tdz'); }

function* retainedOuter() {
  var Inferred = 'before';
  Inferred = class extends (yield 'parent') {
    static seen = Inferred;
    static read() { return Inferred; }
  };
  yield Inferred;
  Inferred = 'after';
}
iterator = retainedOuter();
check(iterator.next().value === 'parent', 'outer-cell-heritage-prefix');
gc();
var Inferred = iterator.next(function Base() {}).value;
check(Inferred.name === 'Inferred' && Inferred.seen === 'before', 'outer-cell-after-heritage');
check(Inferred.read() === Inferred, 'outer-cell-initialized-after-class');
gc(); check(iterator.next().done, 'outer-cell-final-resume');
check(Inferred.read() === 'after', 'outer-cell-after-generator-completion');
print('class-source-names:ok');
