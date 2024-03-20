use std::time::Duration;

use bup::{Bup, BupError};
use rodio::{source::SineWave, Source};

fn main() -> Result<(), BupError> {
    Bup::default()
        .activate(|_| SineWave::new(442f32).take_duration(Duration::from_millis(500)))
        .unwrap();
    Ok(())
}
