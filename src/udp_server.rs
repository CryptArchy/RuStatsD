use mio::*;
use mio::udp::*;
use std::io;
use std::net::{SocketAddr, AddrParseError};
use std::result;
use std::str;
use bytes::{Buf, RingBuf, MutBuf};

// Setup some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(0);
const READ_BUFFER_SIZE: usize = 1024;

#[derive(Debug)]
pub enum UdpError {
    Decode(str::Utf8Error),
    Parse(AddrParseError),
    Io(io::Error),
}

impl From<str::Utf8Error> for UdpError {
    fn from(err: str::Utf8Error) -> UdpError {
        UdpError::Decode(err)
    }
}

impl From<AddrParseError> for UdpError {
    fn from(err: AddrParseError) -> UdpError {
        UdpError::Parse(err)
    }
}

impl From<io::Error> for UdpError {
    fn from(err: io::Error) -> UdpError {
        UdpError::Io(err)
    }
}

pub type Result<T> = result::Result<T, UdpError>;

pub struct UdpReader {
    address: SocketAddr,
    socket: UdpSocket,
}

impl UdpReader {
    pub fn new(host: &str, port: &str) -> Result<UdpReader> {
        let connection_string = format!("{}:{}", host, port);
        let address = try!(connection_string.parse::<SocketAddr>());
        let socket = try!(UdpSocket::bind(&address));
        let server = UdpReader {
            address: address,
            socket: socket,
        };
        Ok(server)
    }

    pub fn run(&mut self) {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);
        let mut buf = RingBuf::new(READ_BUFFER_SIZE);

        // Register the stream with `Poll`
        poll.register(&self.socket, SERVER, Ready::all(), PollOpt::edge()).unwrap();
        info!("Registered listener");

        loop {
            trace!("Polling");
            // Wait for the socket to become ready
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                let tk = (event.token(), event.kind());
                trace!("Event: {:?}", &tk);
                match tk {
                    (SERVER, kind) => {
                        if kind.is_readable() {
                            match unsafe { self.socket.recv_from(buf.mut_bytes()) } {
                                Err(e) => {
                                    error!("Error: {:?}", e);
                                    return;
                                }
                                Ok(None) => {
                                    error!("No Result");
                                    return;
                                }
                                Ok(Some((read_size, addr))) => {
                                    unsafe {
                                        MutBuf::advance(&mut buf, read_size);
                                    }
                                    let msg = str::from_utf8(buf.bytes());
                                    info!("Result: {:?} ({:?} on {:?})", msg, read_size, addr);
                                }
                            };
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
