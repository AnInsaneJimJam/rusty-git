use std::env;

enum Command{
    Add,
    CatFile,
    CheckIgnore,
    Checkout,
    Commit,
    HashObject,
    Init,
    Log,
    LsFiles,
    LsTrees,
    RevParse,
    Rm,
    ShowRef,
    Status,
    Tag,
}

fn main() {
    let args: Vec<String> = env::args().collect();
}
