//! The crate doc lists the entity types met in the corpus that are not implemented; this keeps
//! the list equal to what `Table::unsupported` reports.

use std::collections::BTreeSet;
use truck_stepio::r#in::*;

const STEP_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/step/");
const STEP_FILES: &[&str] = &[
    "occt-cone.step",
    "occt-cube.step",
    "occt-cylinder.step",
    "occt-sphere.step",
    "occt-torus.step",
    "occt-assy.step",
    "abc-0000.step",
    "abc-0006.step",
    "abc-0008.step",
    "abc-0035.step",
];

#[test]
fn unsupported_entities_of_the_corpus_are_listed_in_the_crate_doc() {
    let in_corpus: BTreeSet<String> = STEP_FILES
        .iter()
        .flat_map(|name| {
            let step = std::fs::read_to_string([STEP_DIRECTORY, name].concat()).unwrap();
            let table = Table::try_from_step(&step).unwrap();
            assert!(table.errors.is_empty(), "{name}: {:?}", table.errors);
            table.unsupported().into_keys().collect::<Vec<_>>()
        })
        .collect();
    let in_doc: BTreeSet<String> = include_str!("../../src/lib.rs")
        .lines()
        .filter_map(|line| line.strip_prefix("//! - `")?.strip_suffix('`'))
        .map(str::to_string)
        .collect();
    let expected: Vec<String> = in_corpus
        .iter()
        .map(|name| format!("//! - `{name}`"))
        .collect();
    assert_eq!(
        in_doc,
        in_corpus,
        "the crate doc should list:\n{}",
        expected.join("\n")
    );
}
