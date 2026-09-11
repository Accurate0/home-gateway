use super::*;

#[derive(WorkflowContextVariables)]
struct Day {
    max: i64,
    uv: Option<f64>,
}

#[derive(WorkflowContextVariables)]
struct Forecast {
    description: String,
    #[variables(rename = "wet")]
    raining: bool,
    #[variables(skip)]
    #[allow(unused)]
    hidden: String,
    today: Day,
}

fn forecast() -> Forecast {
    Forecast {
        description: "sunny".to_owned(),
        raining: false,
        hidden: "secret".to_owned(),
        today: Day { max: 30, uv: None },
    }
}

fn path(input: &str) -> Path {
    Path::parse(input).expect("valid path")
}

#[test]
fn derive_builds_nested_shape() {
    let shape = Forecast::shape();

    assert_eq!(
        shape.paths(""),
        vec!["description", "today.max", "today.uv", "wet"]
    );
    assert_eq!(
        shape.lookup(path("today.uv").segments()),
        Some(&Shape::optional(VarType::Float))
    );
    assert_eq!(
        shape.lookup(path("today.max").segments()),
        Some(&Shape::required(VarType::Int))
    );
    assert!(shape.lookup(path("hidden").segments()).is_none());
}

#[test]
fn derive_builds_matching_node() {
    let vars = Vars::default().with("willyweather", forecast().to_node());

    assert_eq!(
        vars.value(&path("willyweather.description")),
        Some(&Value::String("sunny".to_owned()))
    );
    assert_eq!(
        vars.value(&path("willyweather.today.max")),
        Some(&Value::Int(30))
    );
    assert_eq!(
        vars.get(&path("willyweather.today.uv")),
        Some(&Node::Value(None))
    );
    assert_eq!(
        vars.value(&path("willyweather.wet")),
        Some(&Value::Bool(false))
    );
}

#[test]
fn vars_round_trip_keeps_int_and_float() {
    let mut node = Node::empty();
    node.insert("price", Node::Value(Some(Value::Float(3.0))));
    node.insert("count", Node::Value(Some(Value::Int(3))));
    node.insert("missing", Node::Value(None));

    let vars = Vars::default().with("event", node);
    let json = serde_json::to_value(&vars).expect("serialize");
    let restored: Vars = serde_json::from_value(json).expect("deserialize");

    assert_eq!(restored, vars);
    assert_eq!(
        restored.value(&path("event.price")),
        Some(&Value::Float(3.0))
    );
}

#[test]
fn require_unwraps_optional_leaf() {
    let mut shape = Forecast::shape();

    assert!(shape.require(&["today", "uv"]));
    assert_eq!(
        shape.lookup(path("today.uv").segments()),
        Some(&Shape::required(VarType::Float))
    );
    assert!(!shape.require(&["today", "missing"]));
}

#[test]
fn scope_lookup_lists_available_paths() {
    let scope = Scope::default().with("willyweather", Forecast::shape());

    let error = scope
        .lookup(&path("willyweather.nope"))
        .expect_err("unknown");
    assert!(error.contains("willyweather.today.max"));
    assert!(scope.lookup(&path("willyweather.today")).is_ok());
}

#[test]
fn path_rejects_invalid_segments() {
    assert!(Path::parse("event..name").is_err());
    assert!(Path::parse("Event.name").is_err());
    assert!(Path::parse("event.new_price").is_ok());
}

#[test]
fn float_renders_without_trailing_zero() {
    assert_eq!(Value::Float(70.0).render(), "70");
    assert_eq!(Value::Float(3.25).render(), "3.25");
    assert_eq!(
        Value::Int(4).coerce(VarType::Float),
        Some(Value::Float(4.0))
    );
    assert_eq!(Value::Bool(true).coerce(VarType::Int), None);
}
