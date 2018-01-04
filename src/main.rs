extern crate chrono;

use std::io::{self, Read};
use chrono::{Utc, TimeZone, Datelike};
use std::collections::HashMap;


fn main() {

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let mut counters : HashMap<String,u32> = HashMap::new();
    for el in buffer.lines() {
        parse(el, &mut counters);
    }
    for el in &counters {
        println!("{:?}",el);
    }
    let mut counters_keys : Vec<_> = counters.keys().collect();
    counters_keys.sort();
    for el in counters_keys {
        println!("{} {:?}",el,counters.get(el));
    }
}

fn parse(el : &str, counters : &mut HashMap<String,u32>) {
    let mut x = el.split_whitespace();
    let timestamp = x.next();
    let value = x.next();
    if let (Some(timestamp),Some(value)) = (timestamp,value) {
        println!("{} {}", timestamp, value);
        let timestamp = timestamp.parse::<i64>().unwrap();
        let date = Utc.timestamp(timestamp,0);
        let key = format!("{}{:02}",date.year(),date.month());
        println!("{}", key);
        let counter = counters.entry(key).or_insert(1);
        *counter += 1;
    }

}