use {
    ratatui::style::Color,
    solana_sdk::{
        hash::Hash,
        short_vec::ShortVec,
        signature::Signature,
        transaction::{TransactionVersion, VersionedTransaction},
    },
};

pub struct TransactionByteSection {
    pub label: &'static str,
    pub bytes: Vec<u8>,
    pub color: Color,
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
    let num_signatures = transaction.signatures.len();
    let num_signature_bytes = 1 + num_signatures * core::mem::size_of::<Signature>();
    let signature_bytes = bytes[*offset..*offset + num_signature_bytes].to_vec();
    *offset += num_signature_bytes;

    sections.push(TransactionByteSection {
        label: "Signatures",
        bytes: signature_bytes,
        color: Color::LightGreen,
    });
}

fn add_message_header_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let header_length = 3 + match transaction.version() {
        TransactionVersion::Legacy(_) => 0,
        TransactionVersion::Number(_) => 1,
    };
    let header_bytes = get_bytes(bytes, offset, header_length);

    sections.push(TransactionByteSection {
        label: "Message Header",
        bytes: header_bytes,
        color: Color::Blue,
    });
}

fn add_static_account_keys_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let num_static_account_keys_bytes =
        1 + core::mem::size_of_val(transaction.message.static_account_keys());
    let static_account_keys_bytes = get_bytes(bytes, offset, num_static_account_keys_bytes);

    sections.push(TransactionByteSection {
        label: "Static Account Keys",
        bytes: static_account_keys_bytes,
        color: Color::Yellow,
    });
}

fn add_recent_blockhash_section(
    _transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let recent_blockhash_bytes = get_bytes(bytes, offset, core::mem::size_of::<Hash>());
    sections.push(TransactionByteSection {
        label: "Recent Blockhash",
        bytes: recent_blockhash_bytes,
        color: Color::Magenta,
    });
}

fn add_instructions_sections(
    transaction: &VersionedTransaction,
    bytes: &[u8],
    sections: &mut Vec<TransactionByteSection>,
    offset: &mut usize,
) {
    let Ok(num_instruction_bytes) =
        bincode::serialized_size(&ShortVec(transaction.message.instructions().to_vec()))
    else {
        return;
    };
    let instruction_bytes = get_bytes(bytes, offset, num_instruction_bytes as usize);

    sections.push(TransactionByteSection {
        label: "Instructions",
        bytes: instruction_bytes,
        color: Color::Cyan,
    });
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
    let Ok(num_address_table_lookups_bytes) =
        bincode::serialized_size(&ShortVec(address_table_lookups.to_vec()))
    else {
        return;
    };
    let address_table_lookups_bytes =
        get_bytes(bytes, offset, num_address_table_lookups_bytes as usize);

    sections.push(TransactionByteSection {
        label: "Message Address Table Lookups",
        bytes: address_table_lookups_bytes,
        color: Color::Red,
    });
}

fn get_bytes(bytes: &[u8], offset: &mut usize, num_bytes: usize) -> Vec<u8> {
    let result = bytes[*offset..*offset + num_bytes].to_vec();
    *offset += num_bytes;
    result
}
