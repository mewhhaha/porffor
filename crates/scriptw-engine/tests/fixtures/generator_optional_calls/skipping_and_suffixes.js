function check(result, value, done) {
  if (!Object.is(result.value, value) || result.done !== done) throw 'iterator result';
}
let skipped = 0;
let iterators = 0;
let getters = 0;
function forbidden() { skipped++; throw 'shorted operand'; }
const iterable = {[Symbol.iterator]() { iterators++; throw 'shorted iterator'; }};
const absent = null;
const nullMethod = {get method() { getters++; return null; }};
function* nullBase() { return absent?.[yield forbidden()](...iterable, yield forbidden()).child[yield forbidden()]; }
function* nullCall() { return nullMethod.method?.(...iterable, yield forbidden()).child[yield forbidden()]; }
check(nullBase().next(), undefined, true);
check(nullCall().next(), undefined, true);
if (skipped !== 0 || iterators !== 0 || getters !== 1) throw 'complete suffix shorting';

const trace = [];
const noncallable = {get method() { trace.push('get'); return 0; }};
function last() { trace.push('last'); return 2; }
function* ordinary() { return noncallable?.method(yield 'argument', last()); }
const ordinaryIterator = ordinary();
check(ordinaryIterator.next(), 'argument', false);
if (trace.join(',') !== 'get') throw 'Get must precede argument Yield';
let caught;
try { ordinaryIterator.next(1); } catch (error) { caught = error; trace.push('throw'); }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || trace.join(',') !== 'get,last,throw') throw 'noncallable rejects after arguments';

let creates = 0;
const nullChild = {create() { creates++; return null; }};
function* returnedNull() { return nullChild?.create(yield 'first')?.[yield forbidden()]?.(yield forbidden()); }
const returned = returnedNull();
check(returned.next(), 'first', false);
check(returned.next(1), undefined, true);
if (creates !== 1 || skipped !== 0 || iterators !== 0) throw 'later shorted link';

function* grouped() { return (absent?.[yield forbidden()]).method(forbidden()); }
caught = undefined;
try { grouped().next(); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || skipped !== 0) throw 'grouping ends shorting before outer arguments';

const holder = {entry: {value: 17}};
function consume(value) { return value + 1; }
function* asArgument() { return consume(holder?.[yield 'entry'].value); }
const argumentIterator = asArgument();
check(argumentIterator.next(), 'entry', false);
check(argumentIterator.next('entry'), 18, true);
function Constructor(value) { this.value = value; }
const maker = {create() { return Constructor; }};
function* construct() { return new (maker?.create(yield 'constructor'))(5).value; }
const constructorIterator = construct();
check(constructorIterator.next(), 'constructor', false);
check(constructorIterator.next(1), 5, true);

const groupedShortTrace = [];
let groupedSkipped = 0;
function groupedForbidden() { groupedSkipped++; throw 'grouped inner must skip'; }
function groupedFirst() { groupedShortTrace.push('first'); return 1; }
function groupedLast() { groupedShortTrace.push('last'); return 3; }
function* groupedNullCall() { return (absent?.method(yield groupedForbidden()))(groupedFirst(), yield 'grouped-null-outer', groupedLast()); }
const groupedNullIterator = groupedNullCall();
check(groupedNullIterator.next(), 'grouped-null-outer', false);
if (groupedShortTrace.join(',') !== 'first' || groupedSkipped !== 0) throw 'ordinary outer must enter after inner short';
caught = undefined;
try { groupedNullIterator.next(2); } catch (error) { caught = error; groupedShortTrace.push('throw'); }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || groupedShortTrace.join(',') !== 'first,last,throw') throw 'grouped undefined rejects after outer arguments';

const groupedNotCallable = {make(value) {
  'use strict';
  if (this !== groupedNotCallable || value !== 4) throw 'noncallable factory receiver';
  groupedShortTrace.push('factory');
  return 0;
}};
function* groupedInvalidCall() { return (groupedNotCallable?.make(yield 'grouped-factory'))(yield 'grouped-invalid-outer', groupedLast()); }
const groupedInvalidIterator = groupedInvalidCall();
check(groupedInvalidIterator.next(), 'grouped-factory', false);
check(groupedInvalidIterator.next(4), 'grouped-invalid-outer', false);
caught = undefined;
try { groupedInvalidIterator.next(2); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || groupedShortTrace.join(',') !== 'first,last,throw,factory,last') throw 'noncallable completed Value operand order';

