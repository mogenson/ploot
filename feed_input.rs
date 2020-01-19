use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    sleep(Duration::from_millis(1000)); // wait one second before streaming data
    let start = Instant::now();

    loop {
        let x = PI * start.elapsed().as_secs_f64();
        if let Err(_) = writeln!(stdout(), "{:.3} {:.3}", 2.0 * x.sin(), x.cos()) {
            break;
        }
        sleep(Duration::from_millis(20));
    }
}
