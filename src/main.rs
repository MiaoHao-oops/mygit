use std::{fs::{File, self, DirBuilder}, io::Read};
use sha1::{Sha1, Digest};

mod hash;

fn main() {
    init();

    let hash = get_sha1("/home/haooops/Documents/mygit/test/a.c");
    println!("{}", hash);
}

fn init() {
    let dir = fs::read_dir("./").expect("open current directory error!");
    let mut has_init = false;

    // find whether .mygit exists
    for file in dir {
        let file = file.unwrap();
        if file.file_type().unwrap().is_dir() && file.file_name().to_str().unwrap() == ".mygit" {
            has_init = true;
        }
    }

    // if .mygit does not exist, create directory .mygit
    if !has_init {
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
    let mut buf = String::new();
    let mut hash = String::new();
    f.read_to_string(&mut buf).unwrap();

    let mut hasher = Sha1::new();
    hasher.update(&buf);
    let result = hasher.finalize();
    
    for i in result {
        hash.push(hex_digits[(i & 0xf) as usize]);
        hash.push(hex_digits[((i & 0xf0) >> 4) as usize]);
    }
    hash
}
