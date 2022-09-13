//!
//!    Written by Graeme Wilde
//!
//!    Requires the big num-bigint, num-traits, and ctrlc crate dependencies
//!    The following should be at the end of the Cargo.toml
//!
//!    [dependencies]
//!        num-bigint = "0.4.3"
//!        num-traits = "0.2.15"
//!        ctrlc = { version = "3.0", features = ["termination"] }
//!


use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicU32};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use std::time::Instant;

use num_bigint::{BigUint, ToBigUint};
use num_traits::ToPrimitive;
use num_traits::Zero;

// The number of threads that should run
const THREAD_COUNT: u32 = 8;

// The number of powers to check in each thread
const BLOCK_COUNT: u32 = 10000;

// What power to start at
const START_POW: u32 = 5;

// How many seconds between updates
const INFO_UPDATE_SECONDS_TIME: u64 = 10;



fn main() {

    // This is a thread safe variable that is used to tell the threads they should shut down.
    // Either when the user cancels the program, or when another thread has found a value
    let exit = Arc::new(AtomicU32::new(u32::MAX));

    // Clone for handler thread
    let exit_h = exit.clone();

    // Handler that will tell the other threads to shutdown if the main thread is cancelled.
    // AKA if you hit ctrl-c
    ctrlc::set_handler(move || {
        println!("Handler exit.");
        exit_h.store(0, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler.");

    // Hold the threads
    let mut children: Vec<JoinHandle<Option<u32>>> = vec![];

    let start_time = Instant::now();

    println!("Starting. Updates will be provided every ~{} seconds from each thread.", INFO_UPDATE_SECONDS_TIME);

    // Start up the first set of threads
    for i in 0..THREAD_COUNT {

        // Clone the shutdown variable for the new thread
        let exit_t = Arc::clone(&exit);
        children.push(
            pow16_worker_thread(
                START_POW + i,
                THREAD_COUNT,
                BLOCK_COUNT,
                start_time,
                start_time,
                exit_t
            )
        );
    }

    // Set the power for the next to start thread
    let mut current_pos = START_POW + BLOCK_COUNT * THREAD_COUNT;

    // Variable used to provide a bit of information every 10 seconds
    let mut last_time = start_time.clone();

    // Variable to store the first found number, initialized as None (basically a nullable type
    // set to Null)
    let mut found_value_option = None;

    // Main loop that checks if a thread has finished, and whether it found a number or should
    // start another. (I have the threads stop after a certain number of iterations and restart so
    // that faster threads do not end up way ahead of slower threads)
    'outer: loop {
        let mut i = 0;
        while i < children.len() {
            if children[i].is_finished() {

                // Remove the finished thread and swap the last one into its place
                let child = children.swap_remove(i);

                // Check the threads return value
                match child.join().unwrap() {

                    // If a matching number was found, set the found value and tell the other
                    // threads to stop if they are past the matching number.
                    Some(pos) => {
                        found_value_option = Some(pos);
                        exit.store(pos, Ordering::Relaxed);
                        println!("Found Matching Num: {}", pos);
                        println!("Waiting to make sure other threads don't have a lower number.");

                        // Exit the loop
                        break 'outer;
                    },
                    // If no matching number was found
                    None => {
                        // And if the program isn't closing
                        if exit.load(Ordering::Relaxed) > current_pos {

                            // Clone the closing variable for the new thread
                            let exit_t = Arc::clone(&exit);

                            // Start a new thread and add its handle to the children variable
                            children.push(
                                pow16_worker_thread(
                                    current_pos,
                                    THREAD_COUNT,
                                    BLOCK_COUNT,
                                    start_time,
                                    last_time,
                                    exit_t
                                )
                            );

                            // Increment the starting position for the next thread
                            current_pos += 1;

                            // Each section has BLOCK_COUNT * THREAD_COUNT powers of 16 to check.
                            // If we have started THREAD_COUNT of them for this section, jump to
                            // the next section
                            if (current_pos - START_POW) % (BLOCK_COUNT * THREAD_COUNT) == THREAD_COUNT {
                                current_pos = current_pos - THREAD_COUNT + BLOCK_COUNT * THREAD_COUNT;
                            }
                        } else {
                            // Exit the loop if the program is shutting down
                            break 'outer;
                        }
                    }
                }
            } else {
                // Variable i only gets incremented if thread is not finished. Otherwise the
                // finished thread will be removed and the last one will takes its place
                i += 1;
            }
        }

        // Update the user every INFO_UPDATE_SECONDS_TIME about how much time has past
        if last_time.elapsed().ge(&Duration::from_secs(INFO_UPDATE_SECONDS_TIME)) {
            last_time = last_time + Duration::from_secs(INFO_UPDATE_SECONDS_TIME);
            let elapsed =
                start_time.elapsed().as_secs() as f64
                    + start_time.elapsed().subsec_millis() as f64 * 1e-3;

            // Small sleep so that the elapsed text is hopefully always last
            sleep(Duration::from_millis(100));
            println!("Elapsed: {}", elapsed);
        }

        // Sleep for a small bit before doing the loop again
        sleep(Duration::from_millis(100));
    }

    // Wait for all threads to finish
    loop {
        let mut i = 0;
        while i < children.len() {
            if children[i].is_finished() {

                // Remove the current thread handle and return it (also swap the last one into its
                // place)
                let child = children.swap_remove(i);

                // Check if the child found a valid number
                match child.join().unwrap() {
                    Some(new_found_value) => {
                        // Check if we already have a valid number
                        match found_value_option {
                            Some(old_found_value) => {
                                // Update the found value if the new power is smaller
                                if new_found_value < old_found_value {
                                    found_value_option = Some(new_found_value);
                                }
                            }
                            // If we didn't already have a number, then use this new one
                            None => {
                                found_value_option = Some(new_found_value);
                            }
                        }
                    }
                    // Thread exited without a valid value
                    None => ()
                }
                // We don't increment i in this loop because we need to check this place again,
                // since we removed the old one and swapped the last one into its place
            } else {
                // Move on to checking the next thread
                i += 1;
            }
        }

        // Once all threads are finished
        if children.len() == 0 {
            break;
        }
    }

    // Let the user know if a matching value was found.
    match found_value_option {
        Some(power) => {
            let num = 16.to_biguint().unwrap().pow(power);
            println!("---------------------------");
            println!("First Num Found: 16^{} = {}", power, num);
        }
        None => println!("No matching value found.")
    }
}

/// Checks if the BigInt has the digits 1, 2, 4, or 8 in it.
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

/// Launch a thread that checks powers of 16, starting from start_pow, incrementing by step_size
/// checking at most block_count powers, to see if they have a 1, 2, 4, or 8 digit in them.
fn pow16_worker_thread(
    start_pow: u32,
    step_size: u32,
    block_count: u32,
    start_time: Instant,
    last_time: Instant,
    exit_t: Arc<AtomicU32>
) -> JoinHandle<Option<u32>> {

    thread::spawn(move || -> Option<u32> {
        // Calculate what the last power to check is.
        let finish = start_pow + (block_count - 1) * step_size;

        // The current power
        let mut pow = start_pow;

        // 16 as a BigInt
        let bui16: BigUint = 16.to_biguint().unwrap();

        // The last time that an update was given to the user
        let mut last = last_time.clone();

        loop {
            // If the thread has been asked to finish. If a number has been found, keep going until
            // we are greater than it. If the program has been cancelled, it will be 0 so we should
            // just stop the thread.
            if exit_t.load(Ordering::Relaxed) <= pow {
                // Update user where we got to.
                let duration = &start_time.elapsed();
                println!(
                    "Thread exit. Got to: {} in: {}",
                    &pow - 1, duration.as_secs() as f64 + duration.subsec_millis() as f64 *1e-4
                );
                break None;
            }

            // Get the current number we are checking
            let num = bui16.pow(pow);

            // Check if it has a 1, 2, 4, or 8
            if !is1248_v3(&num) {
                // If it does update the user and return the power
                println!("Num is 16^{} = {}", pow, num);
                break Some(pow);
            }

            // Every INFO_UPDATE_SECONDS_TIME update the user on what number this thread is at
            if last.elapsed().ge(&Duration::from_secs(INFO_UPDATE_SECONDS_TIME)) {
                last = last + Duration::from_secs(INFO_UPDATE_SECONDS_TIME);
                println!("Checking Num 16^{}", pow);
            }

            // Increment the power we will check next.
            pow += step_size;

            // If it is greater than our finish point, stop.
            if pow > finish {
                break None;
            }
        }
    })
}
