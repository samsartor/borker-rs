use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::{Array, Uint8Array, Error};
use failure::{bail};

#[wasm_bindgen(raw_module = "../../lib/Bork")] // This is  in./bindings/pkg-*/
extern "C" {
    pub type Bork;

    #[wasm_bindgen(extends = Bork)]
    pub type StandardBork;

    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, nonce: u8) -> StandardBork;

    #[wasm_bindgen(method, getter)]
    pub fn content(_: &StandardBork) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn nonce(_: &StandardBork) -> u8;

    #[wasm_bindgen(extends = StandardBork)]
    pub type Extension;

    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, nonce: u8, index: u8) -> Extension;

    #[wasm_bindgen(method, getter)]
    pub fn index(_: &Extension) -> u8;

    #[wasm_bindgen(extends = StandardBork)]
    pub type ReferBork;

    #[wasm_bindgen(method, getter = referenceId)]
    pub fn reference(_: &ReferBork) -> Vec<u8>;

    #[wasm_bindgen(extends = ReferBork)]
    pub type Comment;

    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, nonce: u8, ref_id: &[u8]) -> Comment;

    #[wasm_bindgen(extends = ReferBork)]
    pub type Rebork;

    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, nonce: u8, ref_id: &[u8]) -> Rebork;
}

#[wasm_bindgen]
pub fn magic_num() -> Vec<u8> {
    use borker_rs::protocol::MAGIC;

    MAGIC.to_vec()
}

fn bytes_to_array(bytes: &[u8]) -> Uint8Array {
    let tmp = unsafe { Uint8Array::view(bytes) };
    tmp.slice(0, bytes.len() as u32)
}

#[wasm_bindgen]
pub fn encode(bork: Bork, parts: Array) -> Result<(), JsValue> {
    use borker_rs::protocol::{encode, NewBork};

    let encoder = || {
        if let Some(bork) = bork.dyn_ref::<Comment>() {
            return encode(NewBork::Comment {
                content: bork.content(),
                reference_id: bork.reference(),
            }, bork.nonce());
        }
        if let Some(bork) = bork.dyn_ref::<Rebork>() {
            return encode(NewBork::Rebork {
                content: bork.content(),
                reference_id: bork.reference(),
            }, bork.nonce());
        }
        if let Some(_) = bork.dyn_ref::<Extension>() {
            bail!("can not encode extension borks yet");
        }
        if let Some(bork) = bork.dyn_ref::<StandardBork>() {
            return encode(NewBork::Bork {
                content: bork.content(),
            }, bork.nonce());
        }
        bail!("unsupported bork type: {:?}", *bork);
    };

    let encoded = match encoder() {
        Ok(e) => e,
        Err(msg) => return Err(Error::new(&msg.to_string()).into()),
    };

    for bytes in encoded {
        parts.push(&bytes_to_array(&bytes));
    }

    Ok(())
}

#[wasm_bindgen]
pub fn decode_block(bytes: &[u8], network: usize, borks: Array) -> Result<(), JsValue> {
    use borker_rs::{process_block, Network, BlockData};
    use borker_rs::protocol::BorkType;

    let network = match network {
        1 => Network::Dogecoin,
        2 => Network::Litecoin,
        3 => Network::Bitcoin,
        _ => return Err(Error::new(&format!("network {} is undefined", network)).into()),
    };

    let process = |data: &BlockData| {
        for tx in &data.borker_txs {
            match tx.bork_type {
                BorkType::Bork => borks.push(&StandardBork::new(
                    tx.content.as_ref().unwrap(),
                    tx.nonce.unwrap(),
                ).into()),
                BorkType::Comment => borks.push(&Comment::new(
                    tx.content.as_ref().unwrap(),
                    tx.nonce.unwrap(),
                    tx.reference_id.as_ref().unwrap().as_bytes(),
                ).into()),
                BorkType::Rebork => borks.push(&Rebork::new(
                    tx.content.as_ref().unwrap(),
                    tx.nonce.unwrap(),
                    tx.reference_id.as_ref().unwrap().as_bytes(),
                ).into()),
                BorkType::Extension => borks.push(&Extension::new(
                    tx.content.as_ref().unwrap(),
                    tx.nonce.unwrap(),
                    tx.index.unwrap(),
                ).into()),
                _ => 0,
            };
        }
        Ok(())
    };

    match process_block(bytes, network, process) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::new(&err.to_string()).into()),
    }
}
