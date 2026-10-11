use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "lila-bytes-catalog-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn catalog_reads_binary_fixtures_and_retains_mixed_records_at_one_url() {
    let directory = Directory::new();
    let source = "import value from './value_FIXTURE.js'; import raw from './value_FIXTURE.js' with {type:'bytes'}; const path = './raw_FIXTURE.png'; import(path, {with:{type:'bytes'}});";
    let entry_path = directory.0.join("entry.js");
    fs::write(&entry_path, source).unwrap();
    fs::write(directory.0.join("value_FIXTURE.js"), b"export default 42;").unwrap();
    fs::write(directory.0.join("raw_FIXTURE.png"), [0, 255, 128, 10]).unwrap();
    fs::write(directory.0.join("empty_FIXTURE.txt"), []).unwrap();
    let graph = declare(&entry_path, EmbeddedModuleGoal::Module, source, source)
        .unwrap()
        .unwrap()
        .graph;
    let referrer = EmbeddedModuleReferrer::Module(graph.entry().identity().into());
    let request = |specifier: &str, bytes: bool| {
        lila_engine::EmbeddedModuleRequest::try_new(
            specifier,
            if bytes {
                vec![("type".into(), "bytes".into())]
            } else {
                Vec::new()
            },
        )
        .unwrap()
    };
    let text = graph
        .resolve(&referrer, &request("./value_FIXTURE.js", false))
        .unwrap();
    let raw = graph
        .resolve(&referrer, &request("./value_FIXTURE.js", true))
        .unwrap();
    assert_eq!(text.kind(), EmbeddedModuleKind::SourceText);
    assert_eq!(raw.kind(), EmbeddedModuleKind::Bytes);
    assert_eq!(text.identity(), raw.identity());
    assert_eq!(text.meta_url(), raw.meta_url());
    assert_eq!(raw.payload_bytes(), b"export default 42;");
    let image = graph
        .resolve(&referrer, &request("./raw_FIXTURE.png", true))
        .unwrap();
    assert_eq!(image.payload_bytes(), &[0, 255, 128, 10]);
    assert!(graph
        .resolve(&referrer, &request("./raw_FIXTURE.png", false))
        .is_none());
    assert_eq!(
        graph
            .resolve(&referrer, &request("./empty_FIXTURE.txt", true))
            .unwrap()
            .payload_bytes(),
        b"" as &[u8]
    );
}
