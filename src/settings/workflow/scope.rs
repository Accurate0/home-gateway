use std::collections::{BTreeMap, HashMap};

use crate::device_registry::DeviceRegistry;
use crate::templating::{Expr, Literal, Template};
use crate::variables::input::input_shape;
use crate::variables::{Scope, VarType};

use super::{
    Combinator, CompareOp, Condition, LeafCondition, ReusableWorkflow, Step, WorkflowDefinition,
};
use crate::settings::NotifyActionKind;

pub fn scope_for(
    definition: &WorkflowDefinition,
    registry: &DeviceRegistry,
) -> Result<Scope, String> {
    match definition {
        WorkflowDefinition::Triggered(workflow) => {
            if workflow.inputs.is_some() {
                return Err(format!(
                    "workflow '{}' has an `on:` trigger so it cannot declare `inputs:`",
                    workflow.name
                ));
            }

            let event = workflow
                .on
                .event_shape(registry)
                .map_err(|error| format!("workflow '{}': {error}", workflow.name))?;

            if let Some(when) = &workflow.when {
                let event_scope = Scope::default().with("event", event.clone());

                check_condition(when, &event_scope)
                    .map_err(|error| format!("workflow '{}' when: {error}", workflow.name))?;
            }

            Ok(context_scope(&workflow.body).with("event", event))
        }
        WorkflowDefinition::Reusable(workflow) => reusable_scope(workflow),
    }
}

pub fn reusable_scope(workflow: &ReusableWorkflow) -> Result<Scope, String> {
    let inputs = workflow.inputs.as_ref().ok_or_else(|| {
        format!(
            "reusable workflow '{}' must declare `inputs:` (use `inputs: {{}}` when it takes none)",
            workflow.name
        )
    })?;

    Ok(context_scope(workflow).with("input", input_shape(inputs)))
}

fn context_scope(workflow: &ReusableWorkflow) -> Scope {
    workflow
        .context
        .iter()
        .fold(Scope::default(), |scope, source| {
            scope.with(source.as_str(), source.shape())
        })
}

pub fn check_steps(workflow: &ReusableWorkflow, scope: &Scope) -> Result<(), String> {
    check_step_list(&workflow.run, scope)
        .map_err(|error| format!("workflow '{}': {error}", workflow.name))
}

fn check_step_list(steps: &[Step], scope: &Scope) -> Result<(), String> {
    let mut scope = scope.clone();

    for (index, step) in steps.iter().enumerate() {
        if let Some(when) = step.guard() {
            check_condition(when, &scope)
                .map_err(|error| format!("step {index} ({}) when: {error}", step.kind()))?;
        }

        for (label, template) in step.templates() {
            template
                .check(&scope)
                .map_err(|error| format!("step {index} ({}) {label}: {error}", step.kind()))?;
        }

        if let Step::Scene { run, .. } = step {
            check_step_list(run, &scope)
                .map_err(|error| format!("step {index} (scene) > {error}"))?;
        }

        if let Step::Lua { returns, .. } = step {
            scope = scope.with("lua", input_shape(returns));
        }
    }

    Ok(())
}

pub fn check_condition(condition: &Condition, scope: &Scope) -> Result<(), String> {
    match condition {
        Condition::Combinator(Combinator::All(conditions) | Combinator::Any(conditions)) => {
            conditions
                .iter()
                .try_for_each(|condition| check_condition(condition, scope))
        }
        Condition::Combinator(Combinator::Not(condition)) => check_condition(condition, scope),
        Condition::Leaf(LeafCondition::Var { var, op, value }) => check_var(var, *op, value, scope),
        Condition::Leaf(_) => Ok(()),
    }
}

fn check_var(var: &Expr, op: CompareOp, value: &Literal, scope: &Scope) -> Result<(), String> {
    let actual = var.check(scope)?;
    let expected = value.var_type();
    let numeric = |ty: VarType| matches!(ty, VarType::Int | VarType::Float);

    let valid = match op {
        CompareOp::Eq => actual == expected || (numeric(actual) && numeric(expected)),
        CompareOp::Gt | CompareOp::Lt | CompareOp::Gte | CompareOp::Lte => {
            numeric(actual) && numeric(expected)
        }
    };

    if valid {
        Ok(())
    } else {
        Err(format!(
            "cannot compare `{var}` ({actual}) with {value} ({expected}) using {op:?}"
        ))
    }
}

