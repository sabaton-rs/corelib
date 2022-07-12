//! Lookup Service Identifiers from service names
//! The service IDs for the platform are pre-defined.

pub type ServiceId = u16;

///Find the id of the given service name.
pub fn id_of_service(name: &str) -> Option<ServiceId> {
    match name {
        "dev.sabaton.ApplicationControl" => Some(1),
        "dev.sabaton.LifecycleControl" => Some(2),
        "dev.sabaton.ApplicationServer" => Some(3),
        _ => None,
    }
}

/// Platform defined service IDs must be below 1024
pub fn is_platform_service_id(id: ServiceId) -> bool {
    id <= 1024
}
