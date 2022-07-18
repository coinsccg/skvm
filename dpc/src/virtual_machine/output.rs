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

use crate::prelude::*;
use snarkvm_algorithms::EncryptionScheme;

use anyhow::Result;
use rand::{CryptoRng, Rng};
use std::convert::TryInto;

#[derive(Derivative)]
#[derivative(Clone(bound = "N: Network"))]
pub struct Output<N: Network> {
    /// The address of the recipient.
    address: Address<N>,
    /// The balance of the recipient.
    value: AleoAmount,
    /// The program data of the recipient.
    payload: Payload<N>,
    /// The program that was run.
    program_id: N::ProgramID,
}

impl<N: Network> Output<N> {
    pub fn new_noop<R: Rng + CryptoRng>(rng: &mut R) -> Result<Self> {
        // Sample a burner noop private key.
        let noop_private_key = PrivateKey::new(rng);
        let noop_address = noop_private_key.try_into()?;

        Self::new(noop_address, AleoAmount::from_i64(0), Payload::default(), None)
    }

    /// Initializes a new instance of `Output`.
    pub fn new(
        address: Address<N>,
        value: AleoAmount,
        payload: Payload<N>,
        program_id: Option<N::ProgramID>,
    ) -> Result<Self> {
        // Retrieve the program ID. If `None` is provided, construct the noop program ID.
        let program_id = match program_id {
            Some(program_id) => program_id,
            None => *N::noop_program_id(),
        };

        Ok(Self {
            address,
            value,
            payload,
            program_id,
        })
    }

    /// Returns `true` if the program ID is the noop program.
    pub fn is_noop(&self) -> bool {
        self.program_id == *N::noop_program_id()
    }

    /// Returns the output record, given the previous serial number.
    pub fn to_record<R: Rng + CryptoRng>(&self, rng: &mut R) -> Result<(Record<N>, EncryptionRandomness<N>)> {
        // Generate the ciphertext parameters.
        let (randomness, randomizer, record_view_key) =
            N::account_encryption_scheme().generate_asymmetric_key(&*self.address, rng);
        let record = Record::from(
            self.address,
            self.value,
            self.payload.clone(),
            self.program_id,
            randomizer.into(),
            record_view_key.into(),
        )?;
        Ok((record, randomness))
    }

    /// Returns the address.
    pub fn address(&self) -> Address<N> {
        self.address
    }

    /// Returns the value.
    pub fn value(&self) -> AleoAmount {
        self.value
    }

    /// Returns a reference to the payload.
    pub fn payload(&self) -> &Payload<N> {
        &self.payload
    }

    /// Returns a reference to the program ID.
    pub fn program_id(&self) -> N::ProgramID {
        self.program_id
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::testnet2::*;
//
//     use rand::{thread_rng, SeedableRng};
//     use rand_chacha::ChaChaRng;
//
//     const ITERATIONS: usize = 100;
//
//     #[test]
//     fn test_new_noop_and_to_record() {
//         for _ in 0..ITERATIONS {
//             // Sample a random seed for the RNG.
//             let seed: u64 = thread_rng().gen();
//
//             // Generate the given inputs.
//             let mut given_rng = ChaChaRng::seed_from_u64(seed);
//             let given_serial_numbers = {
//                 let mut serial_numbers = Vec::with_capacity(Testnet2::NUM_INPUT_RECORDS);
//                 for _ in 0..Testnet2::NUM_INPUT_RECORDS {
//                     let input = Input::<Testnet2>::new_noop(&mut given_rng).unwrap();
//                     serial_numbers.push(input.serial_number().clone());
//                 }
//                 serial_numbers
//             };
//
//             // Checkpoint the RNG and clone it.
//             let mut expected_rng = given_rng.clone();
//             let mut candidate_rng = given_rng.clone();
//
//             // Generate the expected output state.
//             let expected_record = {
//                 let account = Account::<Testnet2>::new(&mut expected_rng).unwrap();
//                 Record::new_noop_output(account.address, given_serial_numbers[0], &mut expected_rng).unwrap()
//             };
//
//             // Generate the candidate output state.
//             let (candidate_record, candidate_address, candidate_value, candidate_payload, candidate_program_id) = {
//                 let output = Output::new_noop(&mut candidate_rng).unwrap();
//                 let record = output.to_record(given_serial_numbers[0], &mut candidate_rng).unwrap();
//                 (
//                     record,
//                     output.address(),
//                     output.value(),
//                     output.payload().clone(),
//                     output.program_id(),
//                 )
//             };
//
//             assert_eq!(expected_record, candidate_record);
//             assert_eq!(expected_record.owner(), candidate_address);
//             assert_eq!(expected_record.value(), candidate_value.0 as u64);
//             assert_eq!(expected_record.payload(), &candidate_payload);
//             assert_eq!(expected_record.program_id(), candidate_program_id);
//         }
//     }
// }
