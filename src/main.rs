/*
 *
	Copyright (C) 2020 Emmybear Arcades

	This program is free software; you can redistribute it and/or
	modify it under the terms of the GNU General Public License
	as published by the Free Software Foundation; version 2
	of the License.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

	See the GNU General Public License for more details.

	You should have received a copy of the GNU General Public License
	along with this program; if not, write to:

		Free Software Foundation, Inc.
		59 Temple Place - Suite 330
		Boston, MA  02111-1307, USA

 *
 */

extern crate clap;
use clap::{Arg, App, SubCommand, Values};
use std::path::Path;
use std::ffi::OsStr;
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::io;
use std::process::Command;

#[derive(Deserialize)]
struct Tweaks {
    i : i8
}

#[derive(Deserialize)]
struct QuickEmuConfigOptions {
    vmname: Option<String>,
    launcher: Option<String>,

    guest_os: Option<String>,

    cpu: Option<String>, //cpu string + tweaks
    ram: Option<String>, //ram
    cpu_cores: Option<String>, //cores

    machine: Option<String>, // default q35

    boot_menu: Option<bool>, //display menu or not
    boot: Option<String>, // Legacy or EFI

    iso: Option<String>, // PATH
    driver_iso: Option<String>, //PATH

    disk_img: Option<String>, //path
    disk: Option<String>, //size

    floppy : Option<String>, //Path

    //devices

}

struct QuickEmuConfig {
    vmname: String,
    launcher: String,

    guest_os: String,

    cpu: String, //cpu string + tweaks
    ram: String, //ram
    cpu_cores: String, //cores

    machine: String, // default q35

    boot_menu: bool, //display menu or not
    boot: String, // Legacy or EFI

    iso: String, // PATH
    driver_iso: String, //PATH

    disk_img: String, //path
    disk: String, //size

    floppy : String, //Path

}


fn get_extension_from_file(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

fn slurp_file(filename: &str) -> Result<String, u8> {
    let mut file = match File::open(filename) {
        Err(e) => return Err(1),
        Ok(f) => f,
    };
    let mut contents = String::new();
    let len = match file.read_to_string(&mut contents)
    {
        Err(e) => return Err(2),
        Ok(f) => f,
    };

    if ( len < 0 ) {
        return Err(4);
    }

    Ok(String::from(contents))
}

fn load_config_from_toml(filename: &str) -> Result<QuickEmuConfigOptions,u8> {
    let config_string = slurp_file(filename)?;
    //let config_string = r#"cpu = '486'"#;
    let config_q = toml::from_str(&*config_string);
    Ok(config_q.unwrap())
}

fn test(i: u8) -> Result<i8,&'static str>{
    match i{
        0..=127 => Ok(1),
        _ =>     Err("noob"),
    }
}

fn main() {
    let matches = App::new("slquickemu")
        .version("0.1")
        .author("HC hc@hackerlan.com")
        .about("Rust implementation of slquickemu by HC\nBased on quickemu by Martin Wimpress")
        .arg(Arg::with_name("config")
            .long("vm")
            .value_name("CONFIG")
            .help("Config File to Run")
            .takes_value(true)
            .required(true)
        )
        .get_matches();

    let r = Command::new("/bin/ls")
        .args(&["-ltra","/"])
        .output()
        .expect("Failed to run LS");

    println!("e {}",r.status);
    println!("stdout: {}", String::from_utf8_lossy(&r.stdout));
    println!("stdout: {}", String::from_utf8_lossy(&r.stderr));


    let config = matches.value_of("config").unwrap();
    let filetype= get_extension_from_file(&config)
        .unwrap_or("none");
    println!("Using config file: {}",config);
    println!("File type is {}",filetype);

    let a = setup_options(&config,&filetype);

    println!("MY CONFIG {}",a.cpu)
}

fn setup_options( config: &str, filetype: &str) -> QuickEmuConfig {
    let myConfig = load_config_file(config, filetype).unwrap();

        QuickEmuConfig {
            vmname: "".to_string(),
            launcher: "".to_string(),
            guest_os: "".to_string(),
            cpu: myConfig.cpu.unwrap_or("host".to_string()),
            ram: "".to_string(),
            cpu_cores: "".to_string(),
            machine: "".to_string(),
            boot_menu: false,
            boot: "".to_string(),
            iso: "".to_string(),
            driver_iso: "".to_string(),
            disk_img: "".to_string(),
            disk: "".to_string(),
            floppy: "".to_string()
        }
}

fn load_config_file(config: &str, filetype: &str) -> Result<QuickEmuConfigOptions,u8>{
    match filetype {
        "toml" => load_config_from_toml(config),
        "yaml" => Err(8),
        _ => Err(16)
    }
}
