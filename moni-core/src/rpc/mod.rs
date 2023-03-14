tonic::include_proto!("moni");

mod server;
pub use server::start as start_server;

mod client;
pub use client::new_client;
pub use client::new_client_with_token;