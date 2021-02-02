
use cmake;

#[cfg(windows)]
fn add_os_deps() {
    println!("cargo:rustc-link-lib={}", "Secur32");
    println!("cargo:rustc-link-lib={}", "Crypt32");
    println!("cargo:rustc-link-lib={}", "Advapi32");
    println!("cargo:rustc-link-lib={}", "BCrypt");
    println!("cargo:rustc-link-lib={}", "Kernel32");
    println!("cargo:rustc-link-lib={}", "Ws2_32");
    println!("cargo:rustc-link-lib={}", "Shlwapi");
}

#[cfg(windows)]
fn add_cmake_overrides(config: &mut cmake::Config) {}

#[cfg(target_vendor = "apple")]
fn add_os_deps() {
    println!("cargo:rustc-link-lib=framework={}", "CoreFoundation");
    println!("cargo:rustc-link-lib=framework={}", "Security");
}

#[cfg(target_vendor = "apple")]
fn add_cmake_overrides(config: &mut cmake::Config) {
    config.define(
        "CMAKE_OSX_SYSROOT",
        "PATH=/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk",
    );
}

#[cfg(all(unix, not(target_vendor = "apple")))]
fn add_os_deps() {
    println!("cargo:rustc-link-lib={}", "s2n");
    println!("cargo:rustc-link-lib={}", "crypto");
    println!("cargo:rustc-link-lib={}", "rt");
}

#[cfg(all(unix, not(target_vendor = "apple")))]
fn add_cmake_overrides(config: &mut cmake::Config) {}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut config = cmake::Config::new("crt/aws-crt-ffi");
    config.profile("RelWithDebInfo")
        //.define("CMAKE_PREFIX_PATH", "build/install")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("BUILD_TESTING", "OFF")
        .out_dir(&out_dir);
    add_cmake_overrides(&mut config);

    config.build();

    println!("cargo:rustc-link-search={}/build", out_dir);
    println!("cargo:rustc-link-lib={}", "aws-crt-ffi");
    add_os_deps();
}
