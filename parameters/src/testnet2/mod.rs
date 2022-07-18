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

pub mod genesis;
pub use genesis::*;

const REMOTE_URL: &str = "https://s3-us-west-1.amazonaws.com/aleo.parameters";

// Universal SRS
impl_remote!(UniversalSRSBytes, REMOTE_URL, "./resources/", "universal", "srs");

// Noop Circuit
impl_local!(NoopProvingKeyBytes, "./resources/", "noop", "proving");
impl_local!(NoopVerifyingKeyBytes, "./resources/", "noop", "verifying");

// Inner Circuit
impl_remote!(InnerProvingKeyBytes, REMOTE_URL, "./resources/", "inner", "proving");
impl_local!(InnerVerifyingKeyBytes, "./resources/", "inner", "verifying");

// Outer Circuit
impl_remote!(OuterProvingKeyBytes, REMOTE_URL, "./resources/", "outer", "proving");
impl_local!(OuterVerifyingKeyBytes, "./resources/", "outer", "verifying");

// PoSW Circuit
impl_remote!(PoSWProvingKeyBytes, REMOTE_URL, "./resources/", "posw", "proving");
impl_local!(PoSWVerifyingKeyBytes, "./resources/", "posw", "verifying");
