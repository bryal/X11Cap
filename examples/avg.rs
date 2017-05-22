extern crate x11cap;
extern crate shuteye;

use shuteye::sleep;
use std::time::Duration;
use x11cap::*;


fn main() {
    let mut capturer = Capturer::new(CaptureSource::Monitor(0)).unwrap();

    loop {
        let ps = capturer.capture_frame().unwrap();

        let geo = capturer.get_geometry();
        let size = geo.width as u64 * geo.height as u64;

        let (mut tot_r, mut tot_g, mut tot_b) = (0, 0, 0);

        for &Bgr8 { r, g, b, .. } in ps.as_slice().iter() {
            tot_r += r as u64;
            tot_g += g as u64;
            tot_b += b as u64;
        }

        println!("Avg: {:?}", (tot_r / size, tot_g / size, tot_b / size));

        sleep(Duration::from_millis(80));
    }
}
