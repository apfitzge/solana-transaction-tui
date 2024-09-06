use {
    byte_section_legend::ByteSectionLegend,
    menu::AppMenuItem,
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
    tui_menu::{Menu, MenuEvent, MenuState},
};

mod byte_section_legend;
mod menu;
mod transaction_byte_block;
mod transaction_byte_sections;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = TransactionApp {
        exit: false,
        mode: Mode::SignatureEntry,

        input: Input::new("".to_string()),
        signature: SignatureStatus::None,
        transaction_byte_sections: vec![],

        menu: menu::menu(),
        endpoint: "https://api.mainnet-beta.solana.com",
    }
    .run(&mut terminal);
    tui::restore()?;
    app_result
}

enum Mode {
    SignatureEntry,
    MenuSelection,
}

enum SignatureStatus {
    None,
    Invalid,
    NotFound(Signature),
    Found(Signature),
}

pub struct TransactionApp {
    exit: bool,
    mode: Mode,

    // Useful for Mode::SignatureEntry
    input: Input,
    signature: SignatureStatus,
    transaction_byte_sections: Vec<TransactionByteSection>,

    // Useful for Mode::MenuSelection
    menu: MenuState<AppMenuItem>,
    endpoint: &'static str,
}

impl TransactionApp {
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
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
                SignatureStatus::None => "".to_string(),
                SignatureStatus::Invalid => "Invalid Signature".to_string(),
                SignatureStatus::NotFound(signature) => format!("{signature} not found"),
                SignatureStatus::Found(signature) => format!("{}", signature),
            });

        let byte_block =
            TransactionByteBlock::new(&self.transaction_byte_sections).block(bytes_block);
        frame.render_widget(&byte_block, middle_block_chunks[0]);

        let legend_block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::uniform(1))
            .title("Legend")
            .style(Style::default());
        let byte_section_legend =
            ByteSectionLegend::new(&self.transaction_byte_sections).block(legend_block);
        frame.render_widget(&byte_section_legend, middle_block_chunks[1]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let footer = Paragraph::new(Text::styled(
            "Press <Esc> to exit",
            Style::default().fg(Color::Red),
        ))
        .block(footer_block);
        frame.render_widget(footer, chunks[3]);

        // Render the menu
        frame.render_stateful_widget(Menu::new(), chunks[0], &mut self.menu)
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
        match self.mode {
            Mode::SignatureEntry => self.signature_entry_handle_key_event(key_event),
            Mode::MenuSelection => self.menu_selection_handle_key_event(key_event),
        }
    }

    fn signature_entry_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Enter => self.on_signature_entry(),
            KeyCode::Up => {
                self.mode = Mode::MenuSelection;
                self.menu.activate()
            }
            _ => {
                self.input.handle_event(&Event::Key(key_event));
            }
        }
    }

    fn menu_selection_handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.mode = Mode::SignatureEntry;
                self.menu.reset();
            }
            KeyCode::Enter => self.menu.select(),
            KeyCode::Up => self.menu.up(),
            KeyCode::Down => self.menu.down(),
            KeyCode::Left => self.menu.left(),
            KeyCode::Right => self.menu.right(),
            _ => {}
        }

        for event in self.menu.drain_events() {
            match event {
                MenuEvent::Selected(item) => match item {
                    AppMenuItem::Endpoint(endpoint) => {
                        self.endpoint = endpoint;
                        self.mode = Mode::SignatureEntry;
                        self.menu.reset();
                    }
                },
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn on_signature_entry(&mut self) {
        self.signature = SignatureStatus::None;
        self.transaction_byte_sections.clear();

        let text = self.input.value();
        let maybe_signature = Signature::from_str(&text);
        self.input.reset(); // Clear the input field

        let Ok(signature) = maybe_signature else {
            self.signature = SignatureStatus::Invalid;
            return;
        };

        // Create client and get transaction details
        let client = RpcClient::new(self.endpoint);
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Binary),
            commitment: Some(CommitmentConfig::confirmed()),
            max_supported_transaction_version: Some(0),
        };
        let Ok(transaction) = client.get_transaction_with_config(&signature, config) else {
            self.signature = SignatureStatus::NotFound(signature);
            return;
        };

        self.signature = SignatureStatus::Found(signature);
        if let Some(transaction) = transaction.transaction.transaction.decode() {
            get_transaction_byte_sections(&transaction, &mut self.transaction_byte_sections);
        }
    }
}
