use {
    ratatui::{
        crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        layout::{Constraint, Direction, Layout},
        style::{Color, Style},
        text::Text,
        widgets::{Block, Borders, Padding, Paragraph},
        Frame,
    },
    solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig},
    solana_sdk::{commitment_config::CommitmentConfig, signature::Signature},
    solana_transaction_status::UiTransactionEncoding,
    std::{io, str::FromStr},
    transaction_byte_block::TransactionByteBlock,
    transaction_byte_sections::{get_transaction_byte_sections, TransactionByteSection},
    tui_input::{backend::crossterm::EventHandler, Input},
};

mod transaction_byte_block;
mod transaction_byte_sections;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = TransactionApp {
        exit: false,
        input: Input::new("".to_string()),
        signature: None,
        transaction_byte_sections: vec![],
    }
    .run(&mut terminal);
    tui::restore()?;
    app_result
}

pub struct TransactionApp {
    exit: bool,
    input: Input,
    signature: Option<Signature>,
    transaction_byte_sections: Vec<TransactionByteSection>,
}

impl TransactionApp {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        // Split the frame into sections.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(Text::styled(
            format!("Transaction Layout App"),
            Style::default().fg(Color::Green),
        ))
        .block(title_block);
        frame.render_widget(title, chunks[0]);

        let width = chunks[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(Style::default().fg(Color::Yellow))
            .scroll((0, scroll as u16))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Input Signature"),
            );
        frame.render_widget(input, chunks[1]);
        // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
        frame.set_cursor_position((
            // Put cursor past the end of the input text
            chunks[1].x + ((self.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[1].y + 1,
        ));

        let middle_block_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(100), Constraint::Fill(1)])
            .split(chunks[2]);
        let bytes_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::uniform(1))
            .style(Style::default())
            .title(match self.signature {
                Some(signature) => format!(" - {}", signature),
                None => "".to_string(),
            });

        let byte_block =
            TransactionByteBlock::new(&self.transaction_byte_sections).block(bytes_block);
        frame.render_widget(&byte_block, middle_block_chunks[0]);

        // For each color, label add a colored text box
        let legend_lines = self
            .transaction_byte_sections
            .iter()
            .map(|section| Text::styled(section.label, Style::default().bg(section.color)))
            .collect::<Vec<_>>();

        let legend_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::uniform(1))
            .title("Legend")
            .style(Style::default());

        // Render the legend lines in vertical layout with equal constraints
        let legend_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                (0..self.transaction_byte_sections.len())
                    .map(|_| Constraint::Length(3))
                    .collect::<Vec<_>>(),
            )
            .split(legend_block.inner(middle_block_chunks[1]));
        frame.render_widget(legend_block, middle_block_chunks[1]);
        for (gauge, layout) in legend_lines.iter().zip(legend_layout.iter()) {
            frame.render_widget(gauge, *layout);
        }

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let footer = Paragraph::new(Text::styled(
            "Press <Esc> to exit",
            Style::default().fg(Color::Red),
        ))
        .block(footer_block);
        frame.render_widget(footer, chunks[3]);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Enter => self.on_signature_entry(),
            _ => {
                self.input.handle_event(&Event::Key(key_event));
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn on_signature_entry(&mut self) {
        self.signature = None;
        self.transaction_byte_sections.clear();

        let text = self.input.value();
        self.signature = Signature::from_str(&text).ok();

        if let Some(signature) = self.signature.as_ref() {
            // Create client and get transaction details
            let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Binary),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };
            let Ok(transaction) = client.get_transaction_with_config(&signature, config) else {
                return;
            };

            if let Some(transaction) = transaction.transaction.transaction.decode() {
                get_transaction_byte_sections(&transaction, &mut self.transaction_byte_sections);
            }
        }
    }
}