pub fn check_calls(
    workflow: &ReusableWorkflow,
    scope: &Scope,
    workflows: &HashMap<String, WorkflowDefinition>,
) -> Result<(), String> {
    let target = |name: &str| {
        workflows.get(name).ok_or_else(|| {
            format!(
                "workflow '{}': run_workflow references unknown workflow '{name}'",
                workflow.name
            )
        })
    };

    for step in workflow.steps() {
        match step {
            Step::RunWorkflow {
                workflow: name,
                with,
                ..
            } => {
                let inputs = callable_inputs(target(name)?)
                    .map_err(|error| format!("workflow '{}': {error}", workflow.name))?;

                check_with(name, &inputs, with, scope)
                    .map_err(|error| format!("workflow '{}': {error}", workflow.name))?;
            }
            Step::Notify { actions, .. } => {
                for action in actions {
                    let NotifyActionKind::RunWorkflow { workflow: name } = &action.action else {
                        continue;
                    };

                    let definition = workflows.get(name).ok_or_else(|| {
                        format!(
                            "workflow '{}': notify action references unknown workflow '{name}'",
                            workflow.name
                        )
                    })?;

                    let inputs = callable_inputs(definition)
                        .map_err(|error| format!("workflow '{}': {error}", workflow.name))?;

                    if !inputs.is_empty() {
                        return Err(format!(
                            "workflow '{}': notify action targets '{name}', which needs inputs a notification cannot pass",
                            workflow.name
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn callable_inputs(
    definition: &WorkflowDefinition,
) -> Result<BTreeMap<String, VarType>, String> {
    let body = definition.body();

    if body.references_namespace("event") {
        return Err(format!(
            "'{}' reads `event.*` so it can only run from its own trigger",
            body.name
        ));
    }

    Ok(body.inputs.clone().unwrap_or_default())
}

fn check_with(
    target: &str,
    inputs: &BTreeMap<String, VarType>,
    with: &BTreeMap<String, Template>,
    scope: &Scope,
) -> Result<(), String> {
    if let Some(unknown) = with.keys().find(|key| !inputs.contains_key(*key)) {
        return Err(format!(
            "run_workflow '{target}' passes unknown input `{unknown}`"
        ));
    }

    for (key, ty) in inputs {
        let template = with
            .get(key)
            .ok_or_else(|| format!("run_workflow '{target}' is missing input `{key}` ({ty})"))?;

        let actual = template
            .check(scope)
            .map_err(|error| format!("run_workflow '{target}' with.{key}: {error}"))?;

        if !ty.accepts(actual) {
            return Err(format!(
                "run_workflow '{target}' with.{key} is a {actual} but the input is a {ty}"
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use config::{Config, File, FileFormat};

    use super::super::ReusableWorkflow;
    use super::{check_steps, reusable_scope};

    fn workflow(yaml: &str) -> ReusableWorkflow {
        Config::builder()
            .add_source(File::from_str(yaml, FileFormat::Yaml))
            .build()
            .expect("the fixture should build")
            .try_deserialize()
            .expect("the fixture should deserialize")
    }

    fn check(yaml: &str) -> Result<(), String> {
        let workflow = workflow(yaml);
        let scope = reusable_scope(&workflow)?;

        check_steps(&workflow, &scope)
    }

    #[test]
    fn a_later_step_may_reference_what_a_lua_step_declares() {
        let result = check(
            r#"
name: Lua scope
slug: lua-scope
inputs: {}
run:
  - type: lua
    returns: { pct: int }
    script: "return { pct = 1 }"
  - type: mqtt_publish
    topic: t
    payload: "${lua.pct}"
    retain: false
"#,
        );

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn an_undeclared_lua_return_is_rejected() {
        let error = check(
            r#"
name: Lua scope
slug: lua-scope
inputs: {}
run:
  - type: lua
    returns: { pct: int }
    script: "return { pct = 1 }"
  - type: mqtt_publish
    topic: t
    payload: "${lua.other}"
    retain: false
"#,
        )
        .expect_err("an undeclared return should not type-check");

        assert!(error.contains("lua.other"), "unexpected error: {error}");
    }

    #[test]
    fn a_lua_return_is_not_visible_before_its_step() {
        let error = check(
            r#"
name: Lua scope
slug: lua-scope
inputs: {}
run:
  - type: mqtt_publish
    topic: t
    payload: "${lua.pct}"
    retain: false
  - type: lua
    returns: { pct: int }
    script: "return { pct = 1 }"
"#,
        )
        .expect_err("a lua return should not be visible to an earlier step");

        assert!(error.contains("lua.pct"), "unexpected error: {error}");
    }
}
