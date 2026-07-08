# Test262 Shortcut Inventory

Source: `crates/porffor-test262/src/lib.rs`

This report is deterministic and intentionally mechanical. Classify each match as a legitimate harness adaptation, temporary diagnostic instrumentation, or semantic shortcut before closing T03.

## Path-Based Rewrite Entrypoints

Count: 107

```text
925:            if include == "testTypedArray.js" && wasm_aot_rewrite_skips_test_typed_array(&case.path)
1086:    if let Some(source) = rewrite_proxy_prevent_extensions_case(&case.path) {
1089:    if let Some(source) = rewrite_proxy_define_property_case(&case.path) {
1092:    if let Some(source) = rewrite_proxy_get_own_property_descriptor_case(&case.path) {
1095:    if let Some(source) = rewrite_proxy_create_case(&case.path) {
1098:    if let Some(source) = rewrite_proxy_revocable_case(&case.path) {
1101:    if let Some(source) = rewrite_proxy_apply_case(&case.path) {
1104:    if let Some(source) = rewrite_proxy_construct_case(&case.path) {
1107:    if let Some(source) = rewrite_iterator_from_return_method_case(&case.path) {
1110:    if let Some(source) = rewrite_iterator_to_array_case(&case.path) {
1113:    if let Some(source) = rewrite_iterator_for_each_case(&case.path) {
1116:    if let Some(source) = rewrite_iterator_every_case(&case.path) {
1119:    if let Some(source) = rewrite_iterator_some_case(&case.path) {
1122:    if let Some(source) = rewrite_iterator_find_case(&case.path) {
1125:    if let Some(source) = rewrite_iterator_reduce_case(&case.path) {
1128:    if let Some(source) = rewrite_iterator_map_case(&case.path) {
1131:    if let Some(source) = rewrite_iterator_filter_case(&case.path) {
1134:    if let Some(source) = rewrite_iterator_flat_map_case(&case.path) {
1137:    if let Some(source) = rewrite_iterator_flat_map_staging_case(&case.path) {
1140:    if let Some(source) = rewrite_iterator_take_case(&case.path) {
1143:    if let Some(source) = rewrite_iterator_drop_case(&case.path) {
1146:    if let Some(source) = rewrite_iterator_constructor_case(&case.path) {
1149:    if let Some(source) = rewrite_iterator_to_string_tag_case(&case.path) {
1152:    if let Some(source) = rewrite_function_tostring_sputnik_case(&case.path) {
1155:    if let Some(source) = rewrite_function_tostring_builtin_object_case(&case.path) {
1158:    if let Some(source) = rewrite_string_char_at_legacy_case(&case.path) {
1161:    if let Some(source) = rewrite_string_char_at_metadata_case(&case.path) {
1164:    if let Some(source) = rewrite_string_char_at_position_case(&case.path) {
1170:    if let Some(source) = rewrite_string_char_code_at_legacy_case(&case.path) {
1173:    if let Some(source) = rewrite_string_char_code_at_metadata_case(&case.path) {
1176:    if let Some(source) = rewrite_string_code_point_at_metadata_case(&case.path) {
1179:    if let Some(source) = rewrite_string_ends_with_metadata_case(&case.path) {
1182:    if let Some(source) = rewrite_string_index_of_legacy_case(&case.path) {
1185:    if let Some(source) = rewrite_string_index_of_metadata_case(&case.path) {
1188:    if let Some(source) = rewrite_string_last_index_of_metadata_case(&case.path) {
1191:    if let Some(source) = rewrite_string_includes_metadata_case(&case.path) {
1194:    if let Some(source) = rewrite_string_match_legacy_case(&case.path) {
1197:    if let Some(source) = rewrite_string_match_metadata_case(&case.path) {
1200:    if let Some(source) = rewrite_string_match_all_metadata_case(&case.path) {
1203:    if let Some(source) = rewrite_string_match_all_unicode_case(&case.path) {
1206:    if let Some(source) = rewrite_string_match_all_nullish_regexp_case(&case.path) {
1209:    if let Some(source) = rewrite_regexp_prototype_symbol_search_metadata_case(&case.path) {
1212:    if let Some(source) = rewrite_string_search_metadata_case(&case.path) {
1215:    if let Some(source) = rewrite_string_repeat_metadata_case(&case.path) {
1218:    if let Some(source) = rewrite_string_pad_start_metadata_case(&case.path) {
1221:    if let Some(source) = rewrite_string_pad_end_metadata_case(&case.path) {
1224:    if let Some(source) = rewrite_string_to_string_value_of_metadata_case(&case.path) {
1227:    if let Some(source) = rewrite_string_to_string_value_of_non_generic_realm_case(&case.path) {
1230:    if let Some(source) = rewrite_string_well_formed_metadata_case(&case.path) {
1233:    if let Some(source) = rewrite_string_at_metadata_case(&case.path) {
1236:    if let Some(source) = rewrite_string_well_formed_primitive_coercion_case(&case.path) {
1239:    if let Some(source) = rewrite_string_trim_metadata_case(&case.path) {
1242:    if let Some(source) = rewrite_string_starts_with_metadata_case(&case.path) {
1245:    if let Some(source) = rewrite_annexb_string_prototype_method_metadata_case(&case.path) {
1248:    if let Some(source) = rewrite_annexb_escape_unescape_metadata_case(&case.path) {
1251:    if let Some(source) = rewrite_undefined_legacy_initial_value_case(&case.path) {
1254:    if let Some(source) = rewrite_global_constant_prop_desc_case(&case.path) {
1257:    if let Some(source) = rewrite_number_constructor_metadata_case(&case.path) {
1260:    if let Some(source) = rewrite_number_constant_metadata_case(&case.path) {
1263:    if let Some(source) = rewrite_number_parse_function_metadata_case(&case.path) {
1266:    if let Some(source) = rewrite_number_predicate_metadata_case(&case.path) {
1269:    if let Some(source) = rewrite_number_prototype_method_metadata_case(&case.path) {
1272:    if let Some(source) = rewrite_boolean_constructor_metadata_case(&case.path) {
1275:    if let Some(source) = rewrite_boolean_prototype_method_metadata_case(&case.path) {
1281:    if let Some(source) = rewrite_boolean_legacy_conversion_case(&case.path) {
1293:    if let Some(source) = rewrite_native_error_metadata_case(&case.path) {
1296:    if let Some(source) = rewrite_array_prototype_method_metadata_case(&case.path) {
1299:    if let Some(source) = rewrite_array_to_string_non_callable_join_case(&case.path) {
1302:    if let Some(source) = rewrite_array_to_string_sputnik_conversion_case(&case.path) {
1305:    if let Some(source) = rewrite_array_at_resizable_case(&case.path) {
1308:    if let Some(source) = rewrite_typedarray_at_case(&case.path) {
1311:    if let Some(source) = rewrite_bigint_typedarray_constructor_case(&case.path) {
1314:    if let Some(source) = rewrite_array_includes_resizable_case(&case.path) {
1317:    if let Some(source) = rewrite_array_index_of_resizable_case(&case.path) {
1320:    if let Some(source) = rewrite_array_last_index_of_resizable_case(&case.path) {
1323:    if let Some(source) = rewrite_array_iteration_resizable_buffer_case(&case.path) {
1326:    if let Some(source) = rewrite_array_to_locale_string_resizable_case(&case.path) {
1329:    if let Some(source) = rewrite_typedarray_to_string_case(&case.path) {
1332:    if let Some(source) = rewrite_typedarray_to_locale_string_case(&case.path) {
1335:    if let Some(source) = rewrite_array_iterator_resizable_case(&case.path) {
1338:    if let Some(source) = rewrite_array_values_resizable_case(&case.path) {
1341:    if let Some(source) = rewrite_array_values_resizable_mid_iteration_case(&case.path) {
1344:    if let Some(source) = rewrite_array_iterator_resizable_mid_iteration_case(&case.path) {
1347:    if let Some(source) = rewrite_array_iteration_resizable_mid_iteration_case(&case.path) {
1350:    if let Some(source) = rewrite_reflect_set_metadata_case(&case.path) {
1353:    if let Some(source) = rewrite_reflect_set_prototype_of_metadata_case(&case.path) {
1356:    if let Some(source) = rewrite_throw_type_error_distinct_cross_realm_case(&case.path) {
1359:    if let Some(source) = rewrite_throw_type_error_metadata_case(&case.path) {
1362:    if let Some(source) = rewrite_error_is_error_prop_desc_case(&case.path) {
1365:    if let Some(source) = rewrite_error_prototype_prop_desc_case(&case.path) {
1368:    if let Some(source) = rewrite_error_prototype_core_case(&case.path) {
1371:    if let Some(source) = rewrite_arraybuffer_accessor_metadata_case(&case.path) {
1374:    if let Some(source) = rewrite_arraybuffer_accessor_wrong_receiver_case(&case.path) {
1377:    if let Some(source) = rewrite_dataview_constructor_case(&case.path) {
1380:    if let Some(source) = rewrite_dataview_accessor_metadata_case(&case.path) {
1383:    if let Some(source) = rewrite_dataview_accessor_wrong_receiver_case(&case.path) {
1386:    if let Some(source) = rewrite_dataview_method_metadata_case(&case.path) {
1389:    if let Some(source) = rewrite_dataview_method_wrong_receiver_case(&case.path) {
1392:    if let Some(source) = rewrite_dataview_method_abrupt_tonumber_case(&case.path) {
1395:    if let Some(source) = rewrite_dataview_method_range_case(&case.path) {
1398:    if let Some(source) = rewrite_dataview_bigint_get_toindex_case(&case.path) {
1401:    if let Some(source) = rewrite_dataview_numeric_set_conversion_case(&case.path) {
1404:    if let Some(source) = rewrite_dataview_method_detached_case(&case.path) {
1407:    if let Some(source) = rewrite_dataview_method_resizable_case(&case.path) {
1410:    if let Some(source) = rewrite_arraybuffer_slice_metadata_case(&case.path) {
1413:    if let Some(source) = rewrite_arraybuffer_slice_wrong_receiver_case(&case.path) {
8950:    if let Some(source) = rewrite_arraybuffer_isview_typed_array_case(&case.path) {
```

## Direct Path Predicates

Count: 358

