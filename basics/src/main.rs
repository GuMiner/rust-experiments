extern crate rand;
extern crate location;

use std::cmp::Ordering;
use std::io;
use std::fmt;
use std::time::Duration;
use std::thread;
use rand::Rng;

const VERSION: u32 = 1_000;

// Enums are a lot more powerful, they can hold random data.
enum Shapes {
    Triangle,
    Rectangle { width: i32, height: i32 },
    Hexagon,
    Circle(i32), // 'Clearly' radius
}

// Demo matching to implement pretty-printing for shapes
impl fmt::Display for Shapes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_value = match *self {
            Shapes::Triangle => String::from("Triangle"),
            Shapes::Circle(radius) => format!("Circle: {}", radius), // This is really cool
            Shapes::Hexagon => String::from("Hexagon"),
            Shapes::Rectangle {width, height} => format!("Rectangle (w: {}, h: {})", width, height)
        };

        return write!(f, "{}", string_value);
    }
}

fn more_control_flow_tests() {
    let mut shape = Shapes::Triangle;
    println!("{}", shape);
    shape = Shapes::Hexagon;
    println!("{}", shape);
    shape = Shapes::Circle(32);
    println!("{}", shape);
    shape = Shapes::Rectangle { width: 11, height: 12 };
    println!("{}", shape);

    // Option type demos
    let null: Option<i32> = Option::None;
    let not_null = Some(4);
    match null {
        None => println!("{}", "null"),
        Some(i) => println!("{}", i + 20),
    }

    // "if let" syntax
    if let Some(4) = not_null {
    println!("four");
}
}

struct Vector {
    x: i32,
    y: i32
}

impl Vector {
    fn distance_sqd(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    // Associated functions
    fn zero() -> Vector {
        Vector { x: 0, y: 0 }
    }
}

// trait implementation -- method on Vector.
impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}: {})", self.x, self.y, self.x * self.y)
    }
}

struct Vector3(i32, i32, i32); // tuple-struct

// field-init demo
fn build_vec(x: i32, y: i32) -> Vector {
    Vector {
        x, 
        y: y
    }
}

fn encapsulation_tests() {
    let v1 = Vector { x: 1, y: 1};
    let v2 = Vector { x: 2, y: 2};
    println!("{} {} {} {}", v1.x, v1.y, v2.x, v2.y);
    println!("{} {}", v1, v2);
    println!("{}", v1.distance_sqd()); // equivalent to (&v1).distanceSqd();
    println!("{}", Vector::zero());

    let mut v3 = build_vec(2, 4);
    println!("{} {} {0} {0}", v3.x, v3.y);

    v3 = Vector { x:1, .. v2}; // struct update syntax
    println!("{} {} {0} {0}", v3.x, v3.y);

    let v4 = Vector3(1, 2, 3);
    println!("{}", v4.0);
}

fn string_owner(mystr: String) {
    println!("{}", mystr);
}

fn string_borrower(mystr: &String) {
    println!("{}", mystr);
}

fn str_borrower(mystr: &str) {
    println!("{}", mystr);
}

fn string_mut_borrower(mystr: &mut String) {
    mystr.push_str("ZZ");
    println!("{}", mystr);
}

fn string_tests() {
    let child = thread::spawn(move || {
        for number in (1..100).rev() {
            println!("{}", number)
        }

        return String::from("Hello"); // This is created in this thread and returned to the other thread
    });

    for number in (1..100).rev() {
        println!("{}", number)
    }

    let mut string = match child.join() {
        Ok(res) => res,
        Err(_) => String::from("Not Success")
    };

    println!("{}", string);
    
    string.push('Z');
    println!("{}", string);

    let s2 = string.clone();
    println!("{}", s2);

    let mut s3 = s2;

    // s2 is now not valid. Copy types (int, bool, char, float, tuples with those types) are copied instead.
    println!("{}", s3);

    string_borrower(&s3);
    string_mut_borrower(&mut s3);

    // Create a new scope so that string_owner does not die.
    {
        let string_slice = &s3[0..2];
        str_borrower(string_slice);

        // We can also slice arrays, type &[i32] or whatever the array type is
    }


    string_owner(s3);
    
    // s3 is moved and now not valid
}

fn vector_add(a: (i32, i32), b: (i32, i32)) -> (i32, i32) {
    return (a.0 + b.0, a.1 + b.1);
}

fn print_vector(v: (i32, i32)) {
    println!("({}, {})", v.0, v.1);
}

fn more_advanced_tests() {
    let a = (1, 0);
    let b = (0, 1);
    let c = vector_add(a, b);
    print_vector(c);

    if a.0 > 0 {
        println!("greater x");
    } else {
        println!("lesser x");
    }

    let mut max_val =
        if a.0 > a.1 { a.0 } else { a.1 };

    println!("{}", max_val);
    while max_val > 0 {
        println!("{}", max_val);
        max_val -= 1;
    }

    let x = 2;
    let y = 4;
    let array = [x, y, x, y, y, x, y, y, x, x];
    println!("{} {} {0}", array[0], array[9]);
    for element in array.iter() {
        println!("{}", element);
    }

    // std::iter
    for number in (1..100).rev() {
        println!("{}", number)
    }
}

fn main() {
    string_tests();
    encapsulation_tests();
    more_control_flow_tests();
    location::write_data();

    let mut vec = location::create_vec();
    vec.add_item(22);
    vec.remove_item();
    thread::sleep(Duration::from_millis(10000));

    more_advanced_tests();

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
    // let mut un8: u8 = 0xFF;
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
    let fl32: f32 = 2.0;

    let ttrue = false;
    let tfalse = true;
    let achar = 'z';
    let unicode = '¼';

    println!(
        "{} {} {} {} {} {} {}",
        2, si8, un16, si16, un32, si32, uchar8
    );
    println!("{} {}", fl64, fl32);
    println!("{0} {1} {2}", z, q, VERSION);
    println!("{} {}", achar, unicode);

    let tuple = (ttrue, tfalse, achar, z);
    let (xx, yy, _zz, _ww) = tuple;
    println!("{} {} {} {}", xx, yy, tuple.0, tuple.3);

    more_advanced_tests();
}
