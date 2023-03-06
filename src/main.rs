use clap::Parser;

#[derive(Parser, Debug)]
struct CliArgs {
    name: String,
    #[clap(long)]
    url: Option<String>,
}

#[tokio::main]
async fn main() {
    let start_time = tokio::time::Instant::now();

    let args = CliArgs::parse();

    let name = args.name.replace('-', "_");
    println!("Run name\t: {name}");

    let arch = "wasm32-wasi";
    println!("Run arch\t: {arch}");

    println!("elapsed\t, {:?}", start_time.elapsed());
}
