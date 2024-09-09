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

fn generate_color_set() -> &'static [Color] {
    const COLORS: [Color; 60] = [
        Color::Rgb(255, 228, 196), // bisque
        Color::Rgb(47, 79, 79),    // darkslategray
        Color::Rgb(85, 107, 47),   // darkolivegreen
        Color::Rgb(107, 142, 35),  // olivedrab
        Color::Rgb(160, 82, 45),   // sienna
        Color::Rgb(46, 139, 87),   // seagreen
        Color::Rgb(128, 0, 0),     // maroon
        Color::Rgb(25, 25, 112),   // midnightblue
        Color::Rgb(112, 128, 144), // slategray
        Color::Rgb(0, 128, 0),     // green
        Color::Rgb(188, 143, 143), // rosybrown
        Color::Rgb(102, 51, 153),  // rebeccapurple
        Color::Rgb(184, 134, 11),  // darkgoldenrod
        Color::Rgb(0, 139, 139),   // darkcyan
        Color::Rgb(205, 133, 63),  // peru
        Color::Rgb(70, 130, 180),  // steelblue
        Color::Rgb(210, 105, 30),  // chocolate
        Color::Rgb(143, 188, 143), // darkseagreen
        Color::Rgb(0, 0, 139),     // darkblue
        Color::Rgb(50, 205, 50),   // limegreen
        Color::Rgb(127, 0, 127),   // purple2
        Color::Rgb(176, 48, 96),   // maroon3
        Color::Rgb(154, 205, 50),  // yellowgreen
        Color::Rgb(72, 209, 204),  // mediumturquoise
        Color::Rgb(153, 50, 204),  // darkorchid
        Color::Rgb(255, 0, 0),     // red
        Color::Rgb(255, 165, 0),   // orange
        Color::Rgb(255, 215, 0),   // gold
        Color::Rgb(199, 21, 133),  // mediumvioletred
        Color::Rgb(0, 0, 205),     // mediumblue
        Color::Rgb(222, 184, 135), // burlywood
        Color::Rgb(0, 255, 0),     // lime
        Color::Rgb(0, 250, 154),   // mediumspringgreen
        Color::Rgb(65, 105, 225),  // royalblue
        Color::Rgb(220, 20, 60),   // crimson
        Color::Rgb(0, 255, 255),   // aqua
        Color::Rgb(0, 191, 255),   // deepskyblue
        Color::Rgb(147, 112, 219), // mediumpurple
        Color::Rgb(0, 0, 255),     // blue
        Color::Rgb(160, 32, 240),  // purple3
        Color::Rgb(240, 128, 128), // lightcoral
        Color::Rgb(173, 255, 47),  // greenyellow
        Color::Rgb(255, 99, 71),   // tomato
        Color::Rgb(218, 112, 214), // orchid
        Color::Rgb(216, 191, 216), // thistle
        Color::Rgb(255, 0, 255),   // fuchsia
        Color::Rgb(219, 112, 147), // palevioletred
        Color::Rgb(240, 230, 140), // khaki
        Color::Rgb(255, 255, 84),  // laserlemon
        Color::Rgb(100, 149, 237), // cornflower
        Color::Rgb(221, 160, 221), // plum
        Color::Rgb(144, 238, 144), // lightgreen
        Color::Rgb(135, 206, 235), // skyblue
        Color::Rgb(255, 20, 147),  // deeppink
        Color::Rgb(255, 160, 122), // lightsalmon
        Color::Rgb(175, 238, 238), // paleturquoise
        Color::Rgb(127, 255, 212), // aquamarine
        Color::Rgb(255, 105, 180), // hotpink
        Color::Rgb(169, 169, 169), // darkgray
        Color::Rgb(255, 182, 193), // lightpink
    ];

    &COLORS[..]
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
        const NUM_NON_ACCOUNT_COLORS: usize = 17;
        let color_set = generate_color_set();
        let non_account_colors = &color_set[..NUM_NON_ACCOUNT_COLORS];

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
            static_account_key_colors: color_set[NUM_NON_ACCOUNT_COLORS..].to_vec(),
        }
    }
}
