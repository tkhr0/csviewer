use log::info;
use promkit::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    PromptSignal, Result,
};

use crate::command;

pub fn default(event: &Event, renderer: &mut crate::Viewer) -> Result<PromptSignal> {
    match event {
        // Exit
        Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            return Ok(PromptSignal::Quit);
        }

        // Erase char
        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => renderer.query_editor_renderer.texteditor.erase(),

        // Input char
        Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }) => {
            renderer.query_editor_renderer.texteditor.insert(*ch);
        }
        _ => {}
    }

    match command::parse(
        &renderer
            .query_editor_renderer
            .texteditor
            .text()
            .chars()
            .into_iter()
            .collect::<String>(),
    ) {
        Ok(exprs) => {
            info!("{:?}", &exprs);
            renderer.csv_renderer.set_exprs(exprs);
        }
        Err(e) => {
            info!("{}", e);
        }
    };

    Ok(PromptSignal::Continue)
}
