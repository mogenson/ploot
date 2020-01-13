use std::f64::consts::PI;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    sleep(Duration::from_millis(1000));
    let start = Instant::now();
    let mut i: usize = 0;

    loop {
        let x = PI * start.elapsed().as_secs_f64();
        let s = 2.0 * x.sin();
        let c = x.cos();
        if i % 200 == 0 {
            println!("{:.3} {:.3} {:.3}", s, c, 1.333); // 3rd blip every now and then
        } else {
            println!("{:.3} {:.3}", s, c); // just sine and cosine
        }
        i += 1;
        sleep(Duration::from_millis(20));
    }
}
