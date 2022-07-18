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

//! An implementation of the [`Groth16`] zkSNARK.
//!
//! [`Groth16`]: https://eprint.iacr.org/2016/260.pdf

use snarkvm_curves::traits::{AffineCurve, PairingCurve, PairingEngine};
use snarkvm_fields::{ConstraintFieldError, Field, ToConstraintField};
use snarkvm_r1cs::{Index, LinearCombination};
use snarkvm_utilities::{
    errors::SerializationError,
    fmt,
    serialize::*,
    str::FromStr,
    FromBytes,
    FromBytesDeserializer,
    ToBytes,
    ToBytesSerializer,
    ToMinimalBits,
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::io::{
    Read,
    Result as IoResult,
    Write,
    {self},
};

/// Reduce an R1CS instance to a *Quadratic Arithmetic Program* instance.
mod r1cs_to_qap;

/// Groth16 zkSNARK construction.
pub mod snark;
pub use snark::*;

/// Generate public parameters for the Groth16 zkSNARK construction.
mod generator;

/// Create proofs for the Groth16 zkSNARK construction.
mod prover;

/// Verify proofs for the Groth16 zkSNARK construction.
mod verifier;

#[cfg(test)]
mod tests;

use crate::Prepare;
pub use generator::*;
pub use prover::*;
pub use verifier::*;

/// A proof in the Groth16 SNARK.
#[derive(Clone, Debug, Eq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Proof<E: PairingEngine> {
    pub a: E::G1Affine,
    pub b: E::G2Affine,
    pub c: E::G1Affine,
    pub(crate) compressed: bool,
}

impl<E: PairingEngine> Proof<E> {
    /// Returns `true` if the proof is in compressed form.
    pub fn is_compressed(&self) -> bool {
        self.compressed
    }

    /// Serialize the proof into bytes in compressed form, for storage
    /// on disk or transmission over the network.
    pub fn write_compressed<W: Write>(&self, mut writer: W) -> IoResult<()> {
        CanonicalSerialize::serialize(self, &mut writer)?;
        Ok(())
    }

    /// Serialize the proof into bytes in uncompressed form, for storage
    /// on disk or transmission over the network.
    pub fn write_uncompressed<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.a.write_le(&mut writer)?;
        self.b.write_le(&mut writer)?;
        self.c.write_le(&mut writer)
    }

    /// Deserialize the proof from compressed bytes.
    pub fn read_compressed<R: Read>(mut reader: R) -> IoResult<Self> {
        Ok(CanonicalDeserialize::deserialize(&mut reader)?)
    }

    /// Deserialize the proof from uncompressed bytes.
    pub fn read_uncompressed<R: Read>(mut reader: R) -> IoResult<Self> {
        let a: E::G1Affine = FromBytes::read_le(&mut reader)?;
        let b: E::G2Affine = FromBytes::read_le(&mut reader)?;
        let c: E::G1Affine = FromBytes::read_le(&mut reader)?;

        Ok(Self {
            a,
            b,
            c,
            compressed: false,
        })
    }

    /// Deserialize a proof from compressed or uncompressed bytes.
    pub fn read<R: Read>(mut reader: R) -> IoResult<Self> {
        let compressed_proof_size = Self::compressed_proof_size()?;
        let uncompressed_proof_size = Self::uncompressed_proof_size()?;

        // Construct the compressed reader.
        let mut proof_reader = vec![0u8; uncompressed_proof_size];
        reader.read_exact(&mut proof_reader[..compressed_proof_size])?;

        // Attempt to read the compressed proof.
        if let Ok(proof) = Self::read_compressed(&proof_reader[..compressed_proof_size]) {
            return Ok(proof);
        }

        // Construct the uncompressed reader.
        reader.read_exact(&mut proof_reader[compressed_proof_size..])?;

        // Attempt to read the uncompressed proof.
        Self::read_uncompressed(&proof_reader[..])
    }

    /// Returns the number of bytes in a compressed proof serialization.
    pub fn compressed_proof_size() -> IoResult<usize> {
        let mut buffer = Vec::new();
        Self::default().write_compressed(&mut buffer)?;
        Ok(buffer.len())
    }

    /// Returns the number of bytes in an uncompressed proof serialization.
    pub fn uncompressed_proof_size() -> IoResult<usize> {
        let mut buffer = Vec::new();
        Self::default().write_uncompressed(&mut buffer)?;
        Ok(buffer.len())
    }
}

