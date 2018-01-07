extern crate chrono;
extern crate time;
extern crate handlebars;
#[macro_use]
extern crate serde_json;

use handlebars::Handlebars;
use std::io::{self, Read};
use chrono::{Utc, TimeZone, Datelike, DateTime};
use std::collections::HashMap;
use time::Duration;
use std::fs::File;

fn main() {

    let mut f = File::open("template.html").expect("template not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("something went wrong reading the template file: 'template.html'");

    let mut buffer = String::new();

    let mut counters : HashMap<String,u32> = HashMap::new();
    let mut counters_per_proto : HashMap<String,u32> = HashMap::new();
    let mut counters_per_proto_last : HashMap<String,u32> = HashMap::new();
    let from = Utc::now() - Duration::days(30); // 1 month ago
    println!("Start");
    loop {
        println!("loop");
        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }

                parse(&buffer, &mut counters, &mut counters_per_proto, &mut counters_per_proto_last, from);
            }
            Err(error) => panic!("error: {}", error),
        }
    }

    let (months, tx_per_month) = print_map_by_key(&counters);
    let (proto, proto_count) = print_map_by_value(&counters_per_proto);
    let (proto_last, proto_last_count) = print_map_by_value(&counters_per_proto_last);

    let reg = Handlebars::new();

    println!(
        "{}",
        reg.template_render(&contents, &json!({
        "months": months,
        "tx_per_month":tx_per_month,
        "proto":proto,
        "proto_count":proto_count,
        "proto_last":proto_last,
        "proto_last_count":proto_last_count,
        })).unwrap()
    );
}


fn print_map_by_value(map : &HashMap<String,u32>) -> (String,String) {
    let mut count_vec: Vec<(&String, &u32)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    let mut name : Vec<String> = vec!();
    let mut value : Vec<u32> = vec!();
    let mut i = 0;
    for (a,b) in count_vec {
        if i>9 {
            break;
        }
        i=i+1;
        name.push(a.to_owned());
        value.push(b.clone());
    }

    (str::replace(&format!("{:?}",name),"\"","'") , format!("{:?}", value) )
}

fn print_map_by_key(map : &HashMap<String,u32>) -> (String,String) {
    let mut map_keys : Vec<_> = map.keys().collect();
    map_keys.sort();
    let mut months : Vec<String> = vec!();
    let mut tx_per_month : Vec<u32> = vec!();
    for el in map_keys {
        let tx_this_month = map.get(el).unwrap();
        //println!("1 {} {}", el, tx_this_month);
        months.push(el.to_owned());
        tx_per_month.push(tx_this_month.clone());
    }
    (str::replace(&format!("{:?}",months),"\"","'"),format!("{:?}",tx_per_month))

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