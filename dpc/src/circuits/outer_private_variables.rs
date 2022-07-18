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

use crate::{Execution, Network};
use snarkvm_algorithms::traits::SNARK;

#[derive(Derivative)]
#[derivative(Clone(bound = "N: Network"))]
pub struct OuterPrivateVariables<N: Network> {
    pub(super) inner_verifying_key: <N::InnerSNARK as SNARK>::VerifyingKey,
    pub(super) inner_proof: N::InnerProof,
    pub(super) execution: Execution<N>,
}

impl<N: Network> OuterPrivateVariables<N> {
    pub(crate) fn blank(
        inner_verifying_key: <N::InnerSNARK as SNARK>::VerifyingKey,
        inner_proof: N::InnerProof,
        execution: Execution<N>,
    ) -> Self {
        Self {
            inner_verifying_key,
            inner_proof,
            execution,
        }
    }

    pub(crate) fn new(
        inner_verifying_key: <N::InnerSNARK as SNARK>::VerifyingKey,
        inner_proof: N::InnerProof,
        execution: Execution<N>,
    ) -> Self {
        Self {
            inner_verifying_key,
            inner_proof,
            execution,
        }
    }
}
