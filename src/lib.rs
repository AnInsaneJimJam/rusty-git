#![allow(warnings)]
use flate2::{read::ZlibDecoder,write::ZlibEncoder,Compression};
use ini::Ini;
use std::{
    env, fs, io::{Read, Write}, panic, path::{Path, PathBuf}
};
use sha1::{Sha1, Digest};

pub fn cmd_add() {}
pub fn cmd_check_ignore() {}
pub fn cmd_checkout() {}
pub fn cmd_commit() {}
pub fn cmd_log() {}
pub fn cmd_ls_files() {}
pub fn cmd_ls_trees() {}
pub fn cmd_rev_parse() {}
pub fn cmd_rm() {}
pub fn cmd_show_ref() {}
pub fn cmd_status() {}
pub fn cmd_tag() {}

struct GitRepository {
    worktree: PathBuf,
    gitdir: PathBuf,
    conf: Ini,
}

impl GitRepository {
    fn new(path: String, force: bool) -> Result<Self, String> {
        let worktree = PathBuf::from(path);
        let gitdir = worktree.join(".git");
        if (force == false && gitdir.is_dir() == false) {
            return Err(format!("Not a Git repository {:?}", worktree));
        }
        let mut repo = GitRepository {
            worktree,
            gitdir,
            conf: Ini::new(),
        };

        let cf = repo.repo_file("config", false);

        if let Some(config_path) = cf {
            if config_path.exists() {
                repo.conf = Ini::load_from_file(config_path).map_err(|e| e.to_string())?;
            } else if !force {
                return Err("Configuration file missing".to_string());
            }
        }
        if (force == false) {
            // Check for repositoryformatversion in the [core] section
            let section = repo
                .conf
                .section(Some("core"))
                .ok_or("Missing [core] section in config")?;

            let vers = section
                .get("repositoryformatversion")
                .ok_or("Missing repositoryformatversion")?;

            if vers != "0" {
                return Err(format!("Unsupported repositoryformatversion: {}", vers));
            }
        }

        Ok(repo)
    }

    // Compute path under repo's gitdir.
    fn repo_path(&self, path: &str) -> PathBuf {
        self.gitdir.join(path)
    }

    // Find the parent folder of the file needed to accessed. Then calls repo_dir with mkdir bool passed.
    // Ex: repo_file("hooks/pre-commit", true) -> uses repo_dir to make sure .git/hooks exsists. And return the full path to .git/hooks/pre-commit.
    pub fn repo_file(&self, path: &str, mkdir: bool) -> Option<PathBuf> {
        let target_path = self.repo_path(path);
        if let Some(parent) = target_path.parent() {
            if self.repo_dir(parent.to_str().unwrap(), mkdir).is_some() {
                return Some(target_path);
            }
        }
        None
    }

    /// Calculates the path using repo_path. If dir exsists => returns path. If does not exsist and mkdir is true. It makes the dir and all the parents folder
    /// repo_dir("refs/tags",true) -> creates .git/refs/tags even if refs doesn't exsist
    pub fn repo_dir(&self, path: &str, mkdir: bool) -> Option<PathBuf> {
        let p = self.repo_path(path);

        if p.exists() {
            if p.is_dir() {
                return Some(p);
            } else {
                panic!("Not a directory: {:?}", p);
            }
        }

        if mkdir {
            fs::create_dir_all(&p).expect("Failed to create directory");
            return Some(p);
        }

        None
    }
}

/// Initializes a new Git repository at the given path.
///
/// This function creates the `.git` directory structure, including:
/// - Subdirectories: `branches`, `objects`, `refs/tags`, and `refs/heads`.
/// - Files: `description`, `HEAD`, and `config`.
fn repo_create(path: String) -> Result<GitRepository, String> {
    let repo = GitRepository::new(path, true).expect("Unable to create a repo");
    let worktree = &repo.worktree;
    let gitdir = &repo.gitdir;
    if worktree.exists() {
        if !worktree.is_dir() {
            return Err(format!("{:?} is not a directory", worktree));
        }
        if gitdir.exists() && fs::read_dir(gitdir).expect("").count() != 0 {
            return Err(format!(" {:?} is not empty", worktree));
        }
    } else {
        fs::create_dir(&worktree).expect("Failed to create directory while running |repo_create|");
    }
    repo.repo_dir("branches", true);
    repo.repo_dir("objects", true);
    repo.repo_dir("refs/tags", true);
    repo.repo_dir("refs/heads", true);

    fs::write(
        repo.repo_file("description", false).unwrap(),
        "Unnamed repository; edit this file 'description' to name the repository.\n",
    );
    fs::write(
        repo.repo_file("HEAD", false).unwrap(),
        "ref: refs/heads/master\n",
    );
    let config_path = repo
        .repo_file("config", false)
        .expect("Coud not create path for config");
    let default_conf = repo_default_config();

    default_conf
        .write_to_file(config_path)
        .expect("Failed to write default config");
    return Ok(repo);
}

fn repo_default_config() -> Ini {
    let mut conf = Ini::new();

    conf.with_section(Some("core"))
        .set("repositoryformatversion", "0")
        .set("filemode", "false")
        .set("bare", "false");

    conf
}

