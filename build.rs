use bootloader::{BootConfig, DiskImageBuilder};
use std::{env, path::PathBuf};

fn main() {
    // set by cargo for the kernel artifact dependency
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();
    let config = {
        let mut c = BootConfig::default();
        c.serial_logging = true;
        c.frame_buffer_logging = true;
        c
    };
    let mut disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));
    disk_builder.set_boot_config(&config);

    // specify output paths
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("jon-uefi.img");
    let bios_path = out_dir.join("jon-bios.img");

    // create the disk images
    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    // pass the disk image paths via environment variables
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}
