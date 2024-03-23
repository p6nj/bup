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

use buplib::{Bup, BupError};
use rodio::{OutputStream, Source};

// source
/// An infinite source that produces a sine.
/// Always has a rate of 48kHz and one channel.
#[derive(Clone, Debug)]
pub struct SineWave {
    freq: f32,
    volume: f32,
    num_sample: usize,
}

// simple generator to make a sine wave with a frequency and a volume
impl SineWave {
    /// Make a new sinewave with a frequency and a volume.
    #[inline]
    pub fn new(freq: f32, volume: u8) -> SineWave {
        SineWave {
            freq,
            volume: volume as f32 / u8::MAX as f32,
            num_sample: 0,
        }
    }
}

// make it iterable over elements that implement the rodio Sample trait (required by the Source trait)
impl Iterator for SineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);
        let value = 2.0 * PI * self.freq * self.num_sample as f32 / 48000.0;
        Some(self.volume * value.sin())
    }
}

// implement Source to make it playable
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

// now the fun part
fn main() -> Result<(), BupError> {
    // open a sound output stream and get a handle
    // ⚠️ we want to keep this OutputStream allocated : dropping it will render the handle unusable which will generate a rodio NoDevice error when playing
    let (_stream, handle) = OutputStream::try_default().unwrap();
    // spawn a thread to activate our buzzer (make it listen for incoming connections) because it will block its thread
    let bup_handle = thread::Builder::new()
        .name("bup".to_string())
        .spawn(move || {
            Bup::new(
                // binding a UnixListener with an abstract name instead of a filename makes things easier
                UnixListener::bind_addr(&SocketAddr::from_abstract_name("bup").unwrap()).unwrap(),
                &handle,
            )
            // now activate it!
            .activate(|(stream, _): (UnixStream, SocketAddr)| {
                SineWave::new(
                    (442f32 / 4f32) * {
                        // we will only send one byte later as a number
                        let mut buf = [0; 1];
                        // read one byte into our little one byte buffer
                        stream.take(1).read_exact(&mut buf).unwrap();
                        *buf.first().unwrap() as f32
                    },
                    5,
                )
                // uncomment the next line to turn this infinite source into a finite one
                // .take_duration(Duration::from_millis(500))
            })
        })
        .unwrap();
    // store our sock address to open multiple connections
    let sock_addr = SocketAddr::from_abstract_name(b"bup").unwrap();
    // this thread will send data to our BUP's unix socket
    thread::Builder::new()
        .name("writer".to_string())
        .spawn(move || {
            (1..=30).for_each(|n| {
                // open a connection
                UnixStream::connect_addr(&sock_addr)
                    .unwrap()
                    // we just write a number in there
                    .write_all(&[n])
                    .unwrap();
                // wait a bit before the next message
                sleep(Duration::from_millis(500));
            });
            // when this part is reached we already sent the numbers 1 to 30 to our BUP, let's stop here
            println!("done!");
        })
        .unwrap();
    // then we join our BUP thread to catch any error it returns
    println!("Joining bup thread. Press Ctrl-C to quit.");
    bup_handle
        .join()
        .expect("bup thread panicked before joining it :(")?;
    Ok(())
}