pub fn cmd_init(path: &str) {
    repo_create(path.to_string()).expect("Failed to create repo");
}

/// Finds the .git file
fn repo_find(path: String, required: bool) -> Option<GitRepository> {
    let path_buf = fs::canonicalize(&path).expect("Failed to resolve path");
    if ((path_buf.join(".git")).is_dir()) {
        return Some(GitRepository::new(path, false).unwrap());
    }
    let parent = path_buf.parent();
    match parent {
        Some(parent) => repo_find(String::from(parent.to_str().unwrap()), required),
        None => {
            // None means root
            if required {
                panic!("No git directory");
            } else {
                None
            }
        }
    }
}

trait GitObject {
    fn new(&mut self, data: Option<String>) {
        match data {
            Some(data) => self.deserialize(data),
            None => self.init(),
        }
    }

    // Must be implemented by whom the trait is implied by
    fn serialize(&self, repo: &Option<GitRepository>) -> String {
        panic!("Implement me")
    }
    fn deserialize(&mut self, data: String) {}
    fn get_format(&self) -> &[u8]{
        panic!("Implement me")
    }
    fn init(&self){}
}

/// Read object sha from Git repository repo. Return a Gitobject whose exact type depends on object
fn object_read(repo: GitRepository, sha: String) -> Option<Vec<u8>> {
    let path = repo
        .repo_file(
            format!("objects/{}/{}", &sha[0..2], &sha[2..]).as_str(),
            false,
        )
        .unwrap();

    if !path.is_file() {
        return None;
    }

    let file = fs::File::open(path).expect("Should have been able to read file");
    let mut decoder = ZlibDecoder::new(file);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw).ok()?;

    // Read object type
    let x = raw.iter().position(|&b| b == b' ')?;
    let fmt = &raw[0..x];

    // Read and validate object size
    let y = raw.iter().position(|&b| b == b'\x00')?;
    let size = &raw[x + 1..y];
    let size = str::from_utf8(size).expect("Should have been able to convert u8 value to string");
    let size: usize = size
        .parse()
        .expect("Parse failed in converting string to usize");

    if size != raw.len() - y - 1 {
        panic!("Malformed object {}: bad length", sha);
    }

    let fmt = str::from_utf8(fmt).unwrap();
    match fmt {
        "commit" => todo!(),
        "tree" => todo!(),
        "tag" => todo!(),
        "blob" => todo!(),
        _ => panic!("Unknown type {} for object {}", fmt, sha),
    }
}

fn object_write(obj: &impl GitObject,repo: Option<GitRepository>) -> String{
    //Treating as option. Python is a dumbass language
    let data = obj.serialize(&repo);
    let mut result = Vec::new();
    result.extend_from_slice(obj.get_format());
    result.push(b' ');
    result.extend_from_slice(data.len().to_string().as_bytes());
    result.push(b'\0');
    result.extend_from_slice(data.as_bytes());

    let mut hasher = Sha1::new();
    hasher.update(&result);
    let sha  = format!("{:x}",hasher.finalize());

    match repo{
        Some(repo) => {
            let path = repo.repo_file(format!("objects/{}/{}", &sha[0..2], &sha[2..]).as_str(), true).unwrap();
            if !path.exists(){
            let file = std::fs::File::create(path).unwrap();
            let mut encoder = ZlibEncoder::new(file, Compression::default());
            encoder.write_all(&result).expect("Failed to compress");
            }
        },
        None => {}
    }
    sha
}

struct GitBlob{
    blobdata: String
}

impl GitObject for GitBlob{
    fn serialize(&self, repo: &Option<GitRepository>) -> String{
        self.blobdata.clone()
    }
    fn deserialize(&mut self, data: String) {
        self.blobdata = data.clone();
    }
    fn get_format(&self) -> &[u8] {
        return b"blob";
    }
}

// rgit cat-file TYPE OBJECT
pub fn cmd_cat_file(obj_type: &str, obj_sha: &str) {
    let repo = repo_find(".".to_string(), true).expect("Not a git repository");
    cat_file(repo, obj_sha, Some(obj_type));
}

// See later this function
fn cat_file(repo: GitRepository, object: &str, fmt: Option<&str>) {
    let sha = object_find(&repo, object, fmt, true);
    
    if let Some(data) = object_read(repo, sha) {
        std::io::stdout().write_all(&data).unwrap();
    }
    todo!()
}

fn object_find(repo: &GitRepository, name: &str, fmt: Option<&str>, follow: bool) -> String {
    name.to_string()
}

pub fn cmd_hash_object(type_:&str,write:&bool,path:&str) {
    let repo = if *write{
        repo_find(".".to_string(), true)
    }else{
        None
    };
    let data = fs::read_to_string(path).expect("Could not read file");
    let sha = object_hash(data,type_,repo);
    println!("{}",sha);

}

fn object_hash(data: String,fmt: &str,repo: Option<GitRepository>) -> String{
    match fmt {
        "blob" => {
            let mut obj = GitBlob { blobdata: data };
            object_write(&obj, repo)
        }
        "commit" => todo!(),
        "tree" => todo!(),
        "tag" => todo!(),
        _ => panic!("Unknown type {}!", fmt),
    }
}
