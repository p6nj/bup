use rodio::{cpal::FromSample, OutputStream, PlayError, Source, StreamError};
use std::{
    io,
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

pub trait Samples: Source + Send
where
    <Self as Iterator>::Item: rodio::Sample,
{
}

pub trait Buzzer {
    fn buzz<S: Samples>(&mut self, incoming: UnixStream) -> S
    where
        <S as Iterator>::Item: rodio::Sample;
}
pub struct Bup<S>(SocketAddr, PhantomData<S>)
where
    S: Samples,
    <S as Iterator>::Item: rodio::Sample;

impl<S> Bup<S>
where
    f32: FromSample<<S as Iterator>::Item>,
    S: Samples + 'static,
    <S as Iterator>::Item: rodio::Sample,
{
    pub fn activate<B: Buzzer>(&self, mut buzzer: B) -> Result<(), BupError>
    where
        <S as Iterator>::Item: rodio::Sample,
    {
        let listener = UnixListener::bind_addr(&self.0)?;
        let (_stream, stream_handle) = OutputStream::try_default()?;
        Ok(stream_handle.play_raw(
            buzzer
                .buzz::<S>(listener.accept().unwrap().0)
                .convert_samples(),
        )?)
    }
}
