use land_worker_server::Opts;

#[tokio::main]
async fn main() {
    land_common::init_logging(false);
    
    let opts = Opts::default();
    land_worker_server::run(opts).await.unwrap();
}
