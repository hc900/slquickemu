use std::fs::File;
use std::io::{BufReader, BufRead};
use std::net::TcpStream;
use crate::qemuconfig;

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
        let number = match kb.next() {
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

fn socket_connect(port:u16) -> std::io::Result<()>
{
    TcpStream::connect(format!("127.0.0.1:{}",port))?;
    Ok(())
}

pub fn find_open_socket(base_port: u16) -> Result<u16, qemuconfig::ERRORCODES>
{
    for i in 1..=5
    {
        let port:u16 = i + base_port;
        debug!("Trying port {}",port);
        let r = socket_connect(port);
        match r {
            Ok(_t) => continue,
            Err(_e) => return Ok(port),
        }
    }
    error!("Exhausted open port search");
    Err(qemuconfig::ERRORCODES::NoOpenPorts)
}