impl<E: PairingEngine> FromBytes for Proof<E> {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Self::read(&mut reader)
    }
}

impl<E: PairingEngine> ToBytes for Proof<E> {
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        match self.compressed {
            true => self.write_compressed(&mut writer),
            false => self.write_uncompressed(&mut writer),
        }
    }
}

impl<E: PairingEngine> FromStr for Proof<E> {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(proof_hex: &str) -> Result<Self, Self::Err> {
        Self::from_bytes_le(&hex::decode(proof_hex)?)
    }
}

impl<E: PairingEngine> fmt::Display for Proof<E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let proof_hex = hex::encode(self.to_bytes_le().expect("Failed to convert proof to bytes"));
        write!(f, "{}", proof_hex)
    }
}

impl<E: PairingEngine> Serialize for Proof<E> {
    #[inline]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match serializer.is_human_readable() {
            true => serializer.collect_str(self),
            false => ToBytesSerializer::serialize(self, serializer),
        }
    }
}

impl<'de, E: PairingEngine> Deserialize<'de> for Proof<E> {
    #[inline]
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match deserializer.is_human_readable() {
            true => {
                let s: String = Deserialize::deserialize(deserializer)?;
                FromStr::from_str(&s).map_err(de::Error::custom)
            }
            false => FromBytesDeserializer::<Self>::deserialize_extended(
                deserializer,
                "proof",
                Self::compressed_proof_size().map_err(de::Error::custom)?,
                Self::uncompressed_proof_size().map_err(de::Error::custom)?,
            ),
        }
    }
}

impl<E: PairingEngine> PartialEq for Proof<E> {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c
    }
}

impl<E: PairingEngine> Default for Proof<E> {
    fn default() -> Self {
        Self {
            a: E::G1Affine::default(),
            b: E::G2Affine::default(),
            c: E::G1Affine::default(),
            compressed: true,
        }
    }
}

/// A verification key in the Groth16 SNARK.
#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct VerifyingKey<E: PairingEngine> {
    pub alpha_g1: E::G1Affine,
    pub beta_g2: E::G2Affine,
    pub gamma_g2: E::G2Affine,
    pub delta_g2: E::G2Affine,
    pub gamma_abc_g1: Vec<E::G1Affine>,
}

impl<E: PairingEngine> ToBytes for VerifyingKey<E> {
    /// Writes the verifying key into little-endian bytes,
    /// for storage on disk or transmission over the network.
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.alpha_g1.write_le(&mut writer)?;
        self.beta_g2.write_le(&mut writer)?;
        self.gamma_g2.write_le(&mut writer)?;
        self.delta_g2.write_le(&mut writer)?;
        (self.gamma_abc_g1.len() as u32).write_le(&mut writer)?;
        for g in &self.gamma_abc_g1 {
            g.write_le(&mut writer)?;
        }
        Ok(())
    }
}

impl<E: PairingEngine> ToMinimalBits for VerifyingKey<E> {
    fn to_minimal_bits(&self) -> Vec<bool> {
        let alpha_g1_bits = self.alpha_g1.to_minimal_bits();
        let beta_g2_bits = self.beta_g2.to_minimal_bits();
        let gamma_g2_bits = self.gamma_g2.to_minimal_bits();
        let delta_g2_bits = self.delta_g2.to_minimal_bits();
        let gamma_abc_g1_bits = self.gamma_abc_g1.to_minimal_bits();

        [
            alpha_g1_bits,
            beta_g2_bits,
            gamma_g2_bits,
            delta_g2_bits,
            gamma_abc_g1_bits,
        ]
        .concat()
    }
}

