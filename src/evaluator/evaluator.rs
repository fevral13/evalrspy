use pyo3::prelude::*;
use serde::{self,  Deserialize};
use serde_json::{Value, from_str, Error};
use anyhow;

use super::constants::JS_PRELUDE;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub script: String,
    pub variables: Value,
    #[serde(default)]
    pub timeout: u64,
}


pub fn _evaluate(request: String) -> PyResult<String> {
    let _request_data = parse_request(&request);

    todo!();
}


fn parse_request(request: &str) -> Result<Request, Error> {
    from_str(request)
}

fn get_argument_list(variables: &Value) -> anyhow::Result<Vec<String>> {
    match variables {
        Value::Object(object) => Ok(object.keys().cloned().collect::<Vec<String>>()),
        _ => Err(anyhow::anyhow!("Variables are not a dict")),
    }
}

fn render_script(script: &str, variables: &Value) -> anyhow::Result<String> {
    let arguments = get_argument_list(variables)?;
    let arguments_string = arguments.join(", ");
    Ok(format!(
        r#" {prelude} function wrapper(script_snippet, {{ {arguments} }} ){{ return eval(script_snippet) }} "#,
        prelude = JS_PRELUDE,
        arguments = arguments_string,
    ))
}


#[cfg(test)]
mod test {
    use serde_json::{json, Value};
    use super::{parse_request, get_argument_list};

    #[test]
    fn test_parse_request() {
        let payload = r#"{"script": "return 1", "variables": {"a": 2, "b": [1,2,3]}}"#;
        let parsed_request = parse_request(&payload).expect("Json parse failed");

        assert_eq!("return 1", parsed_request.script);
        assert_eq!(json!({"a": 2, "b": [1,2,3]}), parsed_request.variables);
        assert_eq!(0, parsed_request.timeout);
    }

    #[test]
    fn test_get_arguments() {
        let args = json!({"alpha": 1, "beta": [], "gamma": Value::Null});
        assert_eq!(vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
                   get_argument_list(&args).unwrap());
    }

    #[test]
    fn test_get_arguments_empty_dict() {
        let args = json!({});
        assert_eq!(Vec::<String>::new(),
                   get_argument_list(&args).unwrap());
    }

    #[test]
    fn test_get_arguments_are_not_a_dict() {
        let args = json!([1,2,3]);
        assert!(get_argument_list(&args).is_err());
    }


}