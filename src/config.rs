use serde::Deserialize;
use std::path::Path;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;

const DEFAULT_QEMU: &str  = "/snap/bin/qemu-virgil";
const DEFAULT_QEMU_IMG: &str = "/snap/bin/qemu-virgil.qemu-img";
const DISK_MIN_SIZE: u32 = 197632 * 8;

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
    pc_spkr: Option<String>,

    //options
    virgl: Option<bool>,
    gl: Option<bool>,
    output: Option<String>,
    output_extras: Option<String>,
    rtc: Option<bool>,
    //bin paths
    qemu_path: Option<String>,
    qemu_img_path: Option<String>,

}

#[derive(Deserialize)]
struct Tweaks {
    i: i8
}

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
    //bin paths
    pub qemu_path: String,
    pub qemu_img_path: String,

}


fn get_extension_from_file(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

fn slurp_file(filename: &str) -> Result<String, u8> {
    let mut file = match File::open(filename) {
        Err(e) => {
            println!("{:?}",e);
            return Err(1)},
        Ok(f) => f,
    };
    let mut contents = String::new();
    let len = match file.read_to_string(&mut contents)
    {
        Err(e) => {
            println!("{:?}",e);
            return Err(2)
        },
        Ok(f) => f,
    };

    /*
    if len < 0 {
        return Err(4);
    }
    */
    Ok(String::from(contents))
}

fn load_config_from_toml(filename: &str) -> Result<QuickEmuConfigOptions, u8> {
    let config_string = slurp_file(filename)?;
    //let config_string = r#"cpu = '486'"#;
    let config_q = toml::from_str(&*config_string);
    Ok(config_q.unwrap())
}

pub fn setup_options(config: &str) -> Result<QuickEmuConfig, u8> {
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

fn load_config_file(config: &str) -> Result<QuickEmuConfigOptions, u8> {
    let filetype = get_extension_from_file(&config)
        .unwrap_or("none");
    match filetype {
        "toml" => load_config_from_toml(config),
        "yaml" => Err(8),
        _ => Err(16)
    }
}
