extern crate core;

use core::resource::ResCache;
use core::utils::serialize;
use std::env;
use std::path::PathBuf;

fn main() {
    if env::args().len() != 4 {
        println!("Usage: compiler <./root/path> <resource.yml> <id.yml>");
        return;
    }
    let root_path = env::args().nth(1).unwrap();
    let res_file = env::args().nth(2).unwrap();
    let id_file = env::args().nth(3).unwrap();

    let cache = ResCache::compile(&root_path, &res_file).unwrap();
    let table = cache.id_table();

    let mut id_path = PathBuf::from(root_path);
    id_path.push(id_file);
    serialize(id_path, table).unwrap();

    println!("Compile resource success");
}
