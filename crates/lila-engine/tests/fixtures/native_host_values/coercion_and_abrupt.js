function check(value, label) { if (!value) throw label; }
var holder = {kind: 'old', element: ['old']};
var printArg = {toString() { holder.kind = 17; holder.element = [true]; return 'native-host-print'; }};
print(printArg, true);
check(holder.kind + 1 === 18 && holder.element[0] === true, 'print callback updates caller facts');
var marker = Symbol('host abrupt'), trace = [], caught, finalized = 0;
try {
  print({toString() { trace.push('first'); return 'unpublished'; }},
    {toString() { trace.push('throw'); throw marker; }},
    {toString() { trace.push('forbidden'); return 'later'; }});
} catch (error) { caught = error; } finally { finalized++; }
check(caught === marker && finalized === 1 && trace.join('|') === 'first|throw', 'print conversion cutoff and atomic output');
var sleepState = {value: 'old'};
__lilaAgentSleep({valueOf() { sleepState.value = 9; return 0; }});
check(sleepState.value + 1 === 10, 'sleep ToNumber updates caller facts');
for (var invoke of [__lilaAgentStart, __lilaAgentReport]) {
  var state = {value: 'old'}, calls = 0;
  caught = undefined;
  try { invoke({toString() { calls++; state.value = true; throw marker; }}); } catch (error) { caught = error; }
  check(caught === marker && calls === 1 && state.value === true, 'agent text whole coercion abrupt before host operation');
}
var idCalls = 0;
try { __lilaAgentBroadcast({}, {valueOf() { idCalls++; return 0; }}); } catch (error) { check(error instanceof TypeError, 'broadcast requires real SAB'); }
check(idCalls === 0, 'SAB admission before id ToInt32');
var sab = new SharedArrayBuffer(0), idState = {value: 'old'};
caught = undefined;
try { __lilaAgentBroadcast(sab, {valueOf() { idCalls++; idState.value = 23; throw marker; }}); } catch (error) { caught = error; }
check(caught === marker && idCalls === 1 && idState.value + 1 === 24, 'broadcast id whole coercion and caller facts');
var realm = __lilaCreateRealm(), realmState = {value: 'old'};
caught = undefined;
try { realm.evalScript({toString() { realmState.value = false; throw marker; }}); } catch (error) { caught = error; }
check(caught === marker && realmState.value === false, 'RealmEvalScript source conversion precedes source admission');
print('native-host-coercion:ok');
262;
