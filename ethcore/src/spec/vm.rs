// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Parameters for a virtual machines

use wasm_utils::rules::Set as WasmRuleSet;

pub enum VirtualMachine {
    Evm(EthereumVirtualMachine),
    Wasm(WasmVirtualMachine),
    Hybrid(HybridVirtualMachine),
}

pub struct EthereumVirtualMachine {
    pub max_depth: usize,
    pub allow_wasm_calls: bool,
}

pub enum WasmAllocator {
    None,
    Arena(usize)
}

pub struct WasmMemory {
    pub allocator: WasmAllocator,
    pub max_total_memory: usize,
    pub stack: usize,
}

pub struct WasmStorage {
    pub read: usize,
    pub write: usize,
}

pub struct WasmVirtualMachine {
    pub cost_table: WasmRuleSet,
    pub memory: WasmMemory,
    pub storage: WasmStorage,
    pub static_charge: usize,
}

pub struct HybridVirtualMachine {
    pub wasm: WasmVirtualMachine,
    pub evm: EthereumVirtualMachine,
}