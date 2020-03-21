pub(crate) fn encode(val: u64, precision: usize) -> String {
    let mut chars = Vec::<char>::new();
    let dict32 = vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p',
        'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

    let mut val = val;

    for _ in 0..precision {
        let ch = dict32[(val & 31) as usize];
        chars.insert(0, ch);
        val >>= 5;
    }

    chars.iter().collect()
}

pub fn decode_chr(c: u8) -> u64 {
    let dec_vec = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
        0,0,0,0,0,0,0, // :-@
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0, // A-Z
        0,0,0,0,0,0, //[..`
        0, // a
        10,11,12,13,14,15,16,
        0, //i
        17,18,
        0, //l
        19,20,
        0, //o
        21,22,23,24,25,26,27,28,29,30,31];

    let c = (c as usize) - 48;
    if c < 0 || c > 75 {
        debug!("bad input");
        return 0;
    }

    dec_vec[c]
}

pub fn decode(string: &str) -> u64 {
    let mut val = 0;

    string.as_bytes()
        .iter()
        .for_each(|c| val = (val << 5) | decode_chr(*c));

    val
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_encode() {
        assert_eq!(super::encode(0b01001_10110_01000_11110_11110, 5),
            "9q8yy");
    }
    #[test]
    fn test_decode() {
        env_logger::init();
        assert_eq!(super::decode("9q8yy"), 10167262);
    }

}
