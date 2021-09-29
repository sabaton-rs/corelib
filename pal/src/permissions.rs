use std::path::Path;


/// Default permissions for the platform.

pub struct FileAttributes {
    pub owner : libc::uid_t,
    pub group : libc::gid_t,
    pub mode : libc::mode_t,
}
pub trait DefaultAttributes {
    /// Get the file attributes for a provided file path. The default implementation
    /// assumes that all devices are owned by root. This routine is used in early startup when
    /// files or directories have to be created.
    fn get_file_attributes(_path:&Path) -> FileAttributes {
        FileAttributes { owner: 0, group: 0, mode: 0o600 }
    }
}

/// The default implementation for this trait.
pub struct DefaultImpl;
impl DefaultAttributes for DefaultImpl{}