```text
931:                    && case.path == "built-ins/AggregateError/order-of-args-evaluation.js"
1440:    if case.path.ends_with("built-ins/Error/isError/errors.js") {
1469:    if case.path.ends_with("built-ins/Error/message_property.js") {
1483:    if case.path.ends_with("built-ins/Error/cause_property.js") {
1540:    if case.path.ends_with("built-ins/AggregateError/length.js") {
1552:    if case.path.ends_with("built-ins/AggregateError/name.js") {
1604:    if case.path.ends_with("built-ins/AggregateError/prop-desc.js") {
1683:    if case.path.ends_with("built-ins/SuppressedError/length.js") {
1695:    if case.path.ends_with("built-ins/SuppressedError/name.js") {
1864:    if case.path.ends_with("built-ins/Error/prop-desc.js") {
1876:    if case.path.ends_with("built-ins/Error/instance-prototype.js") {
2015:    if path.ends_with("built-ins/Proxy/preventExtensions/trap-is-undefined-target-is-proxy.js") {
2037:    if path.ends_with("built-ins/Proxy/defineProperty/trap-is-undefined.js") {
2072:    if path.ends_with("built-ins/Proxy/defineProperty/trap-is-null-target-is-proxy.js") {
2126:    if path.ends_with("built-ins/Proxy/defineProperty/return-boolean-and-define-target.js") {
2209:    if path.ends_with("built-ins/Proxy/getOwnPropertyDescriptor/trap-is-null-target-is-proxy.js") {
2264:    if path.ends_with("built-ins/Proxy/getOwnPropertyDescriptor/trap-is-missing-target-is-proxy.js")
2297:    if path.ends_with("built-ins/Proxy/getOwnPropertyDescriptor/trap-is-undefined.js") {
2331:    if path.ends_with("built-ins/Proxy/create-target-is-not-callable.js") {
2342:    if path.ends_with("built-ins/Proxy/create-target-is-not-a-constructor.js") {
2358:    if path.ends_with("built-ins/Proxy/create-target-is-revoked-function-proxy.js") {
2370:    if path.ends_with("built-ins/Proxy/create-target-is-revoked-proxy.js") {
2386:    if !path.contains("built-ins/Proxy/revocable/") {
2589:    if path.ends_with("built-ins/Proxy/apply/arguments-realm.js") {
2603:    if path.ends_with("built-ins/Proxy/apply/null-handler-realm.js") {
2616:    if path.ends_with("built-ins/Proxy/apply/trap-is-not-callable-realm.js") {
2634:    if path.ends_with("built-ins/Proxy/construct/arguments-realm.js") {
2649:    if path.ends_with(
2664:    if path.ends_with("built-ins/Proxy/construct/trap-is-undefined-proto-from-newtarget-realm.js") {
2683:    match path {
2883:    match path {
3018:    match path {
3272:    match path {
3691:    match path {
4094:    match path {
4537:    match path {
4958:    match path {
7672:    match path {
8031:    match path {
8979:    if !path.ends_with("built-ins/Function/prototype/toString/built-in-function-object.js")
8980:        && !path.ends_with("staging/sm/Function/function-toString-builtin.js")
9024:    if !path.contains("built-ins/Function/prototype/toString/S15.3.4.2_A") {
9125:    if path.ends_with("built-ins/String/prototype/charAt/S15.5.4.4_A1.1.js") {
9147:    if path.ends_with("built-ins/String/prototype/charAt/S15.5.4.4_A1_T1.js") {
9166:    if path.ends_with("built-ins/String/prototype/charAt/S15.5.4.4_A1_T2.js") {
9189:    if path.ends_with("built-ins/String/prototype/charAt/S15.5.4.4_A10.js") {
9212:    if !path.ends_with("built-ins/String/prototype/charAt/name.js") {
9233:    if path.ends_with("built-ins/String/prototype/charAt/pos-coerce-string.js") {
9247:    if path.ends_with("built-ins/String/prototype/charAt/pos-rounding.js") {
9268:    let path = case.path.as_str();
9269:    if !path.starts_with("built-ins/String/prototype/slice/S15.5.4.13_A1_T") {
9282:    match path.rsplit('/').next().unwrap_or_default() {
9429:    if !path.ends_with("built-ins/String/prototype/charCodeAt/S15.5.4.5_A1.1.js") {
9455:    if path.ends_with("built-ins/String/prototype/charCodeAt/S15.5.4.5_A10.js") {
9478:    if !path.ends_with("built-ins/String/prototype/charCodeAt/name.js") {
9499:    if path.ends_with("built-ins/String/prototype/codePointAt/codePointAt.js") {
9515:    if path.ends_with("built-ins/String/prototype/codePointAt/length.js") {
9532:    if !path.ends_with("built-ins/String/prototype/codePointAt/name.js") {
9553:    if path.ends_with("built-ins/String/prototype/startsWith/startsWith.js") {
9569:    if path.ends_with("built-ins/String/prototype/startsWith/length.js") {
9586:    if !path.ends_with("built-ins/String/prototype/startsWith/name.js") {
9607:    if path.ends_with("built-ins/String/prototype/endsWith/endsWith.js") {
9623:    if path.ends_with("built-ins/String/prototype/endsWith/length.js") {
9640:    if !path.ends_with("built-ins/String/prototype/endsWith/name.js") {
9661:    if !path.ends_with("built-ins/String/prototype/indexOf/S15.5.4.7_A3_T2.js") {
9677:    if path.ends_with("built-ins/String/prototype/indexOf/S15.5.4.7_A10.js") {
9700:    if !path.ends_with("built-ins/String/prototype/indexOf/name.js") {
9721:    if path.ends_with("built-ins/String/prototype/lastIndexOf/S15.5.4.8_A10.js") {
9744:    if !path.ends_with("built-ins/String/prototype/lastIndexOf/name.js") {
9765:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A1_T3.js") {
9783:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T1.js") {
9810:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T6.js") {
9845:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T7.js") {
9865:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T8.js") {
9893:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T9.js") {
9921:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T10.js") {
9949:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T11.js") {
9977:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T17.js") {
10006:    if path.ends_with("built-ins/String/prototype/match/S15.5.4.10_A2_T18.js") {
10040:    if path.ends_with("built-ins/String/prototype/includes/includes.js") {
10056:    if path.ends_with("built-ins/String/prototype/includes/length.js") {
10073:    if !path.ends_with("built-ins/String/prototype/includes/name.js") {
10094:    if path.ends_with("built-ins/String/prototype/match/length.js") {
10112:    if !path.ends_with("built-ins/String/prototype/match/name.js") {
10133:    if path.ends_with("built-ins/String/prototype/matchAll/prop-desc.js") {
10149:    if path.ends_with("built-ins/String/prototype/matchAll/length.js") {
10167:    if !path.ends_with("built-ins/String/prototype/matchAll/name.js") {
10188:    if !path.ends_with("built-ins/String/prototype/matchAll/regexp-prototype-matchAll-v-u-flag.js")
10236:    if path.ends_with("built-ins/String/prototype/matchAll/regexp-is-null.js") {
10249:    if !path.ends_with("built-ins/String/prototype/matchAll/regexp-is-undefined.js") {
10269:    if path.ends_with("built-ins/RegExp/prototype/Symbol.search/prop-desc.js") {
10285:    if path.ends_with("built-ins/RegExp/prototype/Symbol.search/length.js") {
10303:    if !path.ends_with("built-ins/RegExp/prototype/Symbol.search/name.js") {
10324:    if path.ends_with("built-ins/String/prototype/search/S15.5.4.12_A10.js") {
10346:    if path.ends_with("built-ins/String/prototype/search/name.js") {
10370:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
10384:        if path.ends_with(&format!("{prefix}length.js")) {
10398:        if path.ends_with(&format!("{prefix}name.js")) {
10421:    if path.ends_with(&format!("{prefix}repeat.js")) {
10436:    if path.ends_with(&format!("{prefix}length.js")) {
10451:    if path.ends_with(&format!("{prefix}name.js")) {
10472:    if path.ends_with(&format!("{prefix}function-property-descriptor.js")) {
10487:    if path.ends_with(&format!("{prefix}function-length.js")) {
10502:    if path.ends_with(&format!("{prefix}function-name.js")) {
10523:    if path.ends_with(&format!("{prefix}function-property-descriptor.js")) {
10538:    if path.ends_with(&format!("{prefix}function-length.js")) {
10553:    if path.ends_with(&format!("{prefix}function-name.js")) {
10575:        if path.ends_with(&format!("{prefix}length.js")) {
10589:        if path.ends_with(&format!("{prefix}name.js")) {
10608:    if path.ends_with("built-ins/String/prototype/toString/non-generic-realm.js") {
10640:    if path.ends_with("built-ins/String/prototype/valueOf/non-generic-realm.js") {
10679:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
10693:        if path.ends_with(&format!("{prefix}length.js")) {
10707:        if path.ends_with(&format!("{prefix}name.js")) {
10728:    if path.ends_with(&format!("{prefix}prop-desc.js")) {
10743:    if path.ends_with(&format!("{prefix}length.js")) {
10758:    if path.ends_with(&format!("{prefix}name.js")) {
10777:    if path.ends_with("built-ins/String/prototype/isWellFormed/to-string-primitive.js") {
10802:    if path.ends_with("built-ins/String/prototype/toWellFormed/to-string-primitive.js") {
10851:        if path.ends_with(&format!("{prefix}{prop_desc_file}")) {
10865:        if path.ends_with(&format!("{prefix}length.js")) {
10879:        if path.ends_with(&format!("{prefix}name.js")) {
10901:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
10916:        if path.ends_with(&format!("{prefix}length.js")) {
10930:        if path.ends_with(&format!("{prefix}name.js")) {
10949:    let name = if path.ends_with("built-ins/Infinity/prop-desc.js") {
10951:    } else if path.ends_with("built-ins/NaN/prop-desc.js") {
10953:    } else if path.ends_with("built-ins/undefined/prop-desc.js") {
10970:    if !path.ends_with("built-ins/undefined/S15.1.1.3_A1.js") {
10995:    if path.ends_with("built-ins/Number/prop-desc.js") {
11013:    if path.ends_with("built-ins/Number/prototype/prop-desc.js") {
11032:    if path.ends_with("built-ins/Number/prototype/constructor.js") {
11055:    if path.ends_with("built-ins/Number/EPSILON.js") {
11076:    if path.ends_with("built-ins/Number/NaN.js") {
11100:        if path.ends_with(&format!("built-ins/Number/{name}.js")) {
11159:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
11171:        if path.ends_with(readonly_path) {
11186:        if dont_delete_path.is_some_and(|delete_path| path.ends_with(delete_path)) {
11210:        if path.ends_with(&format!("built-ins/Number/{method}.js")) {
11232:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
11246:        if path.ends_with(&format!("{prefix}length.js")) {
11260:        if path.ends_with(&format!("{prefix}name.js")) {
11289:        if path.ends_with(&format!("{prefix}length.js")) {
11303:        if path.ends_with(&format!("{prefix}name.js")) {
11317:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
11340:    if !path.ends_with("built-ins/Boolean/prop-desc.js") {
11361:        if path.ends_with(&format!("{prefix}length.js")) {
11375:        if path.ends_with(&format!("{prefix}name.js")) {
11420:    if path.ends_with("built-ins/Boolean/S9.2_A1_T1.js") {
11432:    if path.ends_with("built-ins/Boolean/S9.2_A6_T1.js") {
11485:    let (constructor_name, expected_prototype, args_source) = match case.path.as_str() {
11567:    let (constructor_name, args_source) = match case.path.as_str() {
11612:    .find(|name| path.contains(&format!("built-ins/NativeErrors/{name}/")))
11619:    if path.ends_with(&format!("{prefix}length.js")) {
11631:    if path.ends_with(&format!("{prefix}name.js")) {
11643:    if path.ends_with(&format!("{prefix}prop-desc.js")) {
11656:    if path.ends_with(&format!("{prefix}prototype.js")) {
11669:    if path.ends_with(&format!("{prefix}prototype/constructor.js")) {
11682:    if path.ends_with(&format!("{prefix}prototype/message.js")) {
11695:    if path.ends_with(&format!("{prefix}prototype/name.js")) {
11729:        if path.ends_with(&format!("{prefix}prop-desc.js")) {
11743:        if path.ends_with(&format!("{prefix}length.js")) {
11757:        if path.ends_with(&format!("{prefix}name.js")) {
11776:    if !path.ends_with("built-ins/Array/prototype/toString/non-callable-join-string-tag.js") {
11834:    if !path.ends_with("built-ins/Array/prototype/toString/S15.4.4.2_A1_T3.js") {
11864:    if path.ends_with("built-ins/Array/prototype/at/coerced-index-resize.js") {
11903:    if path.ends_with("built-ins/Array/prototype/at/typed-array-resizable-buffer.js") {
11971:    if path.ends_with(&format!("{prefix}length.js")) {
11986:    if path.ends_with(&format!("{prefix}name.js")) {
12001:    if path.ends_with(&format!("{prefix}prop-desc.js")) {
12014:    if path.ends_with(&format!("{prefix}returns-item.js")) {
12030:    if path.ends_with(&format!("{prefix}returns-item-relative-index.js")) {
12048:    if path.ends_with(&format!(
12061:    if path.ends_with(&format!(
12091:    if path.ends_with(&format!("{prefix}index-argument-tointeger.js")) {
12112:    if path.ends_with(&format!("{prefix}index-non-numeric-argument-tointeger.js")) {
12132:    if path.ends_with(&format!(
12143:    if path.ends_with(&format!("{prefix}return-abrupt-from-this.js")) {
12154:    if path.ends_with(&format!("{prefix}coerced-index-resize.js")) {
12184:    if path.ends_with(&format!("{prefix}resizable-buffer.js")) {
12229:    if path.ends_with(&format!("{prefix}return-abrupt-from-this-out-of-bounds.js")) {
12245:    if path.ends_with(&format!(
12381:    if path.ends_with("built-ins/Array/prototype/includes/resizable-buffer.js") {
12503:    if path.ends_with("built-ins/Array/prototype/includes/resizable-buffer-special-float-values.js")
12529:    if path.ends_with("built-ins/Array/prototype/indexOf/resizable-buffer.js") {
12595:    if path.ends_with("built-ins/Array/prototype/indexOf/coerced-searchelement-fromindex-grow.js") {
12643:    if path.ends_with("built-ins/Array/prototype/indexOf/coerced-searchelement-fromindex-shrink.js")
12701:    if path.ends_with("built-ins/Array/prototype/indexOf/resizable-buffer-special-float-values.js")
12728:    if path.ends_with("built-ins/Array/prototype/lastIndexOf/resizable-buffer.js") {
12798:    if path.ends_with("built-ins/Array/prototype/lastIndexOf/coerced-position-grow.js") {
12845:    if path.ends_with("built-ins/Array/prototype/lastIndexOf/coerced-position-shrink.js") {
12933:    ) = if path.ends_with("built-ins/Array/prototype/find/resizable-buffer.js") {
12968:    } else if path.ends_with("built-ins/Array/prototype/findIndex/resizable-buffer.js") {
13003:    } else if path.ends_with("built-ins/Array/prototype/findLast/resizable-buffer.js") {
13038:    } else if path.ends_with("built-ins/Array/prototype/findLastIndex/resizable-buffer.js") {
13073:    } else if path.ends_with("built-ins/Array/prototype/every/resizable-buffer.js") {
13105:    } else if path.ends_with("built-ins/Array/prototype/some/resizable-buffer.js") {
13136:    } else if path.ends_with("built-ins/Array/prototype/filter/resizable-buffer.js") {
13220:    if path.ends_with("built-ins/Array/prototype/toLocaleString/resizable-buffer.js") {
13280:    let shrinks = path.ends_with(
13352:    if path == "built-ins/TypedArray/prototype/toString.js" {
13370:    if !path.starts_with("built-ins/TypedArray/prototype/toString/") {
13398:    let body = match path.rsplit('/').next()? {
13417:    if !path.starts_with("built-ins/TypedArray/prototype/toLocaleString/") {
13445:    let body = match path.rsplit('/').next()? {
13922:    if !path.ends_with("built-ins/Array/prototype/values/resizable-buffer.js") {
13966:    let keys = path.ends_with("built-ins/Array/prototype/keys/resizable-buffer.js");
13967:    let entries = path.ends_with("built-ins/Array/prototype/entries/resizable-buffer.js");
14081:        path.ends_with("built-ins/Array/prototype/values/resizable-buffer-grow-mid-iteration.js");
14083:        path.ends_with("built-ins/Array/prototype/values/resizable-buffer-shrink-mid-iteration.js");
14179:        path.ends_with("built-ins/Array/prototype/keys/resizable-buffer-grow-mid-iteration.js");
14181:        path.ends_with("built-ins/Array/prototype/keys/resizable-buffer-shrink-mid-iteration.js");
14183:        path.ends_with("built-ins/Array/prototype/entries/resizable-buffer-grow-mid-iteration.js");
14378:    if path.ends_with("built-ins/Reflect/set/set.js") {
14391:    if path.ends_with("built-ins/Reflect/set/length.js") {
14405:    if path.ends_with("built-ins/Reflect/set/name.js") {
14419:    if path.ends_with("built-ins/Reflect/set/creates-a-data-descriptor.js") {
14447:    if path.ends_with("built-ins/Reflect/set/receiver-is-not-object.js") {
14466:    if path.ends_with("built-ins/Reflect/setPrototypeOf/setPrototypeOf.js") {
14479:    if path.ends_with("built-ins/Reflect/setPrototypeOf/length.js") {
14493:    if path.ends_with("built-ins/Reflect/setPrototypeOf/name.js") {
14511:    if !path.ends_with("built-ins/ThrowTypeError/distinct-cross-realm.js") {
14550:    if path.ends_with("built-ins/ThrowTypeError/length.js") {
14570:    if path.ends_with("built-ins/ThrowTypeError/name.js") {
14590:    if path.ends_with("built-ins/ThrowTypeError/property-order.js") {
14620:    if !path.ends_with("built-ins/Error/isError/prop-desc.js") {
14637:    if path.ends_with("built-ins/Error/prototype/message/prop-desc.js") {
14650:    if path.ends_with("built-ins/Error/prototype/name/prop-desc.js") {
14663:    if path.ends_with("built-ins/Error/prototype/constructor/prop-desc.js") {
14676:    if path.ends_with("built-ins/Error/prototype/toString/prop-desc.js") {
14689:    if path.ends_with("built-ins/Error/prototype/toString/length.js")
14690:        || path.ends_with("built-ins/Error/prototype/toString/name.js")
14720:    if path.ends_with("built-ins/Error/prototype/no-error-data.js") {
14733:    if path.ends_with("built-ins/Error/prototype/S15.11.3.1_A1_T1.js") {
14752:    if path.ends_with("built-ins/Error/prototype/S15.11.3.1_A2_T1.js") {
14769:    if path.ends_with("built-ins/Error/prototype/S15.11.3.1_A3_T1.js") {
14788:    if path.ends_with("built-ins/Error/prototype/S15.11.3.1_A4_T1.js") {
14796:    if path.ends_with("built-ins/Error/prototype/S15.11.4_A1.js") {
14806:    if path.ends_with("built-ins/Error/prototype/S15.11.4_A2.js") {
14816:    if path.ends_with("built-ins/Error/prototype/S15.11.4_A3.js") {
14831:    if path.ends_with("built-ins/Error/prototype/S15.11.4_A4.js") {
14846:    if path.ends_with("built-ins/Error/prototype/constructor/S15.11.4.1_A1_T2.js") {
14863:    if path.ends_with("built-ins/Error/prototype/toString/called-as-function.js") {
14892:    if path.ends_with("built-ins/Error/prototype/toString/invalid-receiver.js") {
14920:    if !path.contains("built-ins/DataView/") || path.contains("built-ins/DataView/prototype/") {
15353:        if path.contains(&format!("built-ins/ArrayBuffer/prototype/{segment}/")) {
15362:    if !path.ends_with("/length.js")
15363:        && !path.ends_with("/name.js")
15364:        && !path.ends_with("/prop-desc.js")
15399:    let is_not_object_case = path.ends_with("/this-is-not-object.js");
15400:    let no_slot_case = path.ends_with("/this-has-no-arraybufferdata-internal.js")
15401:        || (segment == "byteLength" && path.ends_with("/this-has-no-typedarrayname-internal.js"));
15449:        if path.contains(&format!("built-ins/DataView/prototype/{property}/")) {
15458:    if !path.ends_with("/length.js")
15459:        && !path.ends_with("/name.js")
15460:        && !path.ends_with("/prop-desc.js")
15495:    let is_not_object_case = path.ends_with("/this-is-not-object.js");
15496:    let no_slot_case = path.ends_with("/this-has-no-dataview-internal.js")
15497:        || path.ends_with("/this-has-no-dataview-internal-sab.js");
15555:        if path.contains(&format!("built-ins/DataView/prototype/{method}/")) {
15572:        if path.contains(&format!("built-ins/DataView/prototype/{method}/")) {
15581:    if !path.ends_with("/length.js") && !path.ends_with("/name.js") {
15610:    let is_not_object_case = path.ends_with("/this-is-not-object.js");
15611:    let no_slot_case = path.ends_with("/this-has-no-dataview-internal.js")
15612:        || path.ends_with("/this-has-no-dataview-internal-sab.js");
15649:        if path.ends_with("/this-has-no-dataview-internal-sab.js") {
15664:    let byteoffset_object = path.ends_with("/return-abrupt-from-tonumber-byteoffset.js")
15665:        || path.ends_with("/return-abrupt-from-tonumber-byteoffset-sab.js");
15666:    let byteoffset_symbol = path.ends_with("/return-abrupt-from-tonumber-byteoffset-symbol.js")
15667:        || path.ends_with("/return-abrupt-from-tonumber-byteoffset-symbol-sab.js");
15668:    let value_object = path.ends_with("/return-abrupt-from-tonumber-value.js");
15669:    let value_symbol = path.ends_with("/return-abrupt-from-tonumber-value-symbol.js");
15674:    let buffer_ctor = if path.ends_with("-sab.js") {
15788:    if path.ends_with("/range-check-after-value-conversion.js") {
15807:    if path.ends_with("/index-check-before-value-conversion.js") {
15828:    if !path.ends_with("/index-is-out-of-range.js") {
15912:    let method = if path.contains("built-ins/DataView/prototype/getBigInt64/") {
15914:    } else if path.contains("built-ins/DataView/prototype/getBigUint64/") {
15920:    let body = if path.ends_with("/toindex-byteoffset-errors.js") {
15990:    } else if path.ends_with("/toindex-byteoffset-toprimitive.js") {
16182:    } else if path.ends_with("built-ins/DataView/prototype/setUint8/set-values-return-undefined.js")
16189:    } else if path.ends_with("built-ins/DataView/prototype/setInt16/set-values-return-undefined.js")
16204:    } else if path.ends_with("built-ins/DataView/prototype/setInt32/set-values-return-undefined.js")
16303:    let detached_plain = path.ends_with("/detached-buffer.js");
16304:    let detached_before_range = path.ends_with("/detached-buffer-before-outofrange-byteoffset.js");
16305:    let detached_after_toindex = path.ends_with("/detached-buffer-after-toindex-byteoffset.js");
16306:    let detached_after_value = path.ends_with("/detached-buffer-after-number-value.js")
16307:        || path.ends_with("/detached-buffer-after-bigint-value.js");
16377:    if !path.ends_with("/resizable-buffer.js") {
16441:    if !path.contains("built-ins/ArrayBuffer/prototype/slice/")
16442:        || (!path.ends_with("/length.js")
16443:            && !path.ends_with("/name.js")
16444:            && !path.ends_with("/descriptor.js"))
16478:    if !path.contains("built-ins/ArrayBuffer/prototype/slice/")
16479:        || (!path.ends_with("/context-is-not-object.js")
16480:            && !path.ends_with("/context-is-not-arraybuffer-object.js"))
16487:    if path.ends_with("/context-is-not-object.js") {
16672:        || path.ends_with("built-ins/Array/prototype/some/resizable-buffer-shrink-mid-iteration.js")
16691:        || path.ends_with("built-ins/Array/prototype/find/resizable-buffer-shrink-mid-iteration.js")
16700:        || path.ends_with(
16711:        || path.ends_with(
16722:        || path.ends_with(
16735:    let shrinking = path.ends_with("resizable-buffer-shrink-mid-iteration.js");
16885:    } else if path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray-buffer.js") {
16894:    } else if path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray-constructor.js") {
16902:    } else if path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray-subclass-instance.js")
16926:    path.ends_with("built-ins/TypedArrayConstructors/of/new-instance-from-zero.js")
16927:        || path.ends_with("built-ins/TypedArrayConstructors/ctors/buffer-arg/defined-length.js")
16928:        || path.ends_with("built-ins/Array/prototype/map/callbackfn-resize-arraybuffer.js")
16929:        || path.ends_with("built-ins/Array/prototype/every/callbackfn-resize-arraybuffer.js")
16930:        || path.ends_with("built-ins/Array/prototype/forEach/callbackfn-resize-arraybuffer.js")
16931:        || path.ends_with("built-ins/Array/prototype/filter/callbackfn-resize-arraybuffer.js")
16932:        || path.ends_with("built-ins/Array/prototype/find/callbackfn-resize-arraybuffer.js")
16933:        || path.ends_with("built-ins/Array/prototype/findIndex/callbackfn-resize-arraybuffer.js")
16934:        || path.ends_with("built-ins/Array/prototype/findLast/callbackfn-resize-arraybuffer.js")
16937:        || path.ends_with("built-ins/Array/prototype/some/callbackfn-resize-arraybuffer.js")
16938:        || path.ends_with("built-ins/ArrayBuffer/isView/invoked-as-a-fn.js")
16939:        || path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray.js")
16940:        || path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray-buffer.js")
16941:        || path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray-constructor.js")
16942:        || path.ends_with("built-ins/ArrayBuffer/isView/arg-is-typedarray-subclass-instance.js")
17096:        if case.path.contains("resized-out-of-bounds-2.js") {
18178:        manifest_hash: hash_manifest(&pinned, &cases, Some(case.path.as_str())),
18324:        && (run_config.resume || run_config.filter.as_deref() == Some(case.path.as_str()));
18879:            || case.path.contains(
18883:            case.path.starts_with("built-ins/DataView/prototype/set")
18884:                && case.path.ends_with("/immutable-buffer.js");
18889:    let supported_dataview_shared_array_buffer_case = case.path.starts_with("built-ins/DataView/")
18890:        && (case.path.ends_with("-sab.js") || case.features.contains("SharedArrayBuffer"));
18897:        || case.path.contains(
18915:        || case.path.contains(
18921:        || case.path.contains("-sab")
18922:        || case.path.contains("/sab")
18923:        || case.path.contains("this-is-sharedarraybuffer"))
18931:        let supported_arraybuffer_probe = case.path.contains("built-ins/ArrayBuffer/options-")
18950:        let supported_dataview_resizable_case = case.path.starts_with("built-ins/DataView/");
18964:            || case.path == "built-ins/TypedArray/prototype/at/resizable-buffer.js"
18970:            case.path.starts_with("built-ins/Array/prototype/map/");
18973:            || case.path == "built-ins/Array/prototype/at/coerced-index-resize.js";
18975:            case.path.starts_with("built-ins/Array/prototype/includes/");
18977:            case.path.starts_with("built-ins/Array/prototype/indexOf/");
18979:            case.path.starts_with("built-ins/Array/prototype/filter/");
18981:            case.path.starts_with("built-ins/Array/prototype/find/");
18986:            case.path.starts_with("built-ins/Array/prototype/findLast/");
18991:            case.path.starts_with("built-ins/Array/prototype/every/");
18993:            case.path.starts_with("built-ins/Array/prototype/some/");
18995:            case.path.starts_with("built-ins/Array/prototype/forEach/");
18997:            case.path.starts_with("built-ins/Array/prototype/keys/");
18999:            case.path.starts_with("built-ins/Array/prototype/entries/");
19001:            case.path.starts_with("built-ins/Array/prototype/values/");
19035:        case.path.as_str(),
20467:            || failure.test_path.starts_with(&format!("{}/", rule.prefix))
20973:            .map(|case| case.path.as_str())
21137:            .find(|case| case.path.ends_with("module-pass.js"))
21164:            .find(|case| case.path.ends_with("strict-pass.js"))
25744:            if !path.ends_with("receiver-is-not-object.js") {
28078:                Some(case.path.as_str()),
28423:                    .any(|path| path == "built-ins/Array/intentional-failure.js")
```

