#![doc(html_no_source)]
pub mod api_trait;
pub mod bootloader;
pub mod context;
pub mod error;
pub mod fstab;
pub mod ids;
pub mod instance_specifier;
pub mod kmsg;
pub mod mount;
pub mod timesync;
pub mod uevent;

mod service_id_lookup;
pub use service_id_lookup::{id_of_service, is_platform_service_id, ServiceId};

#[macro_export]
macro_rules! c_str {
    ($s:expr) => {{
        concat!($s, "\0").as_ptr() as *const std::os::raw::c_char
    }};
}
