use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    pub values: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Memory {
        let values: Vec<u8> = vec![0; size];
        Memory { values }
    }

    pub fn from_values(values: Vec<u8>) -> Memory {
        Memory { values }
    }

    pub fn read(&self, index: usize) -> Option<u8> {
        if index >= self.values.len() {
            return None;
        }
        Some(self.values[index])
    }

    pub fn write(&mut self, index: usize, value: u8) -> bool {
        if index >= self.values.len() {
            return false;
        }
        self.values[index] = value;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_in_bounds() {
        let mut memory = Memory::new(10);
        assert!(memory.write(0, 10));
        assert_eq!(memory.values[0], 10);
    }

    #[test]
    fn test_write_out_of_bounds() {
        let mut memory = Memory::new(10);
        assert!(!memory.write(100, 10));
    }

    #[test]
    fn test_read_in_bounds() {
        let mut memory = Memory::new(10);
        assert!(memory.write(0, 10));
        assert_eq!(memory.read(0), Some(10));
    }

    #[test]
    fn test_read_out_of_bounds() {
        let mut memory = Memory::new(10);
        assert!(memory.write(0, 10));
        assert_eq!(memory.read(100), None);
    }
}
