use dotenv::dotenv;

use std::future::Future;
use std::net::SocketAddr;

use tokio_util::sync::CancellationToken;
use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

use tokio::select;
use tokio::time::sleep;
use tokio::time::Duration;

use tracing::{debug, info};

pub mod config;
use config::{Config, Environment};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    #[tracing::instrument]
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let remote_addr = request.remote_addr();
        let request_future = async move {
            debug!("Got a request from {:?}", request.remote_addr());

            // Take a long time to complete request for the client to cancel early
            sleep(Duration::from_secs(10)).await;

            let reply = hello_world::HelloReply {
                message: format!("Hello {}!", request.into_inner().name),
            };

            debug!("Replying with {:?}", reply);

            Ok(Response::new(reply))
        };
        let cancellation_future = async move {
            debug!("Request from {:?} cancelled by client", remote_addr);
            // If this future is executed it means the request future was dropped,
            // so it doesn't actually matter what is returned here
            Err(Status::cancelled("Request cancelled by client"))
        };
        with_cancellation_handler(request_future, cancellation_future).await
    }
}

async fn with_cancellation_handler<FRequest, FCancellation>(
    request_future: FRequest,
    cancellation_future: FCancellation,
) -> Result<Response<HelloReply>, Status>
where
    FRequest: Future<Output = Result<Response<HelloReply>, Status>> + Send + 'static,
    FCancellation: Future<Output = Result<Response<HelloReply>, Status>> + Send + 'static,
{
    let token = CancellationToken::new();
    // Will call token.cancel() when the future is dropped, such as when the client cancels the request
    let _drop_guard = token.clone().drop_guard();
    let select_task = tokio::spawn(async move {
        // Can select on token cancellation on any cancellable future while handling the request,
        // allowing for custom cleanup code or monitoring
        select! {
            res = request_future => res,
            _ = token.cancelled() => cancellation_future.await,
        }
    });

    select_task.await.unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load color-eyre
    color_eyre::install()?;

    // Load .env
    dotenv().ok();

    // Load config
    let env = Environment::from_env()?;
    let config = Config::with_env(env.clone())?;

    // Setup logging
    // Development: pretty print
    // Production: JSON
    match config.production_mode {
        true => {
            tracing_subscriber::fmt::fmt()
                .json()
                .with_max_level(config.log_level.clone())
                .init();
        }
        false => {
            tracing_subscriber::fmt::fmt()
                .with_max_level(config.log_level.clone())
                .init();
        }
    }

    info!("Environment: {:?}", &env);
    info!("Config: {:?}", &config);

    let addr: SocketAddr = config.binding_address;
    let greeter = MyGreeter::default();

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<GreeterServer<MyGreeter>>()
        .await; // TODO: Check health checker works

    info!("Listening on {}", addr);

    let server = Server::builder()
        .trace_fn(|_| tracing::info_span!("beam-index"))
        .add_service(GreeterServer::new(greeter))
        .add_service(health_service);

    match listenfd::ListenFd::from_env().take_tcp_listener(0)? {
        Some(listener) => {
            listener.set_nonblocking(true)?;
            let listener = tokio_stream::wrappers::TcpListenerStream::new(
                tokio::net::TcpListener::from_std(listener)?,
            );

            server.serve_with_incoming(listener).await?;
        }
        None => {
            server.serve(addr).await?;
        }
    }

    Ok(())
}
