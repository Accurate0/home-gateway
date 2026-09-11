use crate::variables::{Node, Scope, Shape, Value, VarType, Vars};

use super::*;

fn scope() -> Scope {
    let mut event = Shape::empty();
    event.insert("name", Shape::required(VarType::String));
    event.insert("new_price", Shape::required(VarType::Float));
    event.insert("count", Shape::required(VarType::Int));
    event.insert("uv", Shape::optional(VarType::Float));

    let mut today = Shape::empty();
    today.insert("max", Shape::required(VarType::Int));

    let mut willyweather = Shape::empty();
    willyweather.insert("today", today);

    Scope::default()
        .with("event", event)
        .with("willyweather", willyweather)
}

fn vars(uv: Option<f64>) -> Vars {
    let mut event = Node::empty();
    event.insert("name", Node::Value(Some(Value::String("Milk".to_owned()))));
    event.insert("new_price", Node::Value(Some(Value::Float(3.456))));
    event.insert("count", Node::Value(Some(Value::Int(4))));
    event.insert("uv", Node::Value(uv.map(Value::Float)));

    Vars::default().with("event", event)
}

fn template(raw: &str) -> Template {
    Template::parse(raw).expect("valid template")
}

#[test]
fn dollar_before_placeholder_stays_literal() {
    let t = template("$${event.new_price | round(2)}");

    assert_eq!(t.render(&vars(None)).unwrap(), "$3.46");
}

#[test]
fn plain_text_has_no_exprs() {
    let t = template("just text $ 5");

    assert_eq!(t.exprs().count(), 0);
    assert_eq!(t.render(&vars(None)).unwrap(), "just text $ 5");
    assert_eq!(t.check(&scope()), Ok(VarType::String));
}

#[test]
fn whitespace_and_quoted_separators_parse() {
    let t = template("${ event.uv|round(1) | default('n/a | }') }");
    let expr = t.exprs().next().unwrap();

    assert_eq!(
        expr.filters,
        vec![
            Filter::Round(1),
            Filter::Default(Literal::String("n/a | }".to_owned()))
        ]
    );
    assert_eq!(t.render(&vars(None)).unwrap(), "n/a | }");
}

#[test]
fn syntax_errors_are_rejected() {
    assert!(Template::parse("drop ${event.name").is_err());
    assert!(Template::parse("${event.name | upper}").is_err());
    assert!(Template::parse("${event.name | round(x)}").is_err());
    assert!(Template::parse("${event.name | int(1)}").is_err());
    assert!(Template::parse("${}").is_err());
    assert!(Template::parse("${event.name | default(nope)}").is_err());
}

#[test]
fn unknown_path_lists_available() {
    let error = template("${event.bogus}").check(&scope()).unwrap_err();

    assert!(error.contains("event.new_price"));
}

#[test]
fn optional_requires_default() {
    let error = template("${event.uv}").check(&scope()).unwrap_err();
    assert!(error.contains("default"));

    let error = template("${event.uv | round(1)}")
        .check(&scope())
        .unwrap_err();
    assert!(error.contains("default"));

    assert_eq!(
        template("${event.uv | round(1) | default(\"n/a\")}").check(&scope()),
        Ok(VarType::String)
    );
    assert_eq!(
        template("${event.uv | default(0)}").check(&scope()),
        Ok(VarType::Float)
    );
}

#[test]
fn filter_type_errors() {
    assert!(
        template("${event.name | round(1)}")
            .check(&scope())
            .is_err()
    );
    assert!(
        template("${event.count | default(1)}")
            .check(&scope())
            .is_err()
    );
    assert!(
        template("${event.uv | default(true)}")
            .check(&scope())
            .is_err()
    );
    assert!(template("${willyweather.today}").check(&scope()).is_err());
}

#[test]
fn single_expression_keeps_type() {
    assert_eq!(template("${event.count}").check(&scope()), Ok(VarType::Int));
    assert_eq!(
        template("${event.new_price | int}").check(&scope()),
        Ok(VarType::Int)
    );
    assert_eq!(
        template("n ${event.count}").check(&scope()),
        Ok(VarType::String)
    );
    assert_eq!(
        template("${event.new_price | int}").evaluate(&vars(None)),
        Ok(Value::Int(3))
    );
}

#[test]
fn rendering_applies_filters() {
    let with_uv = vars(Some(7.25));

    assert_eq!(
        template("${event.uv | round(1) | default(\"n/a\")}")
            .render(&with_uv)
            .unwrap(),
        "7.3"
    );
    assert_eq!(
        template("${event.uv | default(0)}")
            .render(&vars(None))
            .unwrap(),
        "0"
    );
    assert_eq!(
        template("${event.new_price | round(0)}")
            .render(&with_uv)
            .unwrap(),
        "3"
    );
    assert_eq!(
        template("${event.name}: ${event.count}")
            .render(&with_uv)
            .unwrap(),
        "Milk: 4"
    );
}

#[test]
fn missing_required_value_is_an_error() {
    assert!(template("${event.nope}").render(&vars(None)).is_err());
}
