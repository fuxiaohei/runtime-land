tonic::include_proto!("moni");

mod server;
pub use server::start as start_server;
