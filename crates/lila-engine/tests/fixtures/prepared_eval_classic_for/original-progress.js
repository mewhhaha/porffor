var supreme = 5;
var __evaluated = eval("for(var count=0;;) {if (count===supreme)break;else count++; }");
if (__evaluated !== undefined || count !== supreme) throw new Error('original loop');
print('ok');
true;
