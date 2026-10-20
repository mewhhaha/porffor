function capturedSwitchCheck(value, message) {
  if (!value) throw new Error(message);
}
function capturedSwitchStep(iterator, value, expected, message) {
  var result = iterator.next(value);
  capturedSwitchCheck(!result.done && result.value === expected, message);
}

var capturedSwitchFactory, capturedSwitchAbruptFactory;
var capturedSwitchSelector = 'before-resume';
var capturedSwitchGets = 0;
var capturedSwitchWhole = { marker: 'captured-switch-throw' };
capturedSwitchWhole.self = capturedSwitchWhole;
var capturedSwitchMask = { key: true, value: false, victim: false };
var capturedSwitchOuter = {
  get key() { capturedSwitchGets++; return capturedSwitchSelector; },
  value: 'outer-before', victim: 'outer-victim', local: 'outer-local'
};
var capturedSwitchInner = {
  get key() { throw new Error('unscopable inner selector was read'); },
  get failure() { throw capturedSwitchWhole; },
  value: 1, victim: 'inner-victim', local: 'inner-local'
};
capturedSwitchInner[Symbol.unscopables] = capturedSwitchMask;
with (capturedSwitchOuter) {
  with (capturedSwitchInner) {
    capturedSwitchFactory = function* capturedWithSwitch(local) {
      switch (yield 'captured-discriminant') {
        case (yield 'captured-selector', key):
          const held = 17;
          const reader = () => [value, held, local];
          yield reader;
          value = yield 'captured-write';
          yield value;
          yield delete victim;
          local = yield 'captured-local-write';
          yield delete local;
          return reader;
        default: throw new Error('captured selector chose the wrong body');
      }
    };
    capturedSwitchAbruptFactory = function* capturedWithAbrupt() {
      try {
        switch (yield 'captured-abrupt-discriminant') {
          case (yield 'captured-abrupt-selector', failure):
            throw new Error('throwing captured selector entered a body');
        }
      } finally { yield 'captured-cleanup'; }
    };
  }
}

var capturedSwitchIterator = capturedSwitchFactory('parameter');
capturedSwitchStep(capturedSwitchIterator, undefined, 'captured-discriminant', 'captured discriminant');
gc();
capturedSwitchStep(capturedSwitchIterator, 'chosen', 'captured-selector', 'captured selector suspension');
capturedSwitchCheck(capturedSwitchGets === 0, 'selector Get stays after its suspension');
capturedSwitchSelector = 'chosen';
gc();
var capturedSwitchReaderResult = capturedSwitchIterator.next();
var capturedSwitchReader = capturedSwitchReaderResult.value;
capturedSwitchCheck(!capturedSwitchReaderResult.done && typeof capturedSwitchReader === 'function', 'captured CaseBlock reader');
capturedSwitchCheck(capturedSwitchGets === 1, 'selector reads the outer captured object once');
capturedSwitchCheck(capturedSwitchReader().join(',') === '1,17,parameter', 'captured object and nearer lexical/parameter cells');
capturedSwitchInner.value = 2;
gc();
capturedSwitchCheck(capturedSwitchReader()[0] === 2, 'captured reads retain live object storage');
capturedSwitchStep(capturedSwitchIterator, undefined, 'captured-write', 'Reference selected before yielded RHS');
capturedSwitchMask.value = true;
capturedSwitchOuter.value = 73;
var capturedSwitchWriteResult;
with ({ value: 'caller-value', key: 'caller-key', local: 'caller-local' }) {
  capturedSwitchWriteResult = capturedSwitchIterator.next(41);
}
capturedSwitchCheck(!capturedSwitchWriteResult.done && capturedSwitchWriteResult.value === 73, 'new Get observes changed unscopables in defining environments');
capturedSwitchCheck(capturedSwitchInner.value === 41 && capturedSwitchOuter.value === 73, 'Put retains its pre-suspension object rather than resolving again');
capturedSwitchMask.victim = true;
capturedSwitchStep(capturedSwitchIterator, undefined, true, 'DeleteBinding follows captured unscopables');
capturedSwitchCheck(!('victim' in capturedSwitchOuter) && capturedSwitchInner.victim === 'inner-victim', 'only the selected outer binding is deleted');
capturedSwitchStep(capturedSwitchIterator, undefined, 'captured-local-write', 'parameter Reference precedes captured With');
capturedSwitchStep(capturedSwitchIterator, 'changed', false, 'declarative parameter deletion remains false');
capturedSwitchCheck(capturedSwitchInner.local === 'inner-local' && capturedSwitchOuter.local === 'outer-local', 'parameter write does not mutate captured objects');
capturedSwitchMask.value = false;
gc();
var capturedSwitchDone = capturedSwitchIterator.next();
capturedSwitchCheck(capturedSwitchDone.done && capturedSwitchDone.value === capturedSwitchReader, 'CaseBlock reader escapes with its original environments');
gc();
capturedSwitchCheck(capturedSwitchReader().join(',') === '41,17,changed', 'captured With and CaseBlock cells survive completion');

var capturedSwitchAbrupt = capturedSwitchAbruptFactory();
capturedSwitchStep(capturedSwitchAbrupt, undefined, 'captured-abrupt-discriminant', 'abrupt captured discriminant');
capturedSwitchStep(capturedSwitchAbrupt, 0, 'captured-abrupt-selector', 'abrupt captured selector suspension');
gc();
capturedSwitchStep(capturedSwitchAbrupt, undefined, 'captured-cleanup', 'captured getter Throw enters yielding finalizer');
gc();
var capturedSwitchThrown = false;
try { capturedSwitchAbrupt.next(); }
catch (error) { capturedSwitchThrown = error === capturedSwitchWhole && error.self === error; }
capturedSwitchCheck(capturedSwitchThrown, 'captured selector keeps the whole Throw across cleanup');
