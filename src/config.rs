use serde::Deserialize;
use std::path::Path;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use crate::{config, utils};
use crate::utils::find_open_socket;
use directories::BaseDirs;
use std::process::Command;

#[derive(Debug)]
pub enum ERRORCODES {
    OpenConfigFile,
    ReadConfigFile,
    NoSuchFile,
    ScsiControllerMissing,
    UnknownDiskController,
    MissingXdg,
    NoOpenPorts,
    YAML,
    MISC,
}

const DEFAULT_QEMU: &str  = "/snap/bin/qemu-virgil";
const DEFAULT_QEMU_IMG: &str = "/snap/bin/qemu-virgil.qemu-img";
//const DISK_MIN_SIZE: u32 = 197632 * 8;

#[derive(Deserialize)]
pub struct QuickEmuConfigOptions {
    vmname: Option<String>,
    launcher: Option<String>,

    guest_os: Option<String>,

    cpu: Option<String>,
    kvm: Option<bool>,
    //cpu string + tweaks
    ram: Option<String>,
    //ram
    cpu_cores: Option<u8>, //cores

    machine: Option<String>, // default q35

    boot_menu: Option<bool>,
    //display menu or not
    boot: Option<String>, // Legacy or EFI

    iso: Option<String>,
    // PATH
    driver_iso: Option<String>, //PATH

    disk_img: Option<String>,
    disk2_img: Option<String>,
    //path
    disk: Option<String>, //size
    disk2: Option<String>, //size

    floppy: Option<String>, //Path

    //interfaces
    disk_interface: Option<String>,
    scsi_controller: Option<String>,

    display_device: Option<String>,

    audio: Option<String>,
    audio_output: Option<String>,
    //pc_spkr: Option<String>,

    //options
    virgl: Option<bool>,
    gl: Option<bool>,
    output: Option<String>,
    output_extras: Option<String>,
    rtc: Option<bool>,
    spice: Option<bool>,
    //bin paths
    qemu_path: Option<String>,
    qemu_img_path: Option<String>,

}

/*
#[derive(Deserialize)]
struct Tweaks {
    i: i8
}
*/

pub struct QuickEmuConfig {
    pub vmname: String,
    pub launcher: String,

    pub guest_os: String,

    pub cpu: String,
    pub kvm: bool,
    //cpu string + tweaks
    pub ram: String,
    //ram
    pub cpu_cores: u8, //cores

    pub machine: String, // default q35

    pub boot_menu: bool,
    //display menu or not
    pub boot: String, // Legacy or EFI

    pub iso: String,
    // PATH
    pub driver_iso: String, //PATH

    pub disk_img: String,
    pub disk2_img: String,

    //path
    pub disk: String, //size
    pub disk2: String, //size

    pub floppy: String, //Path

    pub disk_interface: String,
    pub scsi_controller: String,

    pub display_device: String,

    pub audio: String,
    pub audio_output: String,
    pub pc_spkr: String,

    //options
    pub virgl: bool,
    pub gl: bool,
    pub output: String,
    pub output_extras: String,
    pub rtc: bool,
    pub spice: bool,
    //bin paths
    pub qemu_path: String,
    pub qemu_img_path: String,

}


