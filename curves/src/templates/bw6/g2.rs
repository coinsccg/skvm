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
        bw6::{BW6Parameters, TwistType},
        short_weierstrass_jacobian::{Affine, Projective},
    },
    traits::{AffineCurve, ShortWeierstrassParameters},
};
use snarkvm_fields::{Field, One, Zero};
use snarkvm_utilities::{bititerator::BitIteratorBE, errors::SerializationError, serialize::*, ToBytes};

use std::{
    io::{Result as IoResult, Write},
    ops::Neg,
};

pub type G2Affine<P> = Affine<<P as BW6Parameters>::G2Parameters>;
pub type G2Projective<P> = Projective<<P as BW6Parameters>::G2Parameters>;

#[derive(Derivative, CanonicalSerialize, CanonicalDeserialize)]
#[derivative(
    Clone(bound = "P: BW6Parameters"),
    Debug(bound = "P: BW6Parameters"),
    PartialEq(bound = "P: BW6Parameters"),
    Eq(bound = "P: BW6Parameters")
)]
pub struct G2Prepared<P: BW6Parameters> {
    // Stores the coefficients of the line evaluations as calculated in
    // https://eprint.iacr.org/2013/722.pdf
    pub ell_coeffs_1: Vec<(P::Fp, P::Fp, P::Fp)>,
    pub ell_coeffs_2: Vec<(P::Fp, P::Fp, P::Fp)>,
    pub infinity: bool,
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = "P: BW6Parameters"),
    Copy(bound = "P: BW6Parameters"),
    Debug(bound = "P: BW6Parameters")
)]
struct G2HomProjective<P: BW6Parameters> {
    x: P::Fp,
    y: P::Fp,
    z: P::Fp,
}

impl<P: BW6Parameters> Default for G2Prepared<P> {
    fn default() -> Self {
        Self::from(G2Affine::<P>::prime_subgroup_generator())
    }
}

impl<P: BW6Parameters> ToBytes for G2Prepared<P> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        (self.ell_coeffs_1.len() as u32).write_le(&mut writer)?;
        for coeff_1 in &self.ell_coeffs_1 {
            coeff_1.0.write_le(&mut writer)?;
            coeff_1.1.write_le(&mut writer)?;
            coeff_1.2.write_le(&mut writer)?;
        }

        (self.ell_coeffs_2.len() as u32).write_le(&mut writer)?;
        for coeff_2 in &self.ell_coeffs_2 {
            coeff_2.0.write_le(&mut writer)?;
            coeff_2.1.write_le(&mut writer)?;
            coeff_2.2.write_le(&mut writer)?;
        }
        self.infinity.write_le(writer)
    }
}

impl<P: BW6Parameters> FromBytes for G2Prepared<P> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let ell_coeffs_1_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut ell_coeffs_1 = Vec::with_capacity(ell_coeffs_1_len as usize);
        for _ in 0..ell_coeffs_1_len {
            let coeff_1: P::Fp = FromBytes::read_le(&mut reader)?;
            let coeff_2: P::Fp = FromBytes::read_le(&mut reader)?;
            let coeff_3: P::Fp = FromBytes::read_le(&mut reader)?;
            ell_coeffs_1.push((coeff_1, coeff_2, coeff_3));
        }

        let ell_coeffs_2_len: u32 = FromBytes::read_le(&mut reader)?;
        let mut ell_coeffs_2 = Vec::with_capacity(ell_coeffs_2_len as usize);
        for _ in 0..ell_coeffs_2_len {
            let coeff_1: P::Fp = FromBytes::read_le(&mut reader)?;
            let coeff_2: P::Fp = FromBytes::read_le(&mut reader)?;
            let coeff_3: P::Fp = FromBytes::read_le(&mut reader)?;
            ell_coeffs_2.push((coeff_1, coeff_2, coeff_3));
        }

        let infinity: bool = FromBytes::read_le(&mut reader)?;

        Ok(Self {
            ell_coeffs_1,
            ell_coeffs_2,
            infinity,
        })
    }
}

