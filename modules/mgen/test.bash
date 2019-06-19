

SYSROOT=`rustc --print sysroot`
cargo run -- --sysroot "$SYSROOT" example/foo1.rs

#rustup run nightly target/debug/igen --sysroot "$SYSROOT" example/foo1.rs
