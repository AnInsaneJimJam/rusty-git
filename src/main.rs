use std::{clone, env};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Consider writing a command after rgit")
    }
    let command = &args[2];
    match command.as_str() {
        "add" => cmd_add(&args),
        "cat-file" => cmd_cat_file(&args),
        "check-ignore" => cmd_check_ignore(&args),
        "checkout" => cmd_checkout(&args),
        "commit" => cmd_commit(&args),
        "hash-object" => cmd_hash_object(&args),
        "init" => cmd_init(&args),
        "log" => cmd_log(&args),
        "ls-files" => cmd_ls_files(&args),
        "ls-trees" => cmd_ls_trees(&args),
        "rev-parse" => cmd_rev_parse(&args),
        "rm" => cmd_rm(&args),
        "show-ref" => cmd_show_ref(&args),
        "status" => cmd_status(&args),
        "tag" => cmd_tag(&args),
        _ => println!("Bad Command!"),
    }

    fn cmd_add(args: &Vec<String>) {}
    fn cmd_cat_file(args: &Vec<String>) {}
    fn cmd_check_ignore(args: &Vec<String>) {}
    fn cmd_checkout(args: &Vec<String>) {}
    fn cmd_commit(args: &Vec<String>) {}
    fn cmd_hash_object(args: &Vec<String>) {}
    fn cmd_init(args: &Vec<String>) {}
    fn cmd_log(args: &Vec<String>) {}
    fn cmd_ls_files(args: &Vec<String>) {}
    fn cmd_ls_trees(args: &Vec<String>) {}
    fn cmd_rev_parse(args: &Vec<String>) {}
    fn cmd_rm(args: &Vec<String>) {}
    fn cmd_show_ref(args: &Vec<String>) {}
    fn cmd_status(args: &Vec<String>) {}
    fn cmd_tag(args: &Vec<String>) {}

}
