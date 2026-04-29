use crate::elements::ElementId;

pub fn hash_string(key: &str, seed: u32) -> ElementId {
    let mut hash = seed;
    for byte in key.bytes() {
        hash = hash.wrapping_add(byte as u32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    ElementId {
        id: hash.wrapping_add(1),
        offset: 0,
        base_id: hash.wrapping_add(1),
        string_id: key.to_string(),
    }
}

pub fn hash_string_with_offset(key: &str, offset: u32, seed: u32) -> ElementId {
    let mut base = seed;
    for byte in key.bytes() {
        base = base.wrapping_add(byte as u32);
        base = base.wrapping_add(base << 10);
        base ^= base >> 6;
    }
    let mut hash = base;
    hash = hash.wrapping_add(offset);
    hash = hash.wrapping_add(hash << 10);
    hash ^= hash >> 6;

    hash = hash.wrapping_add(hash << 3);
    base = base.wrapping_add(base << 3);
    hash ^= hash >> 11;
    base ^= base >> 11;
    hash = hash.wrapping_add(hash << 15);
    base = base.wrapping_add(base << 15);
    ElementId {
        id: hash.wrapping_add(1),
        offset,
        base_id: base.wrapping_add(1),
        string_id: key.to_string(),
    }
}

pub fn hash_number(offset: u32, seed: u32) -> ElementId {
    let mut hash = seed;
    hash = hash.wrapping_add(offset.wrapping_add(48));
    hash = hash.wrapping_add(hash << 10);
    hash ^= hash >> 6;

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    ElementId {
        id: hash.wrapping_add(1),
        offset,
        base_id: seed,
        string_id: String::new(),
    }
}

pub fn hash_string_contents_with_config(
    text: &str,
    is_statically_allocated: bool,
    font_id: u16,
    font_size: u16,
    letter_spacing: u16,
) -> u32 {
    let mut hash: u32 = 0;
    if is_statically_allocated {
        hash = hash.wrapping_add(text.as_ptr() as u32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
        hash = hash.wrapping_add(text.len() as u32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    } else {
        hash = (hash_data(text.as_bytes()) % u32::MAX as u64) as u32;
    }

    hash = hash.wrapping_add(font_id as u32);
    hash = hash.wrapping_add(hash << 10);
    hash ^= hash >> 6;

    hash = hash.wrapping_add(font_size as u32);
    hash = hash.wrapping_add(hash << 10);
    hash ^= hash >> 6;

    hash = hash.wrapping_add(letter_spacing as u32);
    hash = hash.wrapping_add(hash << 10);
    hash ^= hash >> 6;

    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash.wrapping_add(1)
}

fn hash_data(data: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    for &byte in data {
        hash = hash.wrapping_add(byte as u64);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_string_deterministic() {
        let a = hash_string("hello", 0);
        let b = hash_string("hello", 0);
        assert_eq!(a.id, b.id);
        assert_ne!(a.id, 0);
    }

    #[test]
    fn hash_string_different_seeds() {
        let a = hash_string("hello", 0);
        let b = hash_string("hello", 42);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn hash_number_deterministic() {
        let a = hash_number(5, 100);
        let b = hash_number(5, 100);
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn hash_string_with_offset_deterministic() {
        let a = hash_string_with_offset("test", 3, 0);
        let b = hash_string_with_offset("test", 3, 0);
        assert_eq!(a.id, b.id);
        assert_eq!(a.base_id, b.base_id);
    }
}
