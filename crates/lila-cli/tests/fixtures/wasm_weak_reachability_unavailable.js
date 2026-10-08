const foreign = __lilaCreateRealm().global;
const Constructor = foreign.WeakMap;
foreign.WeakMap = null;
try {
  Reflect.construct(Constructor, []);
} catch (error) {
  print('capability rejection was caught');
}
print('capability rejection was ignored');
true;