impl<E: PairingEngine> ToConstraintField<E::Fq> for VerifyingKey<E> {
    fn to_field_elements(&self) -> Result<Vec<E::Fq>, ConstraintFieldError> {
        let mut res = vec![];
        res.extend_from_slice(&self.alpha_g1.to_field_elements()?);
        res.extend_from_slice(&self.beta_g2.to_field_elements()?);
        res.extend_from_slice(&self.gamma_g2.to_field_elements()?);
        res.extend_from_slice(&self.delta_g2.to_field_elements()?);
        for elem in self.gamma_abc_g1.iter() {
            res.extend_from_slice(&elem.to_field_elements()?);
        }

        Ok(res)
    }
}

impl<E: PairingEngine> FromBytes for VerifyingKey<E> {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Self::read(&mut reader)
    }
}

impl<E: PairingEngine> From<ProvingKey<E>> for VerifyingKey<E> {
    fn from(other: ProvingKey<E>) -> Self {
        other.vk
    }
}

impl<E: PairingEngine> From<PreparedVerifyingKey<E>> for VerifyingKey<E> {
    fn from(other: PreparedVerifyingKey<E>) -> Self {
        other.vk
    }
}

impl<E: PairingEngine> Default for VerifyingKey<E> {
    fn default() -> Self {
        Self {
            alpha_g1: E::G1Affine::default(),
            beta_g2: E::G2Affine::default(),
            gamma_g2: E::G2Affine::default(),
            delta_g2: E::G2Affine::default(),
            gamma_abc_g1: Vec::new(),
        }
    }
}

impl<E: PairingEngine> VerifyingKey<E> {
    /// Deserialize the verification key from bytes.
    pub fn read<R: Read>(mut reader: R) -> IoResult<Self> {
        let alpha_g1: E::G1Affine = FromBytes::read_le(&mut reader)?;
        let beta_g2: E::G2Affine = FromBytes::read_le(&mut reader)?;
        let gamma_g2: E::G2Affine = FromBytes::read_le(&mut reader)?;
        let delta_g2: E::G2Affine = FromBytes::read_le(&mut reader)?;

        let gamma_abc_g1_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut gamma_abc_g1: Vec<E::G1Affine> = Vec::with_capacity(gamma_abc_g1_len as usize);
        for _ in 0..gamma_abc_g1_len {
            let gamma_abc_g1_element: E::G1Affine = FromBytes::read_le(&mut reader)?;
            gamma_abc_g1.push(gamma_abc_g1_element);
        }

        Ok(Self {
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1,
        })
    }
}

impl<E: PairingEngine> Prepare<PreparedVerifyingKey<E>> for VerifyingKey<E> {
    fn prepare(&self) -> PreparedVerifyingKey<E> {
        prepare_verifying_key(self.clone())
    }
}

/// Full public (prover and verifier) parameters for the Groth16 zkSNARK.
#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct ProvingKey<E: PairingEngine> {
    pub vk: VerifyingKey<E>,
    pub beta_g1: E::G1Affine,
    pub delta_g1: E::G1Affine,
    pub a_query: Vec<E::G1Affine>,
    pub b_g1_query: Vec<E::G1Affine>,
    pub b_g2_query: Vec<E::G2Affine>,
    pub h_query: Vec<E::G1Affine>,
    pub l_query: Vec<E::G1Affine>,
}

impl<E: PairingEngine> ToBytes for ProvingKey<E> {
    /// Serialize the proving key into little-endian bytes.
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.vk.write_le(&mut writer)?;

        self.beta_g1.write_le(&mut writer)?;

        self.delta_g1.write_le(&mut writer)?;

        (self.a_query.len() as u32).write_le(&mut writer)?;
        for g in &self.a_query[..] {
            g.write_le(&mut writer)?;
        }

        (self.b_g1_query.len() as u32).write_le(&mut writer)?;
        for g in &self.b_g1_query[..] {
            g.write_le(&mut writer)?;
        }

        (self.b_g2_query.len() as u32).write_le(&mut writer)?;
        for g in &self.b_g2_query[..] {
            g.write_le(&mut writer)?;
        }

        (self.h_query.len() as u32).write_le(&mut writer)?;
        for g in &self.h_query[..] {
            g.write_le(&mut writer)?;
        }

        (self.l_query.len() as u32).write_le(&mut writer)?;
        for g in &self.l_query[..] {
            g.write_le(&mut writer)?;
        }

        Ok(())
    }
}

