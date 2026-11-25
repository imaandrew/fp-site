pub mod romc;
pub mod u8;
pub mod wad;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("index out of bounds")]
    IndexOutOfBounds,
}

macro_rules! read_be {
    ($($t:ident),* $(,)?) => {
        $(
            fn $t(&mut self) -> Result<$t, ReaderError> {
                const SIZE: usize = std::mem::size_of::<$t>();
                if self.offset + SIZE > self.data.len() {
                    return Err(ReaderError::IndexOutOfBounds);
                }
                let bytes: [u8; SIZE] = self.data[self.offset..self.offset + SIZE]
                    .try_into()
                    .unwrap();
                self.offset += SIZE;
                Ok($t::from_be_bytes(bytes))
            }
        )*
    };
}

struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl Reader<'_> {
    fn align(&mut self, align: usize) {
        self.offset = self.offset.div_ceil(align) * align;
    }

    fn skip(&mut self, n: usize) {
        self.offset += n;
    }

    read_be!(u32);
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Reader { data, offset: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ReaderError> {
        if self.offset + n > self.data.len() {
            return Err(ReaderError::IndexOutOfBounds);
        }
        self.offset += n;
        Ok(&self.data[self.offset - n..self.offset])
    }
}

struct Writer {
    data: Vec<u8>
}

impl Writer {
    fn new(n: usize) -> Self {
        Writer { data: Vec::with_capacity(n) }
    }

    fn write(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    fn u32(&mut self, n: usize) {
        self.write(&(n as u32).to_be_bytes());
    }

    fn align(&mut self, align: usize) {
        let new_len = self.data.len().div_ceil(align) * align;
        self.data.resize_with(new_len, Default::default);
    }

    fn get(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }
}
