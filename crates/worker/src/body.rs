use std::pin::Pin;
use std::task::{Context, Poll};

use axum_core::body::{Body, BodyDataStream};
use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use tokio::sync::mpsc;

pub enum HostBody {
    Incoming(BodyDataStream),
    Outgoing(OutgoingStream),
}

impl HostBody {
    pub fn new(body: Body) -> Self {
        HostBody::Incoming(body.into_data_stream())
    }

    pub fn new_outgoing(body: Body) -> Self {
        HostBody::Outgoing(body)
    }

    pub async fn read(&mut self) -> Option<Vec<u8>> {
        match self {
            HostBody::Incoming(ref mut stream) => {
                let chunk = stream.next().await;
                if chunk.is_none() {
                    return None;
                }
                let chunk = chunk.unwrap().unwrap();
                Some(chunk.to_vec())
            }
        }
    }
    pub async fn read_all(&mut self) -> Option<Vec<u8>> {
        match self {
            HostBody::Incoming(ref mut stream) => {
                let mut bytes = Vec::new();
                while let Some(chunk) = stream.next().await {
                    bytes.extend_from_slice(&chunk.unwrap());
                }
                Some(bytes)
            }
        }
    }
    pub fn to_axum_body(self) -> Body {
        match self {
            HostBody::Incoming(stream) => Body::from_stream(stream),
        }
    }
}

pub struct OutgoingStream {
    receiver: mpsc::Receiver<Bytes>,
    sender: mpsc::Sender<Bytes>,
}

impl OutgoingStream {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        OutgoingStream { receiver, sender }
    }

    fn send_data(&mut self, data: Bytes) {
        // Ignore errors here, handle them in a real application
        let _ = self.sender.try_send(data);
    }
}

impl Stream for OutgoingStream {
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(data)) => Poll::Ready(Some(data)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
