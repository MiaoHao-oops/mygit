use std::io::{self, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::fs::{self, File, DirBuilder};
use std::env;
use std::process::exit;
use std::ffi::OsString;
use sha1::{Sha1, Digest};

enum ErrState {
    AddFailed,
    BadUse,
    InitFailed,
    LackOfArg,
    NotARepo,
    UnknownCmd,
}

pub struct MyGit {
    pub repo_path: io::Result<PathBuf>
}

impl MyGit {
    pub fn default() -> Self {
        MyGit { repo_path: (Self::get_repo_root()) }
    }

    pub fn run(&self) {
        let args: Vec<String> = env::args().collect();

        match self.parse_args(&args) {
            Ok(_) => exit(0),
            Err(err) => {
                match err {
                    ErrState::AddFailed => println!("Add file failed!"),
                    ErrState::BadUse => println!("usage: mygit <command> [<args>]"),
                    ErrState::InitFailed => println!("Initialize failed!"),
                    ErrState::LackOfArg => println!("Lack of arguments!"),
                    ErrState::NotARepo => println!("Not a mygit repository!"),
                    ErrState::UnknownCmd => println!("Unknown command!"),
                }
                exit(1)
            },
        }
    }

    fn parse_args(&self, args: &Vec<String>) -> Result<(), ErrState> {
        if args.len() > 1 {
            let cmd = args[1].as_str();
            match cmd {
                "init" => {
                    self.exec_init()
                },
                "add" => {
                    self.exec_add(args)
                },
                _ => {
                    Err(ErrState::UnknownCmd)
                }
            }
        } else {
            Err(ErrState::BadUse)
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

    fn file_exist(file_path: &str) -> bool {
        match File::open(file_path) {
            Ok(_) => true,
            Err(_)=> false,
        }
    }

    fn exec_init(&self) -> Result<(), ErrState> {
        match self.init() {
            Ok(_) => Ok(()),
            Err(_) => Err(ErrState::InitFailed),
        }
    }

    fn exec_add(&self, args: &Vec<String>) -> Result<(), ErrState> {
        if let Err(_) = &self.repo_path {
            Err(ErrState::NotARepo)
        } else {
            if args.len() > 2 {
                match self.add(args[2].as_str()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(ErrState::AddFailed),
                }
            } else {
                Err(ErrState::LackOfArg)
            }
        }
    }

    fn init(&self) -> io::Result<()> {
        // if .mygit does not exist, create directory .mygit and sub directories
        if let Err(_) = self.repo_path {
            println!("Initialize mygit by creating directory ./.mygit");
            DirBuilder::new()
            .recursive(true)
            .create("./.mygit")?;
            DirBuilder::new()
            .recursive(true)
            .create("./.mygit/objects")?;
        }
        Ok(())
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

    fn add(&self, path: &str) -> io::Result<()> {
        // ignore mygit metadata
        if path == "./.mygit" || path == "./target" || path == "./.git" {
            return Ok(());
        }

        let mut f = File::open(path)?;
        if f.metadata().unwrap().is_file() {
            let mut content: Vec<u8> = Vec::new();

            f.read_to_end(&mut content)?;
            let hash = Self::get_sha1(&content);
            let blob_path = self.gen_blob_path(&hash);


            // if the blob of the file dose not exist, create it
            if !Self::file_exist(&blob_path) {
                if let Err(err) = self.mkblob(&content, &blob_path) {
                    return Err(err);
                }
            }
            return Ok(());
        }

        let dir = fs::read_dir(path)?;

        for file in dir {
            let file = file?;
            let file_name = String::from(file.path().to_str().unwrap());

            // traverse project directory tree recursively
            if let Err(err) = self.add(&file_name) {
                return Err(err);
            }
        }
        Ok(())
    }

    fn mkblob(&self, content: &Vec<u8>, blob_path: &str) -> io::Result<()> {
        let mut blob = File::create(&blob_path)?;

        blob.write("blob ".as_bytes())?;
        blob.write(content.len().to_string().as_bytes())?;
        blob.write('\0'.to_string().as_bytes())?;
        blob.write(&content)?;
        Ok(())
    }

    fn gen_repo_path(&self) -> Option<&str> {
        match &self.repo_path {
            Ok(repo_path) => repo_path.to_str(),
            Err(_) => None,
        }
    }
    
    fn gen_blob_path(&self, blob_name: &str) -> String {
        let blob_path = String::from(self.gen_repo_path().unwrap());
        let blob_path = blob_path + "/.mygit/objects/" + blob_name;
        blob_path
    }
}
