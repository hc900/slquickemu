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

use config;
pub mod qemuconfig;
mod utils;

extern crate clap;
use clap::{Arg, App }; // SubCommand, Values};
//use std::process::Command;
use crate::qemuconfig::ERRORCODES;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

#[allow(unused_variables)]
fn main() -> Result<(), ERRORCODES> {
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
/*
    let r = Command::new("/bin/ls")
        .args(&["-ltra","/"])
        .output()
        .expect("Failed to run LS");
    println!("e {}",r.status);
    println!("stdout: {}", String::from_utf8_lossy(&r.stdout));
    println!("stdout: {}", String::from_utf8_lossy(&r.stderr));
*/
    pretty_env_logger::init();


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

