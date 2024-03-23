use rodio::{cpal::FromSample, OutputStreamHandle, PlayError, Sample, Source};
use std::{
    io,
    marker::PhantomData,
    net::{TcpListener, TcpStream, UdpSocket},
    os::unix::net::{UnixListener, UnixStream},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BupError {
    #[error("can't play a sound")]
    Play(#[from] PlayError),
    #[error("can't accept connections")]
    SocketAccept(#[from] io::Error),
}

pub trait Receiver<O, E>
where
    E: Into<BupError>,
{
    fn accept(&self) -> Result<O, E>;
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

pub trait StateBuzzer<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    fn buzz(&mut self, incoming: O) -> S;
}

pub trait Buzzer<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
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

pub struct Bup<S, R, O, E>
where
    S: Source,
    <S as Iterator>::Item: Sample,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    input: R,
    output: OutputStreamHandle,
    _phantom: (PhantomData<S>, PhantomData<O>, PhantomData<E>),
}

impl<S, R, O, E> Bup<S, R, O, E>
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
    f32: FromSample<<S as Iterator>::Item>,
    R: Receiver<O, E>,
    E: Into<BupError>,
{
    pub fn new(input: R, output: OutputStreamHandle) -> Self {
        Self {
            input,
            output,
            _phantom: (PhantomData, PhantomData, PhantomData),
        }
    }
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
