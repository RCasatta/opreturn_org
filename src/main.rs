extern crate chrono;

use std::io::{self, Read};
use chrono::{Utc, TimeZone, Datelike};
use std::collections::HashMap;


fn main() {

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let mut counters : HashMap<String,u32> = HashMap::new();
    let mut counters_per_proto : HashMap<String,u32> = HashMap::new();
    for el in buffer.lines() {
        parse(el, &mut counters, &mut counters_per_proto);
    }
    print_map_by_key(&counters);
    print_map_by_value(&counters_per_proto);
}


fn print_map_by_value(map : &HashMap<String,u32>) {
    let mut count_vec: Vec<(&String, &u32)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    let mut i = 0;
    for (a,b) in count_vec {
        println!("{} {}", a, b);
        if i>99 {
            break;
        }
        i=i+1;
    }
}

fn print_map_by_key(map : &HashMap<String,u32>) {
    let mut map_keys : Vec<_> = map.keys().collect();
    map_keys.sort();
    for el in map_keys {
        println!("{} {}", el, map.get(el).unwrap());
    }
    println!("---");
}

fn parse(el : &str, counters : &mut HashMap<String,u32>, counters_per_proto : &mut HashMap<String,u32>) {
    let mut x = el.split_whitespace();
    let timestamp = x.next();
    let value = x.next();
    if let (Some(timestamp),Some(value)) = (timestamp,value) {
        //println!("{} {}", timestamp, value);
        let timestamp = timestamp.parse::<i64>().expect("found non parsable timestamp");
        let date = Utc.timestamp(timestamp,0);
        let key = format!("{}{:02}",date.year(),date.month());
        //println!("{}", key);
        let counter = counters.entry(key).or_insert(0);
        *counter += 1;

        if value.len()>9 {
            let proto=&value[4..10];
            let counter_per_proto = counters_per_proto.entry(proto.to_owned()).or_insert(0);
            *counter_per_proto += 1;
        }
    }

}