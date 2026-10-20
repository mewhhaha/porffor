//! Product Wasm size controls for prepared direct eval dispatch.
use super::*;

fn main_bytes(artifact: &WasmArtifact) -> usize {
    let main = Parser::new(0)
        .parse_all(&artifact.bytes)
        .find_map(|payload| match payload.unwrap() {
            Payload::ExportSection(exports) => exports
                .into_iter()
                .map(Result::unwrap)
                .find(|export| export.name == "main")
                .map(|export| export.index),
            _ => None,
        })
        .expect("ordinary Script entry is exported");
    let mut index = imported_function_count(artifact);
    for payload in Parser::new(0).parse_all(&artifact.bytes) {
        if let Payload::CodeSectionEntry(body) = payload.unwrap() {
            if index == main {
                return body.range().len();
            }
            index += 1;
        }
    }
    panic!("Script entry has a complete function body");
}

#[test]
fn literal_eval_dispatch_grows_with_sites_without_a_candidate_cross_product() {
    let size = |count| {
        let source = (0..count)
            .map(|index| format!("eval('1 + {index}');\n"))
            .collect::<String>();
        let artifact = emit_script(&source).expect("literal eval source emits");
        expect_valid_module(&artifact, 0);
        main_bytes(&artifact)
    };
    let small = size(8);
    let large = size(32);
    assert!(large > small);
    assert!(
        large < 6 * small,
        "four times the literal sites must not multiply every candidate at every site: {small} -> {large} bytes"
    );
}
