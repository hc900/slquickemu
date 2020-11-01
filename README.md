# slquickemu
A rust program for launching qemu from portable config files.

## Idea
This program is based on the [quickemu](https://github.com/wimpysworld/quickemu) by Martin Wimpress. 
I wanted/needed to add a ton of options to the program.

---

## Opinionated

From Quickemu's Readme:
> `Quickemu is opinionated and will attempt to "do the right thing" rather than expose rich configuration options.`

slquickemu attempts to do this as well. With no defaults or tweak files slquickemu will use internal defaults that match Quickemu.

Even with customization slquickemu will attempt to keep the user from making fatal mistakes that will result in a vm not running properly such as trying to use modern q35 pc architecture with and ISA bus or using SCSI hard disks with unknown SCSI controllers.

### Notable differences
Quickemu will refuse to boot if there is no CDROM or hard disk, slquickemu will allow this to happen as you could be booting from a floppy or net booting.

slquickemu provides for way more customization, such as different sound devices, video cards, an extra hard disk, SCSI, and IDE. 

---

## Config files


By utilizing tweak files the user can change the default behavior of slquickemu. 

slquickemu will attempt to load tweak files from `XDG_CONFIG_HOME`.

Each file should contain at least one definition for settings.

Files are loaded in order based on file names 00-defaults.toml is first 01-linux.conf would be second etc.

Files can be YAML or TOML slquickemu will load both.

Options are progressively overwritten such that 00-defaults.toml would be the base settings and 99-linux.toml would be the last set of options loaded before the actual config file for the vm.

---

## defaults file

You can override all the base defaults (or just have them clearly defined) by using the `00-defaults.toml` file or making your own.

The file must have a section of `[defaults]` in order to work properly.


## tweak files

You can have a base set of overrides by putting tweak files in the `XDG_CONFIG_HOME` directory. 

An example file:

`01-dos.toml`

```
[dos]
cpu = "486"
ram = "16M"
```

You can use these tweaks by setting the `guest_os` option in your vm file to the same name:

`dos-vm.toml`

```
[dos]
disk_img = "/tmp/test.hdd"
disk = "512M"
audio = "sb16"
```

The final settings that would be based on `defaults -> 01-dos.toml -> dos-vm.toml`

---

## Options

The following are options with examples:

| Type   |   |   | Example |
|-------:|---|---|:---|
| String |  |    | "sample"|
| bool | | |true |
| bool | | |false|
| u8   | | |4 |


```
     vmname: String //name of the vm defaults to config file name
     launcher: String
     guest_os: String //defaults to linux
     cpu: String //defaults to 
     kvm: bool // use kvm
     ram: String
     cpu_cores: u8 //cores
     machine: String // default q35
     boot_menu: bool
     boot: String // Legacy or EFI
     iso: String
     driver_iso: String //PATH
     disk_img: String
     disk2_img: String
     disk: String //size "12G"
     disk2: String //size
     floppy: String //Path
     disk_interface: String
     scsi_controller: String
     display_device: String
     audio: String
     audio_output: String
    //options
     virgl: bool
     gl: bool
     output: String
     output_extras: String
     rtc: bool
     spice: bool
    //bin paths
     qemu_path: String
     qemu_img_path: String

```

## TODO

```
[] EFI boot
[] Save State
[] Actual Tweak Files
[] Network cards
[] serial ports
[] samba
[] usb passthrough
[] port forwarding
```