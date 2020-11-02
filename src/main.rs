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

pub mod qemuconfig;
mod utils;
use env_logger::Env;
extern crate clap;
use clap::{Arg, App }; // SubCommand, Values};
//use std::process::Command;
use crate::qemuconfig::ERRORCODES;
use log::LevelFilter;
extern crate pretty_env_logger;
extern crate env_logger;
#[macro_use] extern crate log;

use env_logger::Target;
use std::env;

#[allow(unused_variables)]
fn main() -> Result<(), ERRORCODES> {
    let matches = App::new("slquickemu")
        .version("0.1")
        .author("HC hc@hackerlan.com")
        .about("Rust implementation of slquickemu by HC\
        \nBased on quickemu by Martin Wimpress\
        \nGPL Version 2\nNo Warranty!\n
        ")
        .arg(Arg::with_name("config")
            .long("vm")
            .value_name("CONFIG")
            .help("Config File to Run")
            .takes_value(true)
            .required(true)
        )
        .arg( Arg::with_name("v")
            .short("v")
            .multiple(true)
            .required(false)
            .help("sets the level of verbosity (RUST_LOG=)")
        )
        .get_matches();
/*
    let r = Command::new("/bin/ls")
        .args(&["-ltra","/"])
        .output()
        .expect("Failed to run LS");
    println!("e {}",r.status);
    println!("stdout: {}", String::from_utf8_lossy(&r.stdout));
    println!("stdout: {}", String::from_utf8_lossy(&r.stderr));
*/
    match matches.occurrences_of("v")
    {
        0 => env::set_var("RUST_LOG","error"),
        1 => env::set_var("RUST_LOG","warn"),
        2 => env::set_var("RUST_LOG","info"),
        3 => env::set_var("RUST_LOG","debug"),
        4 | _ => env::set_var("RUST_LOG", "trace"),
    }

    pretty_env_logger::init_custom_env("RUST_LOG");
    let config = matches.value_of("config").unwrap();
    info!("Using config file: {}",config);


    let quick_emu_config = qemuconfig::setup_options(&config);
    let (cfg,config) =
    match quick_emu_config {
        Ok(config) => {
            match qemuconfig::build_config(&config) {
                Ok(t) => (t,config),
                Err(e) => return Err(e),
            }
        },
        Err(e) => {
            error!("Error loading config");
            return Err(e);
        },
    };

    //let mut arguments = Vec::new();
    print!("qemu-system-i386 ");
    for test in cfg {
      //println!("{} ",test);
        let s: Vec<String> = test.trim().split(' ').map(|t|t.to_string()).collect();
        for f in s {
            print!("{} ", f);
        }
    }
    Ok(())
    //for m in arguments {
    //    print!("{} ",m);
    //}

}

