use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub fn answer_to_life() -> i32 {
    42
}

#[wasm_bindgen]
pub fn show_alert(world: &str) {
    alert(&format!("Answer is {}", world));
}

#[wasm_bindgen]
pub fn do_panic() {
    panic!("panicking")
}
