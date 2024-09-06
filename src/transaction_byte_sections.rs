use {
    ratatui::style::Color,
    solana_sdk::{
        hash::Hash,
        pubkey::Pubkey,
        short_vec::ShortU16,
        signature::Signature,
        transaction::{TransactionVersion, VersionedTransaction},
    },
};

pub struct TransactionByteSection {
    pub label: Option<String>,
    pub bytes: Vec<u8>,
    pub color: Color,
}

thread_local! {
    static COLOR_SET: TransactionColorSet = TransactionColorSet::new();
}

pub fn get_transaction_byte_sections(
    transaction: &VersionedTransaction,
    sections: &mut Vec<TransactionByteSection>,
) {
    // Make sure the sections are empty
    sections.clear();

    // Get the transaction raw bytes
    let bytes = bincode::serialize(&transaction).unwrap();

    // Split the bytes into sections by content.
    let mut offset = 0;
    add_signature_sections(transaction, &bytes, sections, &mut offset);
    add_message_header_sections(transaction, &bytes, sections, &mut offset);
    add_static_account_keys_sections(transaction, &bytes, sections, &mut offset);
    add_recent_blockhash_section(transaction, &bytes, sections, &mut offset);
    add_instructions_sections(transaction, &bytes, sections, &mut offset);
    add_message_address_table_lookups_sections(transaction, &bytes, sections, &mut offset);
}

fn add_signature_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    sections.push(TransactionByteSection {
        label: Some("Signature Count".to_owned()),
        bytes: get_bytes(bytes, offset, 1),
        color: COLOR_SET.with(|color_set| color_set.signature_count_color),
    });

    for (index, _signature) in transaction.signatures.iter().enumerate() {
        sections.push(TransactionByteSection {
            label: Some(format!("Signature ({index})")),
            bytes: get_bytes(bytes, offset, core::mem::size_of::<Signature>()),
            color: COLOR_SET.with(|color_set| color_set.static_account_key_colors[index]),
        })
    }
}

fn add_message_header_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    match transaction.version() {
        TransactionVersion::Legacy(_) => {}
        TransactionVersion::Number(_) => {
            sections.push(TransactionByteSection {
                label: Some("Version Byte".to_owned()),
                bytes: get_bytes(bytes, offset, 1),
                color: COLOR_SET.with(|color_set| color_set.version_byte_color),
            });
        }
    }
    sections.push(TransactionByteSection {
        label: Some("num_required_signatures".to_owned()),
        bytes: get_bytes(bytes, offset, 1),
        color: COLOR_SET.with(|color_set| color_set.num_required_signatures_color),
    });
    sections.push(TransactionByteSection {
        label: Some("num_readonly_signed_accounts".to_owned()),
        bytes: get_bytes(bytes, offset, 1),
        color: COLOR_SET.with(|color_set| color_set.num_readonly_signed_accounts_color),
    });
    sections.push(TransactionByteSection {
        label: Some("num_readonly_unsigned_accounts".to_owned()),
        bytes: get_bytes(bytes, offset, 1),
        color: COLOR_SET.with(|color_set| color_set.num_readonly_unsigned_accounts_color),
    });
}

fn add_static_account_keys_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    sections.push(TransactionByteSection {
        label: Some("Static Account Keys Count".to_owned()),
        bytes: get_bytes(bytes, offset, 1),
        color: Color::Yellow,
    });

    for (index, _account_key) in transaction.message.static_account_keys().iter().enumerate() {
        sections.push(TransactionByteSection {
            label: Some(format!("Static Account Key ({index})")),
            bytes: get_bytes(bytes, offset, core::mem::size_of::<Pubkey>()),
            color: COLOR_SET.with(|color_set| color_set.static_account_key_colors[index]),
        });
    }
}

fn add_recent_blockhash_section(
    _transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let recent_blockhash_bytes = get_bytes(bytes, offset, core::mem::size_of::<Hash>());
    sections.push(TransactionByteSection {
        label: Some("Recent Blockhash".to_owned()),
        bytes: recent_blockhash_bytes,
        color: COLOR_SET.with(|color_set| color_set.recent_blockhash_color),
    });
}

