use promkit::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    PromptSignal, Result,
};

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

    Ok(PromptSignal::Continue)
}
