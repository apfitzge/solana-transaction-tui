use std::{io, str::FromStr};

use byte_block::ByteBlock;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Padding, Paragraph},
    Frame,
};
use solana_sdk::signature::{self, Signature};
use tui_input::{backend::crossterm::EventHandler, Input};

mod byte_block;
mod tui;

fn main() -> io::Result<()> {
    let mut terminal = tui::init()?;
    let app_result = TransactionApp {
        exit: false,
        input: Input::new("<Signature>".to_string()),
        signature: None,
    }
    .run(&mut terminal);
    tui::restore()?;
    app_result
}

pub struct TransactionApp {
    exit: bool,
    input: Input,
    signature: Option<Signature>,
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
                Constraint::Length(5),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(Text::styled(
            format!(
                "Transaction Layout App{}",
                match self.signature {
                    Some(signature) => format!(" - {}", signature),
                    None => "".to_string(),
                }
            ),
            Style::default().fg(Color::Green),
        ))
        .block(title_block);
        frame.render_widget(title, chunks[0]);

        let width = chunks[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = self.input.visual_scroll(width as usize);
        let input = Paragraph::new(self.input.value())
            .style(Style::default().fg(Color::Yellow))
            .scroll((0, scroll as u16))
            .block(Block::default().borders(Borders::ALL).title("Input"));
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
            .style(Style::default());

        let owned_byte_sections = vec![
            (0..1u8).into_iter().collect::<Vec<_>>(),
            (1..65u8).into_iter().collect::<Vec<_>>(),
        ];
        let byte_sections = &[
            owned_byte_sections[0].as_ref(),
            owned_byte_sections[1].as_ref(),
        ][..];
        let byte_section_colors = vec![Color::LightGreen, Color::Green];

        let byte_block = ByteBlock::new(byte_sections, &byte_section_colors).block(bytes_block);
        frame.render_widget(&byte_block, middle_block_chunks[0]);

        let footer_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let footer = Paragraph::new(Text::styled(
            "Press <Esc> to exit",
            Style::default().fg(Color::Red),
        ))
        .block(footer_block);
        frame.render_widget(footer, chunks[2]);
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
        let text = self.input.value();
        self.signature = Signature::from_str(&text).ok();
        self.input.reset();

        if self.signature.is_some() {
            // Create a client and fetch the transaction! Hell yeah.
        }
    }
}