fn add_instructions_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let num_instructions_count_bytes =
        bincode::serialized_size(&ShortU16(transaction.message.instructions().len() as u16))
            .unwrap() as usize;
    let num_instructions_count_bytes = get_bytes(bytes, offset, num_instructions_count_bytes);
    sections.push(TransactionByteSection {
        label: Some("Number of Instructions".to_owned()),
        bytes: num_instructions_count_bytes,
        color: COLOR_SET.with(|color_set| color_set.num_instructions_color),
    });

    for instruction in transaction.message.instructions() {
        let program_id_index = instruction.program_id_index as usize;
        sections.push(TransactionByteSection {
            label: None, // color corresponds to the program id
            bytes: get_bytes(bytes, offset, 1),
            color: COLOR_SET.with(|color_set| {
                color_set
                    .static_account_key_colors
                    .get(program_id_index)
                    .copied()
                    .unwrap_or(Color::White)
            }),
        });

        let num_accounts_bytes =
            bincode::serialized_size(&ShortU16(instruction.accounts.len() as u16)).unwrap()
                as usize;
        let num_accounts_bytes = get_bytes(bytes, offset, num_accounts_bytes);
        sections.push(TransactionByteSection {
            label: Some("Instruction Number of Accounts".to_owned()),
            bytes: num_accounts_bytes,
            color: COLOR_SET.with(|color_set| color_set.instruction_num_accounts_color),
        });
        let accounts_bytes = get_bytes(bytes, offset, instruction.accounts.len());
        sections.push(TransactionByteSection {
            label: Some("Instruction Accounts".to_owned()),
            bytes: accounts_bytes,
            color: COLOR_SET.with(|color_set| color_set.instruction_accounts_color),
        });

        let data_length_bytes =
            bincode::serialized_size(&ShortU16(instruction.data.len() as u16)).unwrap() as usize;
        let data_length_bytes = get_bytes(bytes, offset, data_length_bytes);
        sections.push(TransactionByteSection {
            label: Some("Instruction Data Length".to_owned()),
            bytes: data_length_bytes,
            color: COLOR_SET.with(|color_set| color_set.instruction_data_length_color),
        });
        let data = get_bytes(bytes, offset, instruction.data.len());
        sections.push(TransactionByteSection {
            label: Some("Instruction Data".to_owned()),
            bytes: data,
            color: COLOR_SET.with(|color_set| color_set.instruction_data_color),
        });
    }
}

fn add_message_address_table_lookups_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let Some(address_table_lookups) = transaction.message.address_table_lookups() else {
        return;
    };

    let num_address_table_lookups_bytes =
        bincode::serialized_size(&ShortU16(address_table_lookups.len() as u16)).unwrap() as usize;
    let num_address_table_lookups_bytes = get_bytes(bytes, offset, num_address_table_lookups_bytes);
    sections.push(TransactionByteSection {
        label: Some("Message Address Table Lookups Count".to_owned()),
        bytes: num_address_table_lookups_bytes,
        color: COLOR_SET.with(|color_set| color_set.atl_count_color),
    });

    for _atl in address_table_lookups {
        // Address
        let address = get_bytes(bytes, offset, core::mem::size_of::<Pubkey>());
        sections.push(TransactionByteSection {
            label: Some("Message Address Table Lookup Address".to_owned()),
            bytes: address,
            color: COLOR_SET.with(|color_set| color_set.atl_address_color),
        });

        // Write
        let write_count_bytes = get_bytes(bytes, offset, 1);
        let write_count = write_count_bytes[0] as usize;
        sections.push(TransactionByteSection {
            label: Some("Message Address Table Lookup Write Count".to_owned()),
            bytes: write_count_bytes,
            color: COLOR_SET.with(|color_set| color_set.atl_write_count_color),
        });
        sections.push(TransactionByteSection {
            label: Some("Message Address Table Lookup Write Set".to_owned()),
            bytes: get_bytes(bytes, offset, write_count),
            color: COLOR_SET.with(|color_set| color_set.atl_write_set_color),
        });

        // Read
        let read_count_bytes = get_bytes(bytes, offset, 1);
        let read_count = read_count_bytes[0] as usize;
        sections.push(TransactionByteSection {
            label: Some("Message Address Table Lookup Read Count".to_owned()),
            bytes: read_count_bytes,
            color: COLOR_SET.with(|color_set| color_set.atl_read_count_color),
        });
        sections.push(TransactionByteSection {
            label: Some("Message Address Table Lookup Read Set".to_owned()),
            bytes: get_bytes(bytes, offset, read_count),
            color: COLOR_SET.with(|color_set| color_set.atl_read_set_color),
        });
    }
}

