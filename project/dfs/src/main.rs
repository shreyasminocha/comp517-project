use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use clap::Parser;
use dfs::{cas::directory::DirectoryEntry, dfs::fs::Filesystem, fuse::MountedFilesystem};
use fuser::MountOption;

// todo: add an exclude flag or something

#[derive(Parser)]
struct Args {
    /// Address to bind to
    bind: String,
    /// Mount point
    mount: PathBuf,

    /// Paths to add to FS
    #[arg(short, long, use_value_delimiter = true, value_delimiter = ',')]
    files: Vec<PathBuf>,
    /// Peer address
    #[arg(short, long, use_value_delimiter = true, value_delimiter = ',')]
    peers: Option<Vec<String>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut fs = Filesystem::new(args.bind);

    for file in &args.files {
        let entry = add_entry(&mut fs, Path::new(file))?;
        println!("adding {} ({})", file.to_str().unwrap(), entry.file)
    }

    fs.run()?;

    thread::sleep(Duration::from_millis(20));

    if let Some(peers) = args.peers {
        for peer in peers {
            fs.add_peer(peer)?;
        }
    }

    let mount = MountedFilesystem::new(&fs);
    fuser::mount2(
        mount,
        args.mount,
        &[MountOption::AllowRoot, MountOption::AutoUnmount],
    )?;

    Ok(())
}

// todo: move this elsewhere
fn add_entry(filesystem: &mut Filesystem, path: &Path) -> Result<DirectoryEntry, Box<dyn Error>> {
    let item = if path.is_dir() {
        let child_objs = BTreeMap::from_iter(
            fs::read_dir(path)?
                .map(|ch| {
                    let entry = add_entry(filesystem, &ch?.path())?;
                    Ok((entry.name, entry.file))
                })
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?,
        );

        filesystem.create_directory(child_objs)
    } else {
        // todo: don't assume it's a file
        let contents = fs::read(path)?;
        filesystem.create_file(&contents[..])
    };

    let entry = DirectoryEntry {
        name: path.file_name().unwrap().to_str().unwrap().to_string(), // todo: ew
        file: item,
    };

    Ok(entry)
}
