use wasm_bindgen::prelude::*
use borker_js_macros::classy;

// Odd mix of javascript/typescript/rust that is parsed by our proc-macro and
// turned into a JS snippet + typescript_custom_section + Rust bindings.
classy! {
    class AnyBork { };

    class Bork extends AnyBork {
        text: String;

        constructor(text: &str) {
            this.text = text;
        };
    };

    class Comment extends Bork {
        text: String;
        reference: Vec<u8>;

        constructor(text: &str, reference: &[u8]) {
            super(text);
            this.reference = reference;
        };
    };
}
