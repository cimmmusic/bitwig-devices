/// Dag XOR stream cipher used by Bitwig for .bwmodule encryption.
pub(crate) struct Dag {
    swc_idx: usize,
    azd_idx: usize,
    cycle: u32,
    swc: Vec<u8>,
    azd: Vec<u8>,
}

impl Dag {
    pub fn new(key: &[u8], nonce: &[u8]) -> Self {
        let swc = key[16..].to_vec();
        let mut azd = Vec::with_capacity(nonce.len() + 16);
        azd.extend_from_slice(nonce);
        azd.extend_from_slice(&key[..16]);
        Dag { swc_idx: 0, azd_idx: 0, cycle: 0, swc, azd }
    }

    fn next_byte(&mut self, input: u8) -> u8 {
        if self.swc_idx >= self.swc.len() {
            self.swc_idx = 0;
            self.cycle += 1;
        }
        if self.azd_idx >= self.azd.len() {
            self.azd_idx = 0;
        }
        let s = self.swc[self.swc_idx];
        let a = self.azd[self.azd_idx];
        self.swc_idx += 1;
        self.azd_idx += 1;
        let rot = (self.cycle & 7) as u32;
        let rotated = (a as u32) >> rot | ((a as u32) << (8 - rot));
        input ^ s ^ (rotated as u8)
    }

    /// XOR cipher is symmetric — encrypt and decrypt are the same operation.
    pub fn apply(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&b| self.next_byte(b)).collect()
    }
}