fn get_extension_from_file(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

fn slurp_file(filename: &str) -> Result<String, ERRORCODES> {
    let mut file = match File::open(filename) {
        Err(e) => {
            println!("{:?}",e);
            return Err(ERRORCODES::OpenConfigFile)},
        Ok(f) => f,
    };
    let mut contents = String::new();
    let len = match file.read_to_string(&mut contents)
    {
        Err(e) => {
            println!("{:?}",e);
            return Err(ERRORCODES::ReadConfigFile)
        },
        Ok(f) => f,
    };

    if len <= 0 {
        return Err(ERRORCODES::ReadConfigFile);
    }
    Ok(String::from(contents))
}

fn load_config_from_toml(filename: &str) -> Result<QuickEmuConfigOptions, ERRORCODES> {
    let config_string = slurp_file(filename)?;
    //let config_string = r#"cpu = '486'"#;
    let config_q = toml::from_str(&*config_string);
    Ok(config_q.unwrap())
}

pub fn setup_options(config: &str) -> Result<QuickEmuConfig, ERRORCODES> {
    let my_config = load_config_file(config);
    let filename = Path::new(config).file_stem().and_then(OsStr::to_str).unwrap_or("vm");
    match my_config {
        Ok(cfg) => {
            let q = QuickEmuConfig {
                vmname: cfg.vmname.unwrap_or(String::from(filename)),
                launcher: cfg.launcher.unwrap_or("slquickemu".to_string()),
                guest_os: cfg.guest_os.unwrap_or("linux".to_string()),
                kvm: cfg.kvm.unwrap_or(true),
                cpu: cfg.cpu.unwrap_or("-cpu host,kvm=on".to_string()),
                ram: cfg.ram.unwrap_or("auto".to_string()),
                cpu_cores: cfg.cpu_cores.unwrap_or(0u8),
                machine: cfg.machine.unwrap_or("q35".to_string()),
                boot_menu: cfg.boot_menu.unwrap_or(false),
                boot: cfg.boot.unwrap_or("".to_string()),
                iso: cfg.iso.unwrap_or("".to_string()),
                driver_iso: cfg.driver_iso.unwrap_or("".to_string()),
                disk_img: cfg.disk_img.unwrap_or("".to_string()),
                disk: cfg.disk.unwrap_or("128G".to_string()),
                disk2_img: cfg.disk2_img.unwrap_or("".to_string()),
                disk2: cfg.disk2.unwrap_or("128G".to_string()),
                floppy: cfg.floppy.unwrap_or("".to_string()),
                disk_interface: cfg.disk_interface.unwrap_or("none".to_string()),
                scsi_controller: cfg.scsi_controller.unwrap_or("lsi".to_string()),
                display_device: cfg.display_device.unwrap_or("vga".to_string()),
                audio: cfg.audio.unwrap_or("intel-hda".to_string()),
                audio_output: cfg.audio_output.unwrap_or("pa".to_string()),
                pc_spkr: "".to_string(),
                virgl: cfg.virgl.unwrap_or(true),
                gl: cfg.gl.unwrap_or(true),
                rtc: cfg.rtc.unwrap_or(true),
                spice: cfg.spice.unwrap_or(true),
                output: cfg.output.unwrap_or("sdl".to_string()),
                output_extras: cfg.output_extras.unwrap_or("".to_string()),
                qemu_path: cfg.qemu_path.unwrap_or(String::from(DEFAULT_QEMU)),
                qemu_img_path: cfg.qemu_img_path.unwrap_or(String::from(DEFAULT_QEMU_IMG)),
            };
            //left space to do anything I need to correct before passing this out
            Ok(q)
        },
        Err(e) => Err(e),
    }
}

fn load_config_file(config: &str) -> Result<QuickEmuConfigOptions, ERRORCODES> {
    let filetype = get_extension_from_file(&config)
        .unwrap_or("none");
    match filetype {
        "toml" => load_config_from_toml(config),
        "yaml" => Err(ERRORCODES::YAML),
        _ => Err(ERRORCODES::MISC)
    }
}

pub fn build_config(config: &config::QuickEmuConfig) -> Result<Vec<String>,config::ERRORCODES> {
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

    let open_port = find_open_socket(5900)?;
    let spice_port = if config.spice && open_port > 0
    {
        format!("-spice port={},disable-ticketing",open_port)
    } else {
        String::from("")
    };

    //TODO
    //rng
    //serial port
    //extra options

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
    vec.push(spice_port);

    Ok(vec)

}

fn get_xdg_runtime() -> Result<String,config::ERRORCODES>{
    let xdg_dir = BaseDirs::new();
    let l = match xdg_dir {
        Some(x) => {
            x
        },
        None => return Err(config::ERRORCODES::MissingXdg),
    };

    let xdg_runtime_dir = match l.runtime_dir()
    {
        Some(x) => x.to_str(),
        None => return Err(config::ERRORCODES::MissingXdg),
    };

    let acutual_xdg_runtime_dir = match xdg_runtime_dir
    {
        Some(x) => x,
        None => return Err(config::ERRORCODES::MissingXdg),
    };

    Ok(acutual_xdg_runtime_dir.to_string())
}

fn set_cpu_cmd(config: &config::QuickEmuConfig) -> Result<(String,String,String),config::ERRORCODES>
{
    let mut cpu: String;
    cpu = String::from("");
    debug!("Starting cpu as blank {}",cpu);
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

    let mut kvm ;
    kvm = String::from("");
    debug!("kvm is blank {}",kvm);
    if !config.kvm {
        kvm = String::from("");
    } else {
        kvm = String::from("-enable-kvm")
    }
    Ok((cpu,machine,kvm))
}

fn get_output_gl_virgl(config: &config::QuickEmuConfig) -> Result<(String,String,String),config::ERRORCODES>
{
    let mut gl;
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
        let mut temp_oe;
        temp_oe = String::from("");
        debug!("Tempoe is {}",temp_oe);
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

fn set_iso_file(iso: &str) -> Result<String,config::ERRORCODES> {
    if iso.ne("") {
        if Path::new(iso).exists()
        {
            Ok(format!("{}", iso))
        } else {
            error!("MISSING ISO FILE {}", iso);
            Err(config::ERRORCODES::NoSuchFile)
        }
    } else {
        Ok(String::from(""))
    }
}


fn set_drive_cmd(config: &config::QuickEmuConfig, disk_img: String, drive_number: u8) -> Result<String, config::ERRORCODES> {
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
            Err(config::ERRORCODES::ScsiControllerMissing)
        }
    } else {
        let e = format!("DISK CONTROLLER TYPE {} IS UNKNOWN", config.disk_interface);
        error!("{}", e);
        Err(config::ERRORCODES::UnknownDiskController)
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

fn set_floppy(config: &config::QuickEmuConfig) -> Result<String, config::ERRORCODES> {
    if config.floppy.ne("") {
        if Path::new(config.floppy.as_str()).exists() {
            Ok(format!("-fda {}", config.floppy))
        } else {
            error!("File {} does not seem to exist!", config.floppy);
            Err(config::ERRORCODES::NoSuchFile)
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