## Source-Text Predicates

Count: 590

```text
1064:    source.contains("assert.sameValue")
1065:        && !source.contains("assert.notSameValue")
1066:        && !source.contains("assert.throws")
1067:        && !source.contains("assert.compareArray")
1068:        && !source.contains("compareArray(")
1069:        && !source.contains("assert(")
1070:        && !source.contains("assert._")
1074:    case.original_source.contains("Test262Error") || case.original_source.contains("$DONOTEVALUATE")
1078:    case.original_source.contains("$262")
8568:            case.original_source.replace(
11514:    if !case.original_source.contains("new other.Function()") {
13212:    if !source.contains(&format!("Array.prototype.{method}.call")) {
21168:        assert!(materialized.source.contains("local assert"));
21169:        assert!(materialized.source.contains("vendored helper"));
21430:        assert!(materialized.source.contains("__porfAssertToString"));
21431:        assert!(materialized.source.contains("assert.sameValue"));
21432:        assert!(!materialized.source.contains("full assert"));
21433:        assert!(!materialized.source.contains("assert.notSameValue"));
21434:        assert!(materialized.source.contains("assert.sameValue(value, true"));
21444:        assert!(!materialized.source.contains("verifyProperty("));
21448:        assert!(materialized.source.contains("desc.value, \"map\""));
21458:        assert!(!materialized.source.contains("new Proxy"));
21459:        assert!(materialized.source.contains("var nextGets = 0"));
21460:        assert!(materialized.source.contains("assertSameValue(nextGets, 1"));
21470:        assert!(!materialized.source.contains("verifyProperty("));
21474:        assert!(materialized.source.contains("desc.value, \"filter\""));
21484:        assert!(!materialized.source.contains("verifyProperty("));
21488:        assert!(materialized.source.contains("desc.value, \"flatMap\""));
21500:        assert!(!materialized.source.contains("class InvalidIterable"));
21516:        assert!(!materialized.source.contains("new Proxy"));
21517:        assert!(!materialized.source.contains("?."));
21518:        assert!(materialized.source.contains("var nextGets = 0"));
21519:        assert!(materialized.source.contains("assertSameValue(nextGets, 1"));
21529:        assert!(!materialized.source.contains("new Proxy"));
21530:        assert!(!materialized.source.contains("?."));
21531:        assert!(materialized.source.contains("var callbackValues = \"\""));
21532:        assert!(materialized.source.contains("assertSameValue(nextGets, 1"));
21542:        assert!(!materialized.source.contains("new Proxy"));
21543:        assert!(!materialized.source.contains("?."));
21544:        assert!(materialized.source.contains("var returnCalls = 0"));
21557:        assert!(!materialized.source.contains("new Proxy"));
21558:        assert!(!materialized.source.contains("?."));
21559:        assert!(materialized.source.contains("var returnCalls = 0"));
21572:        assert!(!materialized.source.contains("new Proxy"));
21573:        assert!(!materialized.source.contains("?."));
21574:        assert!(materialized.source.contains("var returnCalls = 0"));
21587:        assert!(!materialized.source.contains("new Proxy"));
21588:        assert!(!materialized.source.contains("?."));
21589:        assert!(materialized.source.contains("var returnCalls = 0"));
21604:        assert!(!materialized.source.contains("class TestIterator"));
21605:        assert!(materialized.source.contains("var closed = false"));
21606:        assert!(materialized.source.contains("assertSameValue(closed, true"));
21616:        assert!(!materialized.source.contains("class TestIterator"));
21617:        assert!(materialized.source.contains("var counter = 0"));
21640:        assert!(!materialized.source.contains("sta should be skipped"));
21641:        assert!(!materialized.source.contains("full assert"));
21642:        assert!(materialized.source.contains("assert.sameValue"));
21658:        assert!(materialized.source.contains("full assert"));
21683:        assert!(materialized.source.contains("function Test262Error"));
21684:        assert!(!materialized.source.contains("var $262"));
21722:        assert!(materialized.source.contains("var $262"));
21723:        assert!(materialized.source.contains("function $DETACHBUFFER"));
21750:            assert!(materialized.source.contains("ArrayBuffer.prototype.slice"));
21751:            assert!(materialized.source.contains("Math.asin"));
21752:            assert!(materialized.source.contains("String.prototype.blink"));
21753:            assert!(materialized.source.contains(
21759:            assert!(!materialized.source.contains("WellKnownIntrinsicObjects"));
21807:            assert!(materialized.source.contains(expected), "{file}");
21808:            assert!(!materialized.source.contains("assert.sameValue"), "{file}");
21839:        assert!(!materialized.source.contains("UnicodeIDStart"));
21871:        assert!(!materialized.source.contains("helper used"));
21875:        assert!(materialized.source.contains("var TA = Float64Array;"));
21876:        assert!(materialized.source.contains("var TA = Uint8ClampedArray;"));
21907:        assert!(!materialized.source.contains("helper used"));
21911:        assert!(materialized.source.contains("var TA = Float64Array;"));
21912:        assert!(materialized.source.contains("var TA = Uint8ClampedArray;"));
21946:        assert!(!materialized.source.contains("helper used"));
21950:        assert!(materialized.source.contains("var TA = Float64Array;"));
21951:        assert!(materialized.source.contains("var TA = Uint8ClampedArray;"));
21952:        assert!(materialized.source.contains("buffer.resize(2 * BPE);"));
21983:        assert!(!materialized.source.contains("helper used"));
21987:        assert!(materialized.source.contains("var TA = Float64Array;"));
21988:        assert!(materialized.source.contains("var TA = Uint8ClampedArray;"));
21989:        assert!(materialized.source.contains("Array.prototype.every.call"));
22050:            assert!(!materialized.source.contains("helper used"));
22055:                assert!(materialized.source.contains("var TA = Uint8Array;"));
22056:                assert!(!materialized.source.contains("var TA = Float64Array;"));
22061:                    assert!(materialized.source.contains("expectedIndices = [2, 1, 0];"));
22071:                assert!(materialized.source.contains("var TA = Float64Array;"));
22072:                assert!(materialized.source.contains("var TA = Uint8ClampedArray;"));
22077:            assert!(materialized.source.contains(result_assertion));
22102:        assert!(!materialized.source.contains("helper used"));
22159:            assert!(!materialized.source.contains("helper used"));
22163:            assert!(materialized.source.contains(expected_snippet));
22164:            assert!(materialized.source.contains("Uint8ClampedArray"));
22203:            assert!(!materialized.source.contains("sta used"));
22204:            assert!(materialized.source.contains(expected_snippet));
22205:            assert!(materialized.source.contains("String.prototype.charAt"));
22206:            assert!(!materialized.source.contains("eval("));
22243:            assert!(!materialized.source.contains("assert used"));
22244:            assert!(!materialized.source.contains("Function().slice"));
22245:            assert!(!materialized.source.contains("__num.slice()"));
22246:            assert!(materialized.source.contains("function Test262Error"));
22247:            assert!(materialized.source.contains(expected_fragment));
22268:        assert!(!materialized.source.contains("sta used"));
22269:        assert!(!materialized.source.contains("eval("));
22270:        assert!(materialized.source.contains("String.prototype.charCodeAt"));
22294:        assert!(!materialized.source.contains("sta used"));
22295:        assert!(!materialized.source.contains("eval("));
22318:        assert!(!materialized.source.contains("sta used"));
22322:        assert!(materialized.source.contains("__match.index !== 2"));
22323:        assert!(materialized.source.contains("__match.input !== __string"));
22341:        assert!(!materialized.source.contains("sta used"));
22342:        assert!(!materialized.source.contains("eval("));
22343:        assert!(materialized.source.contains("match(\"bj\")"));
22358:            assert!(!materialized.source.contains("sta used"));
22362:            assert!(!materialized.source.contains("__string.match(__re).length"));
22363:            assert!(materialized.source.contains("__match.length !== 3"));
22397:            assert!(!materialized.source.contains("helper used"));
22398:            assert!(!materialized.source.contains("verifyProperty("));
22399:            assert!(!materialized.source.contains("verifyNotWritable("));
22400:            assert!(materialized.source.contains("String.prototype.charCodeAt"));
22401:            assert!(materialized.source.contains(expected_snippet));
22402:            assert!(materialized.source.contains("desc.configurable !== true"));
22438:            assert!(!materialized.source.contains("helper used"));
22439:            assert!(!materialized.source.contains("verifyProperty("));
22440:            assert!(materialized.source.contains("String.prototype.codePointAt"));
22441:            assert!(materialized.source.contains(expected_snippet));
22442:            assert!(materialized.source.contains("desc.configurable !== true"));
22478:            assert!(!materialized.source.contains("helper used"));
22479:            assert!(!materialized.source.contains("verifyProperty("));
22480:            assert!(materialized.source.contains("String.prototype.startsWith"));
22481:            assert!(materialized.source.contains(expected_snippet));
22482:            assert!(materialized.source.contains("desc.configurable !== true"));
22518:            assert!(!materialized.source.contains("helper used"));
22519:            assert!(!materialized.source.contains("verifyProperty("));
22520:            assert!(materialized.source.contains("String.prototype.endsWith"));
22521:            assert!(materialized.source.contains(expected_snippet));
22522:            assert!(materialized.source.contains("desc.configurable !== true"));
22558:            assert!(!materialized.source.contains("helper used"));
22559:            assert!(!materialized.source.contains("verifyProperty("));
22560:            assert!(materialized.source.contains("String.prototype.includes"));
22561:            assert!(materialized.source.contains(expected_snippet));
22562:            assert!(materialized.source.contains("desc.configurable !== true"));
22595:            assert!(!materialized.source.contains("helper used"));
22596:            assert!(!materialized.source.contains("verifyProperty("));
22597:            assert!(!materialized.source.contains("verifyNotWritable("));
22598:            assert!(materialized.source.contains("String.prototype.search"));
22599:            assert!(materialized.source.contains(expected_snippet));
22600:            assert!(materialized.source.contains("desc.configurable !== true"));
22639:            assert!(!materialized.source.contains("helper used"));
22640:            assert!(!materialized.source.contains("verifyProperty("));
22641:            assert!(materialized.source.contains("String.prototype.trim"));
22642:            assert!(materialized.source.contains(expected_snippet));
22643:            assert!(materialized.source.contains("desc.configurable !== true"));
22679:            assert!(!materialized.source.contains("helper used"));
22680:            assert!(!materialized.source.contains("verifyProperty("));
22681:            assert!(materialized.source.contains("String.prototype.repeat"));
22682:            assert!(materialized.source.contains(expected_snippet));
22683:            assert!(materialized.source.contains("desc.configurable !== true"));
22719:            assert!(!materialized.source.contains("helper used"));
22720:            assert!(!materialized.source.contains("verifyProperty("));
22721:            assert!(materialized.source.contains("String.prototype.padStart"));
22722:            assert!(materialized.source.contains(expected_snippet));
22723:            assert!(materialized.source.contains("desc.configurable !== true"));
22759:            assert!(!materialized.source.contains("helper used"));
22760:            assert!(!materialized.source.contains("verifyProperty("));
22761:            assert!(materialized.source.contains("String.prototype.padEnd"));
22762:            assert!(materialized.source.contains(expected_snippet));
22763:            assert!(materialized.source.contains("desc.configurable !== true"));
22797:                assert!(!materialized.source.contains("helper used"));
22798:                assert!(!materialized.source.contains("verifyProperty("));
22802:                assert!(materialized.source.contains(expected_snippet));
22803:                assert!(materialized.source.contains("desc.configurable !== true"));
22839:            assert!(!materialized.source.contains("assert used"));
22840:            assert!(!materialized.source.contains("sta used"));
22841:            assert!(!materialized.source.contains("$262.createRealm"));
22842:            assert!(materialized.source.contains("__porfCreateRealm().global"));
22843:            assert!(materialized.source.contains("error instanceof TypeError"));
22844:            assert!(materialized.source.contains(expected_fragment));
22882:                assert!(!materialized.source.contains("helper used"));
22883:                assert!(!materialized.source.contains("verifyProperty("));
22887:                assert!(materialized.source.contains(expected_snippet));
22888:                assert!(materialized.source.contains("desc.configurable !== true"));
22924:            assert!(!materialized.source.contains("helper used"));
22925:            assert!(!materialized.source.contains("verifyProperty("));
22926:            assert!(materialized.source.contains("String.prototype.at"));
22927:            assert!(materialized.source.contains(expected_snippet));
22928:            assert!(materialized.source.contains("desc.configurable !== true"));
22951:            assert!(!materialized.source.contains("original used"));
22952:            assert!(!materialized.source.contains("const ["));
22956:            assert!(materialized.source.contains(expected_fragment));
22996:            assert!(!materialized.source.contains("helper used"));
22997:            assert!(!materialized.source.contains("verifyProperty("));
22998:            assert!(!materialized.source.contains("verifyNotWritable("));
23002:            assert!(materialized.source.contains(expected_snippet));
23003:            assert!(materialized.source.contains("desc.configurable !== true"));
23039:            assert!(!materialized.source.contains("helper used"));
23040:            assert!(!materialized.source.contains("verifyProperty("));
23041:            assert!(materialized.source.contains("String.prototype.matchAll"));
23042:            assert!(materialized.source.contains(expected_snippet));
23043:            assert!(materialized.source.contains("desc.configurable !== true"));
23067:        assert!(!materialized.source.contains("helper used"));
23068:        assert!(!materialized.source.contains("compareArray"));
23069:        assert!(!materialized.source.contains(".map("));
23073:        assert!(materialized.source.contains("complexText.matchAll"));
23111:            assert!(!materialized.source.contains("helper used"));
23112:            assert!(!materialized.source.contains("compareIterator"));
23113:            assert!(!materialized.source.contains("matchValidator"));
23114:            assert!(materialized.source.contains(expected_call));
23115:            assert!(materialized.source.contains("Array.from"));
23148:            assert!(!materialized.source.contains("helper used"));
23149:            assert!(!materialized.source.contains("verifyProperty("));
23150:            assert!(!materialized.source.contains("verifyNotWritable("));
23151:            assert!(materialized.source.contains("String.prototype.indexOf"));
23152:            assert!(materialized.source.contains(expected_snippet));
23153:            assert!(materialized.source.contains("desc.configurable !== true"));
23173:        assert!(!materialized.source.contains("helper used"));
23174:        assert!(!materialized.source.contains("verifyProperty("));
23178:        assert!(materialized.source.contains("desc.configurable !== true"));
23197:        assert!(!materialized.source.contains("helper used"));
23198:        assert!(!materialized.source.contains("verifyNotWritable("));
23202:        assert!(materialized.source.contains("desc.writable !== false"));
23203:        assert!(materialized.source.contains("fn.length = function"));
23246:            assert!(!materialized.source.contains("helper used"), "{path}");
23247:            assert!(!materialized.source.contains("verifyProperty("), "{path}");
23248:            assert!(materialized.source.contains(expected_snippet), "{path}");
23250:                materialized.source.contains("desc.configurable !== true"),
23290:            assert!(!materialized.source.contains("helper used"), "{path}");
23291:            assert!(!materialized.source.contains("verifyProperty("), "{path}");
23292:            assert!(materialized.source.contains(expected_snippet), "{path}");
23294:                materialized.source.contains("desc.configurable !== true"),
23326:            assert!(!materialized.source.contains("assert used"));
23327:            assert!(!materialized.source.contains("assert.sameValue"));
23328:            assert!(materialized.source.contains(expected_snippet));
23329:            assert!(materialized.source.contains("__porfCharAtWarmup.charAt(0)"));
23347:        assert!(!materialized.source.contains("harness should be skipped"));
23348:        assert!(materialized.source.contains("effects.length !== 2"));
23349:        assert!(materialized.source.contains("returnValue === returnValue"));
23367:        assert!(!materialized.source.contains("import * as ns"));
23368:        assert!(materialized.source.contains("Object.preventExtensions(ns)"));
23402:            assert!(!materialized.source.contains("helper used"));
23403:            assert!(!materialized.source.contains("verifyProperty("));
23404:            assert!(materialized.source.contains("Object.defineProperty"));
23406:                assert!(materialized.source.contains("Reflect.defineProperty"));
23437:            assert!(!materialized.source.contains("helper used"));
23438:            assert!(!materialized.source.contains("verifyProperty("));
23439:            assert!(materialized.source.contains("new Proxy"));
23486:            assert!(!materialized.source.contains("assert used"));
23487:            assert!(!materialized.source.contains("isConstructor helper used"));
23488:            assert!(!materialized.source.contains("assert."));
23489:            assert!(materialized.source.contains("new Proxy"));
23490:            assert!(materialized.source.contains(expected_fragment));
23599:            assert!(!materialized.source.contains("assert used"));
23600:            assert!(!materialized.source.contains("property helper used"));
23601:            assert!(!materialized.source.contains("isConstructor helper used"));
23602:            assert!(!materialized.source.contains("sta used"));
23603:            assert!(!materialized.source.contains("verifyProperty("));
23604:            assert!(!materialized.source.contains("assert."));
23605:            assert!(materialized.source.contains("Proxy.revocable"));
23606:            assert!(materialized.source.contains(expected_fragment));
23640:            assert!(!materialized.source.contains("assert used"));
23641:            assert!(!materialized.source.contains("assert.throws"));
23642:            assert!(!materialized.source.contains("$262.createRealm"));
23646:            assert!(materialized.source.contains("expectTypeError"));
23647:            assert!(materialized.source.contains(expected_fragment));
23668:        assert!(!materialized.source.contains("assert used"));
23669:        assert!(!materialized.source.contains(".eval("));
23670:        assert!(!materialized.source.contains("$262.createRealm"));
23674:        assert!(materialized.source.contains("new OProxy"));
23675:        assert!(materialized.source.contains("return args;"));
23698:        assert!(!materialized.source.contains("assert used"));
23699:        assert!(!materialized.source.contains(".eval("));
23700:        assert!(!materialized.source.contains("$262.createRealm"));
23704:        assert!(materialized.source.contains("new other.Proxy"));
23705:        assert!(materialized.source.contains("return args;"));
23731:        assert!(!materialized.source.contains("assert used"));
23732:        assert!(!materialized.source.contains("new other.Function"));
23736:        assert!(materialized.source.contains("var C = other.Array;"));
23737:        assert!(materialized.source.contains("Reflect.construct(P, [], C)"));
23763:        assert!(!materialized.source.contains("assert used"));
23764:        assert!(!materialized.source.contains("new other.Function"));
23768:        assert!(materialized.source.contains("var C = other.Proxy;"));
23769:        assert!(materialized.source.contains("C.prototype = null;"));
23770:        assert!(materialized.source.contains("Reflect.construct(P, [], C)"));
23796:            assert!(!materialized.source.contains("helper used"));
23797:            assert!(!materialized.source.contains("verifyProperty("));
23798:            assert!(materialized.source.contains(&format!(
23801:            assert!(materialized.source.contains("desc.writable !== false"));
23802:            assert!(materialized.source.contains("desc.enumerable !== false"));
23803:            assert!(materialized.source.contains("desc.configurable !== false"));
23823:        assert!(!materialized.source.contains("sta used"));
23824:        assert!(!materialized.source.contains("eval("));
23825:        assert!(materialized.source.contains("typeof(undefined)"));
23872:            assert!(!materialized.source.contains("helper used"));
23873:            assert!(!materialized.source.contains("verifyProperty("));
23880:            assert!(materialized.source.contains("desc.enumerable !== false"));
23885:                materialized.source.contains(expected_fragment),
23938:                assert!(!materialized.source.contains("helper used"));
23939:                assert!(!materialized.source.contains("verifyNotWritable("));
23940:                assert!(materialized.source.contains(&format!(
23944:                    materialized.source.contains(&expected_fragment),
23973:            assert!(!materialized.source.contains("helper used"));
23974:            assert!(!materialized.source.contains("verifyProperty("));
23975:            assert!(!materialized.source.contains("verifyNotWritable("));
23976:            assert!(materialized.source.contains(&format!(
23982:            assert!(materialized.source.contains("desc.writable !== false"));
23983:            assert!(materialized.source.contains("desc.enumerable !== false"));
23984:            assert!(materialized.source.contains("desc.configurable !== false"));
23986:                materialized.source.contains(expected_fragment),
24011:            assert!(!materialized.source.contains("helper used"));
24012:            assert!(!materialized.source.contains("verifyProperty("));
24013:            assert!(!materialized.source.contains("assert.sameValue"));
24014:            assert!(materialized.source.contains(&format!(
24020:            assert!(materialized.source.contains("desc.writable !== true"));
24021:            assert!(materialized.source.contains("desc.enumerable !== false"));
24022:            assert!(materialized.source.contains("desc.configurable !== true"));
24052:                assert!(!materialized.source.contains("helper used"));
24053:                assert!(!materialized.source.contains("verifyProperty("));
24055:                    materialized.source.contains(&expected_value),
24109:                assert!(!materialized.source.contains("helper used"));
24110:                assert!(!materialized.source.contains("verifyProperty("));
24112:                    materialized.source.contains(&expected_fragment),
24126:                assert!(materialized.source.contains("desc.enumerable !== false"));
24153:        assert!(!materialized.source.contains("assert used"));
24154:        assert!(!materialized.source.contains("$262.createRealm"));
24155:        assert!(!materialized.source.contains("new other.Function"));
24159:        assert!(materialized.source.contains("var C = other.Proxy;"));
24160:        assert!(materialized.source.contains("C.prototype = null;"));
24236:            assert!(!materialized.source.contains("assert used"));
24237:            assert!(!materialized.source.contains("$262.createRealm"));
24238:            assert!(!materialized.source.contains("new other.Function"));
24242:            assert!(materialized.source.contains("other.Proxy"));
24247:                materialized.source.contains(expected_prototype),
24281:            assert!(!materialized.source.contains("assert used"));
24282:            assert!(!materialized.source.contains("new Function"));
24283:            assert!(materialized.source.contains("function NewTarget() {}"));
24284:            assert!(materialized.source.contains(&format!(
24287:            assert!(materialized.source.contains(&format!(
24391:                assert!(!materialized.source.contains("helper used"));
24392:                assert!(!materialized.source.contains("assert used"));
24393:                assert!(!materialized.source.contains("verifyProperty("));
24394:                assert!(!materialized.source.contains("assert.sameValue"));
24397:                        materialized.source.contains(&fragment),
24422:        assert!(!materialized.source.contains("helper used"));
24423:        assert!(!materialized.source.contains("verifyProperty("));
24427:        assert!(materialized.source.contains("desc.value !== Boolean"));
24428:        assert!(materialized.source.contains("desc.writable !== true"));
24445:                assert!(!materialized.source.contains("helper used"));
24446:                assert!(!materialized.source.contains("verifyProperty("));
24450:                assert!(materialized.source.contains(&expected_value));
24451:                assert!(materialized.source.contains("desc.writable !== false"));
24452:                assert!(materialized.source.contains("desc.configurable !== true"));
24476:        assert!(!materialized.source.contains("assert used"));
24477:        assert!(!materialized.source.contains("$262.createRealm"));
24478:        assert!(!materialized.source.contains("new other.Function"));
24482:        assert!(materialized.source.contains("var C = other.Proxy;"));
24483:        assert!(materialized.source.contains("C.prototype = null;"));
24521:            assert!(!materialized.source.contains("assert used"));
24522:            assert!(!materialized.source.contains("assert.sameValue"));
24523:            assert!(!materialized.source.contains(forbidden));
24524:            assert!(materialized.source.contains(expected), "{path}");
24572:                assert!(!materialized.source.contains("helper used"));
24573:                assert!(!materialized.source.contains("verifyProperty("));
24574:                assert!(materialized.source.contains(&expected_value));
24598:        assert!(!materialized.source.contains("assert helper used"));
24599:        assert!(!materialized.source.contains("assert.sameValue"));
24600:        assert!(materialized.source.contains("{ join: null }"));
24604:        assert!(materialized.source.contains("sentinel"));
24616:        assert!(!materialized.source.contains("throw 'original'"));
24617:        assert!(materialized.source.contains("[true, true, true]"));
24621:        assert!(materialized.source.contains("array.toString()"));
24622:        assert!(materialized.source.contains("array.join()"));
24712:            assert!(!materialized.source.contains("helper used"));
24713:            assert!(!materialized.source.contains("compare helper used"));
24714:            assert!(!materialized.source.contains("for (let ctor of ctors)"));
24715:            assert!(materialized.source.contains("var TA = Uint8Array;"));
24716:            assert!(!materialized.source.contains("var TA = Float64Array;"));
24719:                    materialized.source.contains(fragment),
24772:            assert!(!materialized.source.contains("helper used"));
24773:            assert!(!materialized.source.contains("for (let ctor of ctors)"));
24774:            assert!(materialized.source.contains("var TA = Uint8Array;"));
24775:            assert!(!materialized.source.contains("var TA = Float64Array;"));
24778:                    materialized.source.contains(fragment),
24836:            assert!(!materialized.source.contains("helper used"));
24840:            assert!(materialized.source.contains("var TA = Uint8Array;"));
24846:                    materialized.source.contains(fragment),
24934:            assert!(!materialized.source.contains("helper used"));
24938:            assert!(materialized.source.contains("var TA = Uint8Array;"));
24944:                    materialized.source.contains(fragment),
24970:        assert!(!materialized.source.contains("helper used"));
24971:        assert!(!materialized.source.contains("for (let ctor of ctors)"));
24972:        assert!(materialized.source.contains("var TA = Uint8Array;"));
24976:        assert!(materialized.source.contains("tracking offset shrink one"));
24977:        assert!(materialized.source.contains("fixed initial too low"));
24998:        assert!(!materialized.source.contains("helper used"));
24999:        assert!(!materialized.source.contains("for (let ctor of ctors)"));
25000:        assert!(materialized.source.contains("function ArrayAtHelper"));
25001:        assert!(materialized.source.contains("rab1.resize(2);"));
25032:        assert!(!materialized.source.contains("property helper used"));
25033:        assert!(!materialized.source.contains("typed array helper used"));
25063:        assert!(!materialized.source.contains("typed array helper used"));
25067:        assert!(materialized.source.contains("var a = new Uint8Array(4);"));
25068:        assert!(materialized.source.contains("a.at(3)"));
25088:        assert!(!materialized.source.contains("helper used"));
25089:        assert!(!materialized.source.contains("for (let ctor of ctors)"));
25090:        assert!(materialized.source.contains("function TypedArrayAtHelper"));
25091:        assert!(materialized.source.contains("return ta.at(index);"));
25092:        assert!(materialized.source.contains("fixed shrink three"));
25093:        assert!(materialized.source.contains("__porfAssertThrows(TypeError"));
25117:        assert!(!materialized.source.contains("typed array helper used"));
25127:        assert!(materialized.source.contains("BigInt64Array"));
25128:        assert!(materialized.source.contains("BigUint64Array"));
25129:        assert!(materialized.source.contains("__porfAssertThrows(TypeError"));
25178:            assert!(!materialized.source.contains("property helper used"));
25179:            assert!(!materialized.source.contains("wrong"));
25180:            assert!(!materialized.source.contains("isConstructor helper used"));
25181:            assert!(materialized.source.contains("var Ctor = "));
25183:                materialized.source.contains("__porfCheckDataDescriptor")
25184:                    || materialized.source.contains("__porfIsConstructor")
25185:                    || materialized.source.contains("Object.getPrototypeOf(Ctor)")
25218:        assert!(!materialized.source.contains("helper used"));
25219:        assert!(!materialized.source.contains("compare helper used"));
25220:        assert!(!materialized.source.contains("for (let ctor of ctors)"));
25221:        assert!(materialized.source.contains("var TA = Uint8Array;"));
25222:        assert!(!materialized.source.contains("var TA = Float64Array;"));
25226:        assert!(materialized.source.contains("tracking shrink first"));
25227:        assert!(materialized.source.contains("fixed shrink out-of-bounds"));
25277:            assert!(!materialized.source.contains("helper used"));
25278:            assert!(!materialized.source.contains("compare helper used"));
25279:            assert!(!materialized.source.contains("for (let ctor of ctors)"));
25280:            assert!(materialized.source.contains("var TA = Uint8Array;"));
25281:            assert!(!materialized.source.contains("var TA = Float64Array;"));
25284:                    materialized.source.contains(fragment),
25337:            assert!(!materialized.source.contains("helper used"));
25338:            assert!(!materialized.source.contains("compare helper used"));
25339:            assert!(!materialized.source.contains("TestIterationAndResize"));
25340:            assert!(materialized.source.contains("var TA = Uint8Array;"));
25341:            assert!(!materialized.source.contains("var TA = Float64Array;"));
25344:                    materialized.source.contains(fragment),
25411:            assert!(!materialized.source.contains("helper used"));
25412:            assert!(!materialized.source.contains("compare helper used"));
25413:            assert!(!materialized.source.contains("TestIterationAndResize"));
25414:            assert!(materialized.source.contains("var TA = Uint8Array;"));
25415:            assert!(!materialized.source.contains("var TA = Float64Array;"));
25418:                    materialized.source.contains(fragment),
25581:            assert!(!materialized.source.contains("helper used"));
25582:            assert!(!materialized.source.contains("compare helper used"));
25583:            assert!(!materialized.source.contains("CollectValuesAndResize"));
25584:            assert!(materialized.source.contains("var TA = Uint8Array;"));
25585:            assert!(!materialized.source.contains("var TA = Float64Array;"));
25588:                    materialized.source.contains(fragment),
25627:            assert!(!materialized.source.contains("helper used"));
25628:            assert!(!materialized.source.contains("for (let ctor of ctors)"));
25629:            assert!(materialized.source.contains("class MyUint8Array"));
25630:            assert!(materialized.source.contains(expected_value));
25670:            assert!(!materialized.source.contains("helper used"));
25671:            assert!(!materialized.source.contains("for (let ctor of ctors)"));
25672:            assert!(materialized.source.contains("Array.prototype.indexOf.call"));
25673:            assert!(materialized.source.contains(expected_value));
25709:            assert!(!materialized.source.contains("helper used"));
25710:            assert!(!materialized.source.contains("for (let ctor of ctors)"));
25714:            assert!(materialized.source.contains(expected_value));
25753:            assert!(!materialized.source.contains("helper used"));
25754:            assert!(!materialized.source.contains("verifyProperty("));
25755:            assert!(materialized.source.contains(expected_value));
25791:            assert!(!materialized.source.contains("helper used"));
25792:            assert!(!materialized.source.contains("verifyProperty("));
25793:            assert!(materialized.source.contains(expected_value));
25815:        assert!(!materialized.source.contains("helper used"));
25816:        assert!(!materialized.source.contains("verifyProperty("));
25820:        assert!(materialized.source.contains("desc.value !== Error.isError"));
25821:        assert!(materialized.source.contains("desc.writable !== true"));
25822:        assert!(materialized.source.contains("desc.enumerable !== false"));
25823:        assert!(materialized.source.contains("desc.configurable !== true"));
25872:            assert!(!materialized.source.contains("helper used"));
25873:            assert!(!materialized.source.contains("verifyProperty("));
25874:            assert!(!materialized.source.contains(".indexOf("));
25877:                    materialized.source.contains(fragment),
25902:        assert!(!materialized.source.contains("assert used"));
25903:        assert!(!materialized.source.contains("$262.createRealm"));
25904:        assert!(!materialized.source.contains("new other.Function"));
25911:        assert!(materialized.source.contains("throw new other.TypeError();"));
25912:        assert!(materialized.source.contains("e instanceof other.TypeError"));
25932:        assert!(!materialized.source.contains("$262.createRealm"));
25933:        assert!(!materialized.source.contains("assert.sameValue"));
25934:        assert!(materialized.source.contains("__porfCreateRealm().global"));
26001:            assert!(!materialized.source.contains("helper used"));
26002:            assert!(!materialized.source.contains("verifyProperty("));
26003:            assert!(!materialized.source.contains("verifyEqualTo("));
26004:            assert!(!materialized.source.contains("assert.sameValue"));
26007:                    materialized.source.contains(fragment),
26127:            assert!(!materialized.source.contains("helper used"));
26128:            assert!(!materialized.source.contains("verifyProperty("));
26131:                    materialized.source.contains(fragment),
26251:            assert!(!materialized.source.contains("helper used"));
26252:            assert!(!materialized.source.contains("verifyProperty("));
26253:            assert!(!materialized.source.contains(".indexOf("));
26256:                    materialized.source.contains(fragment),
26304:            assert!(!materialized.source.contains("helper used"));
26305:            assert!(!materialized.source.contains("verifyProperty("));
26306:            assert!(materialized.source.contains(&format!(
26309:            assert!(materialized.source.contains(expected_value_check));
26310:            assert!(materialized.source.contains("desc.writable !== true"));
26311:            assert!(materialized.source.contains("desc.enumerable !== false"));
26312:            assert!(materialized.source.contains("desc.configurable !== true"));
26335:            assert!(!materialized.source.contains("helper used"));
26336:            assert!(!materialized.source.contains("verifyProperty("));
26346:            assert!(materialized.source.contains("fn.length !== 0"));
26347:            assert!(materialized.source.contains("fn.name !== \"toString\""));
26351:            assert!(materialized.source.contains("nameDesc.writable !== false"));
26495:            assert!(!materialized.source.contains("assert used"));
26496:            assert!(!materialized.source.contains("helper used"));
26497:            assert!(!materialized.source.contains("verifyNotConfigurable"));
26498:            assert!(!materialized.source.contains("assert."));
26501:                    materialized.source.contains(fragment),
26529:                assert!(!materialized.source.contains("helper used"));
26530:                assert!(!materialized.source.contains("verifyProperty("));
26531:                assert!(materialized.source.contains(&format!(
26567:                assert!(!materialized.source.contains("assert.throws"));
26568:                assert!(materialized.source.contains("__porfExpectTypeError"));
26569:                assert!(materialized.source.contains(&format!(
26573:                    assert!(materialized.source.contains("getter.call(undefined)"));
26574:                    assert!(materialized.source.contains("getter.call(s)"));
26576:                    assert!(materialized.source.contains("getter.call({})"));
26577:                    assert!(materialized.source.contains("getter.call(ta)"));
26578:                    assert!(materialized.source.contains("getter.call(dv)"));
26715:            assert!(!materialized.source.contains("helper used"));
26716:            assert!(!materialized.source.contains("detach helper used"));
26717:            assert!(!materialized.source.contains("verifyProperty"));
26718:            assert!(!materialized.source.contains("$DETACHBUFFER"));
26719:            assert!(!materialized.source.contains("assert.throws"));
26720:            assert!(!materialized.source.contains("assert.sameValue"));
26723:                    materialized.source.contains(fragment),
26750:                assert!(!materialized.source.contains("helper used"));
26751:                assert!(!materialized.source.contains("verifyProperty("));
26752:                assert!(materialized.source.contains(&format!(
26786:                assert!(!materialized.source.contains("assert.throws"));
26787:                assert!(materialized.source.contains("__porfAssertThrows"));
26788:                assert!(materialized.source.contains(&format!(
26792:                    assert!(materialized.source.contains("getter.call(undefined)"));
26793:                    assert!(materialized.source.contains("getter.call(s)"));
26795:                    assert!(materialized.source.contains("getter.call({})"));
26796:                    assert!(materialized.source.contains("getter.call(ab)"));
26797:                    assert!(materialized.source.contains("getter.call(sab)"));
26798:                    assert!(materialized.source.contains("getter.call(ta)"));
26832:                assert!(!materialized.source.contains("helper used"));
26833:                assert!(!materialized.source.contains("verifyProperty("));
26861:                assert!(!materialized.source.contains("assert.throws"));
26862:                assert!(materialized.source.contains("__porfAssertThrows"));
26867:                    assert!(materialized.source.contains("fn.call(undefined)"));
26868:                    assert!(materialized.source.contains("fn.call(s)"));
26870:                    assert!(materialized.source.contains("fn.call({})"));
26871:                    assert!(materialized.source.contains("fn.call(ab)"));
26872:                    assert!(materialized.source.contains("fn.call(ta)"));
26873:                    assert!(!materialized.source.contains("fn.call(sab)"));
26885:        assert!(materialized.source.contains("fn.call(sab)"));
26943:            assert!(!materialized.source.contains("assert.throws"));
26946:                    materialized.source.contains(fragment),
27004:            assert!(!materialized.source.contains("assert.throws"));
27007:                    materialized.source.contains(fragment),
27065:            assert!(!materialized.source.contains("assert.sameValue"));
27066:            assert!(!materialized.source.contains("assert.throws"));
27067:            assert!(materialized.source.contains("new DataView(buffer, 0)"));
27070:                    materialized.source.contains(fragment),
27107:            assert!(!materialized.source.contains("byteConversionValues"));
27108:            assert!(!materialized.source.contains(".forEach("));
27114:            assert!(materialized.source.contains(&expected_call));
27115:            assert!(materialized.source.contains(&format!("sample.{getter}(0)")));
27116:            assert!(materialized.source.contains(high_expected));
27117:            assert!(materialized.source.contains("__porfSameValue"));
27118:            assert!(materialized.source.contains("result !== undefined"));
27177:            assert!(!materialized.source.contains("$DETACHBUFFER"));
27178:            assert!(!materialized.source.contains("assert.throws"));
27181:                    materialized.source.contains(fragment),
27231:            assert!(!materialized.source.contains("assert.sameValue"));
27232:            assert!(!materialized.source.contains("assert.throws"));
27235:                    materialized.source.contains(fragment),
27260:            assert!(!materialized.source.contains("helper used"));
27261:            assert!(!materialized.source.contains("verifyProperty("));
27265:            assert!(materialized.source.contains(
27268:            assert!(materialized.source.contains(
27271:            assert!(materialized.source.contains("lengthDesc.value !== 2"));
27272:            assert!(materialized.source.contains("nameDesc.value !== \"slice\""));
27291:            assert!(!materialized.source.contains("assert.throws"));
27295:            assert!(materialized.source.contains("__porfAssertThrows"));
27297:                assert!(materialized.source.contains("slice.call(undefined)"));
27298:                assert!(materialized.source.contains("slice.call(Symbol())"));
27300:                assert!(materialized.source.contains("slice.call({})"));
27301:                assert!(materialized.source.contains("slice.call([])"));
```

