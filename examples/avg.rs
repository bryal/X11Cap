extern crate x11cap;
extern crate shuteye;

use x11cap::*;
use shuteye::sleep;
use std::time::Duration;

fn main() {
    let mut capturer = Capturer::new(1920, 0, 1920, 1080).unwrap();

    loop {
        let (ps, (w, h)) = capturer.capture_frame().unwrap();
        let size: u64 = w as u64 * h as u64;

        let (mut tot_r, mut tot_g, mut tot_b) = (0u64, 0u64, 0u64);

        for RGB8 { r, g, b, .. } in ps.into_iter() {
            tot_r += r as u64;
            tot_g += g as u64;
            tot_b += b as u64;
        }

        println!("Avg: {:?}", (tot_r / size, tot_g / size, tot_b / size));

        sleep(Duration::from_millis(80));
    }
}