use futures_lite::{AsyncRead, AsyncWrite, Future};
use glommio::{
    enclose,
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};
use hyper::{
    Error, Request, Response,
    body::{Body as HttpBody, Bytes, Frame, Incoming},
    service::service_fn,
};
use std::{
    io,
    marker::PhantomData,
    net::SocketAddr,
    pin::Pin,
    rc::Rc,
    slice,
    task::{Context, Poll},
};
// use tower::Service;

#[derive(Clone)]
struct HyperExecutor;
impl<F> hyper::rt::Executor<F> for HyperExecutor
where
    F: Future + 'static,
    F::Output: 'static,
{
    fn execute(&self, fut: F) {
        glommio::spawn_local(fut).detach();
    }
}

struct HyperStream(pub TcpStream);

impl hyper::rt::Write for HyperStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

impl hyper::rt::Read for HyperStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<std::io::Result<()>> {
        unsafe {
            let read_slice = {
                let buffer = buf.as_mut();
                buffer.as_mut_ptr().write_bytes(0, buffer.len());
                slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len())
            };
            Pin::new(&mut self.0).poll_read(cx, read_slice).map(|n| {
                if let Ok(n) = n {
                    buf.advance(n);
                }
                Ok(())
            })
        }
    }
}

pub struct ResponseBody {
    // Our ResponseBody type is !Send and !Sync
    _marker: PhantomData<*const ()>,
    data: Option<Bytes>,
}

impl From<&'static str> for ResponseBody {
    fn from(data: &'static str) -> Self {
        ResponseBody {
            _marker: PhantomData,
            data: Some(Bytes::from(data)),
        }
    }
}

impl HttpBody for ResponseBody {
    type Data = Bytes;
    type Error = Error;
    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Poll::Ready(self.get_mut().data.take().map(|d| Ok(Frame::data(d))))
    }
}

pub(crate) async fn serve_http2<S, B, E, A>(
    addr: A,
    service: S,
    max_connections: usize,
) -> io::Result<()>
where
    S: hyper::service::Service<Request<Incoming>, Response = Response<B>, Error = E>
        + Clone
        + 'static,
    E: std::error::Error + 'static + Send + Sync,
    A: Into<SocketAddr>,
    B: hyper::body::Body + 'static,
    B::Error: std::error::Error + 'static + Send + Sync,
{
    let listener = TcpListener::bind(addr.into())?;
    let conn_control = Rc::new(Semaphore::new(max_connections as _));
    loop {
        match listener.accept().await {
            Err(x) => {
                return Err(x.into());
            }
            Ok(stream) => {
                let addr = stream.local_addr().unwrap();
                let io = HyperStream(stream);
                let service = service.clone();
                glommio::spawn_local(enclose! {(conn_control) async move {
                    let _permit = conn_control.acquire_permit(1).await;
                    if let Err(err) = hyper::server::conn::http2::Builder::new(HyperExecutor).serve_connection(io, service).await {
                        if !err.is_incomplete_message() {
                            eprintln!("Stream from {addr:?} failed with error {err:?}");
                        }
                    }
                }}).detach();
            }
        }
    }
}
