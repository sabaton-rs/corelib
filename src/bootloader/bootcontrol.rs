/*
   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/
use std::io::Error;
use sabaton_hal::bootloader::{get_slot_suffix_from_cmd_line, BootControl};
use super::message::BootloaderMessageAB;

pub struct BootControlImpl(BootloaderMessageAB);

impl BootControlImpl {
    pub fn create() -> Result<Self, std::io::Error> {
        BootloaderMessageAB::create_from_misc_partition().map(Self)
    }
}

impl BootControl for BootControlImpl {
    fn number_of_slots(&self) -> Result<usize, std::io::Error> {
        let bl_control = self
            .0
            .get_bootloader_control()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(bl_control.nb_slot() as usize)
    }

    /// Get the current slot from the kernel command line
    fn current_slot(&self) -> Result<usize, std::io::Error> {
        let command_line = std::fs::read_to_string("/proc/cmdline")?;
        match get_slot_suffix_from_cmd_line(&command_line)? {
            "a" => Ok(0),
            "b" => Ok(1),
            s => Err(Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid slot suffix : {}", s),
            )),
        }
    }

    fn set_boot_successful(&mut self) -> Result<(), std::io::Error> {
        let current_slot = self.current_slot()?;

        let bl_control = self
            .0
            .get_bootloader_control_mut()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        bl_control.slot_info[current_slot].set_successful_boot(1);

        self.0.save_to_misc_partition()
    }

    fn set_active_slot(&mut self, slot_index: usize) -> Result<(), std::io::Error> {
        let bl_control = self
            .0
            .get_bootloader_control_mut()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if slot_index < bl_control.nb_slot() as usize {
            let suffix = match slot_index {
                0 => "a",
                1 => "b",
                _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "")),
            };

            bl_control
                .set_slot_suffix(suffix)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            self.0.save_to_misc_partition()
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid slot index : {}", slot_index),
            ))
        }
    }

    fn set_slot_as_unbootable(&mut self, slot_index: usize) -> Result<(), std::io::Error> {
        let bl_control = self
            .0
            .get_bootloader_control_mut()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if slot_index < bl_control.nb_slot() as usize {
            bl_control.slot_info[slot_index].set_tries_remaining(0);
        } else {
            return Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid slot index : {}", slot_index),
            ));
        }

        self.0.save_to_misc_partition()
    }

    fn is_bootable(&self, slot_index: usize) -> Result<bool, std::io::Error> {
        let bl_control = self
            .0
            .get_bootloader_control()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if slot_index < bl_control.nb_slot() as usize {
            Ok(bl_control.slot_info[slot_index].tries_remaining() > 0)
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid slot index : {}", slot_index),
            ))
        }
    }

    fn is_slot_successful(&self, slot_index: usize) -> Result<bool, std::io::Error> {
        let bl_control = self
            .0
            .get_bootloader_control()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if slot_index < bl_control.nb_slot() as usize {
            Ok(bl_control.slot_info[slot_index].successful_boot() == 1)
        } else {
            Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid slot index : {}", slot_index),
            ))
        }
    }

    fn active_slot(&self) -> Result<usize, std::io::Error> {
        let bl_control = self
            .0
            .get_bootloader_control()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let active_slot = bl_control
            .slot_suffix()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(
            match active_slot
                .to_str()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            {
                "a" => 0,
                "b" => 1,
                _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "")),
            },
        )
    }
}
