use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicU32};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use std::time::Instant;

use num_bigint::{BigUint, ToBigUint};
use num_traits::ToPrimitive;
use num_traits::Zero;

const THREAD_COUNT: u32 = 8;

fn main() {

    let exit = Arc::new(AtomicU32::new(u32::MAX));
    let exit_h = exit.clone();
    ctrlc::set_handler(move || {
        println!("Handler exit.");
        //exit_h.store(true, Ordering::Relaxed)
        exit_h.store(0, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler.");

    let mut children: Vec<JoinHandle<Option<u32>>> = vec![];

    let start_time = Instant::now();
    let start_pow = 5_u32;

    let block_count = 10000_u32;
    let mut current_pos = start_pow;

    println!("Starting.");


    let mut i = 0;
    while i < THREAD_COUNT {
        let exit_t = Arc::clone(&exit);
        children.push(
            work_thread(current_pos, block_count, start_time, exit_t)
            // thread::spawn(move || -> Option<u32> {
            //     let mut pos = current_pos;
            //     let finish = current_pos + block_count;
            //
            //     let bui16: BigUint = 16.to_biguint().unwrap();
            //
            //     let mut last = start_time.clone();
            //
            //     loop {
            //         if exit_t.load(Ordering::Relaxed) {
            //             let duration = &start_time.elapsed();
            //             println!("Thread exit. Got to: {} in: {}", &pos - 1, duration.as_secs() as f64 + duration.subsec_millis() as f64 *1e-4);
            //             break None;
            //         }
            //
            //         let num = bui16.pow(pos);
            //
            //         // Much faster
            //         if !is1248_v3(&num) {
            //             println!("Num is 16^{} = {}", pos, num);
            //             break Some(pos);
            //         }
            //
            //         if last.elapsed().ge(&Duration::from_secs(10)) {
            //             last = last + Duration::from_secs(10);
            //             println!("Checking Num 16^{}", pos);
            //         }
            //
            //         pos += 1;
            //         if pos >= finish {
            //             break None;
            //         }
            //     }
            // })
        );

        i += 1;
        current_pos += block_count;
    }

    // for i in 0..THREAD_COUNT {
    //     let exit_t = Arc::clone(&exit);
    //     children.push(
    //         thread::spawn(move || -> Option<u32> {
    //
    //             let step = THREAD_COUNT;
    //             let mut pow = i + start_pow;
    //
    //             let bui16: BigUint = 16.to_biguint().unwrap();
    //
    //             let mut last = start_time.clone();
    //
    //             loop {
    //                 if exit_t.load(Ordering::Relaxed) {
    //                     let duration = &start_time.elapsed();
    //                     println!("Thread exit. Got to: {} in: {}", &pow - 1, duration.as_secs() as f64 + duration.subsec_millis() as f64 *1e-4);
    //                     break None;
    //                 }
    //
    //
    //                 let num = bui16.pow(pow);
    //
    //                 // Much faster
    //                 if !is1248_v3(&num) {
    //                     println!("Num is 16^{} = {}", pow, num);
    //                     break Some(pow);
    //                 }
    //
    //                 if last.elapsed().ge(&Duration::from_secs(10)) {
    //                     last = last + Duration::from_secs(10);
    //                     println!("Checking Num 16^{}", pow);
    //                 }
    //
    //                 pow += step;
    //             }
    //         })
    //     );
    // }

    let mut last_time = start_time.clone();

    let mut found_value = None;

    'outer: loop {
        for i in 0..children.len() {
            if children[i].is_finished() {
                //let child: JoinHandle<Option<u32>> = children[i];

                if exit.load(Ordering::Relaxed) > current_pos {
                    let exit_t = Arc::clone(&exit);
                    let child = std::mem::replace(&mut children[i], work_thread(current_pos, block_count, last_time, exit_t));
                    current_pos += block_count;

                    match child.join().unwrap() {
                        Some(pos) => {
                            found_value = Some(pos);
                            exit.store(pos, Ordering::Relaxed);
                            println!("Found Matching Num: {}", pos);
                            break 'outer;
                        },
                        _ => {}
                    }
                } else {
                    break 'outer;
                }
            }
        }
        // for child in children {
        //     if child.is_finished() {
        //         // match child.join().unwrap() {
        //         //     Some(_) => break 'outer,
        //         //     _ => {
        //         //         break 'outer
        //         //     }
        //         // }
        //         break 'outer;
        //     }
        // }

        if last_time.elapsed().ge(&Duration::from_secs(10)) {
            last_time = last_time + Duration::from_secs(10);
            println!("Elapsed: {}", start_time.elapsed().as_secs() as f64 + start_time.elapsed().subsec_millis() as f64 * 1e-3);
        }

        sleep(Duration::from_millis(500))
    }

    for child in children {
        if child.is_finished() {
            match child.join().unwrap() {
                Some(x) => {
                    match found_value {
                        Some(f) => {
                            if x < f {
                                found_value = Some(x);
                            }
                        },
                        None => {
                            found_value = Some(x);
                        }
                    }
                },
                None => println!("Thread exited.")
            }
        }
    }

    match found_value {
        Some(x) => println!("First Num Found: {}", x),
        None => println!("No matching value found.")
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

fn work_thread(current_pos: u32, block_count: u32, start_time: Instant, exit_t: Arc<AtomicU32>) -> JoinHandle<Option<u32>> {
    thread::spawn(move || -> Option<u32> {
        let mut pos = current_pos;
        let finish = current_pos + block_count;

        let bui16: BigUint = 16.to_biguint().unwrap();

        let mut last = start_time.clone();

        loop {
            if exit_t.load(Ordering::Relaxed) <= pos {
                let duration = &start_time.elapsed();
                println!("Thread exit. Got to: {} in: {}", &pos - 1, duration.as_secs() as f64 + duration.subsec_millis() as f64 *1e-4);
                break None;
            }

            let num = bui16.pow(pos);

            // Much faster
            if !is1248_v3(&num) {
                println!("Num is 16^{} = {}", pos, num);
                break Some(pos);
            }

            if last.elapsed().ge(&Duration::from_secs(10)) {
                last = last + Duration::from_secs(10);
                println!("Checking Num 16^{}", pos);
            }

            pos += 1;
            if pos >= finish {
                break None;
            }
        }
    })
}
