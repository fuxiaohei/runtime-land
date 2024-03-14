use super::host::land::http::body::BodyError;
use super::host::land::http::types::RedirectPolicy;
use axum::body::{Body, BodyDataStream, Bytes};
use futures_util::StreamExt;
use http_body::Frame;
use http_body_util::BodyExt;
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::sync::{atomic::AtomicU32, Once};
use std::task::{Context, Poll};
use std::{collections::HashMap, future::Future, pin::Pin};
use tokio::sync::{mpsc, oneshot};

static CLIENT_INIT_ONCE: Once = Once::new();
static REDIRECT_FOLLOW_POOL: OnceCell<Client> = OnceCell::new();
static REDIRECT_ERROR_POOL: OnceCell<Client> = OnceCell::new();
static REDIRECT_MANUAL_POOL: OnceCell<Client> = OnceCell::new();

fn init_clients() {
    CLIENT_INIT_ONCE.call_once(|| {
        REDIRECT_ERROR_POOL
            .set(
                reqwest::Client::builder()
                    .redirect(RedirectPolicy::Error.try_into().unwrap())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        REDIRECT_FOLLOW_POOL
            .set(
                reqwest::Client::builder()
                    .redirect(RedirectPolicy::Follow.try_into().unwrap())
                    .build()
                    .unwrap(),
            )
            .unwrap();
        REDIRECT_MANUAL_POOL
            .set(
                reqwest::Client::builder()
                    .redirect(RedirectPolicy::Manual.try_into().unwrap())
                    .build()
                    .unwrap(),
            )
            .unwrap();
    });
}

pub struct HttpContext {
    body_seq_id: AtomicU32,
    body_map: HashMap<u32, Body>,
    body_stream_map: HashMap<u32, BodyDataStream>,
    body_buffer_map: HashMap<u32, Vec<u8>>,
    body_sender_map: HashMap<u32, Sender>,
    body_sender_closed: HashMap<u32, bool>,
}

impl HttpContext {
    pub fn new() -> Self {
        init_clients();
        Self {
            body_seq_id: AtomicU32::new(1),
            body_map: HashMap::new(),
            body_stream_map: HashMap::new(),
            body_buffer_map: HashMap::new(),
            body_sender_map: HashMap::new(),
            body_sender_closed: HashMap::new(),
        }
    }

    /// get_http_client returns http client based on redirect policy
    pub fn get_http_client(r: RedirectPolicy) -> Client {
        match r {
            RedirectPolicy::Follow => REDIRECT_FOLLOW_POOL.get().unwrap().clone(),
            RedirectPolicy::Error => REDIRECT_ERROR_POOL.get().unwrap().clone(),
            RedirectPolicy::Manual => REDIRECT_MANUAL_POOL.get().unwrap().clone(),
        }
    }

    /// incr_body_id is used to increase body id
    fn incr_body_id(&mut self) -> u32 {
        self.body_seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// new_body creates new empty body and returns handle id
    pub fn new_empty_body(&mut self) -> u32 {
        self.incr_body_id()
    }

    /// take_body takes body by id, it will remove body from map
    pub fn take_body(&mut self, id: u32) -> Option<Body> {
        self.body_map.remove(&id)
    }

    /// set_sender_closed makes the body sender is closed.
    fn set_sender_closed(&mut self, handle: u32) {
        if self.body_sender_map.contains_key(&handle) {
            let sender = self.body_sender_map.remove(&handle).unwrap();
            let _ = sender.finish();
        }
        self.body_sender_closed.insert(handle, true);
    }

    /// set_body sets body by id, it will return handle id
    pub fn set_body(&mut self, id: u32, body: Body) -> u32 {
        let handle = if id < 1 {
            self.body_seq_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        } else {
            id
        };
        self.body_map.insert(handle, body);
        handle
    }

    /// set_readable_stream sets readable stream by id, it will return handle id
    fn set_readable_stream(&mut self, id: u32, stream: BodyDataStream) -> u32 {
        let handle = if id < 1 {
            self.body_seq_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        } else {
            id
        };
        self.body_stream_map.insert(handle, stream);
        handle
    }

    /// new_writable_stream creates new body stream and returns handle id
    pub fn new_writable_stream(&mut self) -> u32 {
        let (sender, body) = new_channel();
        let handle = self.set_body(0, body);
        self.body_sender_map.insert(handle, sender);
        handle
    }

    /// read_body reads body by id
    pub async fn read_body(
        &mut self,
        handle: u32,
        size: u32,
    ) -> Result<(Vec<u8>, bool), BodyError> {
        // use buffer first
        let mut prev_buffer = self.body_buffer_map.remove(&handle).unwrap_or_default();
        if prev_buffer.len() >= size as usize {
            let (first, second) = prev_buffer.split_at(size as usize);
            self.body_buffer_map.insert(handle, second.to_vec());
            return Ok((first.to_vec(), false));
        }

        // if body in body_map, move it to stream to read chunk
        if self.body_map.contains_key(&handle) {
            let body = self.body_map.remove(&handle).unwrap();
            let stream = body.into_data_stream();
            self.set_readable_stream(handle, stream);
        }

        // if body in body_stream_map, read chunk from stream
        if self.body_stream_map.contains_key(&handle) {
            let stream = self.body_stream_map.get_mut(&handle).unwrap();
            loop {
                let chunk = stream.next().await;
                if chunk.is_none() {
                    // no new chunk, return prev buffer if exist
                    if prev_buffer.is_empty() {
                        return Ok((vec![], true));
                    }
                    return Ok((prev_buffer, false));
                }
                let chunk = chunk.unwrap();
                if chunk.is_err() {
                    return Err(BodyError::ReadFailed(format!(
                        "read body chunk failed: {:?}",
                        chunk.err()
                    )));
                }
                prev_buffer.extend_from_slice(&chunk.unwrap());
                if prev_buffer.len() >= size as usize {
                    let (first, second) = prev_buffer.split_at(size as usize);
                    self.body_buffer_map.insert(handle, second.to_vec());
                    return Ok((first.to_vec(), false));
                }
            }
        }

        Err(BodyError::InvalidHandle)
    }

    /// read_body_all reads all body by id
    pub async fn read_body_all(&mut self, handle: u32) -> Result<Vec<u8>, BodyError> {
        // after read all, the body sender should be write closed
        self.set_sender_closed(handle);

        let mut prev_buffer = self.body_buffer_map.remove(&handle).unwrap_or_default();
        loop {
            let (chunk, flag) = self.read_body(handle, usize::MAX as u32).await?;
            // no chunk read, return prev buffer if exist
            if flag {
                if prev_buffer.is_empty() {
                    return Err(BodyError::ReadClosed);
                }
                return Ok(prev_buffer);
            }
            prev_buffer.extend_from_slice(&chunk);
        }
    }

    /// write_body is used to write data to body
    pub async fn write_body(&mut self, handle: u32, data: Vec<u8>) -> Result<u64, BodyError> {
        if self.body_sender_closed.contains_key(&handle) {
            return Err(BodyError::WriteClosed);
        }
        let data_len = data.len() as u64;
        // if sender exist, write data to sender
        if self.body_sender_map.contains_key(&handle) {
            let sender = self.body_sender_map.get_mut(&handle).unwrap();
            sender.write(Bytes::from(data))?;
            return Ok(data_len);
        }

        // if body exist in body map, return ReadOnly error
        if self.body_map.contains_key(&handle) {
            return Err(BodyError::ReadOnly);
        }

        let body = Body::from(data);
        self.set_body(handle, body);
        Ok(data_len)
    }
}

impl Default for HttpContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum FinishMessage {
    Finished,
    _Abort,
}

type BodyReceiver = mpsc::Receiver<Bytes>;
type FinishReceiver = oneshot::Receiver<FinishMessage>;
type FinishSender = oneshot::Sender<FinishMessage>;

pub struct ChannelBodyImpl {
    body_receiver: BodyReceiver,
    finish_receiver: Option<FinishReceiver>,
}

impl http_body::Body for ChannelBodyImpl {
    type Data = Bytes;
    type Error = BodyError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        use tokio::sync::oneshot::error::RecvError;

        match self.as_mut().body_receiver.poll_recv(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(frame)) => Poll::Ready(Some(Ok(Frame::data(frame)))),
            // This means that the `body_sender` end of the channel has been dropped.
            Poll::Ready(None) => {
                if self.finish_receiver.is_none() {
                    return Poll::Ready(None);
                }
                let mut finish_receiver = self.as_mut().finish_receiver.take().unwrap();
                match Pin::new(&mut finish_receiver).poll(cx) {
                    Poll::Pending => {
                        self.as_mut().finish_receiver = Some(finish_receiver);
                        Poll::Pending
                    }
                    Poll::Ready(Err(RecvError { .. })) => Poll::Ready(None),
                    Poll::Ready(Ok(message)) => match message {
                        FinishMessage::Finished => Poll::Ready(None),
                        FinishMessage::_Abort => {
                            Poll::Ready(Some(Err(BodyError::ReadFailed("abort".to_string()))))
                        }
                    },
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Sender {
    pub writer: mpsc::Sender<Bytes>,
    finish_sender: Option<FinishSender>,
}

impl Sender {
    pub fn finish(mut self) -> Result<(), BodyError> {
        drop(self.writer); // drop writer to notify receiver
        let finish_sender = self.finish_sender.take().expect("finish_sender is illgal");
        let _ = finish_sender.send(FinishMessage::Finished);
        Ok(())
    }

    pub fn _abort(mut self) -> Result<(), BodyError> {
        drop(self.writer); // drop writer to notify receiver
        let finish_sender = self.finish_sender.take().expect("finish_sender is illgal");
        let _ = finish_sender.send(FinishMessage::_Abort);
        Ok(())
    }

    pub fn write(&mut self, bytes: Bytes) -> Result<(), BodyError> {
        let res = self.writer.try_send(bytes);
        match res {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(BodyError::WriteFailed("channel full".to_string()))
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(BodyError::WriteFailed("channel closed".to_string()))
            }
        }
    }
}

pub fn new_channel() -> (Sender, Body) {
    let (body_sender, body_receiver) = mpsc::channel(3);
    let (finish_sender, finish_receiver) = oneshot::channel();

    let body_impl = ChannelBodyImpl {
        body_receiver,
        finish_receiver: Some(finish_receiver),
    }
    .boxed();

    let body = Body::new(body_impl);
    let sender = Sender {
        writer: body_sender,
        finish_sender: Some(finish_sender),
    };
    (sender, body)
}
