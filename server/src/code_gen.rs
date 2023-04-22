use rand::{thread_rng, Rng};

const CODE_CHARS: [char; 31] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
    'W', 'X', 'Y', 'Z', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub fn get_code() -> String {
    let mut rng = thread_rng();
    let mut next = move || rng.gen_range(0..CODE_CHARS.len());
    (0..6)
        .map(|_| CODE_CHARS.get(next()).expect("RNG Code gen out of range"))
        .collect()
}
