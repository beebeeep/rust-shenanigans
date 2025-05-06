use std::pin::Pin;
// use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use async_stream::stream;
use futures_core::Stream;
use glommio::{TaskQueueHandle, channels::local_channel};
use tonic::{Request, Response, Result, Status};
pub mod pb {
    tonic::include_proto!("glommio_test");
}
use pb::{
    HelloResponse,
    test_server::{Test, TestServer},
};

type HelloResponseStream = Pin<Box<dyn Stream<Item = Result<pb::HelloResponse, Status>> + Send>>;

#[derive(Default)]
pub struct MyGreeter {
    tq: TaskQueueHandle,
}

#[tonic::async_trait]
impl Test for MyGreeter {
    async fn hello(
        &self,
        req: Request<pb::HelloRequest>,
    ) -> Result<Response<pb::HelloResponse>, Status> {
        let resp = pb::HelloResponse {
            greet: format!("hello, {}", req.into_inner().name),
        };
        Ok(Response::new(resp))
    }

    type Hello2Stream = HelloResponseStream;
    async fn hello2(
        &self,
        req: Request<pb::HelloRequest>,
    ) -> Result<Response<Self::Hello2Stream>, Status> {
        let req = req.into_inner();
        /*
        let (tx, rx) = local_channel::new_bounded(1);
        let streamer = match glommio::spawn_local_into(
            async move {
                loop {
                    let resp = HelloResponse {
                        greet: format!("hello, {}", req.name),
                    };
                    match tx.send(Ok(resp)).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("error: {e}");
                            break;
                        }
                    };
                }
            },
            self.tq,
        ) {
            Ok(h) => h,
            Err(e) => return Err(Status::internal(format!("starting stream: {e}"))),
        };

        let out_stream = rx.stream();
        Ok(Response::new(Box::pin(out_stream)))
        */

        let s = stream! {
            loop {
                yield Ok(HelloResponse{greet: format!("hello, {}", req.name)});
            }
        };
        Ok(Response::new(Box::pin(s) as Self::Hello2Stream))
    }
}
