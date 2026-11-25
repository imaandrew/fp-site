use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("index out of bounds")]
    IndexOutOfBounds,
}

macro_rules! read_be {
    ($($t:ident),* $(,)?) => {
        $(
            pub fn $t(&mut self) -> Result<$t, ReaderError> {
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

pub struct Reader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl Reader<'_> {
    pub fn align(&mut self, align: usize) {
        self.offset = self.offset.div_ceil(align) * align;
    }

    pub fn skip(&mut self, n: usize) {
        self.offset += n;
    }

    read_be!(u32);
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, offset: 0 }
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], ReaderError> {
        if self.offset + n > self.data.len() {
            return Err(ReaderError::IndexOutOfBounds);
        }
        self.offset += n;
        Ok(&self.data[self.offset - n..self.offset])
    }
}

pub struct Writer {
    data: Vec<u8>,
}

impl Writer {
    pub fn new(n: usize) -> Self {
        Writer {
            data: Vec::with_capacity(n),
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn u32(&mut self, n: usize) {
        self.write(&(n as u32).to_be_bytes());
    }

    pub fn align(&mut self, align: usize) {
        let new_len = self.data.len().div_ceil(align) * align;
        self.data.resize_with(new_len, Default::default);
    }

    pub fn get(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.data)
    }
}
