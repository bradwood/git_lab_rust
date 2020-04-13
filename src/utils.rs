use std::path::{Path, PathBuf};

const DOTGIT: &str = ".git";

pub fn find_git_root(starting_directory: &Path) -> Option<PathBuf> {
    let mut path: PathBuf = starting_directory.into();
    let dotgit = Path::new(DOTGIT);

    loop {
        path.push(dotgit);

        if path.is_dir() {
            trace!("Found git root: {:?}", path.as_path().to_str().unwrap());
            break Some(path);
        }

        // remove DOTGIT && remove parent
        if !(path.pop() && path.pop()) {
            trace!("Did not find git root");
            break None;
        }
    }
}
