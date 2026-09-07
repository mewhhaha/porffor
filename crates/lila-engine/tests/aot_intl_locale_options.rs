use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Intl.Locale options must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn core_overrides_update_every_represented_slot() {
    assert_wasm_true(
        r#"
var locale = new Intl.Locale("en", { language: "DE", script: "lAtN", region: "at" });
locale.toString() === "de-Latn-AT" && locale.baseName === "de-Latn-AT" &&
locale.language === "de" && locale.script === "Latn" && locale.region === "AT";
"#,
    );
}

#[test]
fn replacements_preserve_variants_extensions_and_private_use() {
    assert_wasm_true(
        r#"
var locale = new Intl.Locale("en-Latn-US-1901-a-foo-u-ca-gregory-x-keep", {
  language: "de", script: "cyrl", region: "at"
});
locale.toString() === "de-Cyrl-AT-1901-a-foo-u-ca-gregory-x-keep" &&
locale.baseName === "de-Cyrl-AT-1901" && locale.script === "Cyrl" && locale.region === "AT";
"#,
    );
}

#[test]
fn omitted_fields_preserve_the_tag_and_absent_slots() {
    assert_wasm_true(
        r#"
var a = new Intl.Locale("fr-u-nu-latn", { region: "ca" });
var b = new Intl.Locale("en-Latn-US-x-keep", {
  language: undefined, script: undefined, region: undefined
});
a.toString() === "fr-CA-u-nu-latn" && a.script === undefined &&
a.language === "fr" && a.region === "CA" && a.baseName === "fr-CA" &&
b.toString() === "en-Latn-US-x-keep" && b.baseName === "en-Latn-US";
"#,
    );
}

#[test]
fn each_get_coercion_and_validation_precedes_the_next_get() {
    assert_wasm_true(
        r#"
var log = "";
var options = {
  get language() {
    log += "L";
    return { toString() { log += "l"; return "de"; } };
  },
  get script() {
    log += "S";
    return { toString() { log += "s"; return "latn"; } };
  },
  get region() {
    log += "R";
    return { toString() { log += "r"; return "at"; } };
  }
};
var result = new Intl.Locale("en", options);
log === "LlSsRr" && result.toString() === "de-Latn-AT";
"#,
    );
}

#[test]
fn invalid_language_prevents_script_and_region_observation() {
    assert_wasm_true(
        r#"
var log = "";
var caught = false;
try {
  new Intl.Locale("en", {
    get language() { log += "L"; return "abcd"; },
    get script() { log += "S"; throw 23; },
    get region() { log += "R"; throw 42; }
  });
} catch (error) { caught = error instanceof RangeError; }
caught && log === "L";
"#,
    );
}

#[test]
fn invalid_script_prevents_region_observation() {
    assert_wasm_true(
        r#"
var log = "";
var caught = false;
try {
  new Intl.Locale("en", {
    get language() { log += "L"; return "de"; },
    get script() { log += "S"; return "latin"; },
    get region() { log += "R"; throw 42; }
  });
} catch (error) { caught = error instanceof RangeError; }
caught && log === "LS";
"#,
    );
}

#[test]
fn getter_and_conversion_exceptions_preserve_identity() {
    assert_wasm_true(
        r#"
var sentinel = {};
var log = "";
var getterCaught = false;
var conversionCaught = false;
try {
  new Intl.Locale("en", {
    get language() { throw sentinel; },
    get script() { log += "S"; }
  });
} catch (error) { getterCaught = error === sentinel; }
try {
  new Intl.Locale("en", {
    language: { toString() { throw sentinel; } },
    get script() { log += "S"; }
  });
} catch (error) { conversionCaught = error === sentinel; }
getterCaught && conversionCaught && log === "";
"#,
    );
}

#[test]
fn inherited_accessors_keep_the_original_receiver() {
    assert_wasm_true(
        r#"
var seen = false;
var prototype = {
  get region() { seen = this === options; return this.selectedRegion; }
};
var options = Object.create(prototype);
options.selectedRegion = "ca";
var locale = new Intl.Locale("en", options);
seen && locale.toString() === "en-CA";
"#,
    );
}

#[test]
fn proxy_options_keep_live_reads_and_receiver_identity() {
    assert_wasm_true(
        r#"
var log = "";
var receivers = true;
var options = new Proxy({ language: "de", region: "at" }, {
  get(target, key, receiver) {
    if (key === "language" || key === "script" || key === "region") {
      log += key + ";";
      receivers = receivers && receiver === options;
    }
    if (key === "language") target.script = "Latn";
    return Reflect.get(target, key, receiver);
  }
});
var locale = new Intl.Locale("en", options);
receivers && log === "language;script;region;" && locale.toString() === "de-Latn-AT";
"#,
    );
}

