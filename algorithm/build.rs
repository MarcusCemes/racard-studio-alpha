use std::env::var;

const ENV_VARS: [&str; 4] = [
    "CARGO_CFG_TARGET_ARCH",
    "CARGO_CFG_TARGET_ENV",
    "CARGO_CFG_TARGET_OS",
    "CARGO_CFG_TARGET_VENDOR",
];

fn main() {
    for env in ENV_VARS {
        if let Ok(value) = var(env) {
            println!("cargo:rustc-env={env}={value}");
        }
    }
}
