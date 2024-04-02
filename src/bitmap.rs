#[derive(Debug)]
pub struct Bitmap {
    size: usize,
    len: usize
}


impl Bitmap {
    pub fn new(size: usize) -> Bitmap { 
        Bitmap { size, len: 0}
    }

    pub fn to_bytes(&self) -> &[u8] {
        todo!()
    }

    pub fn set(&mut self, size: usize) {

    }

    pub fn have(&self, id: usize) -> bool {
        todo!()
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