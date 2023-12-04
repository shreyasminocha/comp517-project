use std::{
    collections::HashMap,
    io::{self, Error},
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use fuser::{FileAttr, FileType, Filesystem as FuseFilesystem};
use libc::{EIO, EISDIR, ENOENT, ENOTDIR};

use crate::cas::{
    chunk::CHUNK_SIZE,
    directory::{Directory, DirectoryEntry},
    object::Object,
    resource::Resource,
    ContentAddressedStore,
};

pub struct MountedFilesystem<'a, T: ContentAddressedStore> {
    filesystem: &'a T,
    inode_to_object: HashMap<u64, Object>,
    object_to_inode: HashMap<Object, u64>,
    lowest_free_inode: u64,
}

const ROOT_INODE: u64 = 1;

impl<'a, T: ContentAddressedStore> MountedFilesystem<'a, T> {
    pub fn new(filesystem: &'a T) -> Self {
        Self {
            filesystem,
            inode_to_object: HashMap::new(),
            object_to_inode: HashMap::new(),
            lowest_free_inode: ROOT_INODE + 1,
        }
    }

    fn inode_to_file_attr(&self, ino: u64) -> Result<FileAttr, ()> {
        if ino == ROOT_INODE {
            return Ok(file_attr(ino, 4_096, 1, FileType::Directory));
        }

        let resource = self
            .inode_to_object
            .get(&ino)
            .and_then(|obj| self.filesystem.get(obj))
            .ok_or(())?;

        let (size, blocks) = match &resource {
            Resource::File(o) => (o.size, o.contents.len() as u64),
            Resource::Directory(_) => (4_096, 1),
            Resource::Chunk(_) => return Err(()),
        };

        Ok(file_attr(
            ino,
            size,
            blocks,
            match &resource {
                Resource::File(_) => FileType::RegularFile,
                Resource::Directory(_) => FileType::Directory,
                Resource::Chunk(_) => return Err(()),
            },
        ))
    }

    fn get_inode(&mut self, obj: &Object) -> u64 {
        *self.object_to_inode.entry(*obj).or_insert_with(|| {
            let ino = self.lowest_free_inode;
            self.lowest_free_inode += 1;

            self.inode_to_object.insert(ino, *obj);

            ino
        })
    }

    fn lookup_by_hash(&mut self, name: &std::ffi::OsStr) -> io::Result<FileAttr> {
        let name = name.to_str().ok_or(Error::from_raw_os_error(ENOENT))?;
        let mut digest = [0u8; 32];
        hex::decode_to_slice(name, &mut digest).map_err(|_| Error::from_raw_os_error(ENOENT))?;
        let obj = Object::new(digest);
        let inode = self.get_inode(&obj);

        self.inode_to_file_attr(inode)
            .map_err(|_| Error::from_raw_os_error(ENOENT))
    }

    fn lookup_on_dir(&mut self, parent: u64, name: &std::ffi::OsStr) -> io::Result<FileAttr> {
        let parent_entry = self
            .inode_to_object
            .get(&parent)
            .and_then(|obj| self.filesystem.get(obj))
            .ok_or(Error::from_raw_os_error(ENOENT))?;

        match parent_entry {
            Resource::File(_) => Err(Error::from_raw_os_error(ENOTDIR)),
            Resource::Chunk(_) => Err(Error::from_raw_os_error(ENOTDIR)),
            Resource::Directory(directory) => {
                let name = name.to_str().unwrap().to_string();
                let child = directory
                    .get_child(&name)
                    .map_err(|_| Error::from_raw_os_error(ENOENT))?;

                let child_inode = self.get_inode(&child);
                self.inode_to_file_attr(child_inode)
                    .map_err(|_| Error::from_raw_os_error(ENOENT))
            }
        }
    }
}

impl<'a, T: ContentAddressedStore> FuseFilesystem for MountedFilesystem<'a, T> {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let result = if parent == ROOT_INODE {
            self.lookup_by_hash(name)
        } else {
            self.lookup_on_dir(parent, name)
        };
        match result {
            Ok(attr) => reply.entry(&Duration::new(1, 0), &attr, 1),
            Err(err) => reply.error(err.raw_os_error().unwrap_or(EIO)),
        }
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        match self.inode_to_file_attr(ino) {
            Ok(attr) => reply.attr(&Duration::new(1, 0), &attr),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        let item = self
            .inode_to_object
            .get(&ino)
            .and_then(|obj| self.filesystem.get(obj));
        match item {
            Some(Resource::File(file)) => {
                let res = self.filesystem.read_file(file, offset as u64, size as u64);
                match res {
                    Ok(data) => reply.data(&data),
                    Err(err) => reply.error(err.raw_os_error().unwrap_or(EIO)),
                }
            }
            Some(Resource::Directory(_)) => reply.error(EISDIR),
            None | Some(Resource::Chunk(_)) => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        let entries: Option<Vec<DirectoryEntry>> = if ino == ROOT_INODE {
            self.filesystem.accessible_objects().ok().map(|objects| {
                objects
                    .iter()
                    .map(|obj| DirectoryEntry {
                        name: obj.to_string(),
                        file: *obj,
                    })
                    .collect()
            })
        } else {
            let directory: Option<Arc<Directory>> = self
                .inode_to_object
                .get(&ino)
                .and_then(|obj| self.filesystem.get(obj));

            directory.map(|dir| dir.get_children().collect())
        };

        if let Some(entries) = entries {
            for (i, child) in entries.iter().enumerate().skip(offset as usize) {
                let child_inode = self.object_to_inode.entry(child.file).or_insert_with(|| {
                    let ino = self.lowest_free_inode;
                    self.lowest_free_inode += 1;

                    self.inode_to_object.insert(ino, child.file);

                    ino
                });

                let child_resource = self.filesystem.get(&child.file);
                let file_type = match child_resource {
                    Some(Resource::File(_)) => Some(FileType::RegularFile),
                    Some(Resource::Directory(_)) => Some(FileType::Directory),
                    Some(Resource::Chunk(_)) | None => None,
                };

                if let Some(file_type) = file_type {
                    if reply.add(*child_inode, (i + 1) as i64, file_type, &child.name) {
                        break;
                    }
                } else {
                    reply.error(ENOENT);
                    return;
                }
            }

            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
}

fn file_attr(ino: u64, size: u64, blocks: u64, file_type: FileType) -> FileAttr {
    FileAttr {
        ino,
        size,
        blocks,
        atime: UNIX_EPOCH,
        mtime: UNIX_EPOCH,
        ctime: UNIX_EPOCH,
        crtime: UNIX_EPOCH,
        kind: file_type,
        perm: match file_type {
            FileType::Directory => 0o755,
            _ => 0o644,
        },
        nlink: match file_type {
            FileType::Directory => 2,
            _ => 1,
        },
        uid: 0,
        gid: 0,
        rdev: 0,
        blksize: CHUNK_SIZE as u32,
        flags: 0,
    }
}
