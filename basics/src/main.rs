extern crate rand;

use std::cmp::Ordering;
use std::io;
use rand::Rng;

const VERSION: u32 = 1_000;

fn main() {
    // Get a random number from 0 to 9
    let rand_num = rand::thread_rng().gen_range(0, 10);
    println!("{}", rand_num);

    loop {
        // Read input and write it out
        println!("Requesting user input...");

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {
                println!("{} bytes read", bytes_read);
                println!("Input {}", input);
            }
            Err(error) => {
                println!("{}", error);
            }
        } // Could also use .expect(...) here

        // Perform random number comparison
        let input: u32 = input.trim().parse() // shadowing
            .expect("Failed to parse the input value."); // Could also use matching here

        match input.cmp(&rand_num) {
            Ordering::Less => println!("Small"),
            Ordering::Greater => println!("greater"),
            Ordering::Equal => {
                println!("same");
                break;
            }
        }
    }

    // Test all integral types
    let uchar8 = b'C'; // u8 type
    let mut un8: u8 = 0xFF;
    let si8: i8 = 127;
    let un16: u16 = 0b1111_1111_1111_1111;
    let si16: i16 = 3;
    let un32: u32 = 4;
    let si32: i32 = 5;

    // Mess around with mathematical operations
    let x = 2;
    let y = 4;

    let z = x + y;


    let mut q = z;
    q += 1;
    
    // un8 += 1;  Causes an overflow panic, use wrappings to avoid overflow problems
    let fl64 = 2.0; // 64 bit by default
    let fl32 :f32 = 2.0;
    
    let ttrue = false;
    let tfalse = true;
    let achar = 'z';
    let unicode = '¼';

    println!("{} {} {} {} {} {} {}", un8, si8, un16, si16, un32, si32, uchar8);
    println!("{} {}", fl64, fl32);
    println!("{0} {1} {2}", z, q, VERSION);
    println!("{} {}", achar, unicode);

    let tuple = (ttrue, tfalse, achar, z);
    let (xx, yy, zz, ww) = tuple;
    println!("{} {} {} {}", xx, yy, tuple.0, tuple.3);

    let array = [x, y, x, y, y, x, y, y, x, x];
    println!("{} {} {0}", array[0], array[9]);
}
