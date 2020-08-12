#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::{vec, vec::Vec};

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    ckb_constants::Source,
    entry,
    default_alloc,
    debug,
    high_level::{load_script, load_tx_hash, load_cell_data_hash, load_cell_data, load_witness_args},
    error::SysError,
    ckb_types::{bytes::Bytes, prelude::*},
};
use num::bigint::BigUint;
use bitcoin_spv::types::{RawHeader, Hash256Digest, HeaderArray};

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

pub struct BitcoinHeader {
    /// The double-sha2 digest encoded BE.
    pub hash: Hash256Digest,
    /// The 80-byte raw header.
    pub raw: RawHeader,
    /// The height of the header
    pub height: u32,
    /// The double-sha2 digest of the parent encoded BE.
    pub prevhash: Hash256Digest,
    /// The double-sha2 merkle tree root of the block transactions encoded BE.
    pub merkle_root: Hash256Digest,
}

pub struct SPVProof {
    /// The 4-byte LE-encoded version number. Currently always 1 or 2.
    pub version: RawBytes,
    /// The transaction input vector, length-prefixed.
    pub vin: RawBytes,
    /// The transaction output vector, length-prefixed.
    pub vout: RawBytes,
    /// The 4-byte LE-encoded locktime number.
    pub locktime: RawBytes,
    /// The tx id
    pub tx_id: Hash256Digest,
    /// The transaction index
    pub index: u32,
    /// The confirming Bitcoin header
    pub headers: Vec<BitcoinHeader>,
    /// The intermediate nodes (digests between leaf and root)
    pub intermediate_nodes: RawBytes,
}

pub struct Difficulty {
    pub current: BigUint,
    pub previous: BigUint,
}

fn parse_difficulty(data: &[u8]) -> Result<Difficulty, Error> {
    todo!()
}

fn parse_witness(witness: &[u8]) -> Result<SPVProof, Error> {
    todo!()
}

fn verify(proof: &SPVProof, difficulty: &Difficulty) -> Result<(), Error> {
    todo!()
}

fn main() -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::Input)?;
    let witness = witness_args.input_type();
    debug!("witness args is {:?}", &witness);

    let proof = parse_witness(witness.as_slice())?;

    // TODO: get this index from witness args
    let difficulty_cell_dep_index = 1;
    let dep_data = load_cell_data(difficulty_cell_dep_index, Source::CellDep)?;
    debug!("dep data is {:?}", &dep_data);

    let difficulty = parse_difficulty(&dep_data)?;
    verify(&proof, &difficulty)?;
    Ok(())
}
