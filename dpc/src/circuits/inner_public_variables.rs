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

use crate::{AleoAmount, Network};
use snarkvm_fields::{ConstraintFieldError, ToConstraintField};
use snarkvm_utilities::ToBytes;

use anyhow::Result;

#[derive(Copy, Clone, Debug)]
pub struct InnerPublicVariables<N: Network> {
    /// Transition ID
    transition_id: N::TransitionID,
    value_balance: AleoAmount,
    ledger_root: N::LedgerRoot,
    local_transitions_root: N::TransactionID,
    // These are required in natively verifying an inner circuit proof.
    // However for verification in the outer circuit, these must be provided as witness.
    /// Program ID
    pub(super) program_id: Option<N::ProgramID>,
}

impl<N: Network> InnerPublicVariables<N> {
    pub(crate) fn blank() -> Self {
        Self {
            transition_id: Default::default(),
            value_balance: AleoAmount::ZERO,
            ledger_root: N::LedgerRoot::default(),
            local_transitions_root: Default::default(),
            program_id: Some(N::ProgramID::default()),
        }
    }

    pub(crate) fn new(
        transition_id: N::TransitionID,
        value_balance: AleoAmount,
        ledger_root: N::LedgerRoot,
        local_transitions_root: N::TransactionID,
        program_id: Option<N::ProgramID>,
    ) -> Self {
        Self {
            transition_id,
            value_balance,
            ledger_root,
            local_transitions_root,
            program_id,
        }
    }

    /// Returns the transition ID.
    pub(crate) fn transition_id(&self) -> N::TransitionID {
        self.transition_id
    }

    /// Returns the value balance of the transition.
    pub(crate) fn value_balance(&self) -> AleoAmount {
        self.value_balance
    }

    /// Returns the ledger root.
    pub(crate) fn ledger_root(&self) -> N::LedgerRoot {
        self.ledger_root
    }

    pub(crate) fn local_transitions_root(&self) -> N::TransactionID {
        self.local_transitions_root
    }
}

impl<N: Network> ToConstraintField<N::InnerScalarField> for InnerPublicVariables<N> {
    fn to_field_elements(&self) -> Result<Vec<N::InnerScalarField>, ConstraintFieldError> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.ledger_root.to_field_elements()?);
        v.extend_from_slice(&self.local_transitions_root.to_field_elements()?);

        if let Some(program_id) = &self.program_id {
            v.extend_from_slice(&program_id.to_bytes_le()?.to_field_elements()?);
        }

        v.extend_from_slice(&self.value_balance.to_bytes_le()?.to_field_elements()?);
        v.extend_from_slice(&self.transition_id.to_field_elements()?);

        Ok(v)
    }
}
