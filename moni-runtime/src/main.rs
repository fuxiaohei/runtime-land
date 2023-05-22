use tracing::info;

#[tokio::main]
async fn main() {
    moni_lib::tracing::init();
    
    info!("Hello, world!")
}
