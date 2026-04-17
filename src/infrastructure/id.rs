use cuid2::CuidConstructor;

use crate::core::traits::IdGenerator;

pub struct Cuid2Generator {
    constructor: CuidConstructor,
}

impl Cuid2Generator {
    pub fn new() -> Self {
        Self {
            constructor: CuidConstructor::new(),
        }
    }
}

impl Default for Cuid2Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator for Cuid2Generator {
    fn generate(&self) -> String {
        self.constructor.create_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuid_generation() {
        let generator = Cuid2Generator::new();
        let id1 = generator.generate();
        let id2 = generator.generate();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2, "CUIDs should be unique");
        assert_eq!(id1.len(), 24, "CUID2 length should be 24 characters");
    }
}
