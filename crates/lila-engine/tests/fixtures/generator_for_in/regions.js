function same(a,b,message){if(a!==b)throw new Error(message);}
function check(value,message){if(!value)throw new Error(message);}
var trace='';
var gone=false;
var getterToken={getter:true};
var symbol=Symbol('ignored');
var prototype=new Proxy(Object.create(null),{
  ownKeys(){trace+='pk;';return ['a','deleted','hidden','p'];},
  getOwnPropertyDescriptor(target,key){trace+='pd:'+key+';';return {value:1,enumerable:true,configurable:true};},
  getPrototypeOf(){trace+='pp;';return null;}
});
var target=Object.create(null);
Object.defineProperty(target,'a',{get(){throw getterToken;},enumerable:true,configurable:true});
var proxy=new Proxy(target,{
  ownKeys(){trace+='ok;';return ['a','deleted','hidden',symbol];},
  getOwnPropertyDescriptor(target,key){
    trace+='od:'+key+';';
    if(key==='deleted'&&gone)return undefined;
    return {value:1,enumerable:key!=='hidden',configurable:true};
  },
  getPrototypeOf(){trace+='op;';return prototype;}
});
var savedKeys=Reflect.ownKeys;
var savedDescriptor=Object.getOwnPropertyDescriptor;
Reflect.ownKeys=function(){throw 'public ownKeys';};
Object.getOwnPropertyDescriptor=function(){throw 'public descriptor';};
function* enumerate(view){for(const key in view){yield key;}}
var iterator=enumerate(proxy);
same(iterator.next().value,'a','first actual own key');
same(trace,'ok;od:a;','only first descriptor before suspension');
gone=true;
gc();
same(iterator.next().value,'deleted','missing own descriptor leaves inherited name unvisited');
same(trace,'ok;od:a;od:deleted;od:hidden;op;pk;pd:deleted;','lazy prototype/current descriptor/visited precedence');
gc();
same(iterator.next().value,'p','nonenumerable own name shadows inherited entry');
same(trace,'ok;od:a;od:deleted;od:hidden;op;pk;pd:deleted;pd:p;','visited keys are not re-read');
same(iterator.next().done,true,'prototype chain finishes');
same(trace,'ok;od:a;od:deleted;od:hidden;op;pk;pd:deleted;pd:p;pp;','prototype queried after remaining keys');
Reflect.ownKeys=savedKeys;
Object.getOwnPropertyDescriptor=savedDescriptor;

function* captured(){for(const key in yield 'target'){const local=yield key;yield ()=>[key,local];}return 42;}
var left=captured();
var right=captured();
same(left.next().value,'target','left complete target');
same(right.next().value,'target','right complete target');
same(left.next({a:1,b:1}).value,'a','left first key');
same(right.next({x:1,y:1}).value,'x','right first key');
var readLeft=left.next(11).value;
var readRight=right.next(22).value;
gc();
same(readLeft()[0],'a','captured left original iteration cell');
same(readLeft()[1],11,'captured left yielded declaration');
same(readRight()[0],'x','interleaved cursor is independent');
same(readRight()[1],22,'interleaved lexical chain is independent');
same(left.next().value,'b','next left key selected once');
same(right.next().value,'y','next right key selected once');
var lastLeft=left.next(33).value;
var lastRight=right.next(44).value;
gc();
same(readLeft()[0],'a','new iteration cannot replace captured first cell');
same(lastLeft()[0],'b','new left iteration owns its cell');
same(lastRight()[1],44,'new right local owns its cell');
same(left.next().value,42,'left completion');
same(right.next().value,42,'right completion');

function* controlled(view){outer:for(let key in view){try{for(const inner in {x:1,y:1}){yield key+inner;if(inner==='x')continue outer;}}finally{gc();yield 'finally:'+key;}}return 'done';}
var control=controlled({a:1,b:1});
same(control.next().value,'ax','nested ForIn first key');
same(control.next().value,'finally:a','labelled Continue enters original finalizer');
same(control.next().value,'bx','Continue leaves original iteration before advance');
same(control.next().value,'finally:b','second original finalizer');
same(control.next().value,'done','outer cursor finishes');
var returned={returned:true};returned.self=returned;
control=controlled({a:1,b:1});
same(control.next().value,'ax','return injection point');
same(control.return(returned).value,'finally:a','whole Return retained while finalizer yields');
gc();
var returnedResult=control.next();
check(returnedResult.done&&returnedResult.value===returned,'whole injected Return survives cleanup');
var thrown={thrown:true};thrown.self=thrown;
control=controlled({a:1,b:1});
same(control.next().value,'ax','throw injection point');
same(control.throw(thrown).value,'finally:a','whole Throw retained while finalizer yields');
gc();
var caught;
try{control.next();}catch(error){caught=error;}
same(caught,thrown,'whole injected Throw survives cleanup');

var abrupt={abrupt:true};abrupt.self=abrupt;
function* recover(){try{for(const key in new Proxy({},{ownKeys(){throw abrupt;}})){yield key;}}catch(error){gc();yield error;}for(const key in {after:1})yield key;}
var recovered=recover();
same(recovered.next().value,abrupt,'actual enumeration Throw caught outside owner');
gc();
same(recovered.next().value,'after','new cursor after caught abrupt');
same(recovered.next().done,true,'recovery cursor completes');

var heads=0;
function choose(value){heads++;return value;}
function* primitive(value){for(let key in choose(yield 'head'))yield key;return 'end';}
for(var value of [null,undefined]){
  var nullish=primitive(value);
  same(nullish.next().value,'head','nullish target still evaluated');
  var nullishResult=nullish.next(value);
  check(nullishResult.done&&nullishResult.value==='end','nullish head skips body');
}
Object.defineProperty(Object.prototype,'forInInherited',{value:1,enumerable:true,configurable:true});
for(var value of [7,false,7n,Symbol('boxed')]){
  var boxed=primitive(value);
  same(boxed.next().value,'head','primitive target evaluated');
  same(boxed.next(value).value,'forInInherited','primitive boxes and walks original prototype');
  same(boxed.next().value,'end','primitive cursor terminates');
}
delete Object.prototype.forInInherited;
same(heads,6,'complete targets evaluated once');
var string=primitive('xy');
same(string.next().value,'head','string target evaluated');
same(string.next('xy').value,'0','string first own index');
same(string.next().value,'1','string second own index');
same(string.next().value,'end','string length is nonenumerable');
print('generator-forin-regions:ok');
