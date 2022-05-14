use std::env;

use chrono::Duration;
use clap::*;

use simplelog::*;
use chrono::prelude::*;

use sun2000::params::*;
use sun2000::dump::*;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

fn main() {
    env::set_var("RUST_BACKTRACE", "full");
    sun2000::logging::init(None);
    
    info!("🛡️ Welcome to hard (home automation rust-daemon)");

    decode(2022,5,14);
}

fn decode(year: i32, month: u32, day: u32) -> Vec<Parameter> {

    use std::fs::*;
    use std::io::*;

    let path = format!("sun2000_{}_{}_{}.bin", year, month, day);
    let start_of_day: DateTime<chrono::Utc> = DateTime::<Utc>::from_utc(NaiveDateTime::new(NaiveDate::from_ymd(year, month, day), NaiveTime::from_hms_milli(0, 0, 0, 0)), Utc);

    let mut len_header: [u8;2] = [0,0];
    let mut buff: [u8;4096] = [0; 4096];

    let mut res: Vec<Parameter> = vec![];

    if let Ok(mut f) = OpenOptions::new().read(true).open(path) {
        while f.read(&mut len_header).unwrap() > 0 {
            let len: usize = u16::from_le_bytes(len_header) as usize;
            if f.read(&mut buff[0..len]).unwrap() == len {
                let record: DiskRecord = postcard::from_bytes(&buff[0..len]).unwrap();
                
                
                let timestamp = start_of_day +  Duration::milliseconds(record.time_offset_from_day_start_millis as i64);
                

                let mut prev_addr = record.base_address;
                for v in &record.params {
                    prev_addr = prev_addr + v.addr_offset;
                    let p_ = PARAMETER_MAP.get(&prev_addr);
                    match p_ {
                        Some(&p) => {
                            let mut param: Parameter = p.clone();
                            param.value = v.value.clone();

                            println!("{} {:?}", &timestamp, &param);

                            res.push(param);
                            
                        },
                        None => {
                            error!("PARAMETER NOT FOUND {} {:?}", prev_addr, &record);
                        }
                    }
                }
            } else {
                error!("ERROR PARSING, NOT ENOUGH DATA {} {:?}", len, f.stream_position());
            }
        }
    }

    
    return res
}