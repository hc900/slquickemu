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

pub mod config;
mod utils;

extern crate clap;
use clap::{Arg, App }; // SubCommand, Values};
use std::process::Command;
use num_cpus::get;
use crate::utils::get_system_memory;
use toml::from_str;
use std::path::Path;
use std::fs::File;

extern crate pretty_env_logger;
#[macro_use] extern crate log;

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
    println!("Using config file: {}",config);

    let a = config::setup_options(&config);
    let mut cfg =
    match a {
        Ok(config) => {
            match build_config(&config) {
                Ok(t) => t,
                Err(e) => Vec::new(),
            }
        },
        Err(e) => {
            error!("Error loading config");
            Vec::new()
        },
    };

    //let mut arguments = Vec::new();

    for test in cfg {
      //println!("{} ",test);
        let mut s: Vec<String> = test.trim().split(' ').map(|t|t.to_string()).collect();
        for f in s {
            print!("{} ", f);
        }
    }

    //for m in arguments {
    //    print!("{} ",m);
    //}

}

fn build_config(config: &config::QuickEmuConfig) -> Result<Vec<String>,&str> {
    let cpu_cores = set_cpu_cores(config);
    let ram = set_ram_value(config);
    let floppy = set_floppy(config)?;
    let boot_menu = set_boot_menu(config);
    let disk_img = handle_disk_image(&config.qemu_img_path
                                     , &config.disk_img, &config.disk);
    let disk2_img = handle_disk_image(&config.qemu_img_path
                                     , &config.disk2_img, &config.disk);
    let mut vec = Vec::new();

    if floppy.ne("") {
        vec.push(format!("-fda \"{}\"",floppy));
    }

    let drive_cmd = set_drive_cmd(config,disk_img,0)?;
    let drive2_cmd = set_drive_cmd(config,disk2_img,1)?;

    vec.push(drive_cmd);
    vec.push(drive2_cmd);
    vec.push(format!("-smp {0},sockets=1,cores={0},threads=1",cpu_cores));
    vec.push( format!("-m {}",ram));
    vec.push( format!("{}",boot_menu));

    Ok(vec)

}

fn set_drive_cmd(config: &config::QuickEmuConfig,disk_img:String, drive:u8) -> Result<String, &str> {
    let mut drive_cmd: String =  format!("-drive if={},id=drive{},cache=directsync,aio=native,format=qcow2,file=\"{}\"",config.disk_interface,drive, disk_img);

    if config.disk_interface.eq("") || config.disk_interface.eq("none")
        {
            Ok(format!("{} -device virtio-blk-pci,drive=drive{},scsi=off",drive_cmd,drive))
        } else if config.disk_interface.contains("scsi") {
            if config.scsi_controller.ne("")
            {
                if drive == 0 {
                    drive_cmd = format!("-device {} {}",config.scsi_controller,drive_cmd);
                }
                Ok(format!("{} -device scsi-hd,drive=drive{}",drive_cmd,drive))
            } else {
                let e = "SCSI CONTROLLER TYPE WAS NOT DEFINED!";
                error!("{}",e);
                Err(e)
            }
        } else {
            Ok(String::from(""))
        }
 //   drive_cmd
}

fn handle_disk_image(qemu_img_path: &str, disk_img: &str, disk_size: &str) -> String {
        if disk_img.ne("") {
            if !Path::new(disk_img).exists() {
                //make disk image
                debug!("{} is imger", qemu_img_path);

                let r = Command::new(qemu_img_path)
                    .args(&["create", "-q", "-f", "qcow2",disk_img,disk_size])
                    .output()
                    .expect("Failed to make disk image");
                debug!("e {}",r.status);
                debug!("stdout: {}", String::from_utf8_lossy(&r.stdout));
                debug!("stdout: {}", String::from_utf8_lossy(&r.stderr))

            } else {
                debug!("Image seems to exist, skipping creation!");
            }
            format!("{}", disk_img)
        } else {
            debug!("Disk Image was not set.");
            format!("")
        }
}

fn set_boot_menu(config: &config::QuickEmuConfig) -> String {
    let boot_menu = if config.boot_menu == true {
        format!("-boot menu=on")
    } else {
        format!("-boot menu=off")
    };
    boot_menu
}

fn set_floppy(config: &config::QuickEmuConfig) -> Result<String, &str> {
    if config.floppy.ne("") {
        if Path::new(config.floppy.as_str()).exists() {
            Ok(format!("-fda {}", config.floppy))
        } else {
            error!("File {} does not seem to exist!", config.floppy);
            Err("File does not exist")
        }
    } else {
        Ok(format!(""))
    }
}

fn set_ram_value(config: &config::QuickEmuConfig) -> String {
    let ram = if config.ram.eq("auto") {
        let m = utils::get_system_memory() / 1_000_000;
        if m >= 64 {
            format!("{}G", 4u8)
        } else if m >= 16 {
            format!("{}G", 3u8)
        } else {
            format!("{}G", 2u8)
        }
    } else {
        format!("{}", config.ram)
    };
    ram
}

fn set_cpu_cores(config: &config::QuickEmuConfig) -> u8 {
    let cpu_cores = if config.cpu_cores == 0 {
        if num_cpus::get_physical() >= 8 {
            4u8
        } else {
            2u8
        }
    } else {
        config.cpu_cores
    };
    cpu_cores
}

