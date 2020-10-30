use std::fs::File;
use std::io::{BufReader, BufRead};

pub fn get_system_memory() ->  u64 {
    let file = match File::open("/proc/meminfo") {
        Ok(val) => val,
        Err(_) => return 0,
    };

    let reader = BufReader::new(file);

    for line in reader.lines().filter_map(|result |result.ok()) {
        debug!("{:?}",line);
        let mut it = line.split(':');
        let (key, value) = match (it.next(), it.next()) {
            (Some(key), Some(value)) => (key.trim(), value.trim()),
            _ => continue,
        };
        let mut kb = value.split(' ');
        let number = match(kb.next()) {
            Some(number) => match number.trim().parse() {
                Ok(val) => val,
                Err(_) => break,
            },
            _ => continue,
        };
        let size: u64 =  number;
        match key {
            "MemTotal" => return size,
            _ => continue,
        };
    };
    0
}