use {
    crate::transaction_byte_sections::TransactionByteSection,
    ratatui::{
        buffer::Buffer,
        layout::{Constraint, Direction, Layout, Rect},
        prelude::BlockExt,
        style::Style,
        text::Text,
        widgets::{Block, Widget},
    },
};

pub struct TransactionByteBlock<'a> {
    sections: &'a [TransactionByteSection],
    block: Option<Block<'a>>,
}

impl<'a> TransactionByteBlock<'a> {
    pub fn new(transaction_byte_sections: &'a [TransactionByteSection]) -> Self {
        Self {
            sections: transaction_byte_sections,
            block: None,
        }
    }

    /// Surrounds the `ByteBlock` with a [`Block`].
    ///
    /// The byte block is rendered in the inner portion of the block once space
    /// for borders and padding is reserved. Styles set on the block do **not**
    /// affect the byte block itself.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        let len_bytes = self.sections.iter().map(|s| s.bytes.len()).sum::<usize>();
        if len_bytes == 0 {
            return;
        }

        // Go byte by byte and render them.
        // Make sure to only render as many bytes as can fit in the area.
        let bytes_per_line = (area.width / 3) as usize;
        let num_lines = len_bytes / bytes_per_line
            + if len_bytes % bytes_per_line == 0 {
                0
            } else {
                1
            };

        // Split the current area into lines.
        let lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints((0..num_lines).map(|_| Constraint::Length(1)))
            .split(area);
        let line_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints((0..bytes_per_line).map(|_| Constraint::Length(3)));

        let mut byte_index = 0;
        let mut line_index = 0;

        let mut current_line_layout = line_layout.split(lines[line_index]);
        for section in self.sections.iter() {
            for byte in section.bytes.iter() {
                let byte_text =
                    Text::styled(format!("{:02x} ", byte), Style::default().bg(section.color));
                byte_text.render(current_line_layout[byte_index % bytes_per_line], buf);

                // Update the byte index and line index
                byte_index += 1;
                if byte_index % bytes_per_line == 0 {
                    line_index += 1;
                    current_line_layout = line_layout.split(lines[line_index]);
                }
            }
        }
    }
}

impl<'a> Widget for &TransactionByteBlock<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);
        let inner = self.block.inner_if_some(area);
        self.render_inner(inner, buf);
    }
}
