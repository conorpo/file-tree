//! # project-tree
//! 
//! A simple ascii file tree generator. 
//! 
//! TODO:
//! Make ignore / stop check more elegant, is HashMap<PathBuf> really the best way to do this?
//! 

use clap::Parser;
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;
use std::io;
use clipboard::{ClipboardContext, ClipboardProvider};



#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Files to ignore in the tree
    #[arg(short, long, value_name = "FILE")]
    ignore: Vec<String>,

    /// Files to not recurse into
    #[arg(short, long, value_name = "FILE")]
    stop: Vec<String>,

    /// Output file
    #[arg(short, long, value_name = "FILE")]
    output: Option<String>,

    /// Show node_modules
    #[arg(long)]
    node_modules: bool,

    /// Show .git
    #[arg(long)]
    git: bool,

    /// Show .vscode
    #[arg(long)]
    vscode: bool,

    /// Include root
    #[arg(short, long)]
    root: bool,

    /// Prioritize directories
    #[arg(short, long)]
    dirs: bool
}

struct ProjectTree {
    ignore_list: HashSet<PathBuf>,
    stop_list: HashSet<PathBuf>,
    prioritize_dirs: bool
}

impl ProjectTree {
    fn new(ignore_list: HashSet<PathBuf>, stop_list: HashSet<PathBuf>, prioritize_dirs: bool) -> ProjectTree {
        ProjectTree {
            ignore_list,
            stop_list,
            prioritize_dirs
        }
    }

    fn scan_folder(&self, cur_path: &PathBuf, cur_prefix: String, show_lines: bool) -> io::Result<Vec<String>> {
        let mut files: Vec<String> = Vec::new();

        let mut paths: Vec<PathBuf> = fs::read_dir(&cur_path)?.filter_map(|entry| {
            let entry: fs::DirEntry = entry.ok()?;
            let path: PathBuf = entry.path();
            if self.ignore_list.contains(&path) { None } else { Some(path) }
        }).collect();

        if self.prioritize_dirs {
            paths.sort_by_key(|path| !path.is_dir());
        }

        for (i, path) in paths.iter().enumerate() {
            let is_dir: bool = path.is_dir();
            let is_last: bool = i == paths.len() - 1;

            let affix = match (show_lines, is_last) {
                (true, true) => "└── ",
                (true, false) => "├── ",
                (false, _) => "",
            };
            let filename: &std::ffi::OsStr = path.file_name().unwrap_or_default();
            let filename: &str = filename.to_str().unwrap_or_default();

            files.push(format!("{cur_prefix}{affix}{filename}{}", if is_dir { "/" } else { "" }));     

            if is_dir && !self.stop_list.contains(path) {
                let new_prefix = format!("{cur_prefix}{}", if is_last { "    " } else { "│   " });

                let mut sub_files: Vec<String> = self.scan_folder(path,new_prefix,true)?;
                files.append(&mut sub_files);
            }
        }

        

        Ok(files)
    }
}

fn main() -> io::Result<()>{
    let args = Args::parse();
    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();

    let mut ignore_list: HashSet<PathBuf> = HashSet::new();
    if !args.git { ignore_list.insert(PathBuf::from("./.git")); }
    if !args.vscode { ignore_list.insert(PathBuf::from("./.vscode")); }
    for ignore in args.ignore {
        //See next comment
        if ignore.starts_with("./") {
            ignore_list.insert(PathBuf::from(ignore));
        } else {
            ignore_list.insert(PathBuf::from(format!("./{}", ignore)));
        }
    }

    let mut stop_list: HashSet<PathBuf> = HashSet::new();
    if !args.node_modules { stop_list.insert(PathBuf::from("./node_modules")); }
    for stop in args.stop {
        //Yes yes its ugly but it works, I can't figure out how to hash a PathBuf and support relative paths
        if stop.starts_with("./") {
            stop_list.insert(PathBuf::from(stop));
        } else {
            stop_list.insert(PathBuf::from(format!("./{}", stop)));
        }
    }

    let mut tree: String = ProjectTree::new(ignore_list, stop_list, args.dirs)
                                   .scan_folder(&PathBuf::from("./"), String::from(""), args.root)
                                   .unwrap()
                                   .join("\n");

    //Get Root Dir Name
    if args.root {
        let root_dir: String = std::env::current_dir().unwrap().file_name().unwrap().to_str().unwrap().to_owned();
        tree = format!("{root_dir}\n{tree}");
    }

    println!("{tree}");
    if let Some(output_file) = args.output {
        fs::write(output_file, &tree)?;
    }
    clipboard.set_contents(tree).unwrap();

    Ok(())
}
