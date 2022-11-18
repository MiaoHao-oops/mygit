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
        println!("[usage] mygit ...");
        exit(1);
    };
}

fn has_init() -> bool {
    let dir = fs::read_dir("./").expect("open current directory error!");

    // find whether .mygit exists
    for file in dir {
        let file = file.unwrap();
        if file.file_type().unwrap().is_dir() && file.file_name().to_str().unwrap() == ".mygit" {
            return true;
        }
    }

    false
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

fn get_sha1(file_name: &str) -> String {
    let hex_digits = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];
    let mut f = File::open(file_name).expect("file open error!");
    let mut buf: Vec<u8> = Vec::new();
    let mut hash = String::new();

    f.read_to_end(&mut buf).unwrap();

    let mut hasher = Sha1::new();

    hasher.update(&buf);
    let result = hasher.finalize();

    for i in result {
        hash.push(hex_digits[(i & 0xf) as usize]);
        hash.push(hex_digits[((i & 0xf0) >> 4) as usize]);
    }
    hash
}

fn add(dir_name: &str) {
    // ignore mygit metadata
    if dir_name == "./.mygit" || dir_name == "./target" || dir_name == "./.git" {
        return;
    }

    let dir = fs::read_dir(dir_name).expect("open current directory error!");

    for file in dir {
        let file = file.unwrap();
        let file_name = String::from(file.path().to_str().unwrap());

        if file.file_type().unwrap().is_file() {
            let hash = get_sha1(&file_name);

            // if the blob of the file dose not exist, create it
            if !blob_exist(&hash) {
                mkblob(&file_name, &hash);
            }
        } else if file.file_type().unwrap().is_dir() {
            // traverse project directory tree recursively
            add(&file_name);
        }
    }
}

fn blob_exist(hash: &str) -> bool {
    let obj_dir = fs::read_dir("./.mygit/objects").expect("open ./.mygit/objects error!");

    for obj in obj_dir {
        let obj = obj.unwrap();

        if obj.file_name().to_str().unwrap() == hash {
            return true;
        }
    }

    false
}

fn mkblob(file_path: &str, hash: &str) {
    let blob_path = String::from("./.mygit/objects/");
    let blob_path = blob_path + hash;
    let mut blob = File::create(&blob_path).expect("blob create error!");
    let mut file = File::open(file_path).expect("open file error!");
    let mut content: Vec<u8> = Vec::new();

    file.read_to_end(&mut content).unwrap();
    blob.write("blob ".as_bytes()).unwrap();
    blob.write(content.len().to_string().as_bytes()).unwrap();
    blob.write('\0'.to_string().as_bytes()).unwrap();
    blob.write(&content).unwrap();
}
