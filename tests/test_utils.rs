use fdtd_wasm::utils::set_panic_hook;

#[test]
fn test_set_panic_hook_does_not_crash() {
    // This function sets a global hook. We just want to ensure calling it doesn't panic.
    set_panic_hook();
}
