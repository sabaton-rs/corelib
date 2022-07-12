pub mod handle_events;

use libc::{self, c_void};
use netlink_sys::{protocols::NETLINK_KOBJECT_UEVENT, Socket, SocketAddr};
use nix::cmsg_space;
use nix::poll::{PollFd, PollFlags};
use nix::sys::stat::{SFlag};
use nix::{
    errno::Errno,
    sys::{
        socket::{MsgFlags, UnixCredentials},
        uio::IoVec,
    },
};
use std::fmt;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path};
use std::{convert::TryFrom, mem::size_of, os::unix::prelude::AsRawFd};
/// Uevent processing utilities
use tracing::{debug, error};
use walkdir::WalkDir;

#[derive(PartialEq, Debug)]
pub enum Action {
    Unknown,
    Add,
    Change,
    Remove,
}

#[derive(Debug)]
pub struct UEvent {
    action: Action,
    dev_path: String,
    maybe_subsystem: Option<String>,
    maybe_firmware: Option<String>,
    maybe_major: Option<u64>,
    maybe_minor: Option<u64>,
    maybe_devname: Option<String>,
    maybe_partitionnum: Option<i32>,
    maybe_partitionname: Option<String>,
    maybe_modalias: Option<String>,
}

impl UEvent {
    pub fn get_devname(&self) -> Option<&str> {
        self.maybe_devname.as_deref()
    }

    pub fn get_partition_name(&self) -> Option<&str> {
        self.maybe_partitionname.as_deref()
    }
    pub fn get_dev_type(&self) -> SFlag {
        if let Some(subsystem) = &self.maybe_subsystem {
            if subsystem == "block" {
                SFlag::S_IFBLK
            } else {
                SFlag::S_IFCHR
            }
        } else {
            SFlag::S_IFCHR
        }
    }

    pub fn is_subsystem(&self, name: &str) -> bool {
        if let Some(subsystem) = &self.maybe_subsystem {
            subsystem == name
        } else {
            false
        }
    }
}

impl fmt::Display for UEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Action:{} Devpath: {} Devname: {} Subsystem: {} Major: {} Minor: {}",
            match self.action {
                Action::Add => String::from("Add"),
                Action::Remove => String::from("Remove"),
                Action::Change => String::from("Change"),
                _ => String::from("Unknown"),
            },
            self.dev_path,
            self.maybe_devname
                .as_ref()
                .unwrap_or(&String::from("Unknown")),
            self.maybe_subsystem
                .as_ref()
                .unwrap_or(&String::from("Unknown")),
            self.maybe_major.unwrap_or(0),
            self.maybe_minor.unwrap_or(0)
        )
    }
}

impl TryFrom<&[u8]> for UEvent {
    type Error = &'static str;
    fn try_from(buf: &[u8]) -> Result<UEvent, Self::Error> {
        // println!("Try from: {} bytes :{:?}", buf.len(), buf);
        let lines = buf.split(|b| *b == 0u8).skip(1);

        let mut uevent = UEvent {
            action: Action::Unknown,
            dev_path: String::new(),
            maybe_firmware: None,
            maybe_subsystem: None,
            maybe_major: None,
            maybe_minor: None,
            maybe_devname: None,
            maybe_partitionnum: None,
            maybe_partitionname: None,
            maybe_modalias: None,
        };

        for line in lines {
            //let tokens: Vec<&[u8]> = line.split(|b| *b == b'=').collect();

            let mut tokens = line.split(|b| *b == b'=');

            let key = tokens.next();
            let value = tokens.next();

            if key.is_none() || value.is_none() || tokens.next().is_some() {
                //println!("Ignoring line with missing or bad content: {:?}:{:?}",key,value);
                // process lines with exactly two elements, ignore everything else
                continue;
            }

            let key = key.unwrap();
            let value = value.unwrap();

            match key {
                b"ACTION" => {
                    uevent.action = match value {
                        b"add" => Action::Add,
                        b"remove" => Action::Remove,
                        b"change" => Action::Change,
                        _ => Action::Unknown,
                    }
                }
                b"DEVPATH" => uevent.dev_path = String::from_utf8_lossy(value).to_string(),
                b"SUBSYSTEM" => {
                    uevent.maybe_subsystem = Some(String::from_utf8_lossy(value).to_string())
                }
                b"MAJOR" => {
                    uevent.maybe_major = String::from_utf8_lossy(value).to_string().parse().ok()
                }
                b"MINOR" => {
                    uevent.maybe_minor = String::from_utf8_lossy(value).to_string().parse().ok()
                }
                b"DEVNAME" => {
                    uevent.maybe_devname = Some(String::from_utf8_lossy(value).to_string())
                }
                b"FIRMWARE" => {
                    uevent.maybe_firmware = Some(String::from_utf8_lossy(value).to_string())
                }
                b"PARTN" => {
                    uevent.maybe_partitionnum = Some(
                        String::from_utf8_lossy(value)
                            .to_string()
                            .parse()
                            .unwrap_or(0),
                    )
                }
                b"PARTNAME" => uevent.maybe_partitionname = Some(sanitize_name(value)),
                b"MODALIAS" => {
                    uevent.maybe_modalias = Some(String::from_utf8_lossy(value).to_string())
                }
                _ => {}
            }
        }

        if uevent.action != Action::Unknown {
            Ok(uevent)
        } else {
            Err("Unable to parse uevent")
        }
    }
}

