function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
function name(code,style,languageDisplay){return new Intl.DisplayNames('en-US',{type:'language',style:style,languageDisplay:languageDisplay}).of(code);}
same(name('EN-gb','long','dialect'),'British English','canonical dialect');
same(name('en-GB','short','dialect'),'UK English','actual short dialect');
same(name('en-GB','long','standard'),'English (United Kingdom)','standard qualifiers');
same(name('en-GB','short','standard'),'English (UK)','short qualifier');
same(name('zh-Hans','long','dialect'),'Simplified Chinese','dialect script');
same(name('zh-Hans','long','standard'),'Chinese (Simplified)','composition script distinct from standalone');
same(name('es-Cyrl-MX','long','dialect'),'Mexican Spanish (Cyrillic)','multiple genuine qualifiers');
same(name('IW','long','dialect'),'Hebrew','language alias');
same(new Intl.DisplayNames('ar',{type:'language',style:'long'}).of('en-GB'),'الإنجليزية (المملكة المتحدة)','long does not borrow short-only dialect');
print('ok language_dialects_and_aliases');
262;
