use mio;
use mio::*;
use mio::channel::channel;
use std::io;
use std::result;
use std::str;
use bytes::{Buf, RingBuf, MutBuf};

pub struct LoopHandler {
    token: Token,
    interest: Ready,
}

impl LoopHandler {
    pub fn can_handle(&self, token: Token, interest: Ready) -> bool {
        self.token == token && self.interest.contains(interest)
    }

    pub fn handle(&self, interest: Ready) {

    }
}


pub struct Looper {
    poll: Poll,
}

impl Looper {
    pub fn run(poll: &Poll) {
        let mut events = Events::with_capacity(1024);
        loop {
            if let Err(err) = poll.poll(&mut events, None) {
                error!("poll.poll returned {:?}", err);
                break;
            }

            for event in events.iter() {

            }
        }
    }

    pub fn run(&mut self) {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);
        let mut buf = RingBuf::new(READ_BUFFER_SIZE);
        let (tx, rx) = channel::<String>();

        poll.register(&rx, RX_TOKEN, Ready::all(), PollOpt::edge()).unwrap();
        // Register the stream with `Poll`
        poll.register(&self.socket, INPUT_TOKEN, Ready::all(), PollOpt::edge()).unwrap();
        info!("Registered listener");

        loop {
            trace!("Polling");
            // Wait for the socket to become ready
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                let tk = (event.token(), event.kind());
                trace!("Event: {:?}", &tk);
                match tk {
                    (INPUT_TOKEN, kind) => {
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
                                    let msg = str::from_utf8(buf.bytes()).unwrap();
                                    info!("Result: {:?} ({:?} on {:?})", msg, read_size, addr);
                                    tx.send(msg.to_string());
                                }
                            };
                        }
                    }
                    (RX_TOKEN, kind) => {
                        let msg = rx.try_recv().unwrap();
                        info!("Rx: {:?}", msg);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