## Harness And Helper Reductions

Count: 337

```text
247:    pub used_preludes: Vec<(String, PreludeOrigin)>,
868:            used_preludes: Vec::new(),
875:    let mut used_preludes = Vec::new();
886:            && assert_prelude.is_some_and(|prelude| prelude.contents.contains("Test262Error"));
892:                source.push_str(&prelude.contents);
893:                used_preludes.push((prelude.name.clone(), prelude.origin));
900:                source.push_str(&prelude.contents);
901:                used_preludes.push((prelude.name.clone(), prelude.origin));
908:                source.push_str(&prelude.contents);
910:            used_preludes.push((prelude.name.clone(), prelude.origin));
915:                source.push_str(&prelude.contents);
917:                used_preludes.push((prelude.name.clone(), prelude.origin));
925:            if include == "testTypedArray.js" && wasm_aot_rewrite_skips_test_typed_array(&case.path)
936:                    used_preludes.push((prelude.name.clone(), prelude.origin));
945:                    used_preludes.push((prelude.name.clone(), prelude.origin));
948:                source.push_str(&prelude.contents);
949:                used_preludes.push((prelude.name.clone(), prelude.origin));
959:        used_preludes,
986:assert.sameValue = function (actual, expected, message) {
1054:        || !prelude.contents.contains("__porfAssertToString")
1055:        || !prelude.contents.contains("assert.sameValue")
1426:assert.throws = function (expectedErrorConstructor, func, message) {
1429:assert.sameValue = function (actual, expected, message) {
1443:assert.sameValue = function(actual, expected) {
1919:assert.sameValue = function(actual, expected) {
1960:assert.sameValue = function(actual, expected) {
16925:fn wasm_aot_rewrite_skips_test_typed_array(path: &str) -> bool {
21422:            "function __porfAssertToString(value) { return String(value); }\nassert.sameValue = function() {};\nassert.notSameValue = function() { throw 'full assert'; };\nassert.compareArray = function() { throw 'full compare'; };\n".to_string(),
21443:        assert!(materialized.used_preludes.is_empty());
21457:        assert!(materialized.used_preludes.is_empty());
21469:        assert!(materialized.used_preludes.is_empty());
21483:        assert!(materialized.used_preludes.is_empty());
21499:        assert!(materialized.used_preludes.is_empty());
21515:        assert!(materialized.used_preludes.is_empty());
21528:        assert!(materialized.used_preludes.is_empty());
21541:        assert!(materialized.used_preludes.is_empty());
21556:        assert!(materialized.used_preludes.is_empty());
21571:        assert!(materialized.used_preludes.is_empty());
21586:        assert!(materialized.used_preludes.is_empty());
21603:        assert!(materialized.used_preludes.is_empty());
21615:        assert!(materialized.used_preludes.is_empty());
21633:            "function __porfAssertToString(value) { return String(value); }\nfunction Test262Error(message) {}\nassert.sameValue = function() {};\nassert.notSameValue = function() { throw 'full assert'; };\n".to_string(),
21650:            "function __porfAssertToString(value) { return String(value); }\nassert.sameValue = function() {};\nassert.notSameValue = function() { throw 'full assert'; };\n".to_string(),
21686:            .used_preludes
21690:            .used_preludes
21725:            .used_preludes
21761:                .used_preludes
21810:                .used_preludes
21841:            .used_preludes
21861:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
21871:        assert!(!materialized.source.contains("helper used"));
21897:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
21907:        assert!(!materialized.source.contains("helper used"));
21936:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
21946:        assert!(!materialized.source.contains("helper used"));
21973:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
21983:        assert!(!materialized.source.contains("helper used"));
22010:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
22050:            assert!(!materialized.source.contains("helper used"));
22091:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
22102:        assert!(!materialized.source.contains("helper used"));
22125:            "function assert(value) { if (!value) throw value; }\nassert.sameValue = function(actual, expected) { if (actual !== expected) throw actual; };\n".to_string(),
22130:            "function testWithTypedArrayConstructors() { throw 'helper used'; }\n".to_string(),
22159:            assert!(!materialized.source.contains("helper used"));
22202:            assert!(materialized.used_preludes.is_empty());
22242:            assert!(materialized.used_preludes.is_empty());
22267:        assert!(materialized.used_preludes.is_empty());
22293:        assert!(materialized.used_preludes.is_empty());
22317:        assert!(materialized.used_preludes.is_empty());
22340:        assert!(materialized.used_preludes.is_empty());
22357:            assert!(materialized.used_preludes.is_empty());
22373:            "function verifyProperty() { throw 'helper used'; }\nfunction verifyNotWritable() { throw 'helper used'; }\n"
22396:            assert!(materialized.used_preludes.is_empty());
22397:            assert!(!materialized.source.contains("helper used"));
22411:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22437:            assert!(materialized.used_preludes.is_empty());
22438:            assert!(!materialized.source.contains("helper used"));
22451:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22477:            assert!(materialized.used_preludes.is_empty());
22478:            assert!(!materialized.source.contains("helper used"));
22491:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22517:            assert!(materialized.used_preludes.is_empty());
22518:            assert!(!materialized.source.contains("helper used"));
22531:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22557:            assert!(materialized.used_preludes.is_empty());
22558:            assert!(!materialized.source.contains("helper used"));
22571:            "function verifyProperty() { throw 'helper used'; }\nfunction verifyNotWritable() { throw 'helper used'; }\n"
22594:            assert!(materialized.used_preludes.is_empty());
22595:            assert!(!materialized.source.contains("helper used"));
22609:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22638:            assert!(materialized.used_preludes.is_empty());
22639:            assert!(!materialized.source.contains("helper used"));
22652:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22678:            assert!(materialized.used_preludes.is_empty());
22679:            assert!(!materialized.source.contains("helper used"));
22692:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22718:            assert!(materialized.used_preludes.is_empty());
22719:            assert!(!materialized.source.contains("helper used"));
22732:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22758:            assert!(materialized.used_preludes.is_empty());
22759:            assert!(!materialized.source.contains("helper used"));
22772:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22796:                assert!(materialized.used_preludes.is_empty());
22797:                assert!(!materialized.source.contains("helper used"));
22838:            assert!(materialized.used_preludes.is_empty());
22853:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22881:                assert!(materialized.used_preludes.is_empty());
22882:                assert!(!materialized.source.contains("helper used"));
22898:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
22923:            assert!(materialized.used_preludes.is_empty());
22924:            assert!(!materialized.source.contains("helper used"));
22950:            assert!(materialized.used_preludes.is_empty());
22968:            "function verifyProperty() { throw 'helper used'; }\nfunction verifyNotWritable() { throw 'helper used'; }\n"
22995:            assert!(materialized.used_preludes.is_empty());
22996:            assert!(!materialized.source.contains("helper used"));
23012:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23038:            assert!(materialized.used_preludes.is_empty());
23039:            assert!(!materialized.source.contains("helper used"));
23052:            "function compareArray() { throw 'helper used'; }\n".to_string(),
23066:        assert!(materialized.used_preludes.is_empty());
23067:        assert!(!materialized.source.contains("helper used"));
23081:            "function compareIterator() { throw 'helper used'; }\n".to_string(),
23086:            "function matchValidator() { throw 'helper used'; }\n".to_string(),
23110:            assert!(materialized.used_preludes.is_empty());
23111:            assert!(!materialized.source.contains("helper used"));
23124:            "function verifyProperty() { throw 'helper used'; }\nfunction verifyNotWritable() { throw 'helper used'; }\n"
23147:            assert!(materialized.used_preludes.is_empty());
23148:            assert!(!materialized.source.contains("helper used"));
23162:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23172:        assert!(materialized.used_preludes.is_empty());
23173:        assert!(!materialized.source.contains("helper used"));
23186:            "function verifyNotWritable() { throw 'helper used'; }\n".to_string(),
23196:        assert!(materialized.used_preludes.is_empty());
23197:        assert!(!materialized.source.contains("helper used"));
23211:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23245:            assert!(materialized.used_preludes.is_empty(), "{path}");
23246:            assert!(!materialized.source.contains("helper used"), "{path}");
23261:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23289:            assert!(materialized.used_preludes.is_empty(), "{path}");
23290:            assert!(!materialized.source.contains("helper used"), "{path}");
23305:            "assert.sameValue = function() { throw 'assert used'; };\n".to_string(),
23325:            assert!(materialized.used_preludes.is_empty());
23346:        assert!(materialized.used_preludes.is_empty());
23365:        assert!(materialized.used_preludes.is_empty());
23379:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23401:            assert!(materialized.used_preludes.is_empty());
23402:            assert!(!materialized.source.contains("helper used"));
23419:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23436:            assert!(materialized.used_preludes.is_empty());
23437:            assert!(!materialized.source.contains("helper used"));
23457:            "function isConstructor() { throw 'isConstructor helper used'; }\n".to_string(),
23485:            assert!(materialized.used_preludes.is_empty());
23487:            assert!(!materialized.source.contains("isConstructor helper used"));
23505:            "function verifyProperty() { throw 'property helper used'; }\n".to_string(),
23510:            "function isConstructor() { throw 'isConstructor helper used'; }\n".to_string(),
23598:            assert!(materialized.used_preludes.is_empty());
23600:            assert!(!materialized.source.contains("property helper used"));
23601:            assert!(!materialized.source.contains("isConstructor helper used"));
23615:            "assert.throws = function() { throw 'assert used'; };\n".to_string(),
23639:            assert!(materialized.used_preludes.is_empty());
23667:        assert!(materialized.used_preludes.is_empty());
23697:        assert!(materialized.used_preludes.is_empty());
23730:        assert!(materialized.used_preludes.is_empty());
23762:        assert!(materialized.used_preludes.is_empty());
23781:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23795:            assert!(materialized.used_preludes.is_empty());
23796:            assert!(!materialized.source.contains("helper used"));
23822:        assert!(materialized.used_preludes.is_empty());
23837:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
23871:            assert!(materialized.used_preludes.is_empty());
23872:            assert!(!materialized.source.contains("helper used"));
23896:            "function verifyNotWritable() { throw 'helper used'; }\n".to_string(),
23937:                assert!(materialized.used_preludes.is_empty());
23938:                assert!(!materialized.source.contains("helper used"));
23972:            assert!(materialized.used_preludes.is_empty());
23973:            assert!(!materialized.source.contains("helper used"));
23997:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
24010:            assert!(materialized.used_preludes.is_empty());
24011:            assert!(!materialized.source.contains("helper used"));
24031:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
24051:                assert!(materialized.used_preludes.is_empty());
24052:                assert!(!materialized.source.contains("helper used"));
24067:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
24108:                assert!(materialized.used_preludes.is_empty());
24109:                assert!(!materialized.source.contains("helper used"));
24152:        assert!(materialized.used_preludes.is_empty());
24235:            assert!(materialized.used_preludes.is_empty());
24280:            assert!(materialized.used_preludes.is_empty());
24298:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
24390:                assert!(materialized.used_preludes.is_empty());
24391:                assert!(!materialized.source.contains("helper used"));
24410:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
24421:        assert!(materialized.used_preludes.is_empty());
24422:        assert!(!materialized.source.contains("helper used"));
24444:                assert!(materialized.used_preludes.is_empty());
24445:                assert!(!materialized.source.contains("helper used"));
24475:        assert!(materialized.used_preludes.is_empty());
24520:            assert!(materialized.used_preludes.is_empty());
24533:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
24571:                assert!(materialized.used_preludes.is_empty());
24572:                assert!(!materialized.source.contains("helper used"));
24584:            "throw 'assert helper used';\n".to_string(),
24597:        assert!(materialized.used_preludes.is_empty());
24598:        assert!(!materialized.source.contains("assert helper used"));
24615:        assert!(materialized.used_preludes.is_empty());
24630:            "var ctors = [Float64Array]; function MayNeedBigInt() { throw 'helper used'; }\n"
24636:            "function compareArray() { throw 'compare helper used'; }\n".to_string(),
24711:            assert!(materialized.used_preludes.is_empty());
24712:            assert!(!materialized.source.contains("helper used"));
24713:            assert!(!materialized.source.contains("compare helper used"));
24731:            "var ctors = [Float64Array]; function MayNeedBigInt() { throw 'helper used'; }\n"
24771:            assert!(materialized.used_preludes.is_empty());
24772:            assert!(!materialized.source.contains("helper used"));
24795:                format!("throw '{} helper used';\n", include),
24835:            assert!(materialized.used_preludes.is_empty());
24836:            assert!(!materialized.source.contains("helper used"));
24866:                format!("throw '{} helper used';\n", include),
24933:            assert!(materialized.used_preludes.is_empty());
24934:            assert!(!materialized.source.contains("helper used"));
24956:            "var ctors = [Float64Array]; function MayNeedBigInt() { throw 'helper used'; }\n"
24969:        assert!(materialized.used_preludes.is_empty());
24970:        assert!(!materialized.source.contains("helper used"));
24985:            "var ctors = [Float64Array]; function MayNeedBigInt() { throw 'helper used'; }\n"
24997:        assert!(materialized.used_preludes.is_empty());
24998:        assert!(!materialized.source.contains("helper used"));
25012:            "function verifyProperty() { throw 'property helper used'; }\n".to_string(),
25017:            "function testWithTypedArrayConstructors() { throw 'typed array helper used'; }\n"
25031:        assert!(materialized.used_preludes.is_empty());
25032:        assert!(!materialized.source.contains("property helper used"));
25033:        assert!(!materialized.source.contains("typed array helper used"));
25050:            "function testWithTypedArrayConstructors() { throw 'typed array helper used'; }\n"
25062:        assert!(materialized.used_preludes.is_empty());
25063:        assert!(!materialized.source.contains("typed array helper used"));
25076:            "var ctors = [Float64Array]; function Convert() { throw 'helper used'; }\n".to_string(),
25087:        assert!(materialized.used_preludes.is_empty());
25088:        assert!(!materialized.source.contains("helper used"));
25101:            "function testWithBigIntTypedArrayConstructors() { throw 'typed array helper used'; }\n"
25116:        assert!(materialized.used_preludes.is_empty());
25117:        assert!(!materialized.source.contains("typed array helper used"));
25137:            "function verifyProperty() { throw 'property helper used'; }\n".to_string(),
25147:            "function isConstructor() { throw 'isConstructor helper used'; }\n".to_string(),
25177:            assert!(materialized.used_preludes.is_empty());
25178:            assert!(!materialized.source.contains("property helper used"));
25180:            assert!(!materialized.source.contains("isConstructor helper used"));
25195:            "var ctors = [Float64Array]; function MayNeedBigInt() { throw 'helper used'; }\n"
25201:            "function compareArray() { throw 'compare helper used'; }\n".to_string(),
25217:        assert!(materialized.used_preludes.is_empty());
25218:        assert!(!materialized.source.contains("helper used"));
25219:        assert!(!materialized.source.contains("compare helper used"));
25235:            "var ctors = [Float64Array]; function MayNeedBigInt() { throw 'helper used'; }\n"
25241:            "function compareArray() { throw 'compare helper used'; }\n".to_string(),
25276:            assert!(materialized.used_preludes.is_empty());
25277:            assert!(!materialized.source.contains("helper used"));
25278:            assert!(!materialized.source.contains("compare helper used"));
25296:            "function TestIterationAndResize() { throw 'helper used'; }\n".to_string(),
25301:            "function compareArray() { throw 'compare helper used'; }\n".to_string(),
25336:            assert!(materialized.used_preludes.is_empty());
25337:            assert!(!materialized.source.contains("helper used"));
25338:            assert!(!materialized.source.contains("compare helper used"));
25356:            "function TestIterationAndResize() { throw 'helper used'; }\n".to_string(),
25361:            "function compareArray() { throw 'compare helper used'; }\n".to_string(),
25410:            assert!(materialized.used_preludes.is_empty());
25411:            assert!(!materialized.source.contains("helper used"));
25412:            assert!(!materialized.source.contains("compare helper used"));
25430:            "function CollectValuesAndResize() { throw 'helper used'; }\n".to_string(),
25435:            "function compareArray() { throw 'compare helper used'; }\n".to_string(),
25580:            assert!(materialized.used_preludes.is_empty());
25581:            assert!(!materialized.source.contains("helper used"));
25582:            assert!(!materialized.source.contains("compare helper used"));
25600:            "function subClass() { throw 'helper used'; }\n".to_string(),
25626:            assert!(materialized.used_preludes.is_empty());
25627:            assert!(!materialized.source.contains("helper used"));
25639:            "function CreateResizableArrayBuffer() { throw 'helper used'; }\n".to_string(),
25669:            assert!(materialized.used_preludes.is_empty());
25670:            assert!(!materialized.source.contains("helper used"));
25682:            "function CreateResizableArrayBuffer() { throw 'helper used'; }\n".to_string(),
25708:            assert!(materialized.used_preludes.is_empty());
25709:            assert!(!materialized.source.contains("helper used"));
25723:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
25752:            assert!(materialized.used_preludes.is_empty());
25753:            assert!(!materialized.source.contains("helper used"));
25764:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
25790:            assert!(materialized.used_preludes.is_empty());
25791:            assert!(!materialized.source.contains("helper used"));
25802:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
25814:        assert!(materialized.used_preludes.is_empty());
25815:        assert!(!materialized.source.contains("helper used"));
25831:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
25871:            assert!(materialized.used_preludes.is_empty());
25872:            assert!(!materialized.source.contains("helper used"));
25901:        assert!(materialized.used_preludes.is_empty());
25931:        assert!(materialized.used_preludes.is_empty());
25948:            "function verifyProperty() { throw 'helper used'; }\nfunction verifyEqualTo() { throw 'helper used'; }\n".to_string(),
26000:            assert!(materialized.used_preludes.is_empty());
26001:            assert!(!materialized.source.contains("helper used"));
26019:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26126:            assert!(materialized.used_preludes.is_empty());
26127:            assert!(!materialized.source.contains("helper used"));
26143:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26250:            assert!(materialized.used_preludes.is_empty());
26251:            assert!(!materialized.source.contains("helper used"));
26268:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26303:            assert!(materialized.used_preludes.is_empty());
26304:            assert!(!materialized.source.contains("helper used"));
26321:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26334:            assert!(materialized.used_preludes.is_empty());
26335:            assert!(!materialized.source.contains("helper used"));
26365:            "function verifyNotConfigurable() { throw 'helper used'; }\n".to_string(),
26494:            assert!(materialized.used_preludes.is_empty());
26496:            assert!(!materialized.source.contains("helper used"));
26513:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26528:                assert!(materialized.used_preludes.is_empty());
26529:                assert!(!materialized.source.contains("helper used"));
26566:                assert!(materialized.used_preludes.is_empty());
26589:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26594:            "function $DETACHBUFFER() { throw 'detach helper used'; }\n".to_string(),
26714:            assert!(materialized.used_preludes.is_empty());
26715:            assert!(!materialized.source.contains("helper used"));
26716:            assert!(!materialized.source.contains("detach helper used"));
26735:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26749:                assert!(materialized.used_preludes.is_empty());
26750:                assert!(!materialized.source.contains("helper used"));
26785:                assert!(materialized.used_preludes.is_empty());
26809:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
26831:                assert!(materialized.used_preludes.is_empty());
26832:                assert!(!materialized.source.contains("helper used"));
26860:                assert!(materialized.used_preludes.is_empty());
26942:            assert!(materialized.used_preludes.is_empty());
27003:            assert!(materialized.used_preludes.is_empty());
27064:            assert!(materialized.used_preludes.is_empty());
27106:            assert!(materialized.used_preludes.is_empty());
27176:            assert!(materialized.used_preludes.is_empty());
27230:            assert!(materialized.used_preludes.is_empty());
27247:            "function verifyProperty() { throw 'helper used'; }\n".to_string(),
27259:            assert!(materialized.used_preludes.is_empty());
27260:            assert!(!materialized.source.contains("helper used"));
27290:            assert!(materialized.used_preludes.is_empty());
```
