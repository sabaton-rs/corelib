#![doc(html_no_source)]
pub mod error;
pub mod instance_specifier;
pub mod context;
pub mod ids;
pub mod api_trait;
pub mod kmsg;
pub mod fstab;
pub mod uevent;
pub mod mount;
pub mod timesync;

mod service_id_lookup;
pub use service_id_lookup::{ServiceId, id_of_service, is_platform_service_id};

#[macro_export]
macro_rules! c_str {
    ($s:expr) => {{
        concat!($s, "\0").as_ptr() as *const std::os::raw::c_char
    }};
}

