mod grpc;
mod hyper_compat;

use glommio::{CpuSet, Latency, LocalExecutorPoolBuilder, PoolPlacement, Shares};
use grpc::pb::test_server::{Test, TestServer};
use hyper::{Method, Request, Response, StatusCode, body::Incoming, service::service_fn};
use hyper_compat::ResponseBody;
use std::convert::Infallible;
// use tonic::client::GrpcService;
use tower::Service;

async fn hyper_demo(req: Request<Incoming>) -> Result<Response<ResponseBody>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/hello") => Ok(Response::new(ResponseBody::from("world"))),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(ResponseBody::from("notfound"))
            .unwrap()),
    }
}

fn main() {
    let greeter = grpc::MyGreeter::default();
    let service: TestServer<grpc::MyGreeter> = TestServer::new(greeter);
    LocalExecutorPoolBuilder::new(glommio::PoolPlacement::MaxSpread(8, CpuSet::online().ok()))
        .on_all_shards(|| async move {
            let id = glommio::executor().id();
            println!("starting executor {id}");
            hyper_compat::serve_http2(
                ([0, 0, 0, 0], 8080),
                service_fn(move |req| {
                    glommio::executor().create_task_queue(
                        Shares::default(),
                        Latency::NotImportant,
                        &format!("task queue {id}"),
                    );
                    let mut svc = service.clone();
                    svc.call(req)
                }),
                2,
            )
            .await
            .unwrap();
        })
        .unwrap()
        .join_all();
}
