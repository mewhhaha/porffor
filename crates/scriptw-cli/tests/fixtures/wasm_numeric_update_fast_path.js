(function () {
  function check(a,b,label) { if (a !== b) throw new Error(label); }
  let text = '1'; check(text++, 1, 'postfix string old'); check(text, 2, 'postfix string stored');
  text = '3'; check(++text, 4, 'prefix string');
  let truth = true; const get = () => truth; check(truth++, 1, 'postfix boolean'); check(get(), 2, 'captured store');
  let nil = null; check(++nil, 1, 'null'); let missing; check(Number.isNaN(missing++), true, 'undefined');
  let minusZero = -0; check(Object.is(minusZero++, -0), true, 'postfix signed zero'); check(minusZero, 1, 'zero stored');
  let huge = 1n; check(huge++, 1n, 'BigInt old'); check(huge, 2n, 'BigInt stored');
  let sym = Symbol('value'), threw = false; try { sym++; } catch(error) { threw = error instanceof TypeError; }
  check(threw,true,'Symbol conversion throws'); check(typeof sym,'symbol','Symbol store not performed');
  let calls = ''; const fixed = { valueOf() { calls += 'v'; return 7; } };
  threw=false;try { fixed++; } catch(error) { threw=error instanceof TypeError; }
  check(threw,true,'const assignment throws');check(calls,'v','coercion before const assignment');
  const object = { get x() { calls += 'g'; return '8'; }, set x(value) { calls += 's'; check(value, 9, 'setter number'); } };
  check(object.x++, 8, 'property old');check(calls,'vgs','property order');
  print('numeric-update-controls:ok');
})();
