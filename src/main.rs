use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use std::time::Instant;

use num_bigint::{BigUint, ToBigUint};
use num_traits::ToPrimitive;
use num_traits::Zero;

const THREAD_COUNT: u32 = 8;

fn main() {

    let exit = Arc::new(AtomicBool::new(false));
    let exit_h = exit.clone();
    ctrlc::set_handler(move || {
        println!("Handler exit.");
        exit_h.store(true, Ordering::Relaxed)
    }).expect("Error setting Ctrl-C handler.");

    let mut children: Vec<JoinHandle<Option<u32>>> = vec![];

    let start_time = Instant::now();
    let start_pow = 5;

    println!("Starting.");

    for i in 0..THREAD_COUNT {
        let exit_t = Arc::clone(&exit);
        children.push(
            thread::spawn(move || -> Option<u32> {
                let step = THREAD_COUNT;
                let mut pow = i + start_pow;

                let bui16: BigUint = 16.to_biguint().unwrap();

                let mut last = start_time.clone();

                loop {
                    if exit_t.load(Ordering::Relaxed) {
                        let duration = &start_time.elapsed();
                        println!("Thread exit. Got to: {} in: {}", &pow - 1, duration.as_secs() as f64 + duration.subsec_millis() as f64 *1e-4);
                        break None;
                    }


                    let num = bui16.pow(pow);

                    // Much faster
                    if !is1248_v3(&num) {
                        println!("Num is 16^{} = {}", pow, num);
                        break Some(pow);
                    }

                    if last.elapsed().ge(&Duration::from_secs(10)) {
                        last = last + Duration::from_secs(10);
                        println!("Checking Num 16^{}", pow);
                    }

                    pow += step;
                }
            })
        );
    }

    let mut last = start_time.clone();

    'outer: loop {
        for child in &children {
            if child.is_finished() {
                break 'outer;
            }
        }

        if last.elapsed().ge(&Duration::from_secs(10)) {
            last = last + Duration::from_secs(10);
            println!("Elapsed: {}", start_time.elapsed().as_secs() as f64 + start_time.elapsed().subsec_millis() as f64 * 1e-3);
        }

        sleep(Duration::from_millis(500))
    }

    for child in children {
        if child.is_finished() {
            match child.join().unwrap() {
                Some(x) => println!("First Num Found: {}", x),
                None => println!("Thread exited.")
            }
        }
    }
}

fn is1248_v3(i: &BigUint) -> bool {
    let i10 = 10.to_biguint().unwrap();

    let mut digit = i % &i10;
    let mut num: BigUint = (i - &digit) / &i10;

    loop {

        match digit.to_u8().unwrap() {
            1 | 2 | 4 | 8 => { break true },
            _ => {}
        }

        if num <= Zero::zero() {
            break false;
        }

        digit = &num % &i10;
        num = (&num - &digit) / &i10;
    }

}
