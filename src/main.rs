extern crate core;

mod btree;

fn print(a: [i32; 3]) {
    println!("{:?}", a)
}

fn main() {
    let slice = ['r', 'u', 's', 't'];
    let iter = slice.chunks(2);
    println!("Hello, world!");
    println!("{:?}", iter);
    for i in iter {
        print!("{:?}", i)
    }
    let a = [1, 2, 3];
}

