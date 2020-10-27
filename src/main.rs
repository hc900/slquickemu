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
    match a {
        Ok(config) => {

            println!("CPU {}",config.cpu);

        },
        Err(e) => println!("Error loading config"),
    }

}

fn setup_options( config: &str, filetype: &str) -> Result<QuickEmuConfig,u8> {
    let myConfig = load_config_file(config, filetype);
    match myConfig {
        Ok(cfg) => Ok(
            QuickEmuConfig {
                vmname: cfg.vmname.unwrap_or("".to_string()),
                launcher: cfg.launcher.unwrap_or("".to_string()),
                guest_os: cfg.guest_os.unwrap_or("linux".to_string()),
                cpu: cfg.cpu.unwrap_or("host".to_string()),
                ram: cfg.ram.unwrap_or("2G".to_string()),
                cpu_cores: cfg.cpu_cores.unwrap_or("1".to_string()),
                machine: cfg.machine.unwrap_or( "q35".to_string()),
                boot_menu: cfg.boot_menu.unwrap_or(false),
                boot: cfg.boot.unwrap_or("".to_string()),
                iso : cfg.iso .unwrap_or("".to_string()),
                driver_iso: cfg.driver_iso.unwrap_or("".to_string()),
                disk_img: cfg.disk_img.unwrap_or( "".to_string()),
                disk: cfg.disk.unwrap_or( "128G".to_string()),
                floppy: cfg.floppy.unwrap_or("".to_string())
            }


        ),
        Err(e) =>  Err(e) ,
    }

}

fn load_config_file(config: &str, filetype: &str) -> Result<QuickEmuConfigOptions,u8>{
    match filetype {
        "toml" => load_config_from_toml(config),
        "yaml" => Err(8),
        _ => Err(16)
    }
}
