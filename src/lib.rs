mod sync_token;
use sync_token::*;
use std::thread;
use std::time::Duration;

#[test]
fn test_sync_token() {
    println!("Start");
    let ts = TokenSource::new();
    for x in 0..10 {
        let t = ts.get_token();
        thread::spawn(move || {
            while !t.is_triggered() {
                print!("Work {};", x);
            }
            println!("Done {}", x);
        });
    }
    println!("Trigger");
    ts.trigger();
    thread::sleep(Duration::from_millis(20));
    println!("Done all");
}