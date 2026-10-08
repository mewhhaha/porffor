function check(value, label) { if (!value) throw label; }
var foreign = __lilaCreateRealm().global;
check(foreign.Date.now() === 1234 && new foreign.Date().getTime() === 1234, 'created Realm shares chosen clock');
check(new foreign.Date(0).getHours() === 19 && new foreign.Date(0).getTimezoneOffset() === 300, 'created Realm shares chosen local zone');
check(foreign.Temporal.Now.timeZoneId() === 'America/New_York', 'created Realm publishes primary default');
check(new foreign.Intl.DateTimeFormat('en', {year:'numeric'}).resolvedOptions().timeZone === 'America/New_York', 'created Realm Intl default');
var saved = foreign.TypeError.prototype;
foreign.TypeError = function() { throw 'replaced public error'; };
var caught;
try { foreign.Date.prototype.getTime.call({$DateValue:0}); } catch (error) { caught = error; }
check(caught !== undefined && Object.getPrototypeOf(caught) === saved, 'borrowed Date brand error uses called Realm intrinsic');
caught = undefined;
try { foreign.Date.UTC({valueOf() { new Date(0).setTime(1); return Symbol(); }}, 0); }
catch (error) { caught = error; }
check(caught !== undefined && Object.getPrototypeOf(caught) === saved, 'nested coercion retains called Realm error');
print('ok');
262;
