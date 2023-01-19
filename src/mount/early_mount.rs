use crate::ids;
use libc::umask;
use std::ffi::CStr;
use std::ffi::CString;

use crate::c_str;

/// Perform the early mounts of the system. These are the minimum
/// needed mounts to get started. Call this early in the initrd.
pub fn early_mount() -> Vec<String> {
    let mut errors = Vec::new();

    unsafe {
        umask(0);

        let err = libc::mount(
            c_str!("devtmpfs"),
            c_str!("/dev"),
            c_str!("devtmpfs"),
            libc::MS_NOSUID,
            c_str!("mode=0755") as *const libc::c_void,
        );
        if 0 != err {
            errors.push(format!("mount tmpfs failed:{}", err));
        }

        let err = libc::mkdir(c_str!("/dev/pts"), 0o755);
        if 0 != err {
            errors.push(format!("mkdir /dev/pts failed:{}", err));
        }

        let err = libc::mkdir(c_str!("/dev/socket"), 0o755);
        if 0 != err {
            errors.push(format!("mkdir /dev/socket failed:{}", err));
        }

        let err = libc::mkdir(c_str!("/dev/dm-user"), 0o755);
        if 0 != err {
            errors.push(format!("mkdir /dev/dm-user failed:{}", err));
        }

        let err = libc::mount(
            c_str!("devpts"),
            c_str!("/dev/pts"),
            c_str!("devpts"),
            0,
            std::ptr::null(),
        );
        if 0 != err {
            errors.push(format!("mount devpts failed:{}", err));
        }

        let proc_gid = ids::PlatformDacIds::ReadProc as u32;
        let data = CString::new(format!("hidepid=2,gid={}", proc_gid).as_str()).unwrap();

        let err = libc::mount(
            c_str!("proc"),
            c_str!("/proc"),
            c_str!("proc"),
            0,
            data.as_ptr() as *const libc::c_void,
        );
        if 0 != err {
            errors.push(format!(
                "mount proc failed:{} (Is it enabled in the Kernel?) ",
                *libc::__errno_location()
            ));
        }

        let err = libc::chmod(c_str!("/proc/cmdline"), 0o440);
        if 0 != err {
            errors.push(format!("chmod /proc/cmdline failed:{}", err));
        }
        let _cmdline = if let Ok(cmdline) = std::fs::read_to_string("/proc/cmdline") {
            cmdline
        } else {
            errors.push("Failed to read /proc/cmdline".to_owned());
            String::new()
        };

        let err = libc::chmod(c_str!("/proc/bootconfig"), 0o440);
        if 0 != err {
            //enable CONFIG_BOOT_CONFIG if this fails
            errors.push(format!("chmod /proc/bootconfig failed:{}", err));
        }

        let groups = [ids::PlatformDacIds::ReadProc as libc::gid_t];
        let err = libc::setgroups(groups.len(), groups.as_ptr());
        if 0 != err {
            errors.push(format!("setgroups failed:{}", err));
        }

        let err = libc::mount(
            c_str!("sysfs"),
            c_str!("/sys"),
            c_str!("sysfs"),
            0,
            std::ptr::null(),
        );
        if 0 != err {
            errors.push(format!("mount sysfs failed:{}", *libc::__errno_location()));
        }

        let err = libc::mknod(
            c_str!("/dev/kmsg"),
            libc::S_IFCHR | 0o600,
            libc::makedev(1, 11),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/kmsg failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/random"),
            libc::S_IFCHR | 0o666,
            libc::makedev(1, 8),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/random failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/urandom"),
            libc::S_IFCHR | 0o666,
            libc::makedev(1, 9),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/urandom failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/console"),
            libc::S_IFCHR | 0o666,
            libc::makedev(5, 1),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/console failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/ptmx"),
            libc::S_IFCHR | 0o666,
            libc::makedev(5, 2),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/ptmx failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/null"),
            libc::S_IFCHR | 0o666,
            libc::makedev(1, 3),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/null failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/zero"),
            libc::S_IFCHR | 0o666,
            libc::makedev(1, 5),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/zero failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/full"),
            libc::S_IFCHR | 0o666,
            libc::makedev(1, 7),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/full failed:{}", err));
        }

        let err = libc::mknod(
            c_str!("/dev/tty"),
            libc::S_IFCHR | 0o666,
            libc::makedev(5, 0),
        );
        if 0 != err {
            errors.push(format!("mknod /dev/tty failed:{}", err));
        }

        let err = libc::mount(
            c_str!("tmpfs"),
            c_str!("/mnt"),
            c_str!("tmpfs"),
            libc::MS_NOEXEC | libc::MS_NOSUID | libc::MS_NODEV,
            c_str!("mode=0755,uid=0,gid=1000") as *const libc::c_void,
        );
        if 0 != err {
            errors.push(format!("mount tmpfs failed:{}", err));
        }

        let err = libc::mount(
            c_str!("tmpfs"),
            c_str!("/run"),
            c_str!("tmpfs"),
            libc::MS_NOEXEC | libc::MS_NOSUID | libc::MS_NODEV,
            c_str!("mode=0755,uid=0,nodev,nosuid,strictatime") as *const libc::c_void,
        );
        if 0 != err {
            errors.push(format!("mount tmpfs to /run failed:{}", err));
        }

        // Isolated Device Extensions (IDEXs) are mounted in this folder.
        let err = libc::mount(
            c_str!("tmpfs"),
            c_str!("/idex"),
            c_str!("tmpfs"),
            libc::MS_NOSUID,
            c_str!("mode=0755,uid=0,gid=1000") as *const libc::c_void,
        );
        if 0 != err {
            errors.push(format!("mount idex failed:{}", err));
        }

        let err = libc::mkdir(c_str!("/new_root"), 0o755);
        if 0 != err {
            errors.push(format!("mkdir /new_root failed:{}", err));
        }

        let err = libc::mount(
            c_str!("/new_root"),
            c_str!("/new_root"),
            std::ptr::null(),
            libc::MS_BIND,
            std::ptr::null(),
        );
        if 0 != err {
            errors.push(format!(
                "bind mount of /new_root to itself failed:{}",
                *libc::__errno_location()
            ));
        }

        create_dev_mapper_device_entry(&mut errors);
    }
    errors
}

