extern crate chrono;
extern crate time;

use std::io::{self, Read};
use chrono::{Utc, TimeZone, Datelike, DateTime};
use std::collections::HashMap;
use time::Duration;


fn main() {

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    let mut counters : HashMap<String,u32> = HashMap::new();
    let mut counters_per_proto : HashMap<String,u32> = HashMap::new();
    let mut counters_per_proto_last : HashMap<String,u32> = HashMap::new();
    let from = Utc::now() - Duration::days(30); // 1 month ago
    for el in buffer.lines() {
        parse(el, &mut counters, &mut counters_per_proto, &mut counters_per_proto_last, from);
    }
    print_map_by_key(&counters);
    print_map_by_value(&counters_per_proto, '2');
    print_map_by_value(&counters_per_proto_last, '3');
}


fn print_map_by_value(map : &HashMap<String,u32>, c : char) {
    let mut count_vec: Vec<(&String, &u32)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    let mut i = 0;
    for (a,b) in count_vec {
        println!("{} {} {}",c , a, b);
        if i>49 {
            break;
        }
        i=i+1;
    }
}

fn print_map_by_key(map : &HashMap<String,u32>) {
    let mut map_keys : Vec<_> = map.keys().collect();
    map_keys.sort();
    for el in map_keys {
        println!("1 {} {}", el, map.get(el).unwrap());
    }
}

fn parse(el : &str,
         counters : &mut HashMap<String,u32>,
         counters_per_proto : &mut HashMap<String,u32>,
         counters_per_proto_last : &mut HashMap<String,u32>,
         from : DateTime<Utc>
        ) {
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
            if date>from {
                *counters_per_proto_last.entry(proto.to_owned()).or_insert(0) += 1;
            }
        }
    }

}