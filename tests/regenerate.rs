use std::{fs, path::Path};

use ethos_zero::{FileReader, Manifest, RustEmitter, TypeDeclaration, TypeExpression, VariantPayload};

struct EmptyManifest;

impl Manifest for EmptyManifest {
    fn resolve(&self, _: &str) -> Option<ethos_zero::FileLocation> {
        None
    }
}

#[test]
fn committed_datomic_library_is_exact_ethos_output() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".ethos/data/chronos.ethos");
    let source = fs::read_to_string(path).expect("read Chronos Ethos map");
    let file = FileReader::new(&EmptyManifest).read(&source).expect("read Chronos Ethos map");
    let generated = RustEmitter::new().emit(&file).expect("emit Chronos Datomic library");
    let committed = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(".ethos/data/datomic_library.rs"))
        .expect("read committed Datomic library output");
    assert_eq!(committed, generated, "the committed library is exact Ethos emission");
}

#[test]
fn parsed_ethos_map_witnesses_every_hand_d3_runtime_shape() {
    let ethos_zero::File::Schema(schema) = authored_map() else {
        panic!("Chronos owns a Schema map");
    };
    assert_eq!(schema.types.len(), 9);
    assert_enum(&schema.types[0], "SolarEventKind", &["CivilDawn", "Sunrise", "SolarNoon", "Sunset", "CivilDusk"]);
    assert_enum(&schema.types[1], "LocationSource", &["Geoclue", "Manual"]);
    assert_enum(
        &schema.types[2],
        "ZodiacSign",
        &[
            "Aries",
            "Taurus",
            "Gemini",
            "Cancer",
            "Leo",
            "Virgo",
            "Libra",
            "Scorpio",
            "Sagittarius",
            "Capricorn",
            "Aquarius",
            "Pisces",
        ],
    );
    assert_struct(&schema.types[3], "Location", &[("latitude", "Latitude"), ("longitude", "Longitude")]);
    assert_struct(
        &schema.types[4],
        "SolarEvent",
        &[("kind", "SolarEventKind"), ("when", "EpochTaiNanos"), ("location", "Location")],
    );
    assert_struct(
        &schema.types[5],
        "ZodiacalTime",
        &[("sign", "ZodiacSign"), ("degree", "ZodiacDegree"), ("minute", "ZodiacMinute")],
    );
    assert_tuple_alias(&schema.types[6], "ErrorMessage", "String");
    assert_enum(
        &schema.types[7],
        "Request",
        &["GetTime", "GetSchedule", "GetLocation", "SetLocation", "UseGeoclue", "Subscribe"],
    );
    assert_enum(&schema.types[8], "Response", &["Acked", "Time", "Schedule", "Location", "Event", "Error"]);
    assert_inline(&schema.types[7], "SetLocation", &[("latitude", "Latitude"), ("longitude", "Longitude")]);
    assert_inline(&schema.types[7], "Subscribe", &[("kinds", "Vector")]);
    assert_inline(&schema.types[8], "Time", &[("zodiacal_time", "ZodiacalTime")]);
    assert_inline(&schema.types[8], "Schedule", &[("events", "Vector")]);
    assert_inline(&schema.types[8], "Location", &[("location", "Location"), ("source", "LocationSource")]);
    assert_inline(&schema.types[8], "Event", &[("event", "SolarEvent")]);
    assert_inline(&schema.types[8], "Error", &[("message", "ErrorMessage")]);
    assert!(schema.kinds.iter().any(|kind| kind.name == "Datomic"
        && kind.capabilities.iter().any(|capability| format!("{capability:?}").contains("embody"))));
    assert!(schema.associations.is_empty());
}

fn authored_map() -> ethos_zero::File {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".ethos/data/chronos.ethos");
    let source = fs::read_to_string(path).expect("read Chronos Ethos map");
    FileReader::new(&EmptyManifest).read(&source).expect("parse Chronos Ethos map")
}

fn assert_enum(declaration: &TypeDeclaration, name: &str, variants: &[&str]) {
    let TypeDeclaration::Enum { name: actual, variants: actual_variants, .. } = declaration else {
        panic!("{name} is an enum");
    };
    assert_eq!(actual, name);
    assert_eq!(actual_variants.iter().map(|variant| variant.name.as_str()).collect::<Vec<_>>(), variants);
}

fn assert_struct(declaration: &TypeDeclaration, name: &str, fields: &[(&str, &str)]) {
    let TypeDeclaration::Struct { name: actual, fields: actual_fields, .. } = declaration else {
        panic!("{name} is a struct");
    };
    assert_eq!(actual, name);
    assert_eq!(
        actual_fields.iter().map(|field| (field.name.as_str(), reference(&field.ty))).collect::<Vec<_>>(),
        fields
    );
}

fn assert_tuple_alias(declaration: &TypeDeclaration, name: &str, target: &str) {
    let TypeDeclaration::Alias { name: actual, target: actual_target, .. } = declaration else {
        panic!("{name} is an alias");
    };
    assert_eq!(actual, name);
    assert_eq!(reference(actual_target), target);
}

fn assert_inline(declaration: &TypeDeclaration, variant: &str, fields: &[(&str, &str)]) {
    let TypeDeclaration::Enum { variants, .. } = declaration else {
        panic!("{variant} belongs to an enum");
    };
    let payload = &variants.iter().find(|member| member.name == variant).expect("named variant").payload;
    let VariantPayload::InlineStruct(actual) = payload else {
        panic!("{variant} owns inline fields");
    };
    assert_eq!(actual.iter().map(|field| (field.name.as_str(), constructor(&field.ty))).collect::<Vec<_>>(), fields);
}

fn reference(expression: &TypeExpression) -> &str {
    let TypeExpression::Reference(name) = expression else {
        panic!("expected a named field type");
    };
    name
}

fn constructor(expression: &TypeExpression) -> &str {
    match expression {
        TypeExpression::Reference(name) => name,
        TypeExpression::Application { constructor, .. } => constructor,
        _ => panic!("expected a named field type"),
    }
}
