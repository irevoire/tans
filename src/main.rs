use tans::*;

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
    let mut symbol_tt: SymbolTT = SymbolTT::new();
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
