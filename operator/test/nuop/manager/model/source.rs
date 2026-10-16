use crate::nuop::manager::model::{Source, SourceType};

#[test]
fn test_source_serializes_source_type() -> Result<(), Box<dyn std::error::Error>> {
    let source = Source {
        location: "https://github.com/test/repo".to_string(),
        path: "test/path".to_string(),
        source_type: SourceType::Scripts,
        credentials: None,
    };

    let yaml = serde_yaml::to_string(&source)?;
    assert!(yaml.contains("source_type: scripts"), "yaml was: {yaml}");

    let source = Source {
        source_type: SourceType::Crds,
        ..source
    };

    let yaml = serde_yaml::to_string(&source)?;
    assert!(yaml.contains("source_type: crds"), "yaml was: {yaml}");

    Ok(())
}

#[test]
fn test_source_deserializes_default_source_type() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = "location: https://github.com/test/repo\npath: test/path\n";
    let source: Source = serde_yaml::from_str(yaml)?;

    assert!(matches!(source.source_type, SourceType::Scripts));

    Ok(())
}
