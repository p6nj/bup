#![deny(missing_docs, unused_crate_dependencies)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
use rodio::{cpal::FromSample, OutputStreamHandle, PlayError, Sample, Source};
use std::{
    io,
    marker::PhantomData,
    net::{TcpListener, TcpStream, UdpSocket},
    os::unix::net::{UnixListener, UnixStream},
};
use thiserror::Error;

/// Something bad happened. Supports casting from [`PlayError`] and [`io::Error`].
#[derive(Error, Debug)]
pub enum BupError {
    /// Error encountered while playing samples with rodio.
    #[error("can't play a sound")]
    Play(#[from] PlayError),
    /// Tried the `accept` method of a Socket and failed.
    #[error("can't accept connections")]
    SocketAccept(#[from] io::Error),
}

/// A structure, usually a socket, which can accept connexions (blocking) and return something out of it.
/// Impls are available for common socket types.
pub trait Receiver<O, E>
where
    E: Into<BupError>,
{
    /// Accept some kind of connection or wait for something to happen.
    fn accept(&self) -> Result<O, E>;
}

impl<O, E, F> Receiver<O, E> for F
where
    F: Fn() -> Result<O, E>,
    E: Into<BupError>,
{
    fn accept(&self) -> Result<O, E> {
        self()
    }
}

impl Receiver<(UnixStream, std::os::unix::net::SocketAddr), io::Error> for UnixListener {
    fn accept(&self) -> Result<(UnixStream, std::os::unix::net::SocketAddr), std::io::Error> {
        self.accept()
    }
}

impl Receiver<(TcpStream, std::net::SocketAddr), io::Error> for TcpListener {
    fn accept(&self) -> Result<(TcpStream, std::net::SocketAddr), std::io::Error> {
        self.accept()
    }
}

impl Receiver<[u8; u16::MAX as usize], io::Error> for UdpSocket {
    fn accept(&self) -> Result<[u8; u16::MAX as usize], std::io::Error> {
        let mut buf = [0; u16::MAX as usize];
        self.recv(&mut buf)?;
        Ok(buf)
    }
}

/// [`Buzzer`] with a state! Implement this on a struct to save information between beeps.
pub trait StateBuzzer<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    /// Called at any incoming connexion / event to generate samples out of its fruits (state version).
    fn buzz(&mut self, incoming: O) -> S;
}

/// Classic Buzzer, able to generate samples out of a Receiver's return value.
/// To keep some information between beeps, use [`StateBuzzer`].
pub trait Buzzer<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    /// Called at any incoming connexion / event to generate samples out of its fruits (stateless version).
    fn buzz(&self, incoming: O) -> S;
}

impl<S, R, O, E, F> Buzzer<S, R, O, E> for F
where
    F: Fn(O) -> S,
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    fn buzz(&self, incoming: O) -> S {
        self(incoming)
    }
}

impl<S, R, O, E, F> StateBuzzer<S, R, O, E> for F
where
    F: Fn(O) -> S,
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    fn buzz(&mut self, incoming: O) -> S {
        self(incoming)
    }
}

/// Main struct with a receiver and an audio output handle.
pub struct Bup<'a, S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    /// Socket-like receiver.
    input: R,
    /// Audio output handle.
    output: &'a OutputStreamHandle,
    /// Phantom data to store your generics for impls.
    _phantom: (PhantomData<S>, PhantomData<O>, PhantomData<E>),
}

impl<'a, S, R, O, E> Bup<'a, S, R, O, E>
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
    f32: FromSample<<S as Iterator>::Item>,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    /// Generate a new BUP with ready-to-go socket-like reciever and audio output handle.
    pub fn new(input: R, output: &'a OutputStreamHandle) -> Self {
        Self {
            input,
            output,
            _phantom: (PhantomData, PhantomData, PhantomData),
        }
    }
    /// Activate with a state. Blocks and loops over incoming connections or events.
    /// Used for structs that implement [`StateBuzzer`] to keep information between beeps.
    pub fn activate_with_state<B: StateBuzzer<S, R, O, E>>(
        &self,
        mut buzzer: B,
    ) -> Result<(), BupError> {
        loop {
            self.output.play_raw(
                buzzer
                    .buzz(self.input.accept().map_err(|e| e.into())?)
                    .convert_samples(),
            )?
        }
    }
    /// Activate the BUP. Blocks and loops over incoming connections or events.
    pub fn activate<B: Buzzer<S, R, O, E>>(&self, buzzer: B) -> Result<(), BupError> {
        loop {
            self.output.play_raw(
                buzzer
                    .buzz(self.input.accept().map_err(|e| e.into())?)
                    .convert_samples(),
            )?
        }
    }
}
