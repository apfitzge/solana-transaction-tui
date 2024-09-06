use {
    ratatui::style::Color,
    solana_sdk::{
        hash::Hash,
        pubkey::Pubkey,
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

    // Signatures
    {
        let num_signatures = transaction.signatures.len();
        let num_signature_bytes = 1 + num_signatures * core::mem::size_of::<Signature>();
        let signature_bytes = bytes[offset..offset + num_signature_bytes].to_vec();
        offset += num_signature_bytes;

        sections.push(TransactionByteSection {
            label: "Signatures",
            bytes: signature_bytes,
            color: Color::LightGreen,
        });
    }

    // Message header
    {
        let header_length = 3 + match transaction.version() {
            TransactionVersion::Legacy(_) => 0,
            TransactionVersion::Number(_) => 1,
        };
        let header_bytes = bytes[offset..offset + header_length].to_vec();
        offset += header_length;

        sections.push(TransactionByteSection {
            label: "Message Header",
            bytes: header_bytes,
            color: Color::Blue,
        });
    }

    // Static Account Keys
    {
        let num_static_account_keys = transaction.message.static_account_keys().len();
        let num_static_account_keys_bytes =
            1 + num_static_account_keys * core::mem::size_of::<Pubkey>();
        let static_account_keys_bytes =
            bytes[offset..offset + num_static_account_keys_bytes].to_vec();
        offset += num_static_account_keys_bytes;

        sections.push(TransactionByteSection {
            label: "Static Account Keys",
            bytes: static_account_keys_bytes,
            color: Color::Yellow,
        });
    }

    // Recent Blockhash
    {
        let num_recent_blockhash_bytes = core::mem::size_of::<Hash>();
        let recent_blockhash_bytes = bytes[offset..offset + num_recent_blockhash_bytes].to_vec();
        offset += num_recent_blockhash_bytes;

        sections.push(TransactionByteSection {
            label: "Recent Blockhash",
            bytes: recent_blockhash_bytes,
            color: Color::Magenta,
        });
    }

    // Instructions
    {
        let Ok(num_instruction_bytes) =
            bincode::serialized_size(&ShortVec(transaction.message.instructions().to_vec()))
        else {
            return;
        };
        let instruction_bytes = bytes[offset..offset + num_instruction_bytes as usize].to_vec();
        offset += num_instruction_bytes as usize;

        sections.push(TransactionByteSection {
            label: "Instructions",
            bytes: instruction_bytes,
            color: Color::Cyan,
        });
    }

    // Message Address Table Lookups
    {
        let Some(address_table_lookups) = transaction.message.address_table_lookups() else {
            return;
        };
        let Ok(num_address_table_lookups_bytes) =
            bincode::serialized_size(&ShortVec(address_table_lookups.to_vec()))
        else {
            return;
        };
        let address_table_lookups_bytes =
            bytes[offset..offset + num_address_table_lookups_bytes as usize].to_vec();

        // Still want to update offset for consistency
        #[allow(unused_assignments)]
        {
            offset += num_address_table_lookups_bytes as usize;
        }

        sections.push(TransactionByteSection {
            label: "Message Address Table Lookups",
            bytes: address_table_lookups_bytes,
            color: Color::Red,
        });
    }
}
