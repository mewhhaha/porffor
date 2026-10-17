use super::*;

#[test]
fn native_direct_eval_keeps_caller_lexical_writes_var_admission_and_strictness() {
    let body = "lexical+=2; variable+=3; var added=42; lexical+variable;";
    for (directive, added) in [("", "number"), ("'use strict';", "undefined")] {
        let source = format!(
            r#"
function run(text){{{directive} let lexical=17; var variable=23;
 var result=eval(text);print(lexical);print(variable);print(result);print(typeof added);
}}
run({text});print(typeof added);
"#,
            text = converted_text(body)
        );
        assert_trace(
            &source,
            &["19", "26", "45", added, "undefined"],
            &[units(body)],
            (1, 1, 0),
        );
    }
}

#[test]
fn native_direct_eval_uses_actual_this_arguments_and_new_target_records() {
    let target = "new.target;";
    let value = "this.marker+arguments[1];";
    let source = format!(
        r#"
var target={target}, value={value};
function C(text, amount){{this.marker=17;print(eval(text)===new.target);print(eval(value));}}
new C(target,25);
"#,
        target = converted_text(target),
        value = converted_text(value)
    );
    assert_trace(
        &source,
        &["true", "42"],
        &[units(target), units(value)],
        (2, 2, 0),
    );
}

#[test]
fn native_unicode_private_names_keep_hidden_ancestor_scope_and_escaping_gc_roots() {
    let body = "(function(){gc();return outer.#秘密+(new(class Local {#fresh=25;get(){return this.#fresh;}})).get();});";
    let source = format!(
        r#"
class Outer {{#秘密=17;make(text){{const outer=this;
 class Inner {{accessor backing=23;make(text){{return eval(text);}}}}
 return new Inner().make(text);
}}}}
var retained=new Outer().make({text});gc();print(retained());gc();print(retained());
"#,
        text = converted_text(body)
    );
    let observed = assert_trace(&source, &["42", "42"], &[units(body)], (1, 1, 0));
    assert!(observed.private_environment_floor > observed.visible_private_floor,
        "the actual hidden auto-accessor scope must contribute to the P2 allocation floor: {observed:?}");
}

#[test]
fn native_super_arrow_initializes_the_defining_constructor_after_gc() {
    let body = "(()=>super(25));";
    let source = format!(
        r#"
class Base {{constructor(value){{this.base=value;}}}}
class Derived extends Base {{#secret=17;constructor(text){{
 var invoke=eval(text);gc();invoke();print(this.total());
 }}total(){{return this.base+this.#secret;}}}}
new Derived({text});
"#,
        text = converted_text(body)
    );
    assert_trace(&source, &["42"], &[units(body)], (1, 1, 0));
}
