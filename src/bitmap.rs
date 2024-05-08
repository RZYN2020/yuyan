use bit_vec::BitVec;

#[derive(Debug)]
pub struct Bitmap {
    vec: BitVec,
    size: usize,
    len: usize,
}

impl Bitmap {
    pub fn new(size: usize) -> Bitmap {
        Bitmap {
            vec: BitVec::from_elem(size, false),
            size,
            len: 0,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.vec.to_bytes()
    }

    pub fn set(&mut self, idx: usize) {
        self.len += 1;
        self.vec.set(idx, true);
    }

    pub fn have(&self, id: usize) -> bool {
        self.vec.get(id).unwrap()
    }

    // size: the bytes of the whole file
    pub fn size(&self) -> usize {
        self.size
    }

    // len: downloaded bytes
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn left(&self) -> usize {
        self.size - self.len
    }
}
