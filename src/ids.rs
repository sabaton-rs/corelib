use crate::error::CoreError;
use std::convert::From;

/// A wrapper for uid_t
pub struct DacId(libc::uid_t);
///
/// Ids used for DAC configuration. These ids are
/// user and group ids defined by the Platform.
/// These ids must not be changed
pub enum PlatformDacIds {
    Root = 0,
    Daemon = 1,
    Bin = 2,
    System = 1000,
    Radio = 1001,
    Bluetooth = 1002,
    Graphics = 1003,
    Input = 1004,
    Audio = 1005,
    Camera = 1006,
    Log = 1007,
    Compass = 1008,
    Mount = 1009,
    Wifi = 1010,
    Debug = 1011,
    Install = 1012,
    Media = 1013,
    Dhcp = 1014,
    RemovableStorageRW = 1015,
    Vpn = 1016,
    Keystore = 1017,
    Usb = 1018,
    Drm = 1019,
    Mdns = 1020,
    Gps = 1021,
    Reserved1 = 1022,
    MediaRW = 1023,
    Mtp = 1024,
    Reserved2 = 1025,
    DrmRpc = 1026,
    Nfc = 1027,
    RemovableStorageR = 1028,
    Clat = 1029,
    Reserved3 = 1030,
    MediaDrm = 1031,
    PackageInfo = 1032,
    Reserved4 = 1033,
    Reserved5 = 1034,
    Reserved6 = 1035,
    LogDaemon = 1036,
    Reserved7 = 1037,
    DBus = 1038,
    TlsDate = 1039,
    Reserved8 = 1040,
    AudioManager = 1041,
    MetricsCollector = 1042,
    Reserved9 = 1043,
    Webserver = 1044,
    Debugger = 1045,
    MediaCodec = 1046,
    Unused = 1047,
    Firewall = 1048,
    Reserved10 = 1049,
    Nvram = 1050,
    Dns = 1051,
    Tether = 1052,
    Reserved11 = 1053,
    VehicleNetwork = 1054,
    MediaAudio = 1055,
    MediaVideo = 1056,
    MediaImage = 1057,
    TombstoneD = 1058,
    Reserved12 = 1059,
    EmbeddedSecureElement = 1060,
    OtaUpdate = 1061,
    EarlyAutomotiveSystem = 1062,
    LoWpan = 1063,
    Hsm = 1064,
    ReservedStorage = 1065,
    StatsD = 1066,
    IncidentD = 1067,
    SecureElement = 1068,
    LowMemoryKillerD = 1069,
    Reserved13 = 1070,
    IoReadahead = 1071,
    Gpu = 1072,
    NetworkStack = 1073,
    Reserved14 = 1074,
    FsVerityCertificate = 1075,
    CredentialStore = 1076,
    ExternalStorage = 1077,
    LifecycleManager = 1078,

    Shell = 2000,
    Cache = 2001,
    Diagnostics = 2002,

    ReadProc = 3009,

    OemReservedStart = 5000,
    OemResedvedEnd = 5999,

    SystemReservedStart = 6000,
    SystemReservedEnd = 6499,

    OdmReservedStart = 7000,
    OdmReservedEnd = 7499,

    Everybody = 9997,
    Misc = 9998,
    Nobody = 9999,

    IsolatedProcessIdStart = 90000,
    IsolatedProcessIdEnd = 99999,

    UserStart = 100000,
    UserEnd = 900000,
}

impl DacId {
    pub fn get(&self) -> libc::uid_t {
        self.0
    }
}

impl From<PlatformDacIds> for DacId {
    fn from(id: PlatformDacIds) -> Self {
        DacId(id as u32)
    }
}

impl PlatformDacIds {
    pub fn get_oem_id(offset: u32) -> Result<DacId, CoreError> {
        let id = PlatformDacIds::OdmReservedStart as u32 + offset;
        if id <= PlatformDacIds::OdmReservedEnd as u32 {
            Ok(DacId(id))
        } else {
            Err(CoreError::InvalidArgument)
        }
    }

    pub fn get_system_id(offset: u32) -> Result<DacId, CoreError> {
        let id = PlatformDacIds::SystemReservedStart as u32 + offset;
        if id <= PlatformDacIds::SystemReservedEnd as u32 {
            Ok(DacId(id))
        } else {
            Err(CoreError::InputOutOfRange)
        }
    }

    pub fn get_odm_id(offset: u32) -> Result<DacId, CoreError> {
        let id = PlatformDacIds::OdmReservedStart as u32 + offset;
        if id <= PlatformDacIds::OdmReservedEnd as u32 {
            Ok(DacId(id))
        } else {
            Err(CoreError::InputOutOfRange)
        }
    }

    pub fn get_isolated_id(offset: u32) -> Result<DacId, CoreError> {
        let id = PlatformDacIds::IsolatedProcessIdStart as u32 + offset;
        if id <= PlatformDacIds::IsolatedProcessIdEnd as u32 {
            Ok(DacId(id))
        } else {
            Err(CoreError::InputOutOfRange)
        }
    }

    pub fn get_user_id(offset: u32) -> Result<DacId, CoreError> {
        let id = PlatformDacIds::UserStart as u32 + offset;
        if id <= PlatformDacIds::UserEnd as u32 {
            Ok(DacId(id))
        } else {
            Err(CoreError::InputOutOfRange)
        }
    }
}
