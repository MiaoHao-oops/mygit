use std::fs::{File, self, DirBuilder};
use std::io::{Read, Write, self, ErrorKind};
use std::env;
use std::process::exit;
use std::path::PathBuf;
use std::ffi::OsString;
use sha1::{Sha1, Digest};

fn main() {
    let args: Vec<String> = env::args().collect();
    let repo_root = get_repo_root();

    // parse arguments
    if args.len() > 1 {
        let cmd = args[1].as_str();
        match cmd {
            "init" => {
                if let Err(_) = repo_root {
                    // if .mygit dose not exist in the ancestors of 
                    // current directory, initialize the repository.
                    init();
                }
            },
            "add" => {
                if let Err(_) = repo_root {
                    println!("this is not a mygit repository!");
                    exit(1);
                } else if let Ok(repo_root) = repo_root {
                    // if .mygit exists in the ancestors of current
                    // directory, extract it from Result<>.
                    if args.len() > 2 {
                        add(args[2].as_str(), repo_root.to_str().unwrap());
                    } else {
                        println!("lack of arguments!");
                        exit(1);
                    }
                }
            },
            _ => {
                println!("unknown command!");
                exit(1);
            }
        }
    } else {
        println!("usage: mygit <command> [<args>]");
        exit(1);
    };
}

fn file_exist(file_path: &str) -> bool {
    match File::open(file_path) {
        Ok(_) => true,
        Err(_)=> false
    }
}

// thanks to crate project-root!
fn get_repo_root() -> io::Result<PathBuf> {
    let path = env::current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_mygit =
            fs::read_dir(p)?
                .into_iter()
                .any(|p| p.unwrap().file_name() == OsString::from(".mygit"));
        if has_mygit {
            return Ok(PathBuf::from(p))
        }
    }
    Err(io::Error::new(ErrorKind::NotFound, "Ran out of places to find .mygit"))
}

fn init() {
    // create directory .mygit and sub directories
    println!("Initialize mygit by creating directory ./.mygit");
    DirBuilder::new()
    .recursive(true)
    .create("./.mygit").unwrap();
    DirBuilder::new()
    .recursive(true)
    .create("./.mygit/objects").unwrap();
}

fn get_sha1(buf: &Vec<u8>) -> String {
    let hex_digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let mut hash = String::new();
    let mut hasher = Sha1::new();

    hasher.update(&buf);
    let result = hasher.finalize();

    for i in result {
        hash.push(hex_digits[(i & 0xf) as usize]);
        hash.push(hex_digits[((i & 0xf0) >> 4) as usize]);
    }
    hash
}

fn add(path: &str, repo_root: &str) {
    // ignore mygit metadata
    if path == "./.mygit" || path == "./target" || path == "./.git" {
        return;
    }

    let mut f = File::open(path).expect("open file error");
    if f.metadata().unwrap().is_file() {
        let mut content: Vec<u8> = Vec::new();

        f.read_to_end(&mut content).unwrap();
        let hash = String::from(get_sha1(&content));

        // if the blob of the file dose not exist, create it
        if !blob_exist(&hash, repo_root) {
            mkblob(&content, &hash, repo_root);
        }
        return;
    }

    let dir = fs::read_dir(path).expect("open current directory error!");

    for file in dir {
        let file = file.unwrap();
        let file_name = String::from(file.path().to_str().unwrap());

        // traverse project directory tree recursively
        add(&file_name, repo_root);
    }
}

fn gen_blob_path(repo_root: &str, blob_name: &str) -> String {
    let blob_path = String::from(repo_root);
    let blob_path = blob_path + "/.mygit/objects" + blob_name;
    blob_path
}

fn blob_exist(hash: &str, repo_root: &str) -> bool {
    let blob_path = gen_blob_path(repo_root, hash);
    file_exist(&blob_path)
}

fn mkblob(content: &Vec<u8>, hash: &str, repo_root: &str) {
    let blob_path = gen_blob_path(repo_root, hash);
    let mut blob = File::create(&blob_path).expect("blob create error!");

    blob.write("blob ".as_bytes()).unwrap();
    blob.write(content.len().to_string().as_bytes()).unwrap();
    blob.write('\0'.to_string().as_bytes()).unwrap();
    blob.write(&content).unwrap();
}
