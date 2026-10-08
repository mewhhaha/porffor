var dualGoalValue = this === undefined ? 99 : 17;
let dualGoalRoot = this;
const dualGoalArrow = () => this;
globalThis.dualGoalBodies = (globalThis.dualGoalBodies || 0) + 1;

if (this === undefined) {
  if (dualGoalValue !== 99 || dualGoalRoot !== undefined || dualGoalArrow() !== undefined) {
    throw new Error('the self-imported Module must have its own lexical environment');
  }
  if (globalThis.dualGoalValue !== 17 || globalThis.dualGoalBodies !== 2) {
    throw new Error('Module declarations must preserve the root Script globals');
  }
  globalThis.dualGoalModuleBodies++;
  print('Module self import body');
} else {
  globalThis.dualGoalModuleBodies = 0;
  if (this !== globalThis || dualGoalRoot !== globalThis || dualGoalArrow() !== globalThis) {
    throw new Error('the Script must retain its global this binding');
  }
  const first = import('./entry.js');
  const second = import('./entry.js');
  if (first === second || dualGoalBodies !== 1 || dualGoalModuleBodies !== 0) {
    throw new Error('Script discovery must not evaluate the same-file Module');
  }
  print('Script self import queued');
  Promise.all([first, second]).then(async namespaces => {
    const later = await import('./entry.js');
    if (namespaces[0] !== namespaces[1] || later !== namespaces[0]) {
      throw new Error('the same-file Module namespace must be cached');
    }
    if (Object.keys(later).length !== 0 || dualGoalBodies !== 2 || dualGoalModuleBodies !== 1) {
      throw new Error('the Script and Module bodies must each evaluate once');
    }
    if (dualGoalValue !== 17 || globalThis.dualGoalValue !== 17 ||
        dualGoalRoot !== globalThis || dualGoalArrow() !== globalThis) {
      throw new Error('module jobs must preserve the Script environment');
    }
    print('distinct source goals');
  });
}
true;
