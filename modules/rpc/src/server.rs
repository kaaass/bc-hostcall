use tonic::{transport::Server, Request, Response, Status};

use myadd::greeter_server::{Greeter, GreeterServer};
use myadd::{AddReply, AddRequest};

pub mod myadd {
    tonic::include_proto!("myadd");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn my_add(
        &self,
        request: Request<AddRequest>,
    ) -> Result<Response<AddReply>, Status> {
        println!("Got a request: {:?}", request);
        println!("Got a = {:?}", request.get_ref().a);
        println!("Got b = {:?}", request.get_ref().b);

        let a = request.get_ref().a;
        let b = request.get_ref().b;

        let reply = myadd::AddReply {
            c: a + b,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}