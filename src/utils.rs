use std::path::{Path, PathBuf};

const DOTGIT: &str = ".git";

pub fn find_git_root(starting_directory: &Path) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let dotgit = Path::new(DOTGIT);

    loop {
        path.push(dotgit);

        if path.is_dir() {
            break Some(path);
        }

        // remove DOTGIT && remove parent
        if !(path.pop() && path.pop()) {
            break None;
        }
    }
}
