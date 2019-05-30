use wasm_bindgen::prelude::*;
use borker_js_macros::classy;

// Odd mix of javascript/typescript/rust that is parsed by our proc-macro and
// turned into a JS snippet + typescript_custom_section + Rust bindings.
classy! {
    abstract class Bork { };

    class StandardBork extends Bork {
        content: String;
        nonce: u8;

        constructor(content: &str, nonce: u8) {
            super();
            this.content = content;
            this.nonce = nonce;
        };
    };

    abstract class ReferBork extends StandardBork {
        reference: Vec<u8>;

        constructor(content: &str, nonce: u8, reference: &[u8]) {
            super(content, nonce);
            this.reference = reference;
        };
    };

    class Comment extends ReferBork {
        constructor(content: &str, nonce: u8, reference: &[u8]) {
            super(content, nonce, reference);
        };
    };

    class Rebork extends ReferBork {
        constructor(content: &str, nonce: u8, reference: &[u8]) {
            super(content, nonce, reference);
        };
    };
}

/// Temporary function just to explore possible new lib structure.
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
