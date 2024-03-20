use rodio::{OutputStream, OutputStreamHandle, PlayError, Sample, Source, StreamError};
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

pub trait StateBuzzer<S>
where
    S: Source,
    <S as Iterator>::Item: Sample,
{
    fn buzz(&mut self, incoming: UnixStream) -> S;
}

pub trait Buzzer<S>
where
    S: Source,
    <S as Iterator>::Item: Sample,
{
    fn buzz(&self, incoming: UnixStream) -> S;
}

impl<S, F> Buzzer<S> for F
where
    F: Fn(UnixStream) -> S,
    S: Source,
    <S as Iterator>::Item: Sample,
{
    fn buzz(&self, incoming: UnixStream) -> S {
        self(incoming)
    }
}

pub struct Bup<S: Source>(SocketAddr, PhantomData<S>)
where
    <S as Iterator>::Item: Sample;

impl<S> Bup<S>
where
    S: Source + Read + Seek + Send + Sync + 'static,
    <S as Iterator>::Item: Sample,
{
    fn setup(&self) -> Result<(UnixListener, (OutputStream, OutputStreamHandle)), BupError> {
        Ok((
            UnixListener::bind_addr(&self.0)?,
            OutputStream::try_default()?,
        ))
    }
    pub fn activate_with_state<B: StateBuzzer<S>>(&self, mut buzzer: B) -> Result<(), BupError> {
        let (listener, (_, handle)) = self.setup()?;
        handle.play_once(buzzer.buzz(listener.accept().unwrap().0))?;
        Ok(())
    }

    pub fn activate<B: Buzzer<S>>(&self, buzzer: B) -> Result<(), BupError> {
        let (listener, (_, handle)) = self.setup()?;
        handle.play_once(buzzer.buzz(listener.accept().unwrap().0))?;
        Ok(())
    }
}
