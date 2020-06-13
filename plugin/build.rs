extern crate bindgen;
extern crate regex;

use regex::Regex;
use std::fs;
use std::path::PathBuf;

fn main() {
    gen_bullet_bindings();
}

fn gen_bullet_bindings() {
    println!("cargo:rerun-if-changed=./src_cpp/bullet.h");
    println!("cargo:rerun-if-changed=./src_cpp/bullet.cpp");

    println!("cargo:rustc-link-lib=./bullet3/lib/Release/LinearMath");
    println!("cargo:rustc-link-lib=./bullet3/lib/Release/Bullet3Common");
    println!("cargo:rustc-link-lib=./bullet3/lib/Release/BulletCollision");
    println!("cargo:rustc-link-lib=./bullet3/lib/Release/BulletDynamics");

    cc::Build::new()
        .include("../bullet3/src")
        .file("./src_cpp/bullet.cpp")
        .static_crt(true)
        .compile("bullet_cpp");

    let bindings = bindgen::Builder::default()
        .clang_args(&vec!["-x", "c++"])
        .header("./src_cpp/bullet.h")
        .whitelist_type("bt.*")
        .whitelist_function("bth.*")
        .generate_comments(false)
        .generate_inline_functions(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate C++ bindings");
    bindings
        .write_to_file(PathBuf::from("./src/cpp/bullet.rs"))
        .expect("Couldn't write C++ bindings!");

    let re = Regex::new("_Dbt(\\w+)@@QEAAXXZ").unwrap();
    let bullet = fs::read_to_string("./src/cpp/bullet.rs").unwrap();
    let norm_bullet = re.replace_all(&bullet, "1bt$1@@UEAA@XZ").into_owned();
    fs::write("./src/cpp/bullet.rs", norm_bullet).unwrap();
}
