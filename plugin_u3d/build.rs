use std::fs;

fn main() {
    let _ = fs::remove_file("../target/FFIData.cs");
    let _ = fs::copy("./src/FFIData.cs", "../target/FFIData.cs");
    let _ = fs::remove_file("../target/FFIAgent.cs");
    let _ = fs::copy("./src/FFIAgent.cs", "../target/FFIAgent.cs");
}
