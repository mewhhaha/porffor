
        const first = new ShadowRealm();
        const second = new ShadowRealm();
        const evaluate = first.evaluate;
        const evaluateKey = 'evaluate', callKey = 'call', applyKey = 'apply';
        if (evaluate.call(first, 'globalThis.value = 11; value') !== 11) throw 'first-realm';
        if (evaluate.call(second, 'typeof value') !== 'undefined') throw 'receiver-realm';
        if (second.evaluate?.('globalThis.value = 17; value') !== 17) throw 'optional-evaluate';
        if (first.evaluate('value') !== 11) throw 'independent-state';
        if (first.evaluate("new ShadowRealm().evaluate('41 + 1')") !== 42) throw 'nested-source';
        if (evaluate?.call(first, '101 + 1') !== 102) throw 'optional-alias-call';
        if (evaluate.call?.(second, '102 + 1') !== 103) throw 'optional-call-method';
        if (first?.evaluate.call(second, '103 + 1') !== 104) throw 'optional-direct-call';
        if (first.evaluate.call?.(second, '104 + 1') !== 105) throw 'optional-direct-call-method';
        if (evaluate?.apply(first, ['105 + 1']) !== 106) throw 'optional-alias-apply';
        if (evaluate.apply?.(second, ['106 + 1']) !== 107) throw 'optional-apply-method';
        if (first?.evaluate.apply(second, ['107 + 1']) !== 108) throw 'optional-direct-apply';
        if (first.evaluate.apply?.(second, ['108 + 1']) !== 109) throw 'optional-direct-apply-method';
        if (((source) => first.evaluate(source))?.('109 + 1') !== 110) throw 'arrow-source-owner';
        if ((function(source) { return first.evaluate(source); })?.('110 + 1') !== 111) throw 'function-source-owner';
        if (first?.[evaluateKey]('201 + 1') !== 202) throw 'optional-named-key';
        if (first?.[evaluateKey][callKey](second, '202 + 1') !== 203) throw 'optional-named-call-key';
        if (first[evaluateKey][callKey]?.(second, '203 + 1') !== 204) throw 'optional-call-key-method';
        if (evaluate?.[applyKey](second, ['204 + 1']) !== 205) throw 'optional-named-apply-key';
        const custom = { evaluate(source) { return source; } };
        if (custom.evaluate('let = ;') !== 'let = ;') throw 'replaced-evaluate';
        if (custom?.evaluate.call(custom, 'let = ;') !== 'let = ;') throw 'optional-custom-call';
        if (custom.evaluate.apply?.(custom, ['let = ;']) !== 'let = ;') throw 'optional-custom-apply';

        let trace = '';
        const nativeCall = Function.prototype.call;
        Object.defineProperty(evaluate, 'call', {
          __proto__: null, configurable: true,
          get() { trace += 'get,'; return nativeCall; }
        });
        const result = evaluate?.call(first, (trace += 'arg,',
          Object.defineProperty(evaluate, 'call', {
            __proto__: null, configurable: true,
            value: function(receiver, source) {
              trace += 'custom,';
              if (this !== evaluate || receiver !== first) throw 'custom-reference';
              return source;
            }
          }), '111 + 1'));
        if (result !== 112 || trace !== 'get,arg,') throw 'acquired-call-before-arguments';
        if (evaluate.call?.(first, 'let = ;') !== 'let = ;' || trace !== 'get,arg,custom,')
          throw 'retained-custom-call';

        let skipped = 0;
        const absent = null;
        if (absent?.call(first, (++skipped, 'throw 1')) !== undefined) throw 'null-callee';
        Object.defineProperty(evaluate, 'call', { __proto__: null, value: undefined });
        if (evaluate.call?.(first, (++skipped, 'throw 2')) !== undefined) throw 'missing-call';
        Object.defineProperty(evaluate, 'apply', { __proto__: null, value: null });
        if (evaluate.apply?.(first, [(++skipped, 'throw 3')]) !== undefined) throw 'missing-apply';
        if (skipped !== 0) throw 'shorted-argument-effects';
        true;
