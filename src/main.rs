#![allow(warnings)]
use clap::{Parser, Subcommand};
use flate2::{read::ZlibDecoder,write::ZlibEncoder,Compression};
use ini::Ini;
use std::{
    env, fs, io::{Read, Write}, panic, path::{Path, PathBuf}
};
use sha1::{Sha1, Digest};
use rgit::{cmd_init,cmd_cat_file,cmd_hash_object,cmd_add,cmd_check_ignore,cmd_checkout,cmd_commit,cmd_log,cmd_ls_files,cmd_ls_trees,cmd_rev_parse,cmd_rm,cmd_show_ref,cmd_status,cmd_tag};

#[derive(Parser)]
#[command(name = "rgit")]
#[command(about = "A Rust implementation of Git", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
    CatFile {
        #[arg(name = "TYPE",value_parser = ["blob", "commit", "tag", "tree"])]
        type_: String,
        object: String,
    },
    HashObject {
        #[arg(short = 't', long = "TYPE", default_value = "blob", value_parser = ["blob", "commit", "tag", "tree"])]
        type_: String,
        #[arg(short = 'w')]
        write: bool,
        path: String,
    },
    Add,
    CheckIgnore,
    Checkout,
    Commit,
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
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path } => cmd_init(path),
        Commands::CatFile { type_, object } => cmd_cat_file(type_, object),
        Commands::HashObject { type_, write, path } => cmd_hash_object(),
        Commands::Add => cmd_add(),
        Commands::CheckIgnore => cmd_check_ignore(),
        Commands::Checkout => cmd_checkout(),
        Commands::Commit => cmd_commit(),
        Commands::Log => cmd_log(),
        Commands::LsFiles => cmd_ls_files(),
        Commands::LsTrees => cmd_ls_trees(),
        Commands::RevParse => cmd_rev_parse(),
        Commands::Rm => cmd_rm(),
        Commands::ShowRef => cmd_show_ref(),
        Commands::Status => cmd_status(),
        Commands::Tag => cmd_tag(),
    }
}
