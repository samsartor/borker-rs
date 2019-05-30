use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../../lib/Bork")] // This is  in./bindings/pkg-*/
extern "C" {
    pub type Bork;

    #[wasm_bindgen(extends = Bork)]
    pub type StandardBork;

    #[wasm_bindgen(constructor)]
    pub fn new(content: &str, nonce: u8) -> StandardBork;

    #[wasm_bindgen(method, getter)]
    pub fn content(_: &StandardBork) -> String;

    #[wasm_bindgen(method, setter)]
    pub fn set_content(_: &StandardBork, content: &str);

    #[wasm_bindgen(method, getter)]
    pub fn nonce(_: &StandardBork) -> u8;

    #[wasm_bindgen(method, setter)]
    pub fn set_nonce(_: &StandardBork, nonce: u8);

    #[wasm_bindgen(extends = StandardBork)]
    pub type ReferBork;

    #[wasm_bindgen(method, getter = referenceId)]
    pub fn reference(_: &ReferBork) -> Vec<u8>;

    #[wasm_bindgen(method, setter = referenceId)]
    pub fn set_reference(_: &ReferBork, id: &[u8]);

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

#[wasm_bindgen]
pub fn encode(bork: Bork) -> Vec<u8> {
    use borker_rs::protocol::{encode, NewBork};
    use wasm_bindgen::JsCast;

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
        if let Some(bork) = bork.dyn_ref::<StandardBork>() {
            return encode(NewBork::Bork {
                content: bork.content(),
            }, bork.nonce());
        }
        panic!("bork not encodable");
    };

    encoder()
        .expect("could not encode")
        .into_iter()
        .next()
        .unwrap()
}
