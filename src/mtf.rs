pub fn mtf(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut symbol_table: Vec<u8> = (0..=255).collect();

    for &byte in bytes {
        // Find the position of the byte in the symbol table
        let position = symbol_table.iter().position(|&x| x == byte).unwrap();

        // Output the position
        result.push(position as u8);

        // Move the symbol to the front
        let symbol = symbol_table.remove(position);
        symbol_table.insert(0, symbol);
    }

    result
}

pub fn imtf(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(bytes.len());
    let mut symbol_table: Vec<u8> = (0..=255).collect();

    for &position in bytes {
        // Get the symbol at the given position
        let symbol = symbol_table[position as usize];

        // Output the symbol
        result.push(symbol);

        // Move the symbol to the front
        let symbol = symbol_table.remove(position as usize);
        symbol_table.insert(0, symbol);
    }

    result
}
