#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::vec::Vec;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use bitcoin_spv::{
    btcspv,
    types::{HeaderArray, MerkleArray, SPVError, Vin, Vout},
    validatespv,
};
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    debug, default_alloc, entry,
    error::SysError,
    high_level::{load_cell_data, load_witness_args},
};
use num::bigint::BigUint;

mod types;
use types::{Difficulty, DifficultyReader, SPVProof, SPVProofReader};

const TX_PROOF_DIFFICULTY_FACTOR: u8 = 6;

pub type RawBytes = Vec<u8>;

entry!(entry);
default_alloc!();

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

/// Error
#[repr(i8)]
enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    // Add customized errors here...
    WitnessInvalidEncoding,
    WitnessMissInputType,
    DifficultyDataInvalid,
    InvalidVin,
    InvalidVout,
    WrongTxId,
    SpvError,
    NotAtCurrentOrPreviousDifficulty,
    InsufficientDifficulty,
    BadMerkleProof,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl From<SPVError> for Error {
    fn from(_err: SPVError) -> Self {
        Self::SpvError
    }
}

fn parse_difficulty() -> Result<Difficulty, Error> {
    // TODO: get this index from witness args
    let difficulty_cell_dep_index = 1;
    let dep_data = load_cell_data(difficulty_cell_dep_index, Source::CellDep)?;
    debug!("dep data is {:?}", &dep_data);
    if DifficultyReader::verify(&dep_data, false).is_err() {
        return Err(Error::DifficultyDataInvalid);
    }
    let difficulty = Difficulty::new_unchecked(dep_data.into());
    Ok(difficulty)
}

/// parse proof from witness
fn parse_witness() -> Result<SPVProof, Error> {
    let witness_args = load_witness_args(0, Source::Input)?.input_type();
    if witness_args.is_none() {
        return Err(Error::WitnessMissInputType);
    }
    let witness_args: Bytes = witness_args.to_opt().unwrap().unpack();
    if SPVProofReader::verify(&witness_args, false).is_err() {
        return Err(Error::WitnessInvalidEncoding);
    }
    let proof = SPVProof::new_unchecked(witness_args.into());
    Ok(proof)
}

fn verify(proof: &SPVProof, difficulty: &Difficulty) -> Result<(), Error> {
    if !btcspv::validate_vin(proof.vin().as_slice()) {
        return Err(Error::InvalidVin);
    }
    if !btcspv::validate_vout(proof.vout().as_slice()) {
        return Err(Error::InvalidVout);
    }
    let mut ver = [0u8; 4];
    ver.copy_from_slice(proof.version().as_slice());
    let mut lock = [0u8; 4];
    lock.copy_from_slice(proof.locktime().as_slice());
    let tx_id = validatespv::calculate_txid(
        &ver,
        &Vin::new(proof.vin().as_slice())?,
        &Vout::new(proof.vout().as_slice())?,
        &lock,
    );
    if tx_id.as_ref() != proof.tx_id().as_slice() {
        return Err(Error::WrongTxId);
    }

    // verify difficulty
    let raw_headers = proof.headers();
    let headers = HeaderArray::new(raw_headers.as_slice())?;
    let observed_diff = validatespv::validate_header_chain(&headers, false)?;
    let previous_diff = BigUint::from_bytes_be(difficulty.previous().as_slice());
    let current_diff = BigUint::from_bytes_be(difficulty.current().as_slice());
    let first_header_diff = headers.index(0).difficulty();

    let req_diff = if first_header_diff == current_diff {
        current_diff
    } else if first_header_diff == previous_diff {
        previous_diff
    } else {
        return Err(Error::NotAtCurrentOrPreviousDifficulty);
    };

    if observed_diff < req_diff * TX_PROOF_DIFFICULTY_FACTOR {
        return Err(Error::InsufficientDifficulty);
    }

    // verify tx
    let header = headers.index(headers.len());
    let mut idx = [0u8; 8];
    idx.copy_from_slice(proof.index().as_slice());
    if !validatespv::prove(
        tx_id,
        header.tx_root(),
        &MerkleArray::new(proof.intermediate_nodes().as_slice())?,
        u64::from_le_bytes(idx),
    ) {
        return Err(Error::BadMerkleProof);
    }

    Ok(())
}

fn main() -> Result<(), Error> {
    let proof = parse_witness()?;
    let difficulty = parse_difficulty()?;
    verify(&proof, &difficulty)?;

    Ok(())
}
