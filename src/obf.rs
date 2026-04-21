pub struct Obf<const N: usize> {
    data: [u8; N],
    key: u8,
}

pub const fn obfuscate_literal<const N: usize>(input: &[u8], key: u8) -> Obf<N> {
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N {
        out[i] = input[i] ^ key.wrapping_add((i as u8).wrapping_mul(17));
        i += 1;
    }

    Obf { data: out, key }
}

impl<const N: usize> Obf<N> {
    pub fn reveal(&self) -> SecretString {
        let mut buf = Vec::with_capacity(N);
        let mut i = 0;
        while i < N {
            buf.push(self.data[i] ^ self.key.wrapping_add((i as u8).wrapping_mul(17)));
            i += 1;
        }

        SecretString { buf }
    }
}

pub struct SecretString {
    buf: Vec<u8>,
}

impl SecretString {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf).unwrap_or("")
    }

    pub fn into_string(mut self) -> String {
        let buf = std::mem::take(&mut self.buf);
        String::from_utf8(buf).unwrap_or_default()
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        self.buf.fill(0);
    }
}

#[macro_export]
macro_rules! obfstr {
    ($s:literal) => {{
        const KEY: u8 = 0xA7;
        const OBF: $crate::obf::Obf<{ $s.len() }> =
            $crate::obf::obfuscate_literal($s.as_bytes(), KEY);
        OBF.reveal()
    }};
}