let groupedSubstitutions = 0;
function groupedSubstitution() { groupedSubstitutions++; return 5; }
function* groupedNullTag() { return (absent?.method(yield groupedForbidden()))`head${groupedSubstitution()}middle${yield 'grouped-tag-substitution'}tail`; }
const groupedNullTagIterator = groupedNullTag();
check(groupedNullTagIterator.next(), 'grouped-tag-substitution', false);
if (groupedSubstitutions !== 1 || groupedSkipped !== 0) throw 'ordinary tag substitutions after inner short';
caught = undefined;
try { groupedNullTagIterator.next(6); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || groupedSubstitutions !== 1) throw 'grouped undefined tag rejects after substitutions';
const groupedPropertyShortTrace = [];
let groupedPropertySkipped = 0;
function groupedPropertyForbidden() { groupedPropertySkipped++; throw 'grouped Property skipped key'; }
function groupedPropertyFirst() { groupedPropertyShortTrace.push('first'); return 1; }
function groupedPropertyLast() { groupedPropertyShortTrace.push('last'); return 3; }
function* groupedPropertyNullCall() {
  return (absent?.[yield groupedPropertyForbidden()])(groupedPropertyFirst(), yield 'grouped-property-null-outer', groupedPropertyLast());
}
const groupedPropertyNullIterator = groupedPropertyNullCall();
check(groupedPropertyNullIterator.next(), 'grouped-property-null-outer', false);
if (groupedPropertyShortTrace.join(',') !== 'first' || groupedPropertySkipped !== 0) throw 'grouped Property short must enter ordinary outer Call';
caught = undefined;
try { groupedPropertyNullIterator.next(2); } catch (error) { caught = error; groupedPropertyShortTrace.push('throw'); }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || groupedPropertyShortTrace.join(',') !== 'first,last,throw' || groupedPropertySkipped !== 0) throw 'grouped Property undefined rejects after arguments';

const groupedPropertyInvalid = {get method() { groupedPropertyShortTrace.push('get'); return 0; }};
function* groupedPropertyInvalidCall() {
  return (groupedPropertyInvalid?.[yield 'grouped-property-invalid-key'])(groupedPropertyFirst(), yield 'grouped-property-invalid-outer', groupedPropertyLast());
}
const groupedPropertyInvalidIterator = groupedPropertyInvalidCall();
check(groupedPropertyInvalidIterator.next(), 'grouped-property-invalid-key', false);
check(groupedPropertyInvalidIterator.next('method'), 'grouped-property-invalid-outer', false);
if (groupedPropertyShortTrace.join(',') !== 'first,last,throw,get,first') throw 'grouped Property Get before ordinary operands';
caught = undefined;
try { groupedPropertyInvalidIterator.next(2); } catch (error) { caught = error; groupedPropertyShortTrace.push('throw'); }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || groupedPropertyShortTrace.join(',') !== 'first,last,throw,get,first,last,throw') throw 'grouped Property noncallable operand order';

const groupedPropertyTagTrace = [];
function groupedPropertySubstitutionFirst() { groupedPropertyTagTrace.push('first'); return 4; }
function groupedPropertySubstitutionLast() { groupedPropertyTagTrace.push('last'); return 6; }
function* groupedPropertyNullTag() {
  return (absent?.[yield groupedPropertyForbidden()])`head${groupedPropertySubstitutionFirst()}middle${yield 'grouped-property-tag-outer'}end${groupedPropertySubstitutionLast()}tail`;
}
const groupedPropertyNullTagIterator = groupedPropertyNullTag();
check(groupedPropertyNullTagIterator.next(), 'grouped-property-tag-outer', false);
if (groupedPropertyTagTrace.join(',') !== 'first' || groupedPropertySkipped !== 0) throw 'grouped Property skipped tag still enters substitutions';
caught = undefined;
try { groupedPropertyNullTagIterator.next(5); } catch (error) { caught = error; groupedPropertyTagTrace.push('throw'); }
if (Object.getPrototypeOf(caught) !== TypeError.prototype || groupedPropertyTagTrace.join(',') !== 'first,last,throw' || groupedPropertySkipped !== 0) throw 'grouped Property skipped tag checks callability after substitutions';

const groupedPropertyConstructorOwner = {Constructor};
function* groupedPropertyConstruct() { return new (groupedPropertyConstructorOwner?.[yield 'grouped-property-constructor-key'])(yield 'grouped-property-constructor-argument'); }
const groupedPropertyConstructIterator = groupedPropertyConstruct();
check(groupedPropertyConstructIterator.next(), 'grouped-property-constructor-key', false);
check(groupedPropertyConstructIterator.next('Constructor'), 'grouped-property-constructor-argument', false);
const groupedPropertyConstructResult = groupedPropertyConstructIterator.next(11);
if (groupedPropertyConstructResult.done !== true || Object.getPrototypeOf(groupedPropertyConstructResult.value) !== Constructor.prototype || groupedPropertyConstructResult.value.value !== 11) throw 'grouped Property construction consumes Value';
print('generator-optional-suffixes:ok');
262;
