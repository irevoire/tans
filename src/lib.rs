use std::collections::HashMap;

pub type SymbolTT = HashMap<u8, Transformation>;

#[derive(Debug)]
pub struct Transformation {
    pub delta_nb_bits: usize,
    pub delta_find_state: isize,
}

#[derive(Debug)]
pub struct SymbolDecoding {
    pub symbol: u8,
    pub nb_bits: usize,
    pub new_x: usize,
}

/// Return the Index of the First Non-Zero Bit.
pub fn first1_index(mut val: usize) -> usize {
    let mut counter = 0;

    while val > 1 {
        counter += 1;
        val = val >> 1;
    }
    counter
}

/// Output NbBits to a BitStream
fn output_nb_bits(state: usize, nb_bits: usize) -> String {
    let mask = (1 << nb_bits) - 1;
    let little = state & mask;
    let mut string;
    if nb_bits > 0 {
        string = format!("{little:b}");
    } else {
        return String::from("");
    }
    while string.len() < nb_bits {
        string = format!("0{string}");
    }
    return string;
}

///  Encode a Symbol Using tANS, giving the current state, the symbol, and the bitstream and STT
fn encode_symbol(
    symbol: u8,
    mut state: usize,
    coding_table: &[usize],
    mut bit_stream: String,
    symbol_tt: &SymbolTT,
) -> (usize, String) {
    let symbol_tt = &symbol_tt[&symbol];
    let nb_bits_out = (state + symbol_tt.delta_nb_bits) >> 16;
    bit_stream += &output_nb_bits(state, nb_bits_out);
    state = coding_table[((state >> nb_bits_out) as isize + symbol_tt.delta_find_state) as usize];
    (state, bit_stream)
}

/// Convert Bits from Bitstream to the new State.
fn bits_to_state(bit_stream: &str, nb_bits: usize) -> (usize, &str) {
    let bits = &bit_stream[bit_stream.len() - nb_bits..];
    // let rest = int(bits, 2);
    let rest = usize::from_str_radix(bits, 2).unwrap();
    if nb_bits == bit_stream.len() {
        let remaining = "";
        return (rest, remaining);
    }
    let remaining = &bit_stream[..bit_stream.len() - nb_bits];
    (rest, remaining)
}

/// Return a Symbol + New State + Bitstream from the bitStream and State.
fn decode_symbol<'a>(
    state: usize,
    bit_stream: &'a str,
    state_t: &[SymbolDecoding],
) -> (u8, usize, &'a str) {
    let symbol = state_t[state].symbol;
    let nb_bits = state_t[state].nb_bits;
    let (rest, bit_stream) = bits_to_state(bit_stream, nb_bits);
    let state = state_t[state].new_x + rest;
    (symbol, state, bit_stream)
}

/// Functions to Encode and Decode Streams of Data.
pub fn encode_data(
    input: impl AsRef<[u8]>,
    table_size: usize,
    table_log: usize,
    coding_table: &[usize],
    symbol_tt: &SymbolTT,
) -> String {
    let input = input.as_ref();
    let bit_stream = String::new();
    let (mut state, mut bit_stream) =
        encode_symbol(input[0], 0, coding_table, bit_stream, symbol_tt);
    bit_stream.clear();
    for c in input {
        let (s, bs) = encode_symbol(*c, state, coding_table, bit_stream, symbol_tt);
        state = s;
        bit_stream = bs;
    }
    bit_stream += &output_nb_bits(state - table_size, table_log); // Includes Current Bit
    bit_stream
}

pub fn decode_data(bit_stream: &str, decode_table: &[SymbolDecoding], table_log: usize) -> Vec<u8> {
    let mut output = Vec::new();
    let (mut state, mut bit_stream) = bits_to_state(bit_stream, table_log);
    while bit_stream.len() > 0 {
        let (symbol, st, bs) = decode_symbol(state, bit_stream, decode_table);
        state = st;
        bit_stream = bs;
        output.push(symbol);
    }
    output.reverse();
    output
}
