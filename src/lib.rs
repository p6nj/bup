use rodio::{cpal::FromSample, OutputStreamHandle, PlayError, Sample, Source};
use std::{
    io::{self},
    marker::PhantomData,
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

impl<S, F> StateBuzzer<S> for F
where
    F: Fn(UnixStream) -> S,
    S: Source,
    <S as Iterator>::Item: Sample,
{
    fn buzz(&mut self, incoming: UnixStream) -> S {
        self(incoming)
    }
}

pub struct Bup<S: Source>
where
    <S as Iterator>::Item: Sample,
{
    input: UnixListener,
    output: OutputStreamHandle,
    phantom: PhantomData<S>,
}

impl<S> Bup<S>
where
    S: Source + Send + 'static,
    <S as Iterator>::Item: Sample,
    f32: FromSample<<S as Iterator>::Item>,
{
    pub fn new(input: UnixListener, output: OutputStreamHandle) -> Self {
        Self {
            input,
            output,
            phantom: PhantomData,
        }
    }
    pub fn activate_with_state<B: StateBuzzer<S>>(&self, mut buzzer: B) -> Result<(), BupError> {
        loop {
            self.output
                .play_raw(buzzer.buzz(self.input.accept()?.0).convert_samples())?
        }
    }
    pub fn activate<B: Buzzer<S>>(&self, buzzer: B) -> Result<(), BupError> {
        loop {
            self.output
                .play_raw(buzzer.buzz(self.input.accept()?.0).convert_samples())?
        }
    }
}