fn sanitize_name(input: &[u8]) -> String {
    let allowed = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.";
    let mut sanitized = String::with_capacity(65);
    for i in input {
        if allowed.contains(i) {
            sanitized.push(*i as char)
        } else {
            sanitized.push('_')
        }
    }
    sanitized
}
pub enum UEventGenerateAction {
    Stop,
    Continue,
}

const UEVENT_READ_BUFFER_SIZE: usize = 2048 * 5;
pub struct NLSocket(Socket);

pub fn create_and_bind_netlink_socket() -> Result<NLSocket, std::io::Error> {
    let kernel_multicast: SocketAddr = SocketAddr::new(0u32, 0xFFFF_FFFF);

    match Socket::new(NETLINK_KOBJECT_UEVENT) {
        Ok(mut socket) => match socket.bind(&kernel_multicast) {
            Ok(_) => {
                unsafe {
                    let buf_size = UEVENT_READ_BUFFER_SIZE;
                    let pbuf_size = &buf_size as *const usize;
                    let on: i32 = 1;
                    let p_on = &on as *const i32;
                    let ret = libc::setsockopt(
                        socket.as_raw_fd(),
                        libc::SOL_SOCKET,
                        libc::SO_RCVBUFFORCE,
                        pbuf_size as *const c_void,
                        size_of::<usize>() as u32,
                    );
                    if ret != 0 {
                        log::error!("SO_RCVBUFFORCE failed {}", ret);
                    }
                    // Check peer credentials and only allow messages from root (CVE-2012-3520)
                    let ret = libc::setsockopt(
                        socket.as_raw_fd(),
                        libc::SOL_SOCKET,
                        libc::SO_PASSCRED,
                        p_on as *const c_void,
                        size_of::<i32>() as u32,
                    );

                    if ret != 0 {
                        log::error!("SO_PASSCRED failed {}", ret);
                    }
                }
                Ok(NLSocket(socket))
            }
            Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
        },
        Err(e) => Err(std::io::Error::new(ErrorKind::Other, e)),
    }
}

