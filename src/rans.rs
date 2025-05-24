use std::io;

const RANS_BYTE_L: u32 = 1 << 23; // Lower bound for renormalization

pub struct ANSCoder {
    freq: [u32; 256],
    cum_freq: [u32; 257],
    total_freq: u32,
}

impl ANSCoder {
    pub fn new() -> Self {
        Self {
            freq: [0; 256],
            cum_freq: [0; 257],
            total_freq: 0,
        }
    }

    fn build_frequency_table(&mut self, data: &[u8]) {
        // Reset frequencies
        self.freq.fill(0);

        // Count frequencies
        for &byte in data {
            self.freq[byte as usize] += 1;
        }

        // Ensure no zero frequencies (add 1 to each)
        for freq in &mut self.freq {
            *freq += 1;
        }

        // Build cumulative frequency table
        self.cum_freq[0] = 0;
        for i in 0..256 {
            self.cum_freq[i + 1] = self.cum_freq[i] + self.freq[i];
        }
        self.total_freq = self.cum_freq[256];
    }

    fn rans_encode_put(&self, state: &mut u32, output: &mut Vec<u8>, sym: u8) {
        let symbol = sym as usize;
        let freq = self.freq[symbol];
        let start = self.cum_freq[symbol];

        // Renormalize if needed
        while *state >= ((RANS_BYTE_L >> 8) << 8) * freq {
            output.push(*state as u8);
            *state >>= 8;
        }

        // Encode symbol
        *state = ((*state / freq) << 8) + (*state % freq) + start;
    }

    fn rans_decode_get<'a>(
        &self,
        state: &mut u32,
        input: &mut impl Iterator<Item = &'a u8>,
    ) -> Option<u8> {
        // Renormalize if needed
        while *state < RANS_BYTE_L {
            if let Some(&byte) = input.next() {
                *state = (*state << 8) | (byte as u32);
            } else {
                return None;
            }
        }

        // Decode symbol
        let cum = *state % self.total_freq;

        // Find symbol using binary search
        let mut symbol = 0;
        for i in 0..256 {
            if self.cum_freq[i + 1] > cum {
                symbol = i;
                break;
            }
        }

        let freq = self.freq[symbol];
        let start = self.cum_freq[symbol];

        *state = freq * (*state / self.total_freq) + cum - start;

        Some(symbol as u8)
    }

    pub fn encode(&mut self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        if bytes.is_empty() {
            return Ok(Vec::new());
        }

        self.build_frequency_table(bytes);

        let mut output = Vec::new();
        let mut state = RANS_BYTE_L;

        // Write frequency table to output
        for &freq in &self.freq {
            output.extend_from_slice(&freq.to_le_bytes());
        }

        // Encode symbols in reverse order
        for &byte in bytes.iter().rev() {
            self.rans_encode_put(&mut state, &mut output, byte);
        }

        // Write final state
        output.extend_from_slice(&state.to_le_bytes());

        // Write original length
        output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());

        Ok(output)
    }

    pub fn decode(&mut self, bytes: &[u8]) -> io::Result<Vec<u8>> {
        if bytes.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid compressed data",
            ));
        }

        let mut cursor = bytes.len();

        // Read original length
        cursor -= 4;
        let original_len = u32::from_le_bytes([
            bytes[cursor],
            bytes[cursor + 1],
            bytes[cursor + 2],
            bytes[cursor + 3],
        ]) as usize;

        if original_len == 0 {
            return Ok(Vec::new());
        }

        // Read final state
        cursor -= 4;
        let mut state = u32::from_le_bytes([
            bytes[cursor],
            bytes[cursor + 1],
            bytes[cursor + 2],
            bytes[cursor + 3],
        ]);

        // Read frequency table
        if cursor < 256 * 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid frequency table",
            ));
        }

        cursor -= 256 * 4;
        for i in 0..256 {
            let freq_bytes = [
                bytes[cursor + i * 4],
                bytes[cursor + i * 4 + 1],
                bytes[cursor + i * 4 + 2],
                bytes[cursor + i * 4 + 3],
            ];
            self.freq[i] = u32::from_le_bytes(freq_bytes);
        }

        // Rebuild cumulative frequency table
        self.cum_freq[0] = 0;
        for i in 0..256 {
            self.cum_freq[i + 1] = self.cum_freq[i] + self.freq[i];
        }
        self.total_freq = self.cum_freq[256];

        // Decode symbols
        let encoded_data = &bytes[..cursor];
        let mut input_iter = encoded_data.iter().rev();
        let mut output = Vec::with_capacity(original_len);

        for _ in 0..original_len {
            if let Some(symbol) = self.rans_decode_get(&mut state, &mut input_iter) {
                output.push(symbol);
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Unexpected end of input",
                ));
            }
        }

        Ok(output)
    }
}
