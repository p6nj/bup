#![deny(missing_docs)]
#![warn(clippy::all)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
use rodio::{cpal::FromSample, OutputStreamHandle, PlayError, Sample, Source};
use std::{
    io,
    marker::PhantomData,
    net::{TcpListener, TcpStream, UdpSocket},
    os::unix::net::{UnixListener, UnixStream},
};

/// A structure, usually a socket, which can accept connexions (blocking) and return something out of it.
/// Impls are available for common socket types.
pub trait Receiver<O, E>
where
    E: Sized,
{
    /// Accept some kind of connection or wait for something to happen.
    fn accept(&self) -> Result<O, E>;
}

/// A structure, usually a socket, which can accept connexions (blocking) and return something out of it.
/// Impls are available for common socket types.
/// (mut version)
#[cfg(feature = "mut")]
pub trait MutReceiver<O, E>
where
    E: Sized,
{
    /// Accept some kind of connection or wait for something to happen.
    fn accept(&mut self) -> Result<O, E>;
}

impl<O, E, F> Receiver<O, E> for F
where
    F: Fn() -> Result<O, E>,
    E: Sized,
{
    fn accept(&self) -> Result<O, E> {
        self()
    }
}

#[cfg(feature = "mut")]
impl<O, E, F> MutReceiver<O, E> for F
where
    F: FnMut() -> Result<O, E>,
    E: Sized,
{
    fn accept(&mut self) -> Result<O, E> {
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

/// Supports playing audio.
pub trait AudioPlayer<S, O, E>
where
    S: Source,
    E: Sized,
    <S as Iterator>::Item: Sample,
{
    /// Play audio from a source.
    fn play(&self, source: S) -> Result<O, E>;
}

impl<S> AudioPlayer<S, (), PlayError> for OutputStreamHandle
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: rodio::Sample,
    f32: FromSample<<S as Iterator>::Item>,
{
    fn play(&self, source: S) -> Result<(), PlayError> {
        self.play_raw(source.convert_samples())
    }
}

/// [`Buzzer`] with a state! Implement this on a struct to save information between beeps.
pub trait StateBuzzer<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Sized,
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
    E: Sized,
{
    /// Called at any incoming connexion / event to generate samples out of its fruits (stateless version).
    fn buzz(&self, incoming: O) -> S;
}

/// Classic Buzzer, able to generate samples out of a Receiver's return value.
/// To keep some information between beeps, use [`StateBuzzer`].
#[cfg(feature = "mut")]
pub trait MutBuzzer<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: MutReceiver<O, E>,
    E: Sized,
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
    E: Sized,
{
    fn buzz(&self, incoming: O) -> S {
        self(incoming)
    }
}

#[cfg(feature = "mut")]
impl<S, R, O, E, F> MutBuzzer<S, R, O, E> for F
where
    F: Fn(O) -> S,
    S: Source,
    <S as Iterator>::Item: Sample,
    R: MutReceiver<O, E>,
    E: Sized,
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
    E: Sized,
{
    fn buzz(&mut self, incoming: O) -> S {
        self(incoming)
    }
}

/// Main struct with a receiver and an audio output handle.
pub struct Bup<'a, S, R, O1, E1, A, O2, E2>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O1, E1>,
    A: AudioPlayer<S, O2, E2>,
    E1: Sized,
    E2: Sized,
{
    /// Socket-like receiver.
    input: R,
    /// Audio output handle.
    output: &'a A,
    /// Phantom data to store types. Do not use.
    #[allow(clippy::type_complexity)]
    _phantom: (
        PhantomData<S>,
        PhantomData<O1>,
        PhantomData<O2>,
        PhantomData<E1>,
        PhantomData<E2>,
    ),
}

/// Main struct with a receiver and an audio output handle.
#[cfg(feature = "mut")]
pub struct MutBup<'a, S, R, O1, E1, A, O2, E2>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: MutReceiver<O1, E1>,
    A: AudioPlayer<S, O2, E2>,
    E1: Sized,
    E2: Sized,
{
    /// Socket-like receiver.
    input: R,
    /// Audio output handle.
    output: &'a A,
    /// Phantom data to store types. Do not use.
    #[allow(clippy::type_complexity)]
    _phantom: (
        PhantomData<S>,
        PhantomData<O1>,
        PhantomData<O2>,
        PhantomData<E1>,
        PhantomData<E2>,
    ),
}

impl<'a, S, R, A, O1, O2, E1, E2> Bup<'a, S, R, O1, E1, A, O2, E2>
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
    f32: FromSample<<S as Iterator>::Item>,
    R: Receiver<O1, E1>,
    A: AudioPlayer<S, O2, E2>,
    E1: Sized,
    E2: Sized,
{
    /// Generate a new BUP with ready-to-go socket-like reciever and audio output handle.
    pub fn new(input: R, output: &'a A) -> Self {
        Self {
            input,
            output,
            _phantom: (
                PhantomData,
                PhantomData,
                PhantomData,
                PhantomData,
                PhantomData,
            ),
        }
    }
    /// Activate with a state. Blocks and loops over incoming connections or events.
    /// Used for structs that implement [`StateBuzzer`] to keep information between beeps.
    pub fn activate_with_state<B: StateBuzzer<S, R, O1, E1>, E: Sized + From<E1> + From<E2>>(
        &self,
        mut buzzer: B,
    ) -> Result<(), E> {
        loop {
            self.output
                .play(buzzer.buzz(self.input.accept().map_err(Into::<E>::into)?))?;
        }
    }
    /// Activate the BUP. Blocks and loops over incoming connections or events.
    pub fn activate<B: Buzzer<S, R, O1, E1>, E: Sized + From<E1> + From<E2>>(
        &self,
        buzzer: B,
    ) -> Result<(), E> {
        loop {
            self.output
                .play(buzzer.buzz(self.input.accept().map_err(Into::<E>::into)?))?;
        }
    }
}

#[cfg(feature = "mut")]
impl<'a, S, R, A, O1, O2, E1, E2> MutBup<'a, S, R, O1, E1, A, O2, E2>
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
    f32: FromSample<<S as Iterator>::Item>,
    R: MutReceiver<O1, E1>,
    A: AudioPlayer<S, O2, E2>,
    E1: Sized,
    E2: Sized,
{
    /// Generate a new BUP with ready-to-go socket-like reciever and audio output handle.
    pub fn new(input: R, output: &'a A) -> Self {
        Self {
            input,
            output,
            _phantom: (
                PhantomData,
                PhantomData,
                PhantomData,
                PhantomData,
                PhantomData,
            ),
        }
    }
    /// Activate the BUP. Blocks and loops over incoming connections or events.
    pub fn activate<B: MutBuzzer<S, R, O1, E1>, E: Sized + From<E1> + From<E2>>(
        &mut self,
        buzzer: B,
    ) -> Result<(), E> {
        loop {
            self.output
                .play(buzzer.buzz(self.input.accept().map_err(Into::<E>::into)?))?;
        }
    }
}
