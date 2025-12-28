use shaft_cli::CliApi;
#[cu::cli]
async fn main(cli: CliApi) -> cu::Result<()> {
    cli.run()
}
