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

use crate::{
    templates::{
        bw6::BW6Parameters,
        short_weierstrass_jacobian::{Affine, Projective},
    },
    traits::pairing_engine::AffineCurve,
};
use snarkvm_fields::Zero;
use snarkvm_utilities::{errors::SerializationError, serialize::*, FromBytes, ToBytes};

use std::io::{Read, Result as IoResult, Write};

pub type G1Affine<P> = Affine<<P as BW6Parameters>::G1Parameters>;
pub type G1Projective<P> = Projective<<P as BW6Parameters>::G1Parameters>;

#[derive(Derivative, CanonicalSerialize, CanonicalDeserialize)]
#[derivative(
    Clone(bound = "P: BW6Parameters"),
    Debug(bound = "P: BW6Parameters"),
    PartialEq(bound = "P: BW6Parameters"),
    Eq(bound = "P: BW6Parameters")
)]
pub struct G1Prepared<P: BW6Parameters>(pub G1Affine<P>);

impl<P: BW6Parameters> From<G1Affine<P>> for G1Prepared<P> {
    fn from(other: G1Affine<P>) -> Self {
        G1Prepared(other)
    }
}

impl<P: BW6Parameters> G1Prepared<P> {
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<P: BW6Parameters> Default for G1Prepared<P> {
    fn default() -> Self {
        G1Prepared(G1Affine::<P>::prime_subgroup_generator())
    }
}

impl<P: BW6Parameters> ToBytes for G1Prepared<P> {
    fn write_le<W: Write>(&self, writer: W) -> IoResult<()> {
        self.0.write_le(writer)
    }
}

impl<P: BW6Parameters> FromBytes for G1Prepared<P> {
    fn read_le<R: Read>(reader: R) -> IoResult<Self> {
        Ok(Self(G1Affine::<P>::read_le(reader)?))
    }
}
