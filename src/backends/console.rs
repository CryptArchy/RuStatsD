use mio;
use mio::*;
use std::io;
use std::result;
use std::str;

const RX_TOKEN: Token = Token(0);

pub struct ConsoleBackend {
    rx: channel::Receiver<String>,
    pub tx: channel::Sender<String>,
}

pub trait Backend {
    fn run(&mut self, rx: channel::Receiver<String>);
    // fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>;

    // fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> Result<()>;

    // fn deregister(&self, poll: &Poll) -> Result<()>;
}

impl ConsoleBackend {
    pub fn new() -> ConsoleBackend {
        let (tx, rx) = channel::channel::<String>();
        ConsoleBackend { rx: rx, tx: tx, }
    }

    pub fn run(&mut self) {
        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(1024);

        poll.register(&self.rx, RX_TOKEN, Ready::all(), PollOpt::edge()).unwrap();

        loop {
            poll.poll(&mut events, None).unwrap();

            for event in events.iter() {
                let tk = (event.token(), event.kind());
                trace!("Event: {:?}", &tk);
                match tk {
                    (RX_TOKEN, kind) => {
                        let msg = self.rx.try_recv().unwrap();
                        info!("Rx: {:?}", msg);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}

