use std::collections::HashMap;

const AIRTAG_PREFIX: [u8; 3] = [0x12, 0x19, 0x10];

pub fn is_airtag(manufacturer_data: &HashMap<u16, Vec<u8>>) -> bool {
    let Some(apple_data) = manufacturer_data.get(&0x004C) else { return false };

    if apple_data.len() != 28 {
        return false;
    }

    if apple_data[0..3].eq(&AIRTAG_PREFIX) {
        return true;
    }

    return false;
}