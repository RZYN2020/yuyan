//! Metainfo File Structure (.torrent file)r

use crate::bencode::{BDict, BItem};
use futures::SinkExt;
use sha1::{Digest, Sha1};
use std::fmt::{self, Display};
use std::fs::read;

#[derive(Debug)]
pub struct MetaInfo {
    pub announce: String,
    pub announce_list: Option<Vec<String>>,
    pub creation_date: Option<String>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,

    // info
    pub piece_length: usize,
    pub pieces: Vec<u8>,
    pub private: Option<String>,
    pub name: String,

    pub info_hash: String,
    pub file_info: FileInfo,
}

#[derive(Debug)]
pub enum FileInfo {
    SingleFile {
        length: usize,
        md5sum: Option<String>,
    },
    MultipleFile {
        files: Vec<File>,
    },
}

#[derive(Debug)]
pub struct File {
    length: usize,
    md5sum: Option<String>,
    paths: Vec<String>,
}

pub struct TrackerIterator<'a> {
    metainfo: &'a MetaInfo,
    ptr: usize,
}

impl<'a> Iterator for TrackerIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.ptr += 1;
        if self.ptr - 1 == 0 {
            Some(&self.metainfo.announce)
        } else {
            self.metainfo
                .announce_list
                .as_ref()
                .and_then(|ans| ans.get(self.ptr - 2))
                .map(|x| x.as_str())
        }
    }
}

impl MetaInfo {
    pub fn trackers(&self) -> TrackerIterator {
        TrackerIterator {
            metainfo: self,
            ptr: 0,
        }
    }
}

impl Display for MetaInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Announce: {}", self.announce)?;
        if let Some(announce_list) = &self.announce_list {
            writeln!(f, "Announce List:")?;
            for announce in announce_list {
                writeln!(f, "  - {}", announce)?;
            }
        }
        if let Some(creation_date) = &self.creation_date {
            writeln!(f, "Creation Date: {}", creation_date)?;
        }
        if let Some(comment) = &self.comment {
            writeln!(f, "Comment: {}", comment)?;
        }
        if let Some(created_by) = &self.created_by {
            writeln!(f, "Created By: {}", created_by)?;
        }
        if let Some(encoding) = &self.encoding {
            writeln!(f, "Encoding: {}", encoding)?;
        }
        if let Some(private) = &self.private {
            writeln!(f, "Private: {}", private)?;
        }
        writeln!(f, "Piece Length: {}", self.piece_length)?;
        writeln!(f, "Length: {}", self.pieces.len() / 20)?;
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Info Hash: {}", self.info_hash)?;
        writeln!(f, "File Info: {}", self.file_info)?;
        Ok(())
    }
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileInfo::SingleFile { length, md5sum } => {
                writeln!(f, "Length: {}", length)?;
                if let Some(md5sum) = md5sum {
                    writeln!(f, "MD5 Sum: {}", md5sum)?;
                }
            }
            FileInfo::MultipleFile { files } => {
                writeln!(f, "Files:")?;
                for file in files {
                    writeln!(f, "  - Length: {}", file.length)?;
                    if let Some(md5sum) = &file.md5sum {
                        writeln!(f, "    MD5 Sum: {}", md5sum)?;
                    }
                    write!(f, "    Path: ")?;
                    for path in &file.paths {
                        write!(f, "/{path}")?;
                    }
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

// hex -> string in escape hex format
// http://www.faqs.org/rfcs/rfc1738.html
// byte not in the set 0-9, a-z, A-Z, '.', '-', '_' and '~', must be encoded using the "%nn" format
fn to_escape_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'~') {
                String::from_utf8([*b].to_vec()).unwrap()
            } else {
                format!("%{:x}", b)
            }
        })
        .collect()
}

impl MetaInfo {
    pub fn from(link: &str) -> anyhow::Result<MetaInfo> {
        MetaInfo::from_file(link)
    }

    pub fn from_file(file: &str) -> anyhow::Result<MetaInfo> {
        let bytes = read(file)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<MetaInfo> {
        let bitem = BItem::deseri_cons(bytes)?;
        let mut bdict: BDict = bitem.try_into()?;

        let announce = bdict.remove::<String>("announce")?;

        let announce_list = bdict
            .remove::<Vec<BItem>>("announce-list")?
            .into_iter()
            .map(|x| Vec::<BItem>::try_from(x).ok())
            .collect::<Option<Vec<Vec<BItem>>>>()
            .and_then(|a| {
                a.into_iter()
                    .flat_map(|x| x.into_iter().map(|x| String::try_from(x).ok()))
                    .collect::<Option<Vec<String>>>()
            });

        let creation_date = bdict.remove::<String>("creation date").ok();

        let info = BItem::Dict(bdict.remove::<BDict>("info")?);
        // let info_hash = get_ref(&bdict, "info")?.seri_decons();
        let mut hasher = Sha1::new();
        hasher.update(info.seri_decons());
        let info_hash = to_escape_hex(&hasher.finalize().to_vec());

        let comment = bdict.remove::<String>("comment").ok();
        let created_by = bdict.remove::<String>("created by").ok();
        let encoding = bdict.remove::<String>("encoding").ok();

        // info dict
        let mut info: BDict = info.try_into()?;
        let piece_length = info.remove::<i64>("piece length")? as usize;
        let pieces = info.remove::<Vec<u8>>("pieces")?;
        let private = info.remove::<String>("private").ok();

        let name = info.remove::<String>("name")?;
        let files = info.remove::<Vec<BItem>>("files");

        let file_info = if let Ok(files) = files {
            // Multiple File Mode
            let mut files_ = Vec::new();
            for file in files {
                let mut file = BDict::try_from(file)?;
                let length = file.remove::<i64>("length")? as usize;
                let md5sum = file.remove::<String>("md5sum").ok();
                let paths = file
                    .remove::<Vec<BItem>>("path")?
                    .into_iter()
                    .map(|x| String::try_from(x))
                    .collect::<anyhow::Result<Vec<String>>>()?;

                files_.push(File {
                    length,
                    md5sum,
                    paths,
                });
            }
            FileInfo::MultipleFile { files: files_ }
        } else {
            // Single File Mode
            let length = info.remove::<i64>("length")? as usize;
            let md5sum = info.remove::<String>("md5sum").ok();
            FileInfo::SingleFile { length, md5sum }
        };

        Ok(MetaInfo {
            announce,
            announce_list,
            creation_date,
            comment,
            created_by,
            encoding,

            piece_length,
            pieces,
            private,
            name,

            info_hash,
            file_info,
        })
    }
}
