//https://android.googlesource.com/platform/bootable/recovery/+/refs/tags/android-10.0.0_r25/bootloader_message/include/bootloader_message/bootloader_message.h


use std::{
    ffi::{CStr, CString},
    fs::File,
    iter, str::FromStr, io::Read, convert::TryFrom,
};

use bounded_integer::*;
use error::BootloaderMessageError;

use super::error;

bounded_integer! {   
    pub struct Priority { 0..16}
}

bounded_integer! {
    pub struct TriesRemaining { 0..8}
}

bounded_integer! {
    pub struct NumSlots { 0..5}
}
bounded_integer! {
    pub struct Reserved { 0..128}
}

#[derive(Clone, Copy,PartialEq,Debug)]
pub struct SlotMetadata {
    // Slot priority with 15 meaning highest priority, 1 lowest
    // priority and 0 the slot is unbootable.
    priority: Priority,
    // Number of times left attempting to boot this slot.
    tries_remaining: TriesRemaining,
    // 1 if this slot has booted successfully, 0 otherwise.
    successful_boot: bool,
    // 1 if this slot is corrupted from a dm-verity corruption, 0
    // otherwise.
    verity_corrupted: bool,
    
}

impl TryFrom<&[u8]> for SlotMetadata {
    type Error = BootloaderMessageError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
      // todo!()
      let priority=Priority::new(value[0]).ok_or(BootloaderMessageError::PriorityOutOfRange)?;
      let tries_remaining=TriesRemaining::new(value[1]).ok_or(BootloaderMessageError::PriorityOutOfRange)?;
      let successful_boot:bool=value[2] != 0;
      let verity_corrupted:bool=value[3] != 0;

      let slotmetada=SlotMetadata{
        priority,
        tries_remaining,
        successful_boot,
        verity_corrupted,
    };
    print!("Slotmetada {:?}",slotmetada);
      Ok(slotmetada)
    }
    
}

impl Into<Vec<u8>> for SlotMetadata {
    fn into(self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        v.push(self.priority());
        v.push(self.tries_remaining());
        v.push(self.successful_boot().into());
        v.push(self.verity_corrupted().into());       
        v
    }
}

impl SlotMetadata {
    pub fn priority(&self) -> u8 {
        self.priority.as_ref().clone()
    }

    pub fn tries_remaining(&self) -> u8 {
        self.tries_remaining.as_ref().clone()
    }

    pub fn successful_boot(&self) -> bool {
        self.successful_boot
    }

    pub fn verity_corrupted(&self) -> bool {
        self.verity_corrupted
    }
}
#[derive (Clone,PartialEq,Debug)]
pub struct BootloaderControl {
    // NUL terminated active slot suffix.
    slot_suffix: CString,
    // Bootloader Control AB magic number (see BOOT_CTRL_MAGIC).
    // Number of slots being managed.
    nb_slot: NumSlots,
    // Number of times left attempting to boot recovery.
    recovery_tries_remaining: TriesRemaining,
    // Ensure 4-bytes alignment for slot_info field.
    //reserved0[2];
    // Per-slot information.  Up to 4 slots.
    slot_info:[SlotMetadata;4],
}

impl BootloaderControl {
    pub fn slot_suffix(&self) -> &CStr {
        self.slot_suffix.as_ref()
    }

    pub fn num_slots(&self) -> u8 {
        self.nb_slot.as_ref().clone()
    }

    pub fn recovery_tries_remaining(&self) -> u8 {
        self.recovery_tries_remaining.as_ref().clone()
    }

    pub fn slot_iter(&self) -> std::slice::Iter<'_, SlotMetadata> {
        self.slot_info.iter()
    }
}

impl TryFrom<&[u8]> for BootloaderControl {
    type Error = BootloaderMessageError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        //todo!()
        let slot_suffix_vec=value[0..4].to_vec();
        //let mut slot_suffix_bytes = [0u8; 4];
        let  mut slot_suffix;
        for (i, &item) in slot_suffix_vec.iter().enumerate() {
            let slot_suffix=if item == 0 {
                println!("slot_suffix[i]{},index:{}",slot_suffix_vec[i],i) ;
                let (left, _right) =slot_suffix_vec.split_at(i+1);
                slot_suffix=CString::from_vec_with_nul(left.to_vec()).unwrap();
                println!("SLOT SUFFIX {:?}",slot_suffix);
                slot_suffix
            }
            else{
                continue;
            };
            
        }
        
      //  slot_suffix=slot_suffix
       