impl<P: BW6Parameters> From<G2Affine<P>> for G2Prepared<P> {
    fn from(q: G2Affine<P>) -> Self {
        if q.is_zero() {
            return Self {
                ell_coeffs_1: vec![],
                ell_coeffs_2: vec![],
                infinity: true,
            };
        }

        // f_{u+1,Q}(P)
        let mut r = G2HomProjective {
            x: q.x,
            y: q.y,
            z: P::Fp::one(),
        };

        let bit_iterator = BitIteratorBE::new(P::ATE_LOOP_COUNT_1);
        let mut ell_coeffs_1 = Vec::with_capacity(bit_iterator.len());

        for i in bit_iterator.skip(1) {
            ell_coeffs_1.push(doubling_step::<P>(&mut r));

            if i {
                ell_coeffs_1.push(addition_step::<P>(&mut r, &q));
            }
        }

        // f_{u^3-u^2-u,Q}(P)
        let mut ell_coeffs_2 = Vec::with_capacity(P::ATE_LOOP_COUNT_2.len());
        let mut r = G2HomProjective {
            x: q.x,
            y: q.y,
            z: P::Fp::one(),
        };

        let negq = q.neg();

        for i in (1..P::ATE_LOOP_COUNT_2.len()).rev() {
            ell_coeffs_2.push(doubling_step::<P>(&mut r));

            let bit = P::ATE_LOOP_COUNT_2[i - 1];
            match bit {
                1 => {
                    ell_coeffs_2.push(addition_step::<P>(&mut r, &q));
                }
                -1 => {
                    ell_coeffs_2.push(addition_step::<P>(&mut r, &negq));
                }
                _ => continue,
            }
        }

        Self {
            ell_coeffs_1,
            ell_coeffs_2,
            infinity: false,
        }
    }
}

impl<P: BW6Parameters> G2Prepared<P> {
    pub fn is_zero(&self) -> bool {
        self.infinity
    }
}

#[allow(clippy::many_single_char_names)]
fn doubling_step<B: BW6Parameters>(r: &mut G2HomProjective<B>) -> (B::Fp, B::Fp, B::Fp) {
    // Formula for line function when working with
    // homogeneous projective coordinates, as described in https://eprint.iacr.org/2013/722.pdf.

    let a = r.x * r.y;
    let b = r.y.square();
    let b4 = b.double().double();
    let c = r.z.square();
    let e = B::G2Parameters::COEFF_B * (c.double() + c);
    let f = e.double() + e;
    let g = b + f;
    let h = (r.y + r.z).square() - (b + c);
    let i = e - b;
    let j = r.x.square();
    let e2_square = e.double().square();

    r.x = a.double() * (b - f);
    r.y = g.square() - (e2_square.double() + e2_square);
    r.z = b4 * h;
    match B::TWIST_TYPE {
        TwistType::M => (i, j.double() + j, -h),
        TwistType::D => (-h, j.double() + j, i),
    }
}

#[allow(clippy::many_single_char_names)]
fn addition_step<B: BW6Parameters>(r: &mut G2HomProjective<B>, q: &G2Affine<B>) -> (B::Fp, B::Fp, B::Fp) {
    // Formula for line function when working with
    // homogeneous projective coordinates, as described in https://eprint.iacr.org/2013/722.pdf.
    let theta = r.y - (q.y * r.z);
    let lambda = r.x - (q.x * r.z);
    let c = theta.square();
    let d = lambda.square();
    let e = lambda * d;
    let f = r.z * c;
    let g = r.x * d;
    let h = e + f - g.double();
    r.x = lambda * h;
    r.y = theta * (g - h) - (e * r.y);
    r.z *= &e;
    let j = theta * q.x - (lambda * q.y);

    match B::TWIST_TYPE {
        TwistType::M => (j, -theta, lambda),
        TwistType::D => (lambda, -theta, j),
    }
}
