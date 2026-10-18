use super::succeeds;

#[test]
fn thousand_shared_method_bodies_keep_distinct_objects_names_and_source_text() {
    succeeds(
        r#"
        var code = '({';
        for (var i = 0; i < 1000; i++)
            code += "['foo' + " + i + "]() {return 'ok';}, ";
        code += "['bar']() {return 'all ok'}});";
        var object = eval(code), prior;
        for (var i = 0; i < 1000; i++) {
            var name = 'foo' + i, fn = object[name];
            if (fn === prior || fn.name !== name || fn() !== 'ok') throw 1;
            if (fn.toString().indexOf("['foo' + " + i + "]") < 0) throw 2;
            if (Object.prototype.hasOwnProperty.call(fn, 'prototype')) throw 3;
            prior = fn;
        }
        if (object.bar() !== 'all ok') throw 4;
        1;
    "#,
    );
}

#[test]
fn aliased_constructor_bodies_keep_their_original_instance_element_plan_ids() {
    succeeds(
        r#"
        var create = Function('return [class First { #value = 17; read() { return this.#value; } }, class Second { #value = 29; read() { return this.#value; } }];');
        var classes = create(), First = classes[0], Second = classes[1];
        var a = new First(), b = new Second();
        if (a.read() !== 17 || b.read() !== 29) throw 1;
        if (First === Second || First.name !== 'First' || Second.name !== 'Second') throw 2;
        var throws = 0;
        try { First.prototype.read.call(b); } catch (e) { if (!(e instanceof TypeError)) throw e; throws++; }
        try { Second.prototype.read.call(a); } catch (e) { if (!(e instanceof TypeError)) throw e; throws++; }
        if (throws !== 2) throw 3;
        if (First.toString().indexOf('17') < 0 || Second.toString().indexOf('29') < 0) throw 4;
        1;
    "#,
    );
}
