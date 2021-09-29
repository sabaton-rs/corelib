use std::{ffi::OsStr, io::Read, os::unix::prelude::OsStrExt};

// Kernel logging from /dev/kmsg

// This function does not return. Just read from /proc/kmsg and print to console
// only used for early initrd debugging for now
pub fn log_loop() {
    if let Ok(mut file) = std::fs::File::open("/dev/kmsg") {
        let mut buf = vec![0;2048];
        loop {
            if let Ok(n) = file.read(buf.as_mut_slice()) {
                let msg = &buf[..n];
                print!("{}",OsStr::from_bytes(msg).to_str().unwrap());
            } else {
                println!("Read failed");
            }
        }

    } else {
        println!("Cannot open /dev/kmsg");
    }
}
