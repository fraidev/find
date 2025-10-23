use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug)]
struct Options {
    name: Option<String>,
    name_bytes: Option<Vec<u8>>,
    iname: Option<String>,
    iname_bytes: Option<Vec<u8>>,
    type_filter: Option<char>, // 'f' or 'd'
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: find <path> [-name <pattern> | -iname <pattern>] [-type f|d]");
        std::process::exit(1);
    }

    let start_path = Path::new(&args[1]);
    let opts = parse_args(&args[2..]);

    // Print the start path if it matches filters
    if let Ok(metadata) = start_path.metadata() {
        let file_type = metadata.file_type();
        if should_print_with_metadata(start_path, &file_type, &opts) {
            println!("{}", start_path.display());
        }
    }

    let mut visited_inodes = HashSet::new();
    find_recursive(start_path, &opts, &mut visited_inodes)?;

    Ok(())
}

fn parse_args(args: &[String]) -> Options {
    let mut opts = Options {
        name: None,
        name_bytes: None,
        iname: None,
        iname_bytes: None,
        type_filter: None,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-name" if i + 1 < args.len() => {
                let pattern = args[i + 1].clone();
                opts.name_bytes = Some(pattern.as_bytes().to_vec());
                opts.name = Some(pattern);
                i += 1;
            }
            "-iname" if i + 1 < args.len() => {
                let pattern = args[i + 1].clone();
                let pattern_lower = pattern.to_lowercase();
                opts.iname_bytes = Some(pattern_lower.as_bytes().to_vec());
                opts.iname = Some(pattern);
                i += 1;
            }
            "-type" if i + 1 < args.len() => {
                let t = args[i + 1].chars().next().unwrap_or('_');
                if t == 'f' || t == 'd' {
                    opts.type_filter = Some(t);
                } else {
                    eprintln!("find: invalid argument to '-type': {}", args[i + 1]);
                }
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }

    opts
}

fn find_recursive(path: &Path, opts: &Options, visited: &mut HashSet<(u64, u64)>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();

            // Track visited inodes to avoid symlink loops
            if file_type.is_symlink() {
                if let Ok(metadata) = path.metadata() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        let inode = (metadata.dev(), metadata.ino());
                        if !visited.insert(inode) {
                            continue; // Already visited, skip
                        }
                    }
                }
            }

            if should_print(&path, &file_type, opts) {
                println!("{}", path.display());
            }

            if file_type.is_dir() {
                find_recursive(&path, opts, visited)?;
            }
        }
    }

    Ok(())
}

fn should_print(path: &Path, file_type: &fs::FileType, opts: &Options) -> bool {
    should_print_with_metadata(path, file_type, opts)
}

fn should_print_with_metadata(path: &Path, file_type: &fs::FileType, opts: &Options) -> bool {
    // Apply -type filter
    if let Some(t) = opts.type_filter {
        match t {
            'f' if !file_type.is_file() => return false,
            'd' if !file_type.is_dir() => return false,
            _ => {}
        }
    }

    // Apply -name filter (AND logic)
    if let Some(ref pattern_bytes) = opts.name_bytes {
        if !matches_glob_bytes(path, pattern_bytes, false) {
            return false;
        }
    }

    // Apply -iname filter (AND logic)
    if let Some(ref pattern_bytes) = opts.iname_bytes {
        if !matches_glob_bytes(path, pattern_bytes, true) {
            return false;
        }
    }

    true
}

fn matches_glob_bytes(path: &Path, pattern_bytes: &[u8], case_insensitive: bool) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if case_insensitive {
            // Pattern is already lowercased at startup, just lowercase the name
            let name_lower = name.to_lowercase();
            glob_match(name_lower.as_bytes(), pattern_bytes)
        } else {
            // Zero allocation for case-sensitive matching
            glob_match(name.as_bytes(), pattern_bytes)
        }
    } else {
        false
    }
}

fn glob_match(name: &[u8], pattern: &[u8]) -> bool {
    glob_match_impl(name, pattern, 0, 0)
}

fn glob_match_impl(name: &[u8], pattern: &[u8], ni: usize, pi: usize) -> bool {
    // If we've matched the entire pattern and name
    if pi == pattern.len() && ni == name.len() {
        return true;
    }

    // If pattern is exhausted but name isn't
    if pi == pattern.len() {
        return false;
    }

    // Handle '*' wildcard
    if pattern[pi] == b'*' {
        // Try matching zero characters
        if glob_match_impl(name, pattern, ni, pi + 1) {
            return true;
        }
        // Try matching one or more characters
        if ni < name.len() && glob_match_impl(name, pattern, ni + 1, pi) {
            return true;
        }
        return false;
    }

    // If name is exhausted but pattern isn't (and pattern isn't *)
    if ni == name.len() {
        return false;
    }

    // Handle '?' wildcard or exact character match
    if pattern[pi] == b'?' || pattern[pi] == name[ni] {
        return glob_match_impl(name, pattern, ni + 1, pi + 1);
    }

    false
}
