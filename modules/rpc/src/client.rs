use myadd::greeter_client::GreeterClient;
use myadd::AddRequest;

pub mod myadd {
    tonic::include_proto!("myadd");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(AddRequest {
        a: 2,
        b: 5,
    });

    let response = client.my_add(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}