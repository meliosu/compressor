pub fn bwt(bytes: &[u8]) -> (Vec<u8>, usize) {
    if bytes.is_empty() {
        return (Vec::new(), 0);
    }

    let n = bytes.len();

    // Create indices for rotations instead of storing full rotations
    let mut indices: Vec<usize> = (0..n).collect();

    // Sort indices based on the rotations they represent
    indices.sort_by(|&a, &b| {
        // Compare rotation starting at position a with rotation starting at position b
        for i in 0..n {
            let char_a = bytes[(a + i) % n];
            let char_b = bytes[(b + i) % n];
            match char_a.cmp(&char_b) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        std::cmp::Ordering::Equal
    });

    // Extract the last column and find the index of the original string
    let mut last_column = Vec::with_capacity(n);
    let mut original_index = 0;

    for (i, &rotation_start) in indices.iter().enumerate() {
        // Last character of rotation starting at rotation_start
        last_column.push(bytes[(rotation_start + n - 1) % n]);
        if rotation_start == 0 {
            original_index = i;
        }
    }

    (last_column, original_index)
}

pub fn ibwt(bytes: &[u8], index: usize) -> Vec<u8> {
    if bytes.is_empty() {
        return Vec::new();
    }

    let n = bytes.len();

    // Create first column by sorting the last column
    let mut first_column = bytes.to_vec();
    first_column.sort_unstable();

    // Build the transformation table
    let mut next = vec![0; n];
    let mut count = [0; 256];

    // Count occurrences of each character in the last column
    for &byte in bytes {
        count[byte as usize] += 1;
    }

    // Convert counts to starting positions
    let mut total = 0;
    for i in 0..256 {
        let temp = count[i];
        count[i] = total;
        total += temp;
    }

    // Build the next array
    for (i, &byte) in bytes.iter().enumerate() {
        next[count[byte as usize]] = i;
        count[byte as usize] += 1;
    }

    // Reconstruct the original string
    let mut result = Vec::with_capacity(n);
    let mut current = index;

    for _ in 0..n {
        result.push(first_column[current]);
        current = next[current];
    }

    result
}
