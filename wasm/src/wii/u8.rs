use std::{
    io,
    path::{Component, Path},
};

use thiserror::Error;

const MAGIC: u32 = 0x55aa382d;
const NODE: usize = 12;
const DATA_ALIGN: usize = 0x20;

fn read_be32(d: &[u8], i: usize) -> Result<u32, U8Error> {
    let bytes = d
        .get(i..)
        .and_then(|d| d.get(..4))
        .ok_or(U8Error::InvalidNode)?;
    Ok(u32::from_be_bytes(
        bytes.try_into().expect("should have 4 elements"),
    ))
}

fn write_be32(d: &mut [u8], i: usize, v: u32) {
    d[i..][..4].copy_from_slice(&v.to_be_bytes());
}

fn align_num(num: usize, align: usize) -> usize {
    num.div_ceil(align) * align
}

#[derive(Error, Debug)]
pub enum U8Error {
    #[error("invalid magic")]
    InvalidMagic,
    #[error("invalid node kind")]
    InvalidKind,
    #[error("io error: {0}")]
    IoError(#[from] io::Error),
    #[error("path doesn't exist")]
    InvalidPath,
    #[error("invalid node")]
    InvalidNode,
}

#[derive(Clone, Copy)]
pub struct Node {
    index: usize,
    is_dir: bool,
    name_offset: usize,
    data_offset: usize,
    size: usize,
}

pub struct U8Archive<'a> {
    buf: &'a mut Vec<u8>,
    root: usize,
    nodes: usize,
    strings: usize,
}

impl<'a> U8Archive<'a> {
    pub fn new(buf: &'a mut Vec<u8>) -> Result<Self, U8Error> {
        if read_be32(buf, 0).map_err(|_| U8Error::InvalidMagic)? != MAGIC {
            return Err(U8Error::InvalidMagic);
        }

        let root = read_be32(buf, 4)? as usize;

        let mut arc = U8Archive {
            buf,
            root,
            nodes: 1,
            strings: 0,
        };

        let root_node = arc.read_node(0)?;
        if !root_node.is_dir || root_node.size == 0 {
            return Err(U8Error::InvalidNode);
        }

        let nodes = root_node.size;
        let strings = nodes
            .checked_mul(NODE)
            .and_then(|n| n.checked_add(root))
            .filter(|&s| s <= arc.buf.len())
            .ok_or(U8Error::InvalidNode)?;

        arc.nodes = nodes;
        arc.strings = strings;

        Ok(arc)
    }

    fn node_offset(&self, i: usize) -> usize {
        self.root + i * NODE
    }

    fn read_node(&self, i: usize) -> Result<Node, U8Error> {
        if i >= self.nodes {
            return Err(U8Error::InvalidNode);
        }

        let offset = self.node_offset(i);
        let data = self
            .buf
            .get(offset..)
            .and_then(|d| d.get(..NODE))
            .ok_or(U8Error::InvalidNode)?;

        let a = read_be32(data, 0)?;
        let is_dir = match a >> 24 {
            0 => false,
            1 => true,
            _ => return Err(U8Error::InvalidKind),
        };
        let name_offset = (a & 0xFFFFFF) as usize;
        let data_offset = read_be32(data, 4)? as usize;
        let size = read_be32(data, 8)? as usize;

        Ok(Node {
            index: i,
            is_dir,
            name_offset,
            data_offset,
            size,
        })
    }

    fn read_name(&self, n: &Node) -> Result<&[u8], U8Error> {
        let name_buf = self
            .buf
            .get(self.strings + n.name_offset..)
            .ok_or(U8Error::InvalidNode)?;
        let name_end = name_buf
            .iter()
            .position(|&x| x == 0)
            .ok_or(U8Error::InvalidNode)?;
        Ok(&name_buf[..name_end])
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<Node, U8Error> {
        let mut cur = self.read_node(0)?;

        for c in path.as_ref().components() {
            let name = match c {
                Component::Normal(name) => name.as_encoded_bytes(),
                Component::CurDir => b".",
                Component::RootDir => continue,
                Component::ParentDir => {
                    if !cur.is_dir {
                        return Err(U8Error::InvalidPath);
                    }
                    cur = self.read_node(cur.data_offset)?;
                    continue;
                }
                Component::Prefix(_) => return Err(U8Error::InvalidPath),
            };

            if !cur.is_dir {
                return Err(U8Error::InvalidPath);
            }

            let end = cur.size.min(self.nodes);
            let mut i = cur.index + 1;
            let mut found = None;
            while i < end {
                let n = self.read_node(i)?;
                if self.read_name(&n)? == name {
                    found = Some(n);
                    break;
                }
                i = if n.is_dir {
                    if n.size <= i {
                        return Err(U8Error::InvalidNode);
                    }
                    n.size
                } else {
                    i + 1
                };
            }

            cur = found.ok_or(U8Error::InvalidPath)?;
        }

        Ok(cur)
    }

    pub fn read(&self, n: &Node) -> Result<&[u8], U8Error> {
        if n.is_dir {
            return Err(U8Error::InvalidKind);
        }
        self.buf
            .get(n.data_offset..)
            .and_then(|d| d.get(..n.size))
            .ok_or(U8Error::InvalidNode)
    }

    pub fn replace_file(&mut self, n: &mut Node, data: &[u8]) -> Result<(), U8Error> {
        if n.is_dir {
            return Err(U8Error::InvalidKind);
        }
        if n.data_offset.checked_add(n.size).is_none_or(|end| end > self.buf.len()) {
            return Err(U8Error::InvalidNode);
        }
        let new_size = u32::try_from(data.len()).map_err(|_| U8Error::InvalidNode)?;

        let mut next = self.buf.len();
        for i in 1..self.nodes {
            let node = self.read_node(i)?;
            if !node.is_dir && node.data_offset > n.data_offset {
                next = next.min(node.data_offset);
            }
        }
        let available = next - n.data_offset;

        if data.len() <= available {
            let start = n.data_offset;
            self.buf[start..][..data.len()].copy_from_slice(data);
            let old_end = start + n.size.max(data.len());
            self.buf[start + data.len()..old_end].fill(0);
        } else {
            let shift = align_num(data.len() - available, DATA_ALIGN);
            let padding = available + shift - data.len();
            self.buf.splice(
                n.data_offset..next,
                data.iter().copied().chain(std::iter::repeat_n(0, padding)),
            );

            for i in 1..self.nodes {
                let node = self.read_node(i)?;
                if !node.is_dir && node.data_offset > n.data_offset {
                    let new_offset = u32::try_from(node.data_offset + shift)
                        .map_err(|_| U8Error::InvalidNode)?;
                    let off = self.node_offset(i);
                    write_be32(self.buf, off + 4, new_offset);
                }
            }
        }

        let off = self.node_offset(n.index);
        write_be32(self.buf, off + 8, new_size);
        n.size = data.len();

        Ok(())
    }
}
