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
        let legend_lines = self
            .sections
            .iter()
            .map(|section| Text::styled(section.label, Style::default().bg(section.color)));
        let legend_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints((0..self.sections.len()).map(|_| Constraint::Length(1)))
            .split(area);

        for (line, layout) in legend_lines.zip(legend_layout.into_iter()) {
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
