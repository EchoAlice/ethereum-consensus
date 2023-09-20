#![allow(unused)]
use crate::{crypto::hash, primitives, ssz::prelude::ByteVector};
use alloy_primitives::{uint, U256};
use blst::min_pk::PublicKey;
use c_kzg::{Bytes32, Bytes48, Error, KzgSettings};
use ssz_rs::prelude::*;
use std::ops::Deref;

pub const BLS_MODULUS: U256 =
    uint!(52435875175126190479447740508185965837690552500527637822603658699938581184513_U256);
pub const BYTES_PER_BLOB: usize = 32 * 4096;
pub const BYTES_PER_CONTEXT: usize = 10;
pub const BYTES_PER_COMMITMENT: usize = 48;
pub const BYTES_PER_FIELD_ELEMENT: usize = 32;
pub const BYTES_PER_PROOF: usize = 48;
pub const KZG_COMMITMENT_BYTES_LEN: usize = 48;
pub const KZG_PROOF_BYTES_LEN: usize = 48;

pub type VersionedHash = primitives::Bytes32;
pub type BLSFieldElement = U256;
pub type Polynomial = Vec<BLSFieldElement>; // Should this polynomial type be an array?

const fn create_g1_point_at_infinity() -> [u8; 48] {
    let mut arr: [u8; 48] = [0; 48];
    arr[0] = 0xc0;
    arr
}

/// TODO:  Lean on C-KZG library to implement specs.

pub struct Blob(ByteVector<BYTES_PER_BLOB>);

#[derive(SimpleSerialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KzgCommitment(ByteVector<BYTES_PER_COMMITMENT>);

impl Deref for KzgCommitment {
    type Target = ByteVector<BYTES_PER_COMMITMENT>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(SimpleSerialize, Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KzgProof(ByteVector<BYTES_PER_PROOF>);

/// Uses multicalar multiplication to combine trusted setup with blob
fn blob_to_kzg_commitment(blob: Blob, kzg_settings: &KzgSettings) -> Result<KzgCommitment, Error> {
    let inner = &blob.0;
    let blob = c_kzg::Blob::from_bytes(inner.as_ref()).unwrap();

    let commitment = c_kzg::KzgCommitment::blob_to_kzg_commitment(&blob, kzg_settings)?;
    let inner = ByteVector::try_from(commitment.to_bytes().as_slice()).unwrap();

    Ok(KzgCommitment(inner))
}

/// Compute KZG proof at point `z` for the polynomial represented by 'blob'.
/// Do this by computing the quotient polynomial in evaluation form: q(x) = (p(x) - p(z)) / (x - z).
/// Returns combined proof and the evaluation of the polynomial.
fn compute_kzg_proof(
    blob: Blob,
    z_bytes: Bytes32,
    kzg_settings: &KzgSettings,
) -> Result<(KzgProof, Bytes32), Error> {
    let bytes = blob.0.as_ref();
    let blob = c_kzg::Blob::from_bytes(bytes).unwrap();

    let (ckzg_proof, evaluation) =
        c_kzg::KzgProof::compute_kzg_proof(&blob, &z_bytes, kzg_settings)?;

    let bytes_proof = ckzg_proof.to_bytes();
    let proof = ByteVector::try_from(bytes_proof.as_ref()).unwrap();

    Ok((KzgProof(proof), evaluation))
}

/// Given a blob and a commitment, return the KZG proof that is used to verify it against the
/// commitment.  This function doesn't verify that the commitment is correct with respect to the blob.
fn compute_blob_kzg_proof(
    blob: Blob,
    commitment_bytes: Bytes48,
    kzg_settings: &KzgSettings,
) -> Result<KzgProof, Error> {
    let bytes = blob.0.as_ref();
    let blob = c_kzg::Blob::from_bytes(bytes).unwrap();

    let ckzg_proof =
        c_kzg::KzgProof::compute_blob_kzg_proof(&blob, &commitment_bytes, kzg_settings)?;

    let bytes_proof = ckzg_proof.to_bytes();
    let proof = ByteVector::try_from(bytes_proof.as_ref()).unwrap();

    Ok(KzgProof(proof))
}

fn verify_kzg_proof() {}
fn verify_blob_kzg_proof() {}
fn verify_blob_kzg_proof_batch() {}