fn get_bytes(bytes: &[u8], offset: &mut usize, num_bytes: usize) -> Vec<u8> {
    let result = bytes[*offset..*offset + num_bytes].to_vec();
    *offset += num_bytes;
    result
}

fn generate_color_set(n: usize) -> Vec<Color> {
    let mut colors = Vec::new();
    let step = 256 / (n).max(1); // Spacing colors evenly in RGB space
    for i in 0..n {
        let r = ((i * step) % 256) as u8;
        let g = ((i * step + 85) % 256) as u8; // Shift green by 85 for variety
        let b = ((i * step + 170) % 256) as u8; // Shift blue by 170 for variety
        colors.push((r, g, b));
    }

    let mut sum_r = 0.0;
    let mut sum_g = 0.0;
    let mut sum_b = 0.0;
    let mut count = 0.0;

    let mut ordered = Vec::with_capacity(colors.len());
    ordered.push(colors.pop().unwrap());
    // Update the running sums and count
    let first_color = ordered[0];
    sum_r += first_color.0 as f64;
    sum_g += first_color.1 as f64;
    sum_b += first_color.2 as f64;
    count += 1.0;

    // Greedily pick the next color that is farthest from the current average color.
    while !colors.is_empty() {
        // Calculate the current average color
        let avg_color = (sum_r / count, sum_g / count, sum_b / count);

        let mut max_distance = 0.0;
        let mut max_index = 0;
        for (index, color) in colors.iter().enumerate() {
            let distance =
                euclidean_distance((color.0 as f64, color.1 as f64, color.2 as f64), avg_color);
            if distance > max_distance {
                max_distance = distance;
                max_index = index;
            }
        }

        // Add the farthest color to the ordered list
        let next_color = colors.swap_remove(max_index);
        ordered.push(next_color);

        // Update running sums and count with the new color
        sum_r += next_color.0 as f64;
        sum_g += next_color.1 as f64;
        sum_b += next_color.2 as f64;
        count += 1.0;
    }

    assert!(ordered.len() == n);

    ordered
        .into_iter()
        .map(|(r, g, b)| Color::Rgb(r, g, b))
        .collect()
}

fn euclidean_distance(c1: (f64, f64, f64), c2: (f64, f64, f64)) -> f64 {
    let (r1, g1, b1) = c1;
    let (r2, g2, b2) = c2;
    ((r2 - r1).powi(2) + (g2 - g1).powi(2) + (b2 - b1).powi(2)).sqrt()
}

struct TransactionColorSet {
    signature_count_color: Color,
    version_byte_color: Color,
    num_required_signatures_color: Color,
    num_readonly_signed_accounts_color: Color,
    num_readonly_unsigned_accounts_color: Color,
    recent_blockhash_color: Color,
    num_instructions_color: Color,
    instruction_num_accounts_color: Color,
    instruction_accounts_color: Color,
    instruction_data_length_color: Color,
    instruction_data_color: Color,
    atl_count_color: Color,
    atl_address_color: Color,
    atl_write_count_color: Color,
    atl_read_count_color: Color,
    atl_write_set_color: Color,
    atl_read_set_color: Color,
    static_account_key_colors: Vec<Color>,
}

impl TransactionColorSet {
    fn new() -> Self {
        let color_set = generate_color_set(60);
        let num_static_account_keys = color_set.len() - 17;
        let non_account_colors = &color_set[num_static_account_keys..];

        Self {
            signature_count_color: non_account_colors[0],
            version_byte_color: non_account_colors[1],
            num_required_signatures_color: non_account_colors[2],
            num_readonly_signed_accounts_color: non_account_colors[3],
            num_readonly_unsigned_accounts_color: non_account_colors[4],
            recent_blockhash_color: non_account_colors[5],
            num_instructions_color: non_account_colors[6],
            instruction_num_accounts_color: non_account_colors[7],
            instruction_accounts_color: non_account_colors[8],
            instruction_data_length_color: non_account_colors[9],
            instruction_data_color: non_account_colors[10],
            atl_count_color: non_account_colors[11],
            atl_address_color: non_account_colors[12],
            atl_write_count_color: non_account_colors[13],
            atl_read_count_color: non_account_colors[14],
            atl_write_set_color: non_account_colors[15],
            atl_read_set_color: non_account_colors[16],
            static_account_key_colors: color_set[..num_static_account_keys].to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_color_set() {
        let color_set = generate_color_set(100);
        assert_eq!(color_set.len(), 100);
    }
}
