mod mygit;
use mygit::MyGit;

fn main() {
    let mygit = MyGit::default();

    mygit.run();
}