#[test]
fn primitive_options_are_boxed_once_and_undefined_has_no_prototype() {
    assert_wasm_true(
        r#"
var seen = false;
var numberGets = 0;
var objectGets = 0;
Object.defineProperty(Number.prototype, "region", {
  configurable: true,
  get() {
    numberGets++;
    seen = typeof this === "object" && this.valueOf() === 7;
    return "ca";
  }
});
Object.defineProperty(Object.prototype, "language", {
  configurable: true,
  get() { objectGets++; return "de"; }
});
var absent = new Intl.Locale("en");
var explicitUndefined = new Intl.Locale("en", undefined);
var boxed = new Intl.Locale("en", 7);
delete Number.prototype.region;
delete Object.prototype.language;
seen && numberGets === 1 && objectGets === 1 &&
absent.toString() === "en" && explicitUndefined.toString() === "en" &&
boxed.toString() === "de-CA";
"#,
    );
}

#[test]
fn null_options_fail_after_tag_conversion_but_before_tag_validation() {
    assert_wasm_true(
        r#"
var log = "";
var caught = false;
try {
  new Intl.Locale({ toString() { log += "tag"; return "not_a_tag"; } }, null);
} catch (error) { caught = error instanceof TypeError; }
caught && log === "tag";
"#,
    );
}

#[test]
fn invalid_tag_prevents_every_options_getter() {
    assert_wasm_true(
        r#"
var gets = 0;
var caught = false;
try {
  new Intl.Locale("not_a_tag", {
    get language() { gets++; return "en"; },
    get script() { gets++; return "Latn"; },
    get region() { gets++; return "US"; }
  });
} catch (error) { caught = error instanceof RangeError; }
caught && gets === 0;
"#,
    );
}

#[test]
fn options_cannot_rescue_an_invalid_input_tag() {
    assert_wasm_true(
        r#"
var caught = false;
try { new Intl.Locale("abcd", { language: "en" }); }
catch (error) { caught = error instanceof RangeError; }
caught;
"#,
    );
}

#[test]
fn subtag_validation_rejects_bad_lengths_characters_and_compound_values() {
    assert_wasm_true(
        r#"
var invalid = [
  ["language", ""], ["language", "a"], ["language", "abcd"],
  ["language", "abcdefghi"], ["language", "en-US"], ["language", "12"],
  ["language", "e_"], ["language", "\u00e9n"], ["language", "en\u0000"],
  ["script", ""], ["script", "Lat"], ["script", "Latin"],
  ["script", "1234"], ["script", "L-tN"], ["script", "\u0130atn"],
  ["region", ""], ["region", "U"], ["region", "USA"],
  ["region", "12"], ["region", "1234"], ["region", "U1"],
  ["region", "1A2"], ["region", "US-x-foo"], ["region", "\u00dcS"]
];
var ok = true;
for (var i = 0; i < invalid.length; i++) {
  var options = {};
  options[invalid[i][0]] = invalid[i][1];
  var caught = false;
  try { new Intl.Locale("en", options); }
  catch (error) { caught = error instanceof RangeError; }
  ok = ok && caught;
}
ok;
"#,
    );
}

#[test]
fn supported_grammar_includes_unknown_subtags_and_numeric_region_coercion() {
    assert_wasm_true(
        r#"
var a = new Intl.Locale("en", { language: "AbCdE", script: "qwer", region: 419 });
var b = new Intl.Locale("en", { language: "AbCdEfGh", script: null, region: "zz" });
a.toString() === "abcde-Qwer-419" && a.region === "419" &&
b.toString() === "abcdefgh-Null-ZZ";
"#,
    );
}

#[test]
fn symbol_values_throw_instead_of_becoming_description_strings() {
    assert_wasm_true(
        r#"
var ok = true;
var properties = ["language", "script", "region"];
for (var i = 0; i < properties.length; i++) {
  var options = {};
  options[properties[i]] = Symbol("en");
  var caught = false;
  try { new Intl.Locale("en", options); }
  catch (error) { caught = error instanceof TypeError; }
  ok = ok && caught;
}
ok;
"#,
    );
}

#[test]
fn branded_locale_input_uses_its_slot_before_overrides() {
    assert_wasm_true(
        r#"
var original = new Intl.Locale("fr-Latn-FR-u-ca-gregory-x-keep");
original.toString = function () { throw 42; };
var locale = new Intl.Locale(original, { region: "ca" });
locale.toString() === "fr-Latn-CA-u-ca-gregory-x-keep" &&
locale.baseName === "fr-Latn-CA" && original.region === "FR";
"#,
    );
}

#[test]
fn construction_order_and_result_prototype_are_preserved() {
    assert_wasm_true(
        r#"
var log = "";
var prototype = {};
function Target() {}
var target = new Proxy(Target, {
  get(value, key, receiver) {
    if (key === "prototype") { log += "P"; return prototype; }
    return Reflect.get(value, key, receiver);
  }
});
var locale = Reflect.construct(Intl.Locale, [
  { toString() { log += "T"; return "en"; } },
  { get region() { log += "R"; return "us"; } }
], target);
log === "PTR" && Object.getPrototypeOf(locale) === prototype &&
Intl.Locale.prototype.toString.call(locale) === "en-US";
"#,
    );
}
