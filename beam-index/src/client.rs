//! A simple test client to showcase the server implementation

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

use tokio::time::{timeout, Duration};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    // Cancelling the request by dropping the request future after 1 second
    let response = match timeout(Duration::from_secs(1), client.say_hello(request)).await {
        Ok(response) => response?,
        Err(_) => {
            println!("Cancelled request after 1s");
            return Ok(());
        }
    };

    // // Timeout example
    // let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    // let timeout_channel = Timeout::new(channel, Duration::from_millis(1000));

    // let mut client = GreeterClient::new(timeout_channel);

    // let request = tonic::Request::new(HelloRequest {
    //     name: "Tonic".into(),
    // });

    // let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
