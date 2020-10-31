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
use std::path::{Path, PathBuf};
use std::fs::File;
use directories::BaseDirs;
use std::ffi::OsStr;

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
    info!("Using config file: {}",config);

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
    print!("qemu-system-i386 ");
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
    let (cpu,machine, kvm ) = set_cpu_cmd(config)?;
    let cpu_cores = set_cpu_cores(config);
    let ram = set_ram_value(config);
    let floppy = set_floppy(config)?;
    let boot_menu = set_boot_menu(config);
    let disk_img = handle_disk_image(&config.qemu_img_path
                                     , &config.disk_img, &config.disk);
    let disk2_img = handle_disk_image(&config.qemu_img_path
                                      , &config.disk2_img, &config.disk);
    let mut vec = Vec::new();

    let floppy = set_floppy_cmd(floppy);

    let drive_cmd = set_drive_cmd(config, disk_img, 0)?;
    let drive2_cmd = set_drive_cmd(config, disk2_img, 1)?;

    let cdrom = set_iso_file(config.iso.as_str())?;
    let driver_cdrom = set_iso_file(config.driver_iso.as_str())?;
    let cdrom_cmd = set_cdrom_cmd(config, cdrom, 0);
    let cdrom2_cmd = set_cdrom_cmd(config, driver_cdrom, 1);

    let disp = config.display_device.clone();

    let virgl = String::from("on");
    let video_cmd = set_video_cmd(disp, virgl);

    let (gl,output,output_extras) = get_output_gl_virgl(&config)?;

    let rtc = if config.rtc {
        String::from("-rtc base=localtime,clock=host")
    } else {
        String::new()
    };

    let xdg = get_xdg_runtime()?;

    let mut audio_output = format!("-audiodev {0},id={0}",config.audio_output);
    if config.audio_output.eq("pa")
    {
        audio_output += &*format!(",server=unix:{0}/pulse/native,\
                        out.stream-name={1}-{2},\
                        in.stream-name={1}-{2} \
                        -device {3}", xdg, config.launcher, config.vmname, config.audio);
    }

    if config.audio.contains("hda") || config.audio.contains("intel") {
        audio_output += " -device hda-duplex,mixer=off";
    }
        audio_output += &*format!(",audiodev={}",config.audio_output);


    vec.push(format!("-name {0},process={0}",config.vmname));
    vec.push(format!("{} {} -machine {}",kvm,cpu,machine));
    vec.push(format!("-smp {0},sockets=1,cores={0},threads=1",cpu_cores));
    vec.push(format!("-m {}",ram));
    vec.push(format!("{}",boot_menu));
    vec.push(format!("{} -display {},gl={}{}",video_cmd, output,gl,output_extras));
    vec.push(video_cmd);
    vec.push(floppy);
    vec.push(drive_cmd);
    vec.push(drive2_cmd);
    vec.push(cdrom_cmd);
    vec.push(cdrom2_cmd);
    vec.push(rtc);
    vec.push(audio_output);

    Ok(vec)

}

fn get_xdg_runtime<'a>() -> Result<String,&'a str>{
    let mut my_xdg_dir: String = String::new();
    let xdg_dir = BaseDirs::new();
    let l = match xdg_dir {
        Some(x) => {
            x
        },
        None => return Err("Nope"),
    };

    let xdg_runtime_dir = match l.runtime_dir()
    {
        Some(x) => x.to_str(),
        None => return Err("Nope"),
    };

    let acutual_xdg_runtime_dir = match xdg_runtime_dir
    {
        Some(x) => x,
        None => return Err("Nope"),
    };

    Ok(acutual_xdg_runtime_dir.to_string())
}

fn set_cpu_cmd(config: &config::QuickEmuConfig) -> Result<(String,String,String),&str>
{
    let mut cpu = String::new();
    if !config.cpu.starts_with("-cpu")
    {
        cpu = format!("-cpu {}",config.cpu);
    } else {
        cpu = config.cpu.clone();
    }
    let mut machine = config.machine.clone();

    //final things
    if config.display_device.contains("isa") || config.disk_interface.contains("isa")
    {
        machine = String::from("isapc");
    }

    let mut kvm = String::new();
    if !config.kvm {
        kvm = String::from("");
    } else {
        kvm = String::from("-enable-kvm")
    }
    Ok((cpu,machine,kvm))
}

