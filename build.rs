use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn sdk_include_path_for(sdk: &str) -> String {
    // sdk path find by `xcrun --sdk {iphoneos|macosx} --show-sdk-path`
    let output = Command::new("xcrun")
        .arg("--sdk")
        .arg(sdk)
        .arg("--show-sdk-path")
        .output()
        .expect("failed to execute xcrun");

    let inc_path = Path::new(String::from_utf8_lossy(&output.stdout).trim()).join("usr/include");

    inc_path.to_str().expect("invalid include path").to_string()
}

fn sdk_include_path() -> Option<String> {
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target = env::var("TARGET").unwrap();
    match os.as_str() {
        "ios" => {
            if target == "x86_64-apple-ios" || target == "aarch64-apple-ios-sim" {
                Some(sdk_include_path_for("iphonesimulator"))
            } else {
                Some(sdk_include_path_for("iphoneos"))
            }
        }
        "macos" => Some(sdk_include_path_for("macosx")),
        _ => None,
    }
}

fn compile_lwip() {
    println!("cargo:rerun-if-changed=src/lwip");
    println!("cargo:rerun-if-changed=src/customize");
    let mut build = cc::Build::new();
    build
        .file("src/lwip/src/core/init.c")
        .file("src/lwip/src/core/def.c")
        .file("src/lwip/src/core/inet_chksum.c")
        .file("src/lwip/src/core/ip.c")
        .file("src/lwip/src/core/mem.c")
        .file("src/lwip/src/core/memp.c")
        .file("src/lwip/src/core/netif.c")
        .file("src/lwip/src/core/pbuf.c")
        .file("src/lwip/src/core/raw.c")
        .file("src/lwip/src/core/tcp.c")
        .file("src/lwip/src/core/tcp_in.c")
        .file("src/lwip/src/core/tcp_out.c")
        .file("src/lwip/src/core/timeouts.c")
        .file("src/lwip/src/core/udp.c")
        .file("src/lwip/src/core/ipv4/icmp.c")
        .file("src/lwip/src/core/ipv4/ip4_frag.c")
        .file("src/lwip/src/core/ipv4/ip4.c")
        .file("src/lwip/src/core/ipv4/ip4_addr.c")
        .file("src/lwip/src/core/ipv6/icmp6.c")
        .file("src/lwip/src/core/ipv6/ip6.c")
        .file("src/lwip/src/core/ipv6/ip6_addr.c")
        .file("src/lwip/src/core/ipv6/ip6_frag.c")
        .file("src/lwip/src/core/ipv6/nd6.c")
        .file("src/customize/sys_arch.c")
        .include("src/customize")
        .include("src/lwip/src/include")
        .warnings(false)
        .flag_if_supported("-Wno-everything");
    if let Some(sdk_include_path) = sdk_include_path() {
        build.include(sdk_include_path);
    }

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    if profile == "release" {
        build.debug(false);
    } else {
        build.debug(true);
    }

    build.compile("liblwip.a");
}

fn generate_lwip_bindings() {
    println!("cargo:rustc-link-lib=lwip");
    // println!("cargo:rerun-if-changed=src/headerfiles.h.h");
    println!("cargo:include=src/lwip/src/include");

    let sdk_include_path = sdk_include_path();

    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mut builder = bindgen::Builder::default()
        .header("src/headerfiles.h")
        .size_t_is_usize(false)
        .clang_arg("-I./src/lwip/src/include")
        .clang_arg("-I./src/customize")
        .clang_arg("-Wno-everything")
        .layout_tests(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));
    if arch == "aarch64" && os == "ios" {
        // https://github.com/rust-lang/rust-bindgen/issues/1211
        builder = builder.clang_arg("--target=arm64-apple-ios");
    }
    if let Some(sdk_include_path) = sdk_include_path {
        builder = builder.clang_arg(format!("-I{}", sdk_include_path));
    }
    let bindings = builder.generate().expect("Unable to generate bindings");

    let mut out_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    out_path = out_path.join("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    if env::var("BINDINGS_GEN").is_ok() {
        generate_lwip_bindings();
    }
    compile_lwip();
}
