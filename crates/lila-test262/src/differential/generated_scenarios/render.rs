use super::*;
use std::fmt::Write as _;
mod services;

/// Only the generated accumulator is renamed. It is never reflected, exported,
/// evaluated dynamically, used as a property shorthand or printed by name.
pub(super) fn source(
    program: &Program,
    transformation: Option<MetamorphicTransformation>,
) -> String {
    let state = if transformation == Some(MetamorphicTransformation::BindingRename) {
        "renamed_state"
    } else {
        "scenario_state"
    };
    let mut source = String::new();
    if program.strict {
        source.push_str("'use strict';\n");
    }
    writeln!(
        source,
        "let {state}=0;\nprint('scenario-start:{}');",
        program.family.name()
    )
    .unwrap();
    setup(&mut source, program.family);
    let count = program.actions.len();
    if transformation == Some(MetamorphicTransformation::EquivalentFiniteLoop) {
        writeln!(
            source,
            "let scenario_index=0;while(scenario_index<{count}){{"
        )
        .unwrap();
    } else {
        writeln!(
            source,
            "for(let scenario_index=0;scenario_index<{count};++scenario_index){{"
        )
        .unwrap();
    }
    source.push_str("switch(scenario_index){\n");
    for (index, action) in program.actions.iter().enumerate() {
        writeln!(source, "case {index}:{{").unwrap();
        if transformation == Some(MetamorphicTransformation::NeutralBlocks) {
            source.push_str("{\n");
        }
        match action.operation {
            Operation::Promise(operation) => {
                promise(&mut source, operation, action.value.0, state, index)
            }
            Operation::Array(operation) => {
                array(&mut source, operation, action.slot.0, action.value.0, state);
                step(&mut source, index);
            }
            Operation::Iterator(operation) => {
                iterator(&mut source, operation, action.value.0, state);
                step(&mut source, index);
            }
            Operation::Collection(operation) => {
                collection(&mut source, operation, action.slot.0, action.value.0, state);
                step(&mut source, index);
            }
            Operation::Proxy(operation) => {
                proxy(&mut source, operation, action.slot.0, action.value.0, state);
                step(&mut source, index);
            }
            Operation::Species(operation) => {
                species(&mut source, operation, action.value.0, state);
                step(&mut source, index);
            }
            Operation::Realm(operation) => {
                services::realm(&mut source, operation, action.slot.0, action.value.0, state);
                step(&mut source, index);
            }
            Operation::Temporal(operation) => {
                services::temporal(&mut source, operation, action.slot.0, action.value.0, state);
                step(&mut source, index);
            }
        }
        if transformation == Some(MetamorphicTransformation::NeutralBlocks) {
            source.push_str("}\n");
        }
        source.push_str("break;}\n");
    }
    source.push_str("}\n");
    if transformation == Some(MetamorphicTransformation::EquivalentFiniteLoop) {
        source.push_str("scenario_index+=1;\n");
    }
    source.push_str("}\n");
    match program.family {
        ScenarioFamily::PromiseJobs => {
            writeln!(source, "scenario_promise.then(value=>{{print('scenario-result:'+value);print('scenario-done:'+{state});}},error=>{{print('scenario-failed:'+error);}});\nvoid 0;").unwrap();
        }
        ScenarioFamily::ArrayAndBuffer
        | ScenarioFamily::IteratorClose
        | ScenarioFamily::Collections
        | ScenarioFamily::ProxyAndCoercion
        | ScenarioFamily::SubclassAndSpecies
        | ScenarioFamily::CrossRealm
        | ScenarioFamily::Temporal => {
            writeln!(
                source,
                "print('scenario-result:'+{state});\nprint('scenario-done:'+{state});\n{state};"
            )
            .unwrap();
        }
    }
    source
}
fn step(source: &mut String, index: usize) {
    writeln!(source, "print('scenario-step:{index}');").unwrap();
}
fn setup(source: &mut String, family: ScenarioFamily) {
    source.push_str(match family {
        ScenarioFamily::ArrayAndBuffer => "const scenario_array=[1,,3];const scenario_buffer=new ArrayBuffer(8);const scenario_bytes=new Uint8Array(scenario_buffer);const scenario_view=new DataView(scenario_buffer);\n",
        ScenarioFamily::IteratorClose => "function scenario_iterable(closeThrows,returnGetter){let cursor=0;const record={next(){print('next:'+cursor);if(cursor===0){record.next=function(){print('replacement-next');throw 77;};}if(cursor>=3)return {done:true};const current=++cursor;return {get value(){print('value:'+current);return current;},done:false};}};const close=function(){print('close:'+cursor);if(closeThrows)throw 23;return {done:true};};if(returnGetter){Object.defineProperty(record,'return',{get(){print('return-get');return close;}});}else{record.return=close;}return {[Symbol.iterator](){print('iterator');return record;}};}\n",
        ScenarioFamily::Collections => "const scenario_map=new Map([[0,1],[1,2]]);const scenario_set=new Set([0,1]);\n",
        ScenarioFamily::ProxyAndCoercion => "const scenario_target={0:1,1:2,2:3,3:4};const scenario_proxy=new Proxy(scenario_target,{get(target,key,receiver){print('get:'+String(key));return Reflect.get(target,key,receiver);},set(target,key,value,receiver){print('set:'+String(key)+':'+value);return Reflect.set(target,key,value,receiver);},has(target,key){print('has:'+String(key));return Reflect.has(target,key);},deleteProperty(target,key){print('delete:'+String(key));return Reflect.deleteProperty(target,key);},ownKeys(target){print('ownKeys');return Reflect.ownKeys(target);},getOwnPropertyDescriptor(target,key){print('descriptor:'+String(key));return Reflect.getOwnPropertyDescriptor(target,key);},defineProperty(target,key,descriptor){print('define:'+String(key));return Reflect.defineProperty(target,key,descriptor);}});function scenario_coercible(value){return {[Symbol.toPrimitive](hint){print('coerce:'+hint+':'+value);return value;}};}\n",
        ScenarioFamily::SubclassAndSpecies => "let scenario_species_calls=0;class ScenarioArrayResult extends Array{constructor(length){super(length);print('array-construct:'+length);}}class ScenarioArray extends Array{static get [Symbol.species](){++scenario_species_calls;print('array-species');return ScenarioArrayResult;}}class ScenarioTypedResult extends Uint8Array{constructor(length){super(length);print('typed-construct:'+length);}}class ScenarioTyped extends Uint8Array{static get [Symbol.species](){++scenario_species_calls;print('typed-species');return ScenarioTypedResult;}}const scenario_subclass=new ScenarioArray(1,2,3);const scenario_typed=new ScenarioTyped([1,2,3]);\n",
        ScenarioFamily::PromiseJobs => "let scenario_promise=Promise.resolve(0);\n",
        ScenarioFamily::CrossRealm => services::REALM_SETUP,
        ScenarioFamily::Temporal => services::TEMPORAL_SETUP,
    });
}
fn array(source: &mut String, operation: ArrayOperation, slot: u8, value: i8, state: &str) {
    match operation {
        ArrayOperation::Push => { writeln!(source, "print('push:'+scenario_array.push({value}));").unwrap(); }
        ArrayOperation::Pop => source.push_str("print('pop:'+scenario_array.pop());\n"),
        ArrayOperation::Splice => { writeln!(source, "print('splice:'+scenario_array.splice({},1,{value}).join('|'));", slot % 4).unwrap(); }
        ArrayOperation::CopyWithin => { writeln!(source, "scenario_array.copyWithin({},0,2);", slot % 3).unwrap(); }
        ArrayOperation::SetLength => { writeln!(source, "scenario_array.length={};", slot % 6).unwrap(); }
        ArrayOperation::TypedWrite => { writeln!(source, "scenario_bytes[{slot}]={value};print('typed-write:'+scenario_bytes[{slot}]);").unwrap(); }
        ArrayOperation::ViewWrite => { writeln!(source, "scenario_view.setInt16({},{value},true);print('view:'+scenario_view.getInt16({},false));", (slot % 4) * 2, (slot % 4) * 2).unwrap(); }
        ArrayOperation::Observe => source.push_str("for(let read=0;read<scenario_array.length;++read){print('element:'+read+':'+(read in scenario_array)+':'+scenario_array[read]);}\n"),
    }
    writeln!(source, "print('array:'+scenario_array.length+':'+scenario_array.join('|'));print('bytes:'+scenario_bytes.join('|'));{state}=({state}+scenario_array.length+scenario_bytes[{}])|0;", slot % 8).unwrap();
}
fn iterator(source: &mut String, operation: IteratorOperation, value: i8, state: &str) {
    match operation {
        IteratorOperation::Break => {
            writeln!(source, "for(const item of scenario_iterable(false,false)){{{state}=({state}+item)|0;break;}}").unwrap();
        }
        IteratorOperation::Throw => {
            writeln!(source, "try{{for(const item of scenario_iterable(false,false)){{{state}=({state}+item)|0;throw {value};}}}}catch(error){{if(error!=={value})throw error;print('caught:'+error);}}").unwrap();
        }
        IteratorOperation::Pattern => {
            writeln!(
                source,
                "const [,item]=scenario_iterable(false,false);{state}=({state}+item)|0;"
            )
            .unwrap();
        }
        IteratorOperation::TargetThrow => {
            writeln!(source, "const target={{set value(input){{print('target:'+input);throw {value};}}}};try{{[target.value]=scenario_iterable(false,false);}}catch(error){{if(error!=={value})throw error;print('target-caught:'+error);}}").unwrap();
        }
        IteratorOperation::ReturnThrows => {
            writeln!(source, "try{{for(const item of scenario_iterable(true,false)){{throw {value};}}}}catch(error){{if(error!=={value})throw error;print('close-precedence:'+error);{state}=({state}+error)|0;}}").unwrap();
        }
        IteratorOperation::GetterReturn => {
            writeln!(source, "for(const item of scenario_iterable(false,true)){{{state}=({state}+item)|0;break;}}").unwrap();
        }
        IteratorOperation::ContinueThenBreak => {
            writeln!(source, "for(const item of scenario_iterable(false,false)){{if(item===1)continue;{state}=({state}+item)|0;break;}}").unwrap();
        }
        IteratorOperation::NestedClose => {
            writeln!(source, "outer:for(const first of scenario_iterable(false,false)){{for(const second of scenario_iterable(false,false)){{{state}=({state}+first+second)|0;break outer;}}}}").unwrap();
        }
    }
}
fn collection(
    source: &mut String,
    operation: CollectionOperation,
    slot: u8,
    value: i8,
    state: &str,
) {
    match operation {
        CollectionOperation::MapSet => { writeln!(source, "scenario_map.set({slot},{value});").unwrap(); }
        CollectionOperation::MapDelete => { writeln!(source, "print('map-delete:'+scenario_map.delete({slot}));").unwrap(); }
        CollectionOperation::SetAdd => { writeln!(source, "scenario_set.add({slot});").unwrap(); }
        CollectionOperation::SetDelete => { writeln!(source, "print('set-delete:'+scenario_set.delete({slot}));").unwrap(); }
        CollectionOperation::MapForEach => { writeln!(source, "scenario_map.forEach((item,key)=>{{print('map-each:'+key+':'+item);if(key<8)scenario_map.set(key+8,{value});}});").unwrap(); }
        CollectionOperation::SetForEach => source.push_str("scenario_set.forEach(item=>{print('set-each:'+item);if(item<8)scenario_set.add(item+8);});\n"),
        CollectionOperation::Clear => source.push_str("scenario_map.clear();scenario_set.clear();\n"),
        CollectionOperation::Observe => source.push_str("print('map-has:'+scenario_map.has(0)+':'+scenario_map.get(0));print('set-has:'+scenario_set.has(0));\n"),
    }
    source.push_str("let map_trace='';for(const entry of scenario_map){map_trace+=entry[0]+':'+entry[1]+'|';}let set_trace='';for(const item of scenario_set){set_trace+=item+'|';}print('map:'+map_trace);print('set:'+set_trace);\n");
    writeln!(
        source,
        "{state}=({state}+scenario_map.size+scenario_set.size)|0;"
    )
    .unwrap();
}
fn proxy(source: &mut String, operation: ProxyOperation, slot: u8, value: i8, state: &str) {
    let key = slot % 4;
    match operation {
        ProxyOperation::Get => {
            writeln!(
                source,
                "print('read:'+scenario_proxy[scenario_coercible({key})]);"
            )
            .unwrap();
        }
        ProxyOperation::Put => {
            writeln!(source, "scenario_proxy[scenario_coercible({key})]={value};").unwrap();
        }
        ProxyOperation::Compound => {
            writeln!(
                source,
                "scenario_proxy[scenario_coercible({key})]+=scenario_coercible({value});"
            )
            .unwrap();
        }
        ProxyOperation::Has => {
            writeln!(
                source,
                "print('membership:'+(scenario_coercible({key}) in scenario_proxy));"
            )
            .unwrap();
        }
        ProxyOperation::Define => {
            writeln!(source, "Object.defineProperty(scenario_proxy,scenario_coercible({key}),{{value:{value},writable:true,enumerable:true,configurable:true}});").unwrap();
        }
        ProxyOperation::Delete => {
            writeln!(
                source,
                "print('deleted:'+(delete scenario_proxy[scenario_coercible({key})]));"
            )
            .unwrap();
        }
        ProxyOperation::Keys => {
            source.push_str("print('keys:'+Object.keys(scenario_proxy).join('|'));\n")
        }
        ProxyOperation::Coerce => {
            writeln!(source, "print('number:'+Number(scenario_coercible({value})));print('string:'+String(scenario_coercible({value})));print('sum:'+(scenario_coercible({value})+scenario_coercible({key})));").unwrap();
        }
    }
    // Reads of the ordinary backing object do not introduce extra Proxy traps.
    writeln!(
        source,
        "{state}=({state}+(scenario_target[{key}]|0))|0;print('backing:'+scenario_target[{key}]);"
    )
    .unwrap();
}
fn species(source: &mut String, operation: SpeciesOperation, value: i8, state: &str) {
    match operation {
        SpeciesOperation::Map => { writeln!(source, "const result=scenario_subclass.map(item=>item+({value}));print('map-result:'+result.join('|')+':'+(result instanceof ScenarioArrayResult));").unwrap(); }
        SpeciesOperation::Filter => source.push_str("const result=scenario_subclass.filter(item=>item%2);print('filter-result:'+result.join('|')+':'+(result instanceof ScenarioArrayResult));\n"),
        SpeciesOperation::Slice => source.push_str("const result=scenario_subclass.slice(0,2);print('slice-result:'+result.join('|')+':'+(result instanceof ScenarioArrayResult));\n"),
        SpeciesOperation::Splice => { writeln!(source, "const result=scenario_subclass.splice(0,1,{value});print('splice-result:'+result.join('|')+':'+(result instanceof ScenarioArrayResult));").unwrap(); }
        SpeciesOperation::Concat => { writeln!(source, "const result=scenario_subclass.concat([{value}]);print('concat-result:'+result.join('|')+':'+(result instanceof ScenarioArrayResult));").unwrap(); }
        SpeciesOperation::TypedMap => { writeln!(source, "const result=scenario_typed.map(item=>item+({value}));print('typed-map:'+result.join('|')+':'+(result instanceof ScenarioTypedResult));").unwrap(); }
        SpeciesOperation::TypedSlice => source.push_str("const result=scenario_typed.slice(1);print('typed-slice:'+result.join('|')+':'+(result instanceof ScenarioTypedResult));\n"),
        SpeciesOperation::Observe => source.push_str("print('subclass:'+scenario_subclass.join('|')+':'+(scenario_subclass instanceof ScenarioArray));print('typed-subclass:'+scenario_typed.join('|')+':'+(scenario_typed instanceof ScenarioTyped));\n"),
    }
    writeln!(source, "{state}=({state}+scenario_subclass.length+scenario_species_calls)|0;print('species-count:'+scenario_species_calls);").unwrap();
}
fn promise(source: &mut String, operation: PromiseOperation, value: i8, state: &str, index: usize) {
    match operation {
        PromiseOperation::Then => { writeln!(source, "scenario_promise=scenario_promise.then(input=>{{print('then:'+input);return input+({value});}});").unwrap(); }
        PromiseOperation::Thenable => { writeln!(source, "scenario_promise=scenario_promise.then(input=>({{get then(){{print('then-get');return function(resolve,reject){{print('then-call:'+input);resolve(input+({value}));reject(99);resolve(98);}};}}}}));").unwrap(); }
        PromiseOperation::Recover => { writeln!(source, "scenario_promise=scenario_promise.then(input=>Promise.reject({value})).catch(error=>{{if(error!=={value})throw error;print('recover:'+error);return error;}});").unwrap(); }
        PromiseOperation::Finally => source.push_str("scenario_promise=scenario_promise.finally(()=>{print('promise-finally');return Promise.resolve(41);});\n"),
        PromiseOperation::NestedJobs => { writeln!(source, "scenario_promise=scenario_promise.then(input=>{{const first=Promise.resolve().then(()=>{{print('job-first');return input;}});const second=Promise.resolve().then(()=>{{print('job-second');return {value};}});return Promise.all([first,second]).then(values=>values[0]+values[1]);}});").unwrap(); }
        PromiseOperation::Await => { writeln!(source, "scenario_promise=scenario_promise.then(async input=>{{print('await-before');const addition=await Promise.resolve({value});print('await-after');return input+addition;}});").unwrap(); }
        PromiseOperation::All => { writeln!(source, "scenario_promise=scenario_promise.then(input=>Promise.all([Promise.resolve(input),{value}]).then(values=>{{print('all:'+values.join('|'));return values[0]+values[1];}}));").unwrap(); }
        PromiseOperation::Race => { writeln!(source, "scenario_promise=scenario_promise.then(input=>Promise.race([Promise.resolve(input),Promise.resolve({value})]).then(result=>{{print('race:'+result);return result;}}));").unwrap(); }
    }
    writeln!(source, "scenario_promise=scenario_promise.then(result=>{{{state}=({state}+result)|0;print('scenario-step:{index}');return result;}});").unwrap();
}
