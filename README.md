# 🔘 BUP
> A small beeper / buzzer socket wrapper library

## Introduction
BUP is a wrapper for socket-like structs (Unix, UDP, TCP...) that beeps at every connection using a provided [Source](https://docs.rs/rodio/latest/rodio/source/trait.Source.html) to generate the samples from using the result of the connection (a stream, some bytes...).

## Example
This crate comes with an [example](examples/sine.rs) and some comments.
It creates a BUP with an infinite sinewave source and sends increasing values to it via a Unix socket.

To run it, use cargo :
```sh
cargo run --example sine
```

## Q&A
### Is BUP an acronym?
Almost.
### Rodio says "NoDevice"
You probably dropped your `OutputStream`.
### Is this the future?
Not *yet*.