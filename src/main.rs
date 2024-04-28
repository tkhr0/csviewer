use std::collections::HashSet;
use std::env;

use anyhow::Result;
use log4rs;
use promkit::crossterm::event::Event;
use promkit::crossterm::style::ContentStyle;
use promkit::impl_as_any;
use promkit::keymap::KeymapManager;
use promkit::text_editor;
use promkit::{Prompt, PromptSignal, Renderer};

mod command;
mod csv_renderer;
mod keymap;
use csv_renderer::CsvRenderer;

struct Viewer {
    query_editor_renderer: text_editor::Renderer,
    csv_renderer: CsvRenderer,
    keymap: KeymapManager<Self>,
}

impl Renderer for Viewer {
    fn create_panes(&self, width: u16) -> Vec<promkit::pane::Pane> {
        let mut panes = Vec::new();
        panes.append(&mut self.query_editor_renderer.create_panes(width));
        panes.append(&mut self.csv_renderer.create_panes(width));
        panes
    }
}

impl_as_any!(Viewer);

fn main() -> Result<()> {
    match log4rs::init_file("log4rs.yml", Default::default()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to initialize log4rs: {}", e);
            panic!();
        }
    };

    let file = std::fs::File::open(&(env::args().collect::<Vec<String>>())[1])?;

    let viewer = Viewer {
        query_editor_renderer: text_editor::Renderer {
            texteditor: text_editor::TextEditor::default(),
            history: None,
            prefix: "".to_string(),
            mask: None,
            prefix_style: ContentStyle::new(),
            active_char_style: ContentStyle::new(),
            inactive_char_style: ContentStyle::new(),
            edit_mode: text_editor::Mode::Insert,
            word_break_chars: HashSet::new(),
            lines: Some(1),
        },
        csv_renderer: CsvRenderer::new(file),
        keymap: KeymapManager::new("default", keymap::default),
    };

    let mut prompt = Prompt::try_new(
        Box::new(viewer),
        Box::new(
            move |event: &Event,
                  renderer: &mut Box<dyn Renderer + 'static>|
                  -> promkit::Result<PromptSignal> {
                let viewer = renderer.as_any_mut().downcast_mut::<Viewer>().unwrap();

                let signal = match viewer.keymap.get() {
                    Some(f) => f(event, viewer),
                    None => Ok(PromptSignal::Quit),
                }?;

                return Ok(signal);
            },
        ),
        |renderer: &(dyn Renderer + '_)| -> promkit::Result<String> {
            let viewer = renderer.as_any().downcast_ref::<Viewer>().unwrap();
            Ok(viewer
                .query_editor_renderer
                .texteditor
                .text_without_cursor()
                .to_string())
        },
    )?;
    let _ = prompt.run()?;

    Ok(())
}
