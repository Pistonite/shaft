use shaft_cli::CliApi;
#[cu::cli(preprocess = CliApi::preprocess)]
fn main(cli: CliApi) -> cu::Result<()> {
    cli.run()
}
