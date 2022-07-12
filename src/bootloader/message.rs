/*
  This code is converted from the Android code at 
  https://android.googlesource.com/platform/bootable/recovery/+/refs/tags/android-10.0.0_r25/bootloader_message/include/bootloader_message/bootloader_message.h
  hence retaining the License header from Android.

  Uboot implements the Android A/B bootloader message so reusing the layout of the data
  allows us to use the implementation from UBoot.

*/

/*
 * Copyright (C) 2008 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fmt::Display;

use super::error::BootloaderMessageError;
use c2rust_bitfields::BitfieldStruct;
use crc::{Crc, Algorithm, CRC_32_ISCSI, CRC_32_AIXM, CRC_32_AUTOSAR, CRC_32_BASE91_D, CRC_32_BZIP2, CRC_32_CD_ROM_EDC, CRC_32_CKSUM, CRC_32_ISO_HDLC};

/// Spaces used by misc partition are as below:
/// 0   - 2K     For bootloader_message
/// 2K  - 16K    Used by Vendor's bootloader (the 2K - 4K range may be optionally used
///              as bootloader_message_ab struct)
/// 16K - 64K    Used by uncrypt and recovery to store wipe_package for A/B devices
/// Note that these offsets are admitted by bootloader,recovery and uncrypt, so they
/// are not configurable without changing all of them.
pub const BOOTLOADER_MESSAGE_OFFSET_IN_MISC: usize = 0usize;
pub const VENDOR_SPACE_OFFSET_IN_MISC: usize = 2 * 1024usize;

/// Bootloader Message (2-KiB)
///
/// This structure describes the content of a block in flash
/// * that is used for recovery and the bootloader to talk to
/// * each other.
///
/// The command field is updated by linux when it wants to
/// reboot into recovery or to update radio or bootloader firmware.
/// It is also updated by the bootloader when firmware update
/// is complete (to boot into recovery for any final cleanup)
///
/// The status field was used by the bootloader after the completion
/// of an "update-radio" or "update-hboot" command, which has been
/// deprecated since Froyo.
///
/// The recovery field is only written by linux and used
/// for the system to send a message to recovery or the
/// other way around.
///
/// The stage field is written by packages which restart themselves
/// multiple times, so that the UI can reflect which invocation of the
/// package it is.  If the value is of the format "#/#" (eg, "1/3"),
/// the UI will add a simple indicator of that status.
///
/// We used to have slot_suffix field for A/B boot control metadata in
/// this struct, which gets unintentionally cleared by recovery or
/// uncrypt. Move it into struct bootloader_message_ab to avoid the
/// issue.
///
#[derive(Debug,Clone)]
#[repr(C, packed)]
pub struct BootloaderMessage {
    command : [u8;32],
    status : [u8;32],
    recovery: [u8;768],
    // The 'recovery' field used to be 1024 bytes.  It has only ever
    // been used to store the recovery command line, so 768 bytes
    // should be plenty.  We carve off the last 256 bytes to store the
    // stage string (for multistage packages) and possible future
    // expansion.
    stage : [u8;32],
    // The 'reserved' field used to be 224 bytes when it was initially
    // carved off from the 1024-byte recovery field. Bump it up to
    // 1184-byte so that the entire bootloader_message struct rounds up
    // to 2048-byte.
    reserved: [u8;1184],
}

/**
 * We must be cautious when changing the bootloader_message struct size,
 * because A/B-specific fields may end up with different offsets.
 */
 


///
/// The A/B-specific bootloader message structure (4-KiB).
///
/// We separate A/B boot control metadata from the regular bootloader
/// message struct and keep it here. Everything that's A/B-specific
/// stays after struct bootloader_message, which should be managed by
/// the A/B-bootloader or boot control HAL.
///
/// The slot_suffix field is used for A/B implementations where the
/// bootloader does not set the androidboot.ro.boot.slot_suffix kernel
/// commandline parameter. This is used by fs_mgr to mount /system and
/// other partitions with the slotselect flag set in fstab. A/B
/// implementations are free to use all 32 bytes and may store private
/// data past the first NUL-byte in this field. It is encouraged, but
/// not mandatory, to use 'struct bootloader_control' described below.
///
/// The update_channel field is used to store the Omaha update channel
/// if update_engine is compiled with Omaha support.
///

#[derive(Debug,Clone)]
#[repr(C,packed)]
pub struct BootloaderMessageAB {
    pub message : BootloaderMessage,
    pub slot_suffix: [u8;32],
    pub update_channel :[u8;128],
    // Round up the entire struct to 4096-byte.
    reserved:[u8;1888],
}

impl BootloaderMessageAB {

    pub fn get_bootloader_control(&self) -> Result<&BootloaderControl,BootloaderMessageError> {
        let crc32 : u32 = u32::from_ne_bytes([self.slot_suffix[28],self.slot_suffix[29],self.slot_suffix[30],self.slot_suffix[31]]);
        let data = &self.slot_suffix[0..28];
        let algo = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let computed_checksum = algo.checksum(&data);

        if crc32 != computed_checksum {
            Err(BootloaderMessageError::CrcFailure)
        } else {
            let bolo_ctrl_ptr = data.as_ptr() as  *const  BootloaderControl;
            let bolo_message_ab = unsafe {bolo_ctrl_ptr.as_ref().unwrap()}; 
            Ok(bolo_message_ab)
        }  
    }

