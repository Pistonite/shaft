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
#[inline(always)]
pub fn run<S: Serialize>(input: &S, script: &str) -> cu::Result<json::Value> {
    let input_string = cu::check!(
        json::stringify(input),
        "failed to serialize script input to string"
    )?;
    let output = run_str(&input_string, script)?;
    cu::check!(json::parse(&output), "failed to parse javascript output")
}
pub fn run_str(input_str: &str, script: &str) -> cu::Result<String> {
    if !script.as_bytes().starts_with(b"function main") {
        cu::bail!("invalid script: must start with function main");
    }

    let mut limits = RuntimeLimits::default();
    limits.set_loop_iteration_limit(2048);
    limits.set_recursion_limit(2048);
    limits.set_stack_size_limit(20480);

    let mut context = JsContext::default();
    context.set_runtime_limits(limits);

    let script = format!(
        "{}{}\n/**/JSON.stringify(main({}))",
        include_str!("./lib.js"),
        script,
        input_str
    );
    match context.eval(JsSource::from_bytes(&script)) {
        Ok(x) => {
            let s = cu::check!(x.as_string(), "javascript output is not a string: {x:?}")?;
            cu::check!(
                s.to_std_string(),
                "javascript output cannot be converted to utf-8"
            )
        }
        Err(e) => {
            cu::bail!("error evaluating javascript: {e:?}");
        }
    }
}
