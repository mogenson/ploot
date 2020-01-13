use std::f64::consts::PI;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    sleep(Duration::from_millis(1000));

    let start = Instant::now();

    loop {
        let x = PI * start.elapsed().as_secs_f64();
        //println!("{}", x.sin());
        println!("{:.3} {:.3}", x.sin(), x.cos());
        sleep(Duration::from_millis(50));
    }
}
