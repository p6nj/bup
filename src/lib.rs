use rodio::{OutputStream, PlayError, Sample, Source, StreamError};
use std::{
    io::{self, Read, Seek},
    marker::PhantomData,
    os::unix::net::{SocketAddr, UnixListener, UnixStream},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BupError {
    #[error("can't bind socket address")]
    UnixSocket(#[from] io::Error),
    #[error("can't open default output sound device")]
    OutputStream(#[from] StreamError),
    #[error("can't play a sound")]
    Play(#[from] PlayError),
}

pub trait Buzzer {
    fn buzz<S: Source>(&mut self, incoming: UnixStream) -> S
    where
        <S as Iterator>::Item: Sample;
}
pub struct Bup<S: Source>(SocketAddr, PhantomData<S>)
where
    <S as Iterator>::Item: Sample;

impl<S> Bup<S>
where
    S: Source + Read + Seek + Send + Sync + 'static,
    <S as Iterator>::Item: Sample,
{
    pub fn activate<B: Buzzer>(&self, mut buzzer: B) -> Result<(), BupError> {
        let listener = UnixListener::bind_addr(&self.0)?;
        let (_stream, stream_handle) = OutputStream::try_default()?;
        stream_handle.play_once(buzzer.buzz::<S>(listener.accept().unwrap().0))?;
        Ok(())
    }
}
