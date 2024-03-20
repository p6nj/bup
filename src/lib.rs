use rodio::{
    cpal::FromSample, OutputStream, OutputStreamHandle, PlayError, Sample, Source, StreamError,
};
use std::{
    io::{self},
    marker::PhantomData,
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener, UnixStream},
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BupError {
    #[error("can't bind socket to address")]
    UnixSocket(#[from] io::Error),
    #[error("can't open default output sound device")]
    OutputStream(#[from] StreamError),
    #[error("can't play a sound")]
    Play(#[from] PlayError),
    #[error("socket name is too long")]
    SocketName(io::Error),
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
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
    f32: FromSample<<S as Iterator>::Item>,
{
    pub fn try_with_name<A: AsRef<[u8]>>(addr: A) -> Result<Self, BupError> {
        Ok(Self(
            SocketAddr::from_abstract_name(addr).map_err(|e| BupError::SocketName(e))?,
            PhantomData,
        ))
    }
    fn setup(&self) -> Result<(UnixListener, (OutputStream, OutputStreamHandle)), BupError> {
        Ok((
            UnixListener::bind_addr(&self.0)?,
            OutputStream::try_default()?,
        ))
    }
    pub fn activate_with_state<B: StateBuzzer<S>>(&self, mut buzzer: B) -> Result<(), BupError> {
        let (listener, (_, handle)) = self.setup()?;
        handle.play_raw(buzzer.buzz(listener.accept().unwrap().0).convert_samples())?;
        Ok(())
    }

    pub fn activate<B: Buzzer<S>>(&self, buzzer: B) -> Result<(), BupError> {
        let (listener, (_, handle)) = self.setup()?;
        handle.play_raw(buzzer.buzz(listener.accept().unwrap().0).convert_samples())?;
        Ok(())
    }
}

impl<S> Default for Bup<S>
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
{
    fn default() -> Self {
        Self(SocketAddr::from_abstract_name("bup").unwrap(), PhantomData)
    }
}
