pub fn bwt(bytes: &[u8]) -> (Vec<u8>, usize) {
    if bytes.is_empty() {
        return (Vec::new(), 0);
    }

    let n = bytes.len();

    let mut rotations: Vec<(Vec<u8>, usize)> = (0..n)
        .map(|i| {
            let mut rotation = Vec::with_capacity(n);
            rotation.extend_from_slice(&bytes[i..]);
            rotation.extend_from_slice(&bytes[..i]);
            (rotation, i)
        })
        .collect();

    rotations.sort_by(|a, b| a.0.cmp(&b.0));

    let mut last_column = Vec::with_capacity(n);
    let mut original_index = 0;

    for (i, (rotation, orig_idx)) in rotations.iter().enumerate() {
        last_column.push(rotation[n - 1]);
        if *orig_idx == 0 {
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

    let mut first_column = bytes.to_vec();
    first_column.sort_unstable();

    let mut next = vec![0; n];
    let mut count = [0; 256];

    for &byte in bytes {
        count[byte as usize] += 1;
    }

    let mut total = 0;
    for i in 0..256 {
        let temp = count[i];
        count[i] = total;
        total += temp;
    }

    for (i, &byte) in bytes.iter().enumerate() {
        next[count[byte as usize]] = i;
        count[byte as usize] += 1;
    }

    let mut result = Vec::with_capacity(n);
    let mut current = index;

    for _ in 0..n {
        result.push(first_column[current]);
        current = next[current];
    }

    result
}
