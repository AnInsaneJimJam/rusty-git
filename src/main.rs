#![allow(warnings)]
use flate2::read::ZlibDecoder;
use ini::Ini;
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
};

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
    fn cmd_log(args: &Vec<String>) {}
    fn cmd_ls_files(args: &Vec<String>) {}
    fn cmd_ls_trees(args: &Vec<String>) {}
    fn cmd_rev_parse(args: &Vec<String>) {}
    fn cmd_rm(args: &Vec<String>) {}
    fn cmd_show_ref(args: &Vec<String>) {}
    fn cmd_status(args: &Vec<String>) {}
    fn cmd_tag(args: &Vec<String>) {}
}

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

fn cmd_init(args: &Vec<String>) {
    if (args.len() == 4) {
        repo_create(args[3].clone());
    } else if (args.len() == 3) {
        repo_create(String::from(".")).expect("Not able to create repo in source");
    } else {
        panic!("Too little or too many arguements");
    }
}

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
    fn new(&self, data: Option<String>) {
        match data {
            Some(data) => self.deserialize(data),
            None => self.init(),
        }
    }

    // Must be implemented by whom the trait is implied by
    fn serialize(&self, repo: GitRepository) {}
    fn deserialize(&self, data: String) {}
    fn init(&self) {}
}

/// Read object sha from Git repository repo. Return a Gitobject whose exact type depends on object
fn object_read(repo: GitRepository, sha: &str) -> Option<Vec<u8>> {
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
