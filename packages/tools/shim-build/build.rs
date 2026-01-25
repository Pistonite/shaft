use std::collections::BTreeMap;
use std::path::PathBuf;

use cu::pre::*;

fn main() -> cu::Result<()> {
    use std::fmt::Write as _;

    let config_path = env!("SHAFT_SHIM_BUILD_CONFIG");
    let config_json = json::parse::<BTreeMap<String, Vec<String>>>(&cu::fs::read_string(config_path)?)?;
    let mut output = String::new();
    if config_json.is_empty() {
        writeln!(output, "fn main() {{}}")?;
    } else {
        writeln!(output, "use shaft_shim_build as lib;")?;
        writeln!(output, "use std::process::{{ExitCode, Command}};")?;
        writeln!(output, "fn main() -> ExitCode {{")?;
        writeln!(output, "    let mut args = std::env::args_os();")?;
        writeln!(output, "    let Some(arg0) = args.next() else {{ return ExitCode::FAILURE; }};")?;
        writeln!(output, "    let mut cmd = match lib::exe_name(&arg0) {{")?;

        for (cmd, args) in config_json {
            writeln!(output, "        {:?} => {{", cmd.as_bytes())?;
            if args.len() < 1 {
                println!("cargo::error='{cmd}': arg length must be > 1");
                cu::bail!("config error");
            }
            let arg0 = &args[0];
            let arg_rest = &args[1..];
            if args.len() > 1 {
                writeln!(output, "            let mut c = Command::new({arg0:?});")?;
                writeln!(output, "            c.args({arg_rest:?}); c }}")?;
            } else {
                writeln!(output, "            Command::new({arg0:?}) }}")?;
            }
        }
        writeln!(output, "        _ => {{ eprintln!(\"invalid argument\"); return ExitCode::FAILURE; }}")?;

        writeln!(output, "    }};")?;
        writeln!(output, "    cmd.args(args); lib::exec_replace(cmd)")?;

        writeln!(output, "}}")?;
    }

    let mut main_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    main_path.push("main.rs");
    cu::fs::write(main_path, output)?;
    Ok(())
}