    pub fn get_bootloader_control_mut(&mut self) -> Result<&mut BootloaderControl,BootloaderMessageError> {
        let crc32 : u32 = u32::from_ne_bytes([self.slot_suffix[28],self.slot_suffix[29],self.slot_suffix[30],self.slot_suffix[31]]);
        let data = &self.slot_suffix[0..28];
        let algo = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let computed_checksum = algo.checksum(&data);

        if crc32 != computed_checksum {
            Err(BootloaderMessageError::CrcFailure)
        } else {
            let bolo_ctrl_ptr = data.as_ptr() as  *mut  BootloaderControl;
            let bolo_message_ab = unsafe {bolo_ctrl_ptr.as_mut().unwrap()}; 
            Ok(bolo_message_ab)
        }        
    }


    pub fn set_bootloader_control(&mut self, ctrl : &BootloaderControl) -> Result<(), BootloaderMessageError> {
        todo!()
    }

     
    /// Call this function to recompute the checksum
    fn set_checksum(&mut self) {
        let data = &self.slot_suffix[0..28];
        let algo = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        let computed_checksum = algo.checksum(&data).to_ne_bytes();
        self.slot_suffix[28] = computed_checksum[0];
        self.slot_suffix[29] = computed_checksum[1];
        self.slot_suffix[30] = computed_checksum[2];
        self.slot_suffix[31] = computed_checksum[3];
    }
    
}

#[derive(Clone,Copy)]
#[repr(C,packed)]
pub struct BootloaderControl{
    // NUL terminated active slot suffix.
    pub slot_suffix: [u8;4],
    // Bootloader Control AB magic number (see BOOT_CTRL_MAGIC).
    pub magic:u32,
    version:u8,
    // Number of slots being managed.
    bitfield1 : u8,
    //uint8_t nb_slot : 3;
    // Number of times left attempting to boot recovery.
   // uint8_t recovery_tries_remaining : 3;
    // Ensure 4-bytes alignment for slot_info field.
    reserved0: [u8;2],
    // Per-slot information.  Up to 4 slots.
    slot_info:[SlotMetadata;4],
    // Reserved for further use.
    reserved1:[u8;8],
    // CRC32 of all 28 bytes preceding this field (little endian
    // format).
    crc32_le:u32,

}


#[derive(Debug,Clone,Copy)]
#[derive(BitfieldStruct)]
#[repr(C,packed)]
struct SlotMetadata {
    // Slot priority with 15 meaning highest priority, 1 lowest
    // priority and 0 the slot is unbootable.
    #[bitfield(name="priority", ty="u8", bits="0..=3")]
    // Number of times left attempting to boot this slot.
    #[bitfield(name="tries_remaining", ty="u8", bits="4..=6")]
    // 1 if this slot has booted successfully, 0 otherwise.
    #[bitfield(name="successful_boot", ty="u8", bits="7..=7")]
    data0 : [u8;1],
    // 1 if this slot is corrupted from a dm-verity corruption, 0
    #[bitfield(name="verity_corrupted", ty="u8", bits="0..=0")]
    data1 : [u8;1],
} 

impl Display for SlotMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Priority:{} TriesRemaining:{} SuccessfulBoot:{} VerityCorrupted:{}", self.priority(),self.tries_remaining(),self.successful_boot(),self.verity_corrupted())
    }
}

#[cfg(test)]
mod test {

    use thiserror::private::DisplayAsDisplay;

    use super::*;
    #[test]
    fn check_sizes() {
        assert_eq!(std::mem::size_of::<BootloaderMessage>(),2048);
        assert_eq!(std::mem::size_of::<BootloaderMessageAB>(),4096);
        assert_eq!(std::mem::size_of::<SlotMetadata>(),2);
        assert_eq!(std::mem::size_of::<BootloaderControl>(),32);
    }

    #[test]
    fn read_bolo_message() {
        let bytes = include_bytes!("./testdata/bolomessage.dat").as_ptr();
        let bolo_message_ab = bytes as  *const  BootloaderMessageAB;
        let bolo_message_ab = unsafe {bolo_message_ab.as_ref().unwrap()};


        let ctrl = bolo_message_ab.get_bootloader_control().unwrap();
        assert_eq!(ctrl.slot_info[0].priority(),15);
        assert_eq!(ctrl.slot_info[1].priority(),15);
        assert_eq!(ctrl.slot_info[2].priority(),0);
        assert_eq!(ctrl.slot_info[3].priority(),0);

        assert_eq!(ctrl.slot_info[0].tries_remaining(),6);
        assert_eq!(ctrl.slot_info[1].tries_remaining(),7);
        assert_eq!(ctrl.slot_info[2].tries_remaining(),0);
        assert_eq!(ctrl.slot_info[3].tries_remaining(),0);

        assert_eq!(ctrl.slot_info[0].successful_boot(),0);
        assert_eq!(ctrl.slot_info[1].successful_boot(),0);
        assert_eq!(ctrl.slot_info[2].successful_boot(),0);
        assert_eq!(ctrl.slot_info[3].successful_boot(),0);

        assert_eq!(ctrl.slot_info[0].verity_corrupted(),0);
        assert_eq!(ctrl.slot_info[1].verity_corrupted(),0);
        assert_eq!(ctrl.slot_info[2].verity_corrupted(),0);
        assert_eq!(ctrl.slot_info[3].verity_corrupted(),0);

        for s in ctrl.slot_info.as_ref().iter() {
            println!("SlotMetadata:{:?}",s.to_string());
        }

        let mut copy = bolo_message_ab.clone();
        let mut control = copy.get_bootloader_control_mut().unwrap();
        control.slot_info[0].set_successful_boot(1);
        for s in control.slot_info.as_ref().iter() {
            println!("SlotMetadata:{:?}",s.to_string());
        }
        //copy.set_checksum();

        

        
        
    }

}
