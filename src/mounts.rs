use get_sys_info::{Filesystem, Platform, System};

pub fn get_mounts() -> Vec<Filesystem> {
    let sys = System::new();

    let mounts = sys.mounts().unwrap_or_else(|r| {
        println!("get mounts error:{}", r);
        return vec![];
    });

    mounts
        .into_iter()
        .filter(|mount| {
            if cfg!(unix) {
                mount.fs_mounted_from.starts_with("/dev/")
                    && mount.fs_mounted_on != "/esp"
                    && mount.fs_mounted_on != "/efi"
                    && mount.fs_mounted_on != "/boot/efi"
            } else {
                true
            }
        })
        .collect()
}
