use std::{
    f32::consts::PI,
    io::{Read, Write},
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener, UnixStream},
    },
    thread::{self, sleep},
    time::Duration,
};

use bup::{Bup, BupError};
use rodio::{OutputStream, Source};

/// An infinite source that produces a sine.
///
/// Always has a rate of 48kHz and one channel.
#[derive(Clone, Debug)]
pub struct SineWave {
    freq: f32,
    volume: f32,
    num_sample: usize,
}

impl SineWave {
    /// The frequency of the sine.
    #[inline]
    pub fn new(freq: f32, volume: f32) -> SineWave {
        SineWave {
            freq,
            volume,
            num_sample: 0,
        }
    }
}

impl Iterator for SineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);
        let value = 2.0 * PI * self.freq * self.num_sample as f32 / 48000.0;
        Some(self.volume * value.sin())
    }
}

impl Source for SineWave {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        48000
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() -> Result<(), BupError> {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let bup_handle = thread::Builder::new()
        .name("bup".to_string())
        .spawn(|| {
            Bup::new(
                UnixListener::bind_addr(&SocketAddr::from_abstract_name("bup").unwrap()).unwrap(),
                handle,
            )
            .activate(|(stream, _): (UnixStream, SocketAddr)| {
                SineWave::new(
                    (442f32 / 4f32) * {
                        let mut buf = [0; 1];
                        stream.take(1).read_exact(&mut buf).unwrap();
                        *buf.first().unwrap() as f32
                    },
                    0.02,
                )
                // .take_duration(Duration::from_millis(500))
            })
        })
        .unwrap();
    sleep(Duration::from_secs(1));
    let sock = SocketAddr::from_abstract_name(b"bup").unwrap();
    thread::Builder::new()
        .name("writer".to_string())
        .spawn(move || {
            (1..=30).for_each(|n| {
                UnixStream::connect_addr(&sock)
                    .unwrap()
                    .write_all(&[n])
                    .unwrap();
                sleep(Duration::from_millis(500));
            });
            println!("Finished writing.");
        })
        .unwrap();
    println!("Joining bup thread. Press Ctrl-C to quit.");
    bup_handle.join().expect("bup failed before join")?;
    Ok(())
}
