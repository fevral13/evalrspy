use js_sandbox::{AnyError, Script};
use pyo3::prelude::*;
use serde::{self, Deserialize};
use serde_json::{self, Value};
use std::time::Duration;
use thiserror;

use super::constants::{DEFAULT_TIMEOUT, JS_PRELUDE};

#[derive(Debug, Deserialize)]
pub struct Request {
    pub script: String,
    pub variables: Value,
    #[serde(default = "Request::default_timeout")]
    pub timeout: u64,
}

impl Request {
    fn default_timeout() -> u64 {
        DEFAULT_TIMEOUT
    }
}

#[derive(thiserror::Error, Debug)]
enum EvalrsError {
    #[error("Wrong argument structure")]
    WrongArguments,

    #[error("Variables must be a dict")]
    WrongVariablesType,

    #[error("Script is not a valid JS code")]
    WrongScriptCode { source: AnyError },

    #[error("Script evaluation error")]
    ScriptEvaluationError { source: AnyError },
}

#[pyfunction]
pub fn evaluate(request: String) -> String {
    let payload = match evaluate_inner(request) {
        Ok(result) => {
            serde_json::json!(
                {"status": "ok", "result": result}
            )
        },
        Err(error) => {
            let formatted_error = format!("{:?}", error);
            serde_json::json!(
                {"status": "error", "error": formatted_error}
            )
        }
    };
    serde_json::to_string(&payload).unwrap()
}


fn evaluate_inner(request: String) -> Result<Value, EvalrsError> {
    let request_data = parse_request(&request)?;
    let raw_script = render_script(&request_data.variables)?;
    let evaluator = get_script_evaluator(&raw_script, request_data.timeout)?;
    evaluate_script(evaluator, &request_data.script, &request_data.variables)
}

fn parse_request(request_string: &str) -> Result<Request, EvalrsError> {
    let parse_result: Result<Request, serde_json::Error> = serde_json::from_str(request_string);

    match parse_result {
        Ok(request) => Ok(request),
        Err(_) => Err(EvalrsError::WrongArguments),
    }
}

fn get_argument_list(variables: &Value) -> Result<Vec<String>, EvalrsError> {
    match variables {
        Value::Object(object) => Ok(object.keys().cloned().collect::<Vec<String>>()),
        _ => Err(EvalrsError::WrongVariablesType),
    }
}

fn render_script(variables: &Value) -> Result<String, EvalrsError> {
    let arguments = get_argument_list(variables)?;
    let arguments_string = arguments.join(", ");
    Ok(format!(
        r#" {prelude} function wrapper(script_snippet, {{ {arguments} }} ){{ return eval(script_snippet) }}"#,
        prelude = JS_PRELUDE,
        arguments = arguments_string,
    ))
}

fn get_script_evaluator(script_code: &str, timeout: u64) -> Result<Script, EvalrsError> {
    let duration = Duration::from_millis(timeout);

    match Script::from_string(script_code) {
        Ok(evaluator) => Ok(evaluator.with_timeout(duration)),
        Err(error) => Err(EvalrsError::WrongScriptCode { source: error }),
    }
}

fn evaluate_script(
    mut evaluator: Script,
    script: &String,
    variables: &Value,
) -> Result<Value, EvalrsError> {
    match evaluator.call(
        "wrapper",
        (Value::String(script.clone()), variables.clone()),
    ) {
        Ok(result) => Ok(result),
        Err(error) => Err(EvalrsError::ScriptEvaluationError { source: error }),
    }
}

#[cfg(test)]
mod test {
    use super::{get_argument_list, parse_request};
    use crate::evaluator::evaluator::{render_script, EvalrsError, evaluate_inner};
    use serde_json::{json, Value};

    #[test]
    fn test_parse_request_ok() {
        let payload = r#"{"script": "return 1", "variables": {"a": 2, "b": [1,2,3]}}"#;
        let parsed_request = parse_request(&payload).expect("Json parse failed");

        assert_eq!("return 1", parsed_request.script);
        assert_eq!(json!({"a": 2, "b": [1,2,3]}), parsed_request.variables);
        assert_eq!(500, parsed_request.timeout);
    }

    #[test]
    fn test_parse_request_error() {
        let payload = r#"{}"#;
        let _parsed_request_error = parse_request(&payload).unwrap_err();

        assert!(matches!(EvalrsError::WrongArguments, _parsed_request_error));
    }

    #[test]
    fn test_get_arguments_ok() {
        let args = json!({"alpha": 1, "beta": [], "gamma": Value::Null});
        assert_eq!(
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
            get_argument_list(&args).unwrap()
        );
    }

    #[test]
    fn test_get_arguments_error() {
        let args = json!([]);
        let _result_error = get_argument_list(&args).unwrap_err();

        assert!(matches!(EvalrsError::WrongVariablesType, _result_error));
    }

    #[test]
    fn test_get_arguments_empty_dict() {
        let args = json!({});
        assert_eq!(Vec::<String>::new(), get_argument_list(&args).unwrap());
    }

    #[test]
    fn test_script_render_ok() {
        let variables = json!({"a": 1, "b": 2, "e": []});
        let rendered_script = render_script(&variables).expect("Error rendering the script");
        assert!(rendered_script.ends_with(
            "function wrapper(script_snippet, { a, b, e } ){ return eval(script_snippet) }"
        ));
    }

    #[test]
    fn test_script_render_error() {
        let variables = json!([]);
        let _rendered_script_error = render_script(&variables).unwrap_err();
        assert!(matches!(
            EvalrsError::WrongVariablesType,
            _rendered_script_error
        ));
    }

    #[test]
    fn test_evaluate_ok() {
        let request_string = r#"{
"script": "a+3",
"variables": {"a": 3}

} "#
        .to_string();
        let result = evaluate_inner(request_string).unwrap();

        assert_eq!(6, result.as_i64().unwrap());
    }

    #[test]
    fn test_evaluate_error() {
        let request_string = r#"{
"script": "return",
"variables": {"a": 3}
} "#
        .to_string();
        assert!(evaluate_inner(request_string).is_err());
    }
}