/// This function calls blocking functions.
pub fn read_uevent(socket: &mut Socket) -> Result<UEvent, Error> {
    log::debug!("read_uevent");

    let mut buf = vec![0u8; UEVENT_READ_BUFFER_SIZE];
    let mut msg_space = cmsg_space!(UnixCredentials);
    let iov = [IoVec::from_mut_slice(&mut buf[..])];

    let mut uid = None;
    match nix::sys::socket::recvmsg(
        socket.as_raw_fd(),
        &iov,
        Some(&mut msg_space),
        MsgFlags::empty(),
    ) {
        Ok(msg) => {
            for m in msg.cmsgs() {
                match m {
                    nix::sys::socket::ControlMessageOwned::ScmCredentials(c) => {
                        uid = Some(c.uid());
                        if c.uid() != 0 {
                            return Err(std::io::Error::new(
                                ErrorKind::PermissionDenied,
                                "Blocked message from non root user",
                            ));
                        }
                    }
                    _ => { // skip over the other headers}
                    }
                }
            }
            //
            if uid.is_none() {
                return Err(std::io::Error::new(
                    ErrorKind::PermissionDenied,
                    "Ignoring message without credentials",
                ));
            }

            if let Some(nix::sys::socket::SockAddr::Netlink(add)) = msg.address {
                if add.groups() == 0 || add.pid() != 0 {
                    /* ignoring non-kernel or unicast netlink message */
                    log::debug!("add.groups({})  pid({})", add.groups(), add.pid());
                    return Err(std::io::Error::new(
                        ErrorKind::PermissionDenied,
                        "Ignoring non-kernel or unicast netlink message",
                    ));
                } else {
                    match UEvent::try_from(&iov[0].as_slice()[..msg.bytes]) {
                        Ok(e) => Ok(e),
                        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                    }
                }
            } else {
                Err(std::io::Error::new(
                    ErrorKind::PermissionDenied,
                    "Not a netlink address",
                ))
            }
        }
        Err(e) => {
            if e != nix::Error::Sys(Errno::EAGAIN) && e != nix::Error::Sys(Errno::EINTR) {
                log::error!("reading uevent failed:{}", e);
            } else {
                log::error!("Error reading uvent {}", e);
            }
            Err(std::io::Error::new(ErrorKind::Interrupted, ""))
        }
    }

    //}
}

/// Regenerate Uevents for the give directory. Will
/// recursively go into the directory as long as the
/// callback returns UEventGenerateAction::Continue
pub fn regenerate_uevent_for_dir(
    dir: &Path,
    socket: &mut NLSocket,
    cb: &mut dyn FnMut(&UEvent) -> UEventGenerateAction,
) -> UEventGenerateAction {
    // don't go deeper than 4 elements in the path
    // /sys/class/block/vda4

    if !dir.is_dir() || dir.components().count() > 5 {
        return UEventGenerateAction::Continue;
    }
    //log::debug!("Regen for {}", dir.display());

    let entry_path = dir.join("uevent");

    if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&entry_path) {
        if let Ok(()) = file.write_all("add\n".as_bytes()) {
            drop(file);
            //log::debug!(" Wrote to {} Going to read data", &entry_path.display());

            let mut pollfd = [PollFd::new(socket.0.as_raw_fd(), PollFlags::POLLIN)];

            // drain the socket
            while let Ok(count) = nix::poll::poll(&mut pollfd, 5) {
                if count == 0 {
                    break;
                } else {
                    match read_uevent(&mut socket.0) {
                        Ok(uevent) => match cb(&uevent) {
                            UEventGenerateAction::Stop => return UEventGenerateAction::Stop,
                            UEventGenerateAction::Continue => {}
                        },
                        Err(e) => {
                            log::error!("Error reading uevent:{}", e);
                        }
                    }
                }
            }
        } else {
            log::error!("Cannot write into uevent");
        }
    } else {
        log::debug!("Cannot open {}", &entry_path.display());
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| {
        if let Ok(d) = e {
            if !d.path().is_file()
                && d.path().join("uevent").is_file()
                && d.path().join("dev").is_file()
                && d.path() != dir
            {
                Some(d)
            } else {
                None
            }
        } else {
            None
        }
    }) {
        if let UEventGenerateAction::Stop = regenerate_uevent_for_dir(entry.path(), socket, cb) {
            return UEventGenerateAction::Stop;
        }
    }
    UEventGenerateAction::Continue
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    //use crate::{create_and_bind_netlink_socket, regenerate_uevent_for_dir};

    #[test]
    fn test_regen() {
        let path = PathBuf::from("/sys/block/sda");
        let mut socket = create_and_bind_netlink_socket().unwrap();
        regenerate_uevent_for_dir(&path, &mut socket, &mut |e| {
            println!("Event {:?}", e);
            UEventGenerateAction::Stop
        });
    }
}
