use boa_engine::vm::RuntimeLimits;
use boa_engine::{Context as JsContext, Source as JsSource};

use cu::pre::*;

/// Run a JavaScript using the json as input.
///
/// The script should contain a single function main(input),
/// and return a JSON output.
///
/// The input object is inlined into the script and the output
/// will be `JSON.stringify`-ed
pub fn run(input: &json::Value, script: &str) -> cu::Result<json::Value> {
    if !script.as_bytes().starts_with(b"function main") {
        cu::bail!("invalid script: must start with function main");
    }

    let mut limits = RuntimeLimits::default();
    limits.set_loop_iteration_limit(2048);
    limits.set_recursion_limit(2048);
    limits.set_stack_size_limit(20480);

    let mut context = JsContext::default();
    context.set_runtime_limits(limits);

    let input_string = cu::check!(
        json::stringify(input),
        "failed to serialize script input to string"
    )?;

    let script = format!(
        "{}{}\n/**/JSON.stringify(main({}))",
        include_str!("./lib.js"),
        script,
        input_string
    );
    let output = match context.eval(JsSource::from_bytes(&script)) {
        Ok(x) => {
            let s = cu::check!(x.as_string(), "javascript output is not a string: {x:?}")?;
            let s = cu::check!(
                s.to_std_string(),
                "javascript output cannot be converted to utf-8"
            )?;
            s
        }
        Err(e) => {
            cu::bail!("error evaluating javascript: {e:?}");
        }
    };

    cu::check!(json::parse(&output), "failed to parse javascript output")
}
