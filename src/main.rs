extern crate protobuf;

mod user;

use user::User;
use std::fs::File;
use std::io::BufReader;
use protobuf::{CodedInputStream, Message};

fn main() {
    let file = File::open("./reserved.bin").expect("fail to open file");
    let mut buffered_reader = BufReader::new(file);
    let mut cis = CodedInputStream::from_buffered_reader(&mut buffered_reader);

    let mut u = User::new();
    u.merge_from(&mut cis).expect("fail to merge");

    println!("Name: {}", u.get_name());
    println!("Age: {}", u.get_age());
}