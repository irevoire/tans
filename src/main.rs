use std::collections::HashMap;

fn main() {
    let table_log = 5;
    let table_size = 1 << table_log;

    // Define how often a symbol is seen, total should equal the
    // table size.
    let symbol_occurrences = [(b'0', 10), (b'1', 10), (b'2', 12)];

    // Define the Initial Positions of States in StateList.
    let symbol_list: Vec<_> = symbol_occurrences.iter().map(|(key, _)| key).collect();
    let mut cumulative = vec![0];
    for (_symbol, occurences) in symbol_occurrences {
        let last = cumulative.last().unwrap();
        cumulative.push(last + occurences);
    }
    let last = cumulative.last().unwrap();
    cumulative.push(last + 1);
    println!("cumsum: {cumulative:?}");
    println!();

    // Spread Symbols to Create the States Table
    let high_thresh = table_size - 1;
    let mut state_table = vec![b'a'; table_size];
    let table_mask = table_size - 1;
    let step = (table_size >> 1) + (table_size >> 3) + 3;
    let mut pos = 0;
    for (symbol, occurrences) in symbol_occurrences {
        for _ in 0..occurrences {
            state_table[pos] = symbol;
            pos = (pos + step) & table_mask;
            // while pos > highThresh {
            //     TODO: What is this position?
            //           it wasn't used in the original code
            //     position = (pos + step) & tableMask
            // }
        }
    }
    assert!(pos == 0);
    println!("state table: {state_table:?}");
    println!();

    // Build Coding Table from State Table
    let mut output_bits = vec![0; table_size];
    let mut coding_table = vec![0; table_size];
    let mut cumulative_cp = cumulative.clone();
    for i in 0..table_size {
        let s = state_table[i];
        let index = symbol_list.iter().position(|symbol| **symbol == s).unwrap();
        coding_table[cumulative_cp[index]] = table_size + i;
        cumulative_cp[index] += 1;
        output_bits[i] = table_log - first1_index(table_size + i);
    }
    println!("output bits: {output_bits:?}");
    println!("coding table: {coding_table:?}");
    println!();

    // Create the Symbol Transformation Table
    let mut total: usize = 0;
    let mut symbol_tt: SymbolTT = HashMap::new();
    for (symbol, occurrences) in symbol_occurrences {
        let transform = if occurrences == 1 {
            Transformation {
                delta_nb_bits: (table_log << 16) - (1 << table_log),
                delta_find_state: total as isize - 1,
            }
        } else if occurrences > 0 {
            let max_bits_out = table_log - first1_index(occurrences - 1);
            let min_state_plus = occurrences << max_bits_out;
            let transform = Transformation {
                delta_nb_bits: (max_bits_out << 16) - min_state_plus,
                delta_find_state: (total as isize) - (occurrences as isize),
            };
            total += occurrences;
            transform
        } else {
            panic!();
        };
        symbol_tt.insert(symbol, transform);
    }
    println!("symbol TT: {symbol_tt:?}");
    println!();

    // Generate a Decoding Table
    let mut decode_table = Vec::with_capacity(table_size);
    let mut nextt = symbol_occurrences.to_vec();
    for i in 0..table_size {
        let symbol = state_table[i];
        let index = symbol_list.iter().position(|s| **s == symbol).unwrap();
        let x = nextt[index].1;
        nextt[index] = (nextt[index].0, nextt[index].1 + 1);
        let nb_bits = table_log - first1_index(x);
        let new_x = (x << nb_bits) - table_size;
        decode_table.push(SymbolDecoding {
            symbol,
            nb_bits,
            new_x,
        });
    }

    println!("decode table: {decode_table:?}");
    println!();

    // Test Encoding
    let input = "1102010120";
    let bit_stream = encode_data(input, table_size, table_log, &coding_table, &symbol_tt);

    // Test Decoding
    let output = decode_data(&bit_stream, &decode_table, table_log);

    // Assert that input and Output are the same
    println!("{:?} = input", input.as_bytes());
    println!("{:?} = bitStream", bit_stream);
    println!("{:?} = output", output);
    assert_eq!(input.as_bytes(), output);
}

type SymbolTT = HashMap<u8, Transformation>;

#[derive(Debug)]
struct Transformation {
    delta_nb_bits: usize,
    delta_find_state: isize,
}

#[derive(Debug)]
struct SymbolDecoding {
    symbol: u8,
    nb_bits: usize,
    new_x: usize,
}

/// Return the Index of the First Non-Zero Bit.
fn first1_index(mut val: usize) -> usize {
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

fn decode_data(bit_stream: &str, decode_table: &[SymbolDecoding], table_log: usize) -> Vec<u8> {
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
