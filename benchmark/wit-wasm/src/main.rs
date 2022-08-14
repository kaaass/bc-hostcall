wit_bindgen_rust::import!("imports.wit");
wit_bindgen_rust::export!("exports.wit");

use imports::*;

struct Exports;

impl exports::Exports for Exports {
    fn do_service(url: String) -> String {
        let result = http_get(&url);

        result
    }
}

fn main() {
}
