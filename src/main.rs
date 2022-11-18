use std::{fs::{File, self, DirBuilder}, io::{Read, Write}, env, process::exit};
use sha1::{Sha1, Digest};

fn main() {
    let args: Vec<String> = env::args().collect();

    // parse arguments
    if args.len() > 1 {
        let cmd = args[1].as_str();
        match cmd {
            "init" => init(),
            "add" => {
                if !has_init() {
                    println!("this is not a mygit repository!");
                    exit(1);
                }
                if args.len() > 2 {
                    add(args[2].as_str());
                } else {
                    println!("lack of arguments!");
                    exit(1);
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

fn dir_exist(dir_path: &str) -> bool {
    match fs::read_dir(dir_path) {
        Ok(_) => true,
        Err(_) => false
    }
}

fn file_exist(file_path: &str) -> bool {
    match File::open(file_path) {
        Ok(_) => true,
        Err(_)=> false
    }
}

fn has_init() -> bool {
    dir_exist("./.mygit")
}

fn init() {
    // if .mygit does not exist, create directory .mygit
    if !has_init() {
        println!("Initialize mygit by creating directory ./.mygit");
        DirBuilder::new()
        .recursive(true)
        .create("./.mygit").unwrap();
        DirBuilder::new()
        .recursive(true)
        .create("./.mygit/objects").unwrap();
    }
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

fn add(path: &str) {
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
        if !blob_exist(&hash) {
            mkblob(&content, &hash);
        }
        return;
    }

    let dir = fs::read_dir(path).expect("open current directory error!");

    for file in dir {
        let file = file.unwrap();
        let file_name = String::from(file.path().to_str().unwrap());

        // traverse project directory tree recursively
        add(&file_name);
    }
}

fn blob_exist(hash: &str) -> bool {
    let blob_path = String::from("./.mygit/objects/");
    let blob_path = blob_path + hash;
    file_exist(blob_path.as_str())
}

fn mkblob(content: &Vec<u8>, hash: &str) {
    let blob_path = String::from("./.mygit/objects/");
    let blob_path = blob_path + hash;
    let mut blob = File::create(&blob_path).expect("blob create error!");

    blob.write("blob ".as_bytes()).unwrap();
    blob.write(content.len().to_string().as_bytes()).unwrap();
    blob.write('\0'.to_string().as_bytes()).unwrap();
    blob.write(&content).unwrap();
}
