extern crate core;

mod btree;

fn main() {
    let slice = ['r', 'u', 's', 't'];
    let iter = slice.chunks(2);
    println!("Hello, world!");
    println!("{:?}", iter);
    for i in iter {
        print!("{:?}", i)
    }
}

