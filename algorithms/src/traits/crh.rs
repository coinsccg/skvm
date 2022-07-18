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

use crate::errors::CRHError;
use snarkvm_utilities::{FromBytes, ToBytes};

use snarkvm_fields::PrimeField;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
};

pub trait CRH: Clone + Debug + ToBytes + FromBytes + Send + Sync + From<<Self as CRH>::Parameters> {
    type Output: Copy + Clone + Debug + Display + ToBytes + FromBytes + Eq + Hash + Default + Send + Sync;
    type Parameters: Clone + Debug + Eq;

    fn setup(message: &str) -> Self;

    fn hash(&self, input: &[u8]) -> Result<Self::Output, CRHError> {
        let bits = input
            .iter()
            .flat_map(|&byte| (0..8).map(move |i| (byte >> i) & 1u8 == 1u8))
            .collect::<Vec<bool>>();
        self.hash_bits(&bits)
    }

    fn hash_bits(&self, input_bits: &[bool]) -> Result<Self::Output, CRHError>;

    fn hash_field_elements<F: PrimeField>(&self, input: &[F]) -> Result<Self::Output, CRHError> {
        let mut input_bytes = vec![];
        for elem in input.iter() {
            input_bytes.extend_from_slice(&elem.to_bytes_le()?);
        }
        self.hash(&input_bytes)
    }

    fn parameters(&self) -> &Self::Parameters;
}
