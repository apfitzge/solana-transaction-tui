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
    std::collections::HashSet,
};

pub struct ByteSectionLegend<'a> {
    sections: &'a [TransactionByteSection],
    block: Option<Block<'a>>,
}

impl<'a> ByteSectionLegend<'a> {
    pub fn new(transaction_byte_sections: &'a [TransactionByteSection]) -> Self {
        Self {
            sections: transaction_byte_sections,
            block: None,
        }
    }

    /// Surrounds the `ByteSectionLegend` with a [`Block`].
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
        let mut unique_lines = self
            .sections
            .iter()
            .filter(|section| section.label.is_some())
            .map(|section: &TransactionByteSection| &section.label)
            .collect::<HashSet<_>>();
        let num_unique_lines = unique_lines.len();
        let legend_lines = self
            .sections
            .iter()
            .filter(|section| unique_lines.remove(&section.label))
            .map(|section| {
                Text::styled(
                    section.label.as_ref().unwrap(),
                    Style::default().bg(section.color),
                )
            });
        let legend_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints((0..num_unique_lines).map(|_| Constraint::Length(1)))
            .split(area);

        for (line, layout) in legend_lines.zip(legend_layout.iter()) {
            line.render(*layout, buf);
        }
    }
}

impl<'a> Widget for &ByteSectionLegend<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);
        let inner = self.block.inner_if_some(area);
        self.render_inner(inner, buf);
    }
}
