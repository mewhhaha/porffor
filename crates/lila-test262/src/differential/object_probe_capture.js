(function (probe) {
    "use strict";
    // Capture the actual primordials before the selected probe executes.
    var apply = Reflect.apply;
    var ownKeys = Reflect.ownKeys;
    var descriptor = Object.getOwnPropertyDescriptor;
    var prototype = Object.getPrototypeOf;
    var extensible = Object.isExtensible;
    var setPrototype = Object.setPrototypeOf;
    var isArray = Array.isArray;
    var stringify = JSON.stringify;
    var numberString = Number.prototype.toString;
    var bigintString = BigInt.prototype.toString;
    var charCode = String.prototype.charCodeAt;
    var keyFor = Symbol.keyFor;
    var symbolDescription = descriptor(Symbol.prototype, "description").get;
    var RangeErrorConstructor = RangeError;
    var wellKnownNames = ["asyncIterator", "hasInstance", "isConcatSpreadable", "iterator", "match", "matchAll", "replace", "search", "species", "split", "toPrimitive", "toStringTag", "unscopables", "dispose", "asyncDispose"];
    var wellKnownValues = [];
    for (var w = 0; w < wellKnownNames.length; w++) {
        wellKnownValues[w] = Symbol[wellKnownNames[w]];
    }
    var objects = list([]);
    var symbolValues = list([]);
    var nodes = list([]);
    var symbols = list([]);
    var totalUnits = 0;
    var totalKeys = 0;
    function fail(message) { throw new RangeErrorConstructor(message); }
    function list(value) { setPrototype(value, null); return value; }
    function record(value) { setPrototype(value, null); return value; }
    function append(array, value) { array[array.length] = value; }
    function units(value) {
        if (value.length > 32768 || totalUnits + value.length > 32768) {
            fail("selected object probe UTF-16 budget exceeded");
        }
        totalUnits += value.length;
        var output = list([]);
        for (var i = 0; i < value.length; i++) {
            append(output, apply(charCode, value, [i]));
        }
        return output;
    }
    function find(values, value) {
        for (var i = 0; i < values.length; i++) {
            if (values[i] === value) { return i; }
        }
        return -1;
    }
    function symbol(value) {
        var id = find(symbolValues, value);
        if (id !== -1) { return id; }
        if (symbolValues.length === 256) { fail("selected object probe Symbol budget exceeded"); }
        id = symbolValues.length;
        append(symbolValues, value);
        var name = apply(symbolDescription, value, []);
        var origin = record({kind: "local"});
        var registered = keyFor(value);
        if (registered !== undefined) {
            origin = record({kind: "registry", key: units(registered)});
        } else {
            for (var i = 0; i < wellKnownValues.length; i++) {
                if (wellKnownValues[i] === value) {
                    origin = record({kind: "well_known", name: wellKnownNames[i]});
                    break;
                }
            }
        }
        append(symbols, record({id: id, description: name === undefined ? null : units(name), origin: origin}));
        return id;
    }
    function value(input) {
        switch (typeof input) {
            case "undefined": return record({type: "undefined"});
            case "boolean": return record({type: "boolean", value: input});
            case "number": {
                var decimal;
                if (input !== input) { decimal = "NaN"; }
                else if (input === Infinity) { decimal = "Infinity"; }
                else if (input === -Infinity) { decimal = "-Infinity"; }
                else if (input === 0 && 1 / input === -Infinity) { decimal = "-0"; }
                else { decimal = apply(numberString, input, []); }
                return record({type: "number_decimal", decimal: decimal});
            }
            case "string": return record({type: "string", units: units(input)});
            case "bigint": return record({type: "big_int", decimal: apply(bigintString, input, [])});
            case "symbol": return record({type: "symbol", id: symbol(input)});
            case "object": if (input === null) { return record({type: "null"}); }
            case "function": {
                var id = find(objects, input);
                if (id === -1) {
                    if (objects.length === 512) { fail("selected object probe object budget exceeded"); }
                    id = objects.length;
                    append(objects, input);
                }
                return record({type: "object", id: id});
            }
        }
        fail("selected object probe value domain rejected");
    }
    function ownData(object, name) {
        var property = descriptor(object, name);
        if (property === undefined || descriptor(property, "value") === undefined) {
            fail("selected object probe plan requires own data fields");
        }
        return property.value;
    }
    function named(input, identityOnly) {
        if (!isArray(input) || input.length > 16) { fail("selected object probe root budget exceeded"); }
        var output = list([]);
        for (var i = 0; i < input.length; i++) {
            var entry = ownData(input, apply(numberString, i, []));
            var name = ownData(entry, "name");
            var selected = ownData(entry, "value");
            if (typeof name !== "string" || name.length === 0 || name.length > 64) {
                fail("selected object probe root name rejected");
            }
            for (var n = 0; n < name.length; n++) {
                var code = apply(charCode, name, [n]);
                if (!((code >= 97 && code <= 122) || (code >= 48 && code <= 57) || code === 45 || code === 95)) {
                    fail("selected object probe root name rejected");
                }
            }
            for (var n = 0; n < output.length; n++) {
                if (output[n].name === name) { fail("selected object probe root name repeated"); }
            }
            if (identityOnly && !((typeof selected === "object" && selected !== null) || typeof selected === "function" || typeof selected === "symbol")) {
                fail("selected object probe anchor requires an identity");
            }
            append(output, record({name: name, value: value(selected)}));
        }
        return output;
    }
    // These reflection operations intentionally observe the selected objects.
    // Proxy traps/throws are part of the probe; none is silently bypassed.
    var plan = probe();
    var roots = named(ownData(plan, "roots"), false);
    if (roots.length === 0) { fail("selected object probe requires a root"); }
    var anchors = named(ownData(plan, "anchors"), true);
    for (var i = 0; i < objects.length; i++) {
        var object = objects[i];
        var parent = value(prototype(object));
        var keys = ownKeys(object);
        if (keys.length > 4096 || totalKeys + keys.length > 4096) { fail("selected object probe own-key budget exceeded"); }
        totalKeys += keys.length;
        var properties = list([]);
        for (var k = 0; k < keys.length; k++) {
            var key = keys[k];
            var encodedKey = typeof key === "symbol" ? record({type: "symbol", id: symbol(key)}) : record({type: "string", units: units(key)});
            var property = descriptor(object, key);
            if (property === undefined) { fail("selected object probe key disappeared during snapshot"); }
            var encodedDescriptor;
            if (descriptor(property, "value") !== undefined) {
                encodedDescriptor = record({kind: "data", value: value(property.value), writable: property.writable, enumerable: property.enumerable, configurable: property.configurable});
            } else {
                encodedDescriptor = record({kind: "accessor", get: value(property.get), set: value(property.set), enumerable: property.enumerable, configurable: property.configurable});
            }
            append(properties, record({key: encodedKey, descriptor: encodedDescriptor}));
        }
        var kind = typeof object === "function" ? "function" : isArray(object) ? "array" : "object";
        append(nodes, record({id: i, kind: kind, extensible: extensible(object), prototype: parent, properties: properties}));
    }
    var output = stringify(record({version: 1, roots: roots, anchors: anchors, nodes: nodes, symbols: symbols}));
    if (output.length > 131072) { fail("selected object probe wire budget exceeded"); }
    return output;
})
