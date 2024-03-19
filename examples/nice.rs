use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:34254")?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 65527]; // max UDP datagram size according to https://stackoverflow.com/a/77043561
        let (amt, src) = socket.recv_from(&mut buf)?;

        println!("From {} : {:?}", src, &buf[..amt])
    } // the socket is closed here
    Ok(())
}
