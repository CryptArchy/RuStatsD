# RuStatsD

A reimplementation of etsy's StatsD server in Rust.

## Under Construction

This project is heavily under construction and is full of junk files and commented out code and such.
It is no where even remotely close to usable yet.

## Testing

There are a few unit tests that can be run with `cargo test` and some of the utility code can be tested
in isolation by using `cargo test --lib -- --nocapture`. The `--nocapture` flag is because some tests
use `println!` which won't show up in the console from unit tests without it.

After running with `cargo run` you can test the UDP connection by using another terminal to send
`echo "testing.udp:1|c" | nc -u -w0 127.0.0.1 13265` into the server. The log output should show
the result of the message being processed.

## Pronunciation

Is it "ruh-stats-dee" or "rust-ats-d" or what? Doesn't really matter, call it however you like.
Personally I pronounce it "Roo Stats Drag Race" but that's just me!