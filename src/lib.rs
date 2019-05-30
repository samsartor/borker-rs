#![feature(slice_concat_ext)]

#[macro_use]
extern crate failure;

use failure::Error;
use serde::{Deserialize, Serialize};

mod big_array;
#[macro_use]
mod macros;
mod protocol;
mod wallet;

pub use self::wallet::{ChildWallet, Wallet};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockData<'a> {
    borker_txs: Vec<protocol::BorkTxData<'a>>,
    spent: Vec<protocol::UtxoId>,
    created: Vec<protocol::NewUtxo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    address: String,
    value: u64,
}
impl Output {
    pub fn as_tup(&self) -> (&str, u64) {
        (self.address.as_str(), self.value)
    }
}

#[derive(Clone, Copy)]
pub enum Network {
    Dogecoin,
    Litecoin,
    Bitcoin,
}

#[allow(non_snake_case)]
pub fn processBlock<T>(block: String, network: Network, process: impl FnOnce(&BlockData) -> Result<T, Error>) -> Result<T, Error> {
    use bitcoin::consensus::encode::Decodable;

    let block = hex::decode(&block)?;
    let mut cur = std::io::Cursor::new(&block);
    let block_header: bitcoin::BlockHeader = Decodable::consensus_decode(&mut cur)?;
    match network {
        Network::Dogecoin | Network::Litecoin if block_header.version & 1 << 8 != 0 => {
            let _: bitcoin::Transaction = Decodable::consensus_decode(&mut cur)?;
            let pos = cur.position() + 32;
            cur.set_position(pos);
            let len: bitcoin::VarInt = Decodable::consensus_decode(&mut cur)?;
            let pos = cur.position() + 32 * len.0;
            cur.set_position(pos + 4);

            let len: bitcoin::VarInt = Decodable::consensus_decode(&mut cur)?;
            let pos = cur.position() + 32 * len.0;
            cur.set_position(pos + 4);
            let _: bitcoin::BlockHeader = Decodable::consensus_decode(&mut cur)?;
        }
        _ => (),
    }

    let count: bitcoin::VarInt = Decodable::consensus_decode(&mut cur)?;
    let timestamp = chrono::DateTime::from_utc(
        chrono::NaiveDateTime::from_timestamp(block_header.time as i64, 0),
        chrono::Utc,
    );
    let mut block_data = BlockData {
        borker_txs: Vec::new(),
        spent: Vec::new(),
        created: Vec::new(),
    };
    for _ in 0..count.0 {
        let (bork, spent, created) = protocol::parse_tx(
            Decodable::consensus_decode(&mut cur)?,
            &timestamp,
            network,
        );
        if let Some(bork) = bork {
            block_data.borker_txs.push(bork);
        }
        block_data.spent.extend(spent);
        block_data.created.extend(created);
    }
    process(&block_data)
}
