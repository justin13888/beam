pub mod config;
pub mod grpc;
pub mod repositories;
pub mod services;
pub mod utils;

pub mod proto {
    tonic::include_proto!("beam_index");
}
