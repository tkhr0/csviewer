use std::collections::HashSet;

use anyhow::Result;
use promkit::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use promkit::crossterm::style::ContentStyle;
use promkit::text_editor;
use promkit::{Prompt, PromptSignal, Renderer};

fn main() -> Result<()> {
    let renderer = text_editor::Renderer {
        texteditor: text_editor::TextEditor::default(),
        history: None,
        prefix: "prefix".to_string(),
        mask: None,
        prefix_style: ContentStyle::new(),
        active_char_style: ContentStyle::new(),
        inactive_char_style: ContentStyle::new(),
        edit_mode: text_editor::Mode::Insert,
        word_break_chars: HashSet::new(),
        lines: Some(10),
    };

    let mut prompt = Prompt::try_new(
        Box::new(renderer),
        Box::new(
            move |event: &Event,
                  renderer: &mut Box<dyn Renderer + 'static>|
                  -> promkit::Result<PromptSignal> {
                match event {
                    Event::Key(key_event) => {
                        if key_event == &KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL) {
                            return Ok(PromptSignal::Quit);
                        } else {
                            renderer
                                .as_any_mut()
                                .downcast_mut::<text_editor::Renderer>()
                                .unwrap()
                                .texteditor
                                .insert('a');
                            return Ok(PromptSignal::Continue);
                        }
                    }
                    _ => {
                        return Ok(PromptSignal::Continue);
                    }
                };
            },
        ),
        |renderer: &(dyn Renderer + '_)| -> promkit::Result<String> {
            let texteditor_renderer = renderer
                .as_any()
                .downcast_ref::<text_editor::Renderer>()
                .unwrap();
            Ok(texteditor_renderer
                .texteditor
                .text_without_cursor()
                .to_string())
        },
    )?;
    let _ = prompt.run()?;

    Ok(())
}
