use std::collections::HashMap;


pub fn is_airtag(manufacturer_data: &HashMap<u16, Vec<u8>>) -> bool {
    let Some(apple_data) = manufacturer_data.get(&0x004C) else { return false };
    log::debug!("Found apple device!");
    
    // After removing the Data Type (Manufacturer Specific = 0xFF)
    // and the Company ID (Apple = 0x004C), we should have 27 bytes for airtag
    if apple_data.len() != 27 {
        log::debug!("Advetising Data Length != 27, not airtag...");
        return false;
    }

    // Check the prefix to make sure it _is_ an airtag
    if apple_data[0..3].eq(&super::constants::AIRTAG_PREFIX) {
        log::debug!("Advertising data matches airtag prefix!");
        return true;
    }

    log::debug!("Advertising data does not match airtag prefix, not airtag...");
    return false;
}