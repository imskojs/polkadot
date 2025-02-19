// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! WASM validation for adder parachain.

#![no_std]

#![feature(core_intrinsics, lang_items, core_panic_info, alloc_error_handler)]

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

use core::{intrinsics, panic};
use parachain::ValidationResult;
use parachain::codec::{Encode, Decode};
use adder::{HeadData, BlockData};

#[panic_handler]
#[no_mangle]
pub fn panic(_info: &panic::PanicInfo) -> ! {
	unsafe {
		intrinsics::abort()
	}
}

#[alloc_error_handler]
#[no_mangle]
pub fn oom(_: core::alloc::Layout) -> ! {
	unsafe {
		intrinsics::abort();
	}
}

#[no_mangle]
pub extern fn validate_block(params: *const u8, len: usize) -> usize {
	let params = unsafe { parachain::wasm_api::load_params(params, len) };
	let parent_head = HeadData::decode(&mut &params.parent_head[..])
		.expect("invalid parent head format.");

	let block_data = BlockData::decode(&mut &params.block_data[..])
		.expect("invalid block data format.");

	let parent_hash = tiny_keccak::keccak256(&params.parent_head[..]);

	// we also add based on incoming data from messages. ignoring unknown message
	// kinds.
	let from_messages = adder::process_messages(
		params.ingress.iter().map(|incoming| &incoming.data[..])
	);

	match adder::execute(parent_hash, parent_head, &block_data, from_messages) {
		Ok(new_head) => parachain::wasm_api::write_result(
			ValidationResult { head_data: new_head.encode() }
		),
		Err(_) => panic!("execution failure"),
	}
}
