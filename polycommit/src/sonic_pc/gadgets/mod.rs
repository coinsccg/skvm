// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

/// Gadgets for Sonic-KZG10 commitments.
pub mod commitment;
pub(crate) use commitment::*;

/// Gadget for Sonic-KZG10 polynomial commitment scheme.
pub mod sonic_kzg10;

/// Gadgets for Sonic-KZG10 proofs.
pub mod proof;

/// Verifier key gadgets for the Sonic-KZG10 polynomial commitment scheme.
pub mod verifier_key;
