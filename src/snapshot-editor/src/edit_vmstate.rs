// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::Subcommand;
use vmm::persist::MicrovmState;

use crate::utils::{open_vmstate, save_vmstate, UtilsError};

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum EditVmStateError {
    /// {0}
    Utils(#[from] UtilsError),
}

#[derive(Debug, Subcommand)]
pub enum EditVmStateSubCommand {
    /// Update the rootfs path
    UpdateRootfsPath {
        /// Path to the vmstate file.
        #[arg(short, long)]
        vmstate_path: PathBuf,
        /// Path of output file.
        #[arg(short, long)]
        output_path: PathBuf,
    },
}

pub fn edit_vmstate_command(command: EditVmStateSubCommand) -> Result<(), EditVmStateError> {
    match command {
        EditVmStateSubCommand::UpdateRootfsPath {
            vmstate_path,
            output_path,
        } => edit(&vmstate_path, &output_path, |state| {
            rewrite_virtio_block_disk_path(state)
        })?,
    }
    Ok(())
}

fn edit(
    vmstate_path: &PathBuf,
    output_path: &PathBuf,
    f: impl Fn(MicrovmState) -> Result<MicrovmState, EditVmStateError>,
) -> Result<(), EditVmStateError> {
    let (microvm_state, version) = open_vmstate(vmstate_path)?;
    let microvm_state = f(microvm_state)?;
    save_vmstate(microvm_state, output_path, version)?;
    Ok(())
}

fn rewrite_virtio_block_disk_path(
    mut state: MicrovmState,
) -> Result<MicrovmState, EditVmStateError> {
    // device_states is not an Option, but a DeviceStates struct
    for block in state.device_states.block_devices.iter_mut() {
        if let vmm::devices::virtio::block::persist::BlockState::Virtio(ref mut blk_state) = block.device_state {
            blk_state.set_disk_path("/mnt/disks/fc-envs/v1/rootfs.ext4".to_string());
        }
    }
    Ok(state)
}