        let nb_slot=NumSlots::new(value[9]).unwrap();
        let recovery_tries_remaining=TriesRemaining::new(value[10]).unwrap();
        let mut initial_index=13;
        let slot=&value[initial_index..initial_index+5];
        let slotmetadata0:SlotMetadata=SlotMetadata::try_from(slot)?;
        initial_index=initial_index+5;
        let slot=&value[initial_index..initial_index+5];
        let slotmetadata1:SlotMetadata=SlotMetadata::try_from(slot)?;
        initial_index=initial_index+5;
        let slot=&value[initial_index..initial_index+5];
        let slotmetadata2:SlotMetadata=SlotMetadata::try_from(slot)?;
        initial_index=initial_index+5;
        let slot=&value[initial_index..initial_index+5];
        let slotmetadata3:SlotMetadata=SlotMetadata::try_from(slot)?;
        initial_index=initial_index+5;
        
        
        let bootloadercontrol=BootloaderControl{
            slot_suffix:todo!(),
            nb_slot,
            recovery_tries_remaining,
            slot_info:[slotmetadata0,slotmetadata1,slotmetadata2,slotmetadata3],
      };
      Ok(bootloadercontrol)
    }
}

impl Into<Vec<u8>> for BootloaderControl {
    fn into(self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        let value0 = self.slot_suffix().to_bytes();
        let mut slot_suffix_bytes = [0u8; 4];
        let check = self.slot_suffix.to_bytes_with_nul();
        if check.len() <= 4 {
            for (index, byte) in check.iter().enumerate() {
                slot_suffix_bytes[index] = *byte;
            }
            v.extend(&slot_suffix_bytes);
            let magic_bytes=0x42414342u32.to_ne_bytes();
            v.extend(magic_bytes);
            let version=0x01u8.to_ne_bytes();
            v.extend(version);
            v.push(self.num_slots());
            v.push(self.recovery_tries_remaining());
            let reserved= [0u8; 2];
            v.extend(reserved);
            //slot meta data
            let x=self.slot_iter(); 
            let mut slot_metadata_vec: Vec<u8> = Vec::new();
            for  slotmetada in self.slot_iter(){
                let value:Vec<u8>= (*slotmetada).into();
                slot_metadata_vec.extend(value);
                //let s= Reserved::new(0).unwrap();
                //v.push(reserved1.into());
            }
            println!("SLOTMETADATA{:?}\n",slot_metadata_vec);
            v.extend(slot_metadata_vec);
            //v.extend(x);)
            let reserved= [0u8; 8];
            v.extend(reserved);
            let crc32_le=0x00000000u32.to_le_bytes();
            v.extend(crc32_le);


        } else {
            panic!();
        }
        
        v
        //todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use super::SlotMetadata;
    #[test]
    fn it_works() {
        let slot_metadata0 = SlotMetadata {
            priority: Priority::new(0).unwrap(),
            tries_remaining: TriesRemaining::new(7).unwrap(),
            successful_boot: false,
            verity_corrupted: false,
        };

        let slot_metadata1 = SlotMetadata {
            priority: Priority::new(0).unwrap(),
            tries_remaining: TriesRemaining::new(7).unwrap(),
            successful_boot: false,
            verity_corrupted: false,
        };

        let slot_metadata2 = SlotMetadata {
            priority: Priority::new(0).unwrap(),
            tries_remaining: TriesRemaining::new(7).unwrap(),
            successful_boot: false,
            verity_corrupted: false,
        };

        let slot_metadata3 = SlotMetadata {
            priority: Priority::new(0).unwrap(),
            tries_remaining: TriesRemaining::new(7).unwrap(),
            successful_boot: false,
            verity_corrupted: false,
        };
        
        let control = BootloaderControl {
            slot_suffix: CString::new("a").expect("error"),
            nb_slot: NumSlots::new(4).unwrap(),
            recovery_tries_remaining: TriesRemaining::new(7).unwrap(),
            slot_info: [
                slot_metadata0,
                slot_metadata1,
                slot_metadata2,
                slot_metadata3

            ],
        };

        // This converts the Bootloader control structure into a Vector of bytes
        let vec: Vec<u8> = control.clone().into();
        let slice = vec.as_slice();
        println!("VECTOR: {:?}\n",vec);

        // This will convert the raw buffer into a Bootloader control
        let reverse_control: BootloaderControl = slice.try_into().unwrap();
        println!("\nREVERSE_CONTROL: {:?}\n",reverse_control);
        assert!(control==reverse_control);
    }
}
