extern crate core;

use core::resource::ResCache;
use core::utils::serialize;
use std::env;

fn main() {
    if env::args().len() != 3 {
        println!("Usage: compiler <resource.yaml>");
        return;
    }
    let res_file = env::args().nth(1).unwrap();
    let id_file = env::args().nth(2).unwrap();

    let cache = ResCache::compile(&res_file).unwrap();
    let table = cache.id_table();
    serialize(id_file, table).unwrap();

    println!("Compile resource success");
}
