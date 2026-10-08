function check(condition, label) { if (!condition) throw label; }
var events = [];
var leaked;
var observedClass = false;
class Base {}

class Owner {
  #secret = 47;
  *evaluate(stage, input, direct) {
    let Inner = 23;
    let selected;
    try {
      if (direct) {
        if (stage === 'heritage') {
          selected = class Inner extends (leaked = () => Inner, yield 'heritage') {
            #nested = 3;
            static observed = (observedClass = true);
          };
        } else {
          selected = class Inner {
            #nested = 3;
            [(leaked = () => Inner, yield 'key')]() {}
            static observed = (observedClass = true);
          };
        }
      } else {
        if (stage === 'heritage') {
          [[selected = class Inner extends (leaked = () => Inner, yield 'heritage') {
            #nested = 3;
            static observed = (observedClass = true);
          }]] = input;
        } else {
          [[selected = class Inner {
            #nested = 3;
            [(leaked = () => Inner, yield 'key')]() {}
            static observed = (observedClass = true);
          }]] = input;
        }
      }
    } finally {
      // This source binding and private environment belong to the Owner,
      // not the abandoned class-name or inner private environments.
      check(Inner === 23, 'restored-outer-name-before-finally');
      check(this.#secret === 47, 'restored-owner-private-before-finally');
      class After {
        read(owner) { return owner.#secret; }
      }
      check(new After().read(this) === 47, 'new-class-captures-restored-private-parent');
      events.push('finally-enter');
      yield 'finally';
      gc();
      check(Inner === 23 && this.#secret === 47, 'restored-environment-after-finally-resume');
      events.push('finally-exit');
    }
    return selected;
  }
}

function iterable(label, value, closeError) {
  return {
    [Symbol.iterator]() {
      let consumed = false;
      return {
        next() {
          if (consumed) return { done: true };
          consumed = true;
          return { done: false, value };
        },
        get return() {
          events.push(label + '-get-return');
          gc();
          return function () {
            events.push(label + '-close');
            gc();
            if (closeError) throw closeError;
            return {};
          };
        }
      };
    }
  };
}

for (const stage of ['heritage', 'key']) {
  for (const direct of [true, false]) {
    for (const outcome of ['return', 'throw', 'normal']) {
      for (const closeFails of [false, true]) {
        if (direct && closeFails) continue;
        events = [];
        leaked = undefined;
        observedClass = false;
        const incoming = { incoming: stage + outcome };
        const closeError = { close: stage + outcome };
        const inner = iterable('inner', undefined, closeFails ? closeError : undefined);
        const outer = iterable('outer', inner, closeFails ? closeError : undefined);
        const iterator = new Owner().evaluate(stage, outer, direct);
        let result = iterator.next();
        check(!result.done && result.value === stage, 'exact-class-prefix-yield');
        gc();
        if (outcome === 'return') result = iterator.return(incoming);
        else if (outcome === 'throw') result = iterator.throw(incoming);
        else result = iterator.next(stage === 'heritage' ? Base : 'method');
        check(!result.done && result.value === 'finally', 'outer-finally-owns-cleaned-class-outcome');
        check(events.join(',') === (direct ? 'finally-enter' :
          'inner-get-return,inner-close,outer-get-return,outer-close,finally-enter'),
          'class-cleanup-before-inner-to-outer-close-and-finally');
        check(observedClass === (outcome === 'normal'), 'abrupt-prefix-never-runs-static-elements');
        if (outcome !== 'normal') {
          try { leaked(); throw 'missing-abandoned-class-tdz'; }
          catch (error) { check(error instanceof ReferenceError, 'abandoned-name-closure-retains-original-tdz'); }
        } else {
          check(leaked().name === 'Inner', 'normal-original-class-name-cell');
        }
        gc();
        let thrown;
        try { result = iterator.next(); }
        catch (error) { thrown = error; }
        if (outcome === 'throw') check(thrown === incoming, 'whole-incoming-throw-wins-close-error');
        else if (closeFails) check(thrown === closeError, 'close-throw-replaces-return-or-normal');
        else if (outcome === 'return') check(!thrown && result.done && result.value === incoming, 'whole-incoming-return-retained');
        else check(!thrown && result.done && result.value === leaked(), 'normal-class-result-retained');
        check(events[events.length - 1] === 'finally-exit', 'finally-resumes-once');
        result = iterator.next();
        check(result.done && result.value === undefined, 'exhausted-prefix-does-not-replay');
        gc();
      }
    }
  }
}
print('generator-class-abrupt-cleanup:ok');
