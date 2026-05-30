// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Pistonite

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("vipath is only applicable on windows");
    std::process::ExitCode::FAILURE
}
#[cfg(windows)]
mod imp_win;
#[cfg(windows)]
#[cu::cli(flags = "flags")]
fn main(cli: imp_win::Cli) -> cu::Result<()> {
    imp_win::run(cli)
}
