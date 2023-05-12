//! # project-tree
//! 
//! A simple ascii file tree generator. 
//! 
//! TODO:
//! Implement --git, --vscode flags
//! Make -i, -s HashSet initialization more elegant
//! Make main iterator filter out ignores so that we can check if a file is the last in the tree.
//! Implement --dirs flag

use clap::Parser;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::env;
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
    root: bool
}

fn scan_folder(path: PathBuf, ignore_list: &HashSet<PathBuf>, stop_list: &HashSet<PathBuf>, depth: usize) -> io::Result<Vec<String>> {
    let mut files = Vec::new();

    // Custom Functionality for root folder
    let mut file_itr = fs::read_dir(path)?.peekable();

    while let Some(file) = file_itr.next() {
        let file: fs::DirEntry = file?;
        let path: PathBuf = file.path();
        let is_dir: bool = path.is_dir();
        let path: PathBuf = match path.strip_prefix("./") {
            Ok(path) => path.to_path_buf(),
            Err(_) => path,
        };

        if ignore_list.contains(&path) { continue; }
        
        let prefix = if depth == 0 {"".to_owned()} else {"│  ".repeat(depth - 1) + "├─ "};
        let filename = path.file_name().unwrap().to_str().unwrap().to_owned() + if is_dir {"/"} else {""};

        files.push(format!("{prefix}{filename}"));
        
        if is_dir && !stop_list.contains(&path) {
            let mut sub_files: Vec<String> = scan_folder(path, ignore_list, stop_list, depth + 1)?;
            files.append(&mut sub_files);
        }
    }

    //Change the last file to the different prefix
    files.last_mut().map(|s| *s = s.replace("├─", "└─"));
    
    Ok(files)
}

fn main() -> io::Result<()>{
    let args = Args::parse();
    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();

    let mut ignore_list: HashSet<PathBuf> =  vec![".git", ".vscode"].into_iter().map(|s| Path::new(s).to_path_buf()).collect();
    let mut stop_list: HashSet<PathBuf> = vec![if args.node_modules {None} else {Some("node_modules")}]
        .into_iter()
        .filter_map(|s| s)
        .map(|s| Path::new(s).to_path_buf())
        .collect();

    ignore_list.extend(args.ignore.into_iter().map(|s| Path::new(&s).to_path_buf()));
    stop_list.extend(args.stop.into_iter().map(|s| Path::new(&s).to_path_buf()));

    if args.node_modules {
        stop_list.remove(&Path::new("node_modules").to_path_buf());
    }
    let current_dir = env::current_dir().unwrap().file_name().unwrap().to_str().unwrap().to_owned();
    let file_tree: String = scan_folder("./".into(), &ignore_list, &stop_list, if args.root {1} else {0}).unwrap().join("\n");
    let file_tree = if args.root {format!("{}/\n{}", current_dir, file_tree)} else {file_tree};
   
    println!("{file_tree}");
    clipboard.set_contents(file_tree.clone()).unwrap();
    if let Some(output_file) = args.output {
        fs::write(output_file, file_tree)?;
    }

    Ok(())
}
