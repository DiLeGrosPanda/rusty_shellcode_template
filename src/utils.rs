macro_rules! obfuscate {
    ($s:expr) => {{
        const KEY: u8 = 0x42;
        const LEN: usize = $s.len();

        const fn xor_encode<const N: usize>(s: &[u8]) -> [u8; N] {
            let mut buf = [0u8; N];
            let mut i = 0usize;
            while i < s.len() {
                buf[i] = s[i] ^ KEY;
                i += 1;
            }
            buf
        }

        #[unsafe(link_section = ".text")]
        static OBFUSCATED: [u8; LEN] = xor_encode::<LEN>($s);
        (&OBFUSCATED, KEY)
    }};
}

macro_rules! obf_str {
    ($s:expr) => {{
        let (encoded, key) = crate::utils::obfuscate!($s);
        let mut buf = [0u8; $s.len()];
        let mut i = 0usize;
        while i < $s.len() {
            buf[i] = encoded[i] ^ key;
            i += 1;
        }
        buf
    }};
}

pub(crate) use {obf_str, obfuscate};
