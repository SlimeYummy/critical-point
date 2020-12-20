extern crate core;
extern crate serde_yaml;

use core::resource::ResCache;
use std::env;
use std::fs;

fn main() {
    if env::args().len() != 3 {
        println!("Usage: compiler <resource.yaml>");
        return;
    }
    let res_file = env::args().nth(1).unwrap();
    let id_file = env::args().nth(2).unwrap();

    let cache = ResCache::compile(&res_file).unwrap();
    let table = cache.id_table();
    let yaml = serde_yaml::to_string(table).unwrap();
    fs::write(&id_file, yaml).unwrap();

    println!("Compile resource success");
}
