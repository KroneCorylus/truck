use std::error::Error;
use truck_base::diagnostics::Code;
use truck_stepio::r#in::{diagnostics::ConversionError, Table};

const CUBE: &str = include_str!("../../../resources/step/occt-cube.step");

#[test]
fn syntax_is_coded_and_preserves_the_parser_source() {
    let error = Table::from_step_with_diagnostics("not STEP").unwrap_err();
    assert_eq!(error.code, Code::StepSyntax);
    assert!(error.source().is_some());
}

#[test]
fn unsupported_surface_reports_omitted_face_and_dependency() {
    let step = CUBE.replace("#32 = PLANE('',#33);", "#32 = HOGE_SURFACE('',#33);");
    let table = Table::try_from_step(&step).unwrap();
    let shell = table.shell.values().next().unwrap();
    let report = table.to_compressed_shell_with_diagnostics(shell).unwrap();
    assert_eq!(report.value.faces.len(), 5);
    assert_eq!(report.diagnostics.len(), 1);
    let error = &report.diagnostics[0];
    assert_eq!(error.code, Code::StepUnsupportedEntity);
    assert_eq!(error.context.entity_id, Some(17));
    assert_eq!(error.context.related_entity_id, Some(32));
    assert!(error
        .source()
        .unwrap()
        .downcast_ref::<ConversionError>()
        .is_some());
    assert!(report.into_complete().is_err());
}

#[test]
fn unreadable_records_and_dependency_omissions_have_distinct_codes() {
    let step = CUBE.replace(
        "#23 = CARTESIAN_POINT('',(0.,0.,0.));",
        "#23 = CARTESIAN_POINT('',(0.,0.,'x'));",
    );
    let parsed = Table::from_step_with_diagnostics(&step).unwrap();
    assert_eq!(parsed.diagnostics.len(), 1);
    assert_eq!(parsed.diagnostics[0].code, Code::StepUnreadableEntity);
    assert_eq!(parsed.diagnostics[0].context.entity_id, Some(23));
    let table = parsed.value;
    let report = table
        .to_compressed_shell_with_diagnostics(table.shell.values().next().unwrap())
        .unwrap();
    assert_eq!(report.value.faces.len(), 3);
    assert!(report
        .diagnostics
        .iter()
        .any(|e| e.code == Code::StepSkippedDependency && e.context.related_entity_id == Some(22)));
}
