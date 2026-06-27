mod scan;

use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").expect("HOME not set");
    let data_path = PathBuf::from(home).join("Data");
    let nodes = scan::scan_folder(&data_path);

    for node in &nodes {
        println!("{}", node);
    }

    println!("Total: {} nodes", nodes.len());
}