fn get_output_gl_virgl(config: &config::QuickEmuConfig) -> Result<(String,String,String),&str>
{
    let mut gl = String::from("on");
    let mut output_extras = String::new();

    if config.gl
    {
        gl = String::from("on")
    } else {
        gl = String::from("off")
    }

    if config.output.eq("gtk")
    {
        if gl.eq("on") {
            gl = String::from("es");
        }
        output_extras = String::from(",grab-on-hover=on,zoom-to-fit=on");
    }

    if config.output.eq("curses")
    {
        gl = String::from("off");
    }

    output_extras = set_output_extras(config, &output_extras);
    let output = config.output.clone();
    Ok((gl,output,output_extras))
}

fn set_output_extras(config: &config::QuickEmuConfig, output_extras: &String) -> String{
    if config.output_extras.ne("")
    {
        let mut temp_oe = String::from("");
        if config.output_extras.starts_with(',') {
            temp_oe = format!("{}", config.output_extras);
        } else {
            temp_oe = format!(",{}", config.output_extras);
        }
        format!("{}{}", output_extras,temp_oe)
    } else {
        format!("{}", output_extras)
    }
}


fn set_video_cmd(disp: String, virgl: String) -> String {
   if disp.contains("cirrus") {
        if disp.contains("isa") {
            format!("-device isa-cirrus-vga")
        } else {
            format!("-device cirrus-vga")
        }
    } else if disp.contains("bochs") {
        format!("-device bochs-display")
    } else if disp.contains("ati") {
        format!("-device ati-vga")
    } else if disp.contains("vmware") {
        format!("-device vmware-svga")
    } else if disp.contains("qxl") {
        format!("-device qxl-vga")
    } else if disp.contains("virtio") {
        format!("-device virtio-vga,virgl={}", virgl)
    } else if disp.contains("vga") {
        if disp.contains("isa") {
            format!("-device isa-vga")
        } else {
            format!("-device VGA,vgamem_mb=128")
        }
    } else {
        format!("-device VGA,vgamem_mb=128")
    }
}

fn set_floppy_cmd(floppy: String) -> String {
    if floppy.ne("") {
        format!("-fda \"{}\"", floppy)
    } else {
        format!("")
    }
}

fn set_cdrom_cmd(config: &config::QuickEmuConfig, cdrom: String, cdrom_index: u8) -> String {
    let cdrom_cmd: String = if cdrom.ne("") {
        let mut index = cdrom_index;
        if config.disk_interface.contains("ide") {
            if config.disk_img.ne("") {
                index = index + 1;
            }
            if config.disk2_img.ne("") {
                index = index + 1;
            }
        }
        format!("-drive media=cdrom,index={},file=\"{}\"", index, cdrom)
    } else {
        format!("")
    };
    cdrom_cmd
}

fn set_iso_file(iso: &str) -> Result<String,&str> {
    if iso.ne("") {
        if Path::new(iso).exists()
        {
            Ok(format!("{}", iso))
        } else {
            error!("MISSING ISO FILE {}", iso);
            Err("Missing ISO")
        }
    } else {
        Ok(String::from(""))
    }
}


fn set_drive_cmd(config: &config::QuickEmuConfig, disk_img: String, drive_number: u8) -> Result<String, &str> {
    let iface = if config.disk_interface.eq("") ||
        config.disk_interface.eq("none") || config.disk_interface.contains("scsi")
    {
        "none"
    } else
    {
        "ide"
    };
    let mut drive_cmd: String = format!("-drive if={},id=drive{},cache=directsync,\
    aio=native,format=qcow2,file=\"{}\"", iface, drive_number, disk_img);

    if config.disk_interface.eq("") || config.disk_interface.eq("none") || config.disk_interface.contains("ide")
    {
        let res: String = format!("{} -device virtio-blk-pci,drive=drive{},scsi=off", drive_cmd, drive_number);
        Ok(res)
    } else if config.disk_interface.contains("scsi") {
        if config.scsi_controller.ne("")
        {
            if drive_number == 0 {
                drive_cmd = format!("-device {} {}", config.scsi_controller, drive_cmd);
            }
            Ok(format!("{} -device scsi-hd,drive=drive{}", drive_cmd, drive_number))
        } else {
            let e = "SCSI CONTROLLER TYPE WAS NOT DEFINED!";
            error!("{}", e);
            Err(e)
        }
    } else {
        let e = format!("DISK CONTROLLER TYPE {} IS UNKNOWN", config.disk_interface);
        error!("{}", e);
        Err("BUMMER")
    }
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