fn create_dev_mapper_device_entry(errors: &mut Vec<String>) {
    let proc_misc = std::fs::read_to_string("/proc/misc").unwrap();
    let mut lines = proc_misc.lines();

    let dev_mapper_minor: Option<u32> = lines.find_map(|e| {
        let mut split_iter = e.trim().split(' ');
        let (minor, device) = (split_iter.next().unwrap(), split_iter.next().unwrap());
        if device == "device-mapper" {
            let minor = minor.parse::<u32>().expect("Cannot parse minor number");
            Some(minor)
        } else {
            None
        }
    });

    unsafe {
        let err = libc::mkdir(c_str!("/dev/mapper"), 0o755);
        if 0 != err {
            errors.push("Error creating /dev/mapper".to_string());
        }
        let err = libc::mknod(
            c_str!("/dev/mapper/control"),
            libc::S_IFCHR | 0o600,
            libc::makedev(10, dev_mapper_minor.unwrap()),
        );
        if 0 != err {
            errors.push("Error creating /dev/mapper/control".to_string());
        }
    }
}

/// Lots of unsafe code used here as we need to operate at a very low level.
/// Idea for this is from the Android init code.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn cleanup_ramdisk(dir: *mut libc::DIR, dev: u64) {
    if !dir.is_null() {
        log::info!("Cleaning up RAMDISK");
        let dfd = unsafe { libc::dirfd(dir) };

        loop {
            let de = unsafe { libc::readdir(dir) };
            if de.is_null() {
                break;
            }
            let de = unsafe { &mut *de };

            let dname = unsafe { CStr::from_ptr(de.d_name.as_ptr()) };
            if dname == unsafe { CStr::from_ptr(c_str!(".")) }
                || dname == unsafe { CStr::from_ptr(c_str!("..")) }
            {
                continue;
            }

            let is_dir = false;

            if de.d_type == libc::DT_DIR || de.d_type == libc::DT_UNKNOWN {
                let mut info: libc::stat = unsafe { std::mem::MaybeUninit::zeroed().assume_init() };
                //let ptr =
                if 0 != unsafe {
                    libc::fstatat(
                        dfd,
                        de.d_name.as_ptr(),
                        &mut info as *mut libc::stat,
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } {
                    continue;
                }

                if info.st_dev != dev {
                    continue;
                }

                // TODO: recurse. Right now, only the /init is freed-up. This is the largest file anyway
            }
            // directory and subdir is cleaned up
            unsafe {
                let _ret = libc::unlinkat(
                    dfd,
                    de.d_name.as_ptr(),
                    if is_dir { libc::AT_REMOVEDIR } else { 0 },
                );
                //println!("deleting : {:?} {}", CStr::from_ptr(de.d_name.as_ptr()), ret);
            }
        }
        unsafe { libc::closedir(dir) };
    }
}
