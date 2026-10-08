'use strict';
function check(value, label) { if (!value) throw new Error(label); }
function uninitialized(read, label) {
  try { read(); throw new Error(label + ':missing'); }
  catch (error) { check(error instanceof ReferenceError, label); }
}
async function rejected(promise, expected, label) {
  try { await promise; throw new Error(label + ':missing'); }
  catch (error) { check(error === expected, label); }
}

async function run() {
  var events = [], resume, pending, invoke, readThis, target;
  var gate = new Promise(function (resolve) { resume = resolve; });
  class Base {
    constructor(first, middle, last) {
      events.push('base');
      this.args = [first, middle, last];
      target = new.target;
    }
  }
  class OtherBase { constructor() { events.push('other-base'); } }
  class Derived extends Base {
    field = (events.push('field'), 17);
    constructor(wait) {
      readThis = () => this;
      invoke = async (value) => {
        const result = super((events.push('first'), 1), await value, (events.push('last'), 3));
        check(result === this, 'original-this-binding-after-super');
        return result;
      };
      pending = invoke(wait);
      return {};
    }
  }
  var placeholder = new Derived(gate);
  check(events.join(',') === 'first', 'early-argument-before-await');
  uninitialized(readThis, 'this-remains-uninitialized-while-pending');
  Object.setPrototypeOf(Derived, OtherBase);
  gc(); resume(2);
  var result = await pending;
  check(events.join(',') === 'first,last,base,field', 'captured-super-before-prototype-change');
  check(result !== placeholder && readThis() === result, 'constructor-return-does-not-replace-original-this-cell');
  check(result.args.join(',') === '1,2,3' && result.field === 17 && target === Derived, 'original-construction-and-fields');
  check(Object.getPrototypeOf(result) === Derived.prototype, 'original-derived-new-target-without-arrow-new-target-syntax');
  try { await invoke(Promise.resolve(9)); throw new Error('missing-duplicate-super'); }
  catch (error) { check(error instanceof ReferenceError, 'duplicate-super-binds-after-construction'); }
  check(events.slice(-3).join(',') === 'first,last,other-base', 'second-construction-precedes-duplicate-binding-error');
  check(readThis() === result && result.field === 17, 'duplicate-super-preserves-original-this-and-fields');

  Object.setPrototypeOf(Derived, Base);
  function AlternateNewTarget() {}
  gate = new Promise(function (resolve) { resume = resolve; });
  placeholder = Reflect.construct(Derived, [gate], AlternateNewTarget);
  gc(); resume(8);
  result = await pending;
  check(target === AlternateNewTarget && Object.getPrototypeOf(result) === AlternateNewTarget.prototype,
    'original-reflect-new-target-survives-return-and-await');
  check(result.field === 17 && readThis() === result, 'fields-belong-to-active-derived-constructor');

  var whole = {}, attempts = 0, fields = 0, recover, recoverThis;
  class RecoveryBase { constructor(value) { attempts++; this.value = value; } }
  class Recovery extends RecoveryBase {
    field = (++fields);
    constructor() {
      recoverThis = () => this;
      recover = async (value) => super(await value);
      return {};
    }
  }
  new Recovery();
  await rejected(recover(Promise.reject(whole)), whole, 'rejected-argument-preserves-whole-error');
  check(attempts === 0 && fields === 0, 'rejected-await-does-not-construct-or-initialize-fields');
  uninitialized(recoverThis, 'rejected-await-leaves-original-this-tdz');
  gc(); result = await recover(Promise.resolve(23));
  check(result.value === 23 && recoverThis() === result && attempts === 1 && fields === 1,
    'later-call-initializes-the-same-activation');

  var fail, failureThis, baseCalls = 0;
  class ThrowBase { constructor() { baseCalls++; throw whole; } }
  class ThrowDerived extends ThrowBase {
    constructor() { failureThis = () => this; fail = async (value) => super(await value); return {}; }
  }
  new ThrowDerived();
  await rejected(fail(Promise.resolve(0)), whole, 'base-construction-error-is-preserved');
  check(baseCalls === 1, 'throwing-base-runs-once');
  uninitialized(failureThis, 'throwing-base-leaves-original-this-uninitialized');

  var invalid, invalidThis, argumentRuns = 0;
  class InvalidDerived extends Base {
    constructor() {
      invalidThis = () => this;
      invalid = async (value) => super((argumentRuns++, 1), await value, (argumentRuns++, 3));
      return {};
    }
  }
  new InvalidDerived();
  Object.setPrototypeOf(InvalidDerived, () => 0);
  try { await invalid(Promise.resolve(2)); throw new Error('missing-invalid-super'); }
  catch (error) { check(error instanceof TypeError, 'is-constructor-follows-complete-argument-list'); }
  check(argumentRuns === 2, 'invalid-super-still-evaluates-all-arguments');
  uninitialized(invalidThis, 'invalid-super-does-not-bind-this');

  var readBefore, readBeforeThis, observations = 0;
  var untouched = { get then() { observations++; return function () {}; } };
  class BeforeThis extends Base {
    constructor() { readBeforeThis = () => this; readBefore = async () => super(this, await untouched); return {}; }
  }
  new BeforeThis();
  try { await readBefore(); throw new Error('missing-before-this'); }
  catch (error) { check(error instanceof ReferenceError, 'early-argument-this-tdz'); }
  check(observations === 0, 'early-this-failure-prevents-await-observation');
  uninitialized(readBeforeThis, 'argument-failure-does-not-initialize-this');

  events = []; var spreadNext = 0, spreadReturn = 0, spreadCall, spreadPending;
  var values = {
    [Symbol.iterator]() {
      events.push('iterator');
      return {
        get next() {
          events.push('next-get');
          return function () {
            events.push('step' + spreadNext);
            if (spreadNext < 2) return { done: false, value: ++spreadNext };
            return { done: true };
          };
        },
        return() { spreadReturn++; throw whole; }
      };
    }
  };
  class SpreadBase { constructor() { events.push('spread-base'); this.args = Array.from(arguments); } }
  class SpreadDerived extends SpreadBase {
    constructor(value, wait) {
      spreadCall = async () => super(...value, await wait, (events.push('spread-last'), 4));
      spreadPending = spreadCall(); return {};
    }
  }
  gate = new Promise(function (resolve) { resume = resolve; });
  new SpreadDerived(values, gate);
  check(events.join(',') === 'iterator,next-get,step0,step1,step2', 'spread-drains-before-later-await');
  values[Symbol.iterator] = function () { throw whole; };
  gc(); resume(3);
  result = await spreadPending;
  check(result.args.join(',') === '1,2,3,4' && spreadReturn === 0, 'captured-private-argument-list-is-not-reiterated');
  check(events.slice(-2).join(',') === 'spread-last,spread-base', 'late-argument-before-construct');

  var chosen, choice;
  class BranchDerived extends Base {
    constructor() { choice = async (flag, first, second) => super(flag ? await first : await second); return {}; }
  }
  new BranchDerived();
  chosen = await choice(false, untouched, Promise.resolve(31));
  check(chosen.args[0] === 31 && observations === 0, 'argument-branch-keeps-unselected-await-unobserved');
}
run().then(function () { print('async-super-construct:ok'); }, function (error) { print(error); throw error; });