impl<E: PairingEngine> FromBytes for ProvingKey<E> {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        Self::read(&mut reader, false)
    }
}

impl<E: PairingEngine> ProvingKey<E> {
    /// Deserialize the public parameters from bytes.
    pub fn read<R: Read>(mut reader: R, checked: bool) -> IoResult<Self> {
        let read_g1_affine = |mut reader: &mut R| -> IoResult<E::G1Affine> {
            let g1_affine: E::G1Affine = FromBytes::read_le(&mut reader)?;

            if checked && !g1_affine.is_in_correct_subgroup_assuming_on_curve() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "point is not in the correct subgroup",
                ));
            }

            Ok(g1_affine)
        };

        let read_g2_affine = |mut reader: &mut R| -> IoResult<E::G2Affine> {
            let g2_affine: E::G2Affine = FromBytes::read_le(&mut reader)?;

            if checked && !g2_affine.is_in_correct_subgroup_assuming_on_curve() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "point is not in the correct subgroup",
                ));
            }

            Ok(g2_affine)
        };

        let vk = VerifyingKey::<E>::read(&mut reader)?;

        let beta_g1: E::G1Affine = FromBytes::read_le(&mut reader)?;

        let delta_g1: E::G1Affine = FromBytes::read_le(&mut reader)?;

        let a_query_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut a_query = Vec::with_capacity(a_query_len as usize);
        for _ in 0..a_query_len {
            a_query.push(read_g1_affine(&mut reader)?);
        }

        let b_g1_query_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut b_g1_query = Vec::with_capacity(b_g1_query_len as usize);
        for _ in 0..b_g1_query_len {
            b_g1_query.push(read_g1_affine(&mut reader)?);
        }

        let b_g2_query_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut b_g2_query = Vec::with_capacity(b_g2_query_len as usize);
        for _ in 0..b_g2_query_len {
            b_g2_query.push(read_g2_affine(&mut reader)?);
        }

        let h_query_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut h_query = Vec::with_capacity(h_query_len as usize);
        for _ in 0..h_query_len {
            h_query.push(read_g1_affine(&mut reader)?);
        }

        let l_query_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut l_query = Vec::with_capacity(l_query_len as usize);
        for _ in 0..l_query_len {
            l_query.push(read_g1_affine(&mut reader)?);
        }

        Ok(Self {
            vk,
            beta_g1,
            delta_g1,
            a_query,
            b_g1_query,
            b_g2_query,
            h_query,
            l_query,
        })
    }
}

/// Preprocessed verification key parameters that enable faster verification
/// at the expense of larger size in memory.
#[derive(Clone, Debug)]
pub struct PreparedVerifyingKey<E: PairingEngine> {
    pub vk: VerifyingKey<E>,
    pub alpha_g1_beta_g2: E::Fqk,
    pub gamma_g2_neg_pc: <E::G2Affine as PairingCurve>::Prepared,
    pub delta_g2_neg_pc: <E::G2Affine as PairingCurve>::Prepared,
}

impl<E: PairingEngine> PreparedVerifyingKey<E> {
    fn gamma_abc_g1(&self) -> &[E::G1Affine] {
        &self.vk.gamma_abc_g1
    }
}

impl<E: PairingEngine> From<ProvingKey<E>> for PreparedVerifyingKey<E> {
    fn from(other: ProvingKey<E>) -> Self {
        prepare_verifying_key(other.vk)
    }
}

impl<E: PairingEngine> From<VerifyingKey<E>> for PreparedVerifyingKey<E> {
    fn from(other: VerifyingKey<E>) -> Self {
        prepare_verifying_key(other)
    }
}

fn push_constraints<F: Field>(l: LinearCombination<F>, constraints: &mut Vec<Vec<(F, Index)>>) {
    let vars_and_coeffs = l.as_ref();
    let mut vec = Vec::with_capacity(vars_and_coeffs.len());

    for (var, coeff) in vars_and_coeffs {
        match var.get_unchecked() {
            Index::Public(i) => vec.push((*coeff, Index::Public(i))),
            Index::Private(i) => vec.push((*coeff, Index::Private(i))),
        }
    }
    constraints.push(vec);
}
