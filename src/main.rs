use std::cmp;
use std::collections::HashSet;
use std::env;
use std::fs;

use anyhow::Result;
use csv;
use log::info;
use log4rs;
use promkit::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use promkit::crossterm::style::ContentStyle;
use promkit::grapheme::{trim, StyledGraphemes};
use promkit::impl_as_any;
use promkit::text_editor;
use promkit::{Prompt, PromptSignal, Renderer};

#[derive(Debug)]
struct Cell {
    value: String,
    width: usize,
    max_width: Option<usize>, // other cell's max width
}

impl Cell {
    fn new(value: &str) -> Self {
        let width = unicode_width::UnicodeWidthStr::width(value);
        Self {
            value: value.to_string(),
            width,
            max_width: None,
        }
    }

    fn width(&self) -> usize {
        self.width
    }
}

struct CsvRenderer {
    headers: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
}

impl CsvRenderer {
    fn new(file: fs::File) -> Self {
        let mut reader = csv::Reader::from_reader(file);

        let mut headers = vec![];
        let mut header_max_width = 0;

        for header in reader.headers().unwrap() {
            let cell = Cell::new(header);
            header_max_width = cmp::max(header_max_width, cell.width());
            headers.push(cell);
        }
        headers
            .iter_mut()
            .for_each(|cell| cell.max_width = Some(header_max_width));
        info!("{:?}", headers);

        let mut rows = vec![];

        for row in reader.records() {
            let row = row.unwrap();
            let mut cells = vec![];
            let mut max_width = 0;

            for (i, cell) in row.iter().enumerate() {
                let cell = Cell::new(cell);
                if let Some(header) = headers.get(i) {
                    max_width = cmp::max(max_width, header.width());
                }
                cells.push(cell);
            }
            cells
                .iter_mut()
                .for_each(|cell| cell.max_width = Some(max_width));
            rows.push(cells);
        }
        info!("{:?}", rows);

        Self { headers, rows }
    }
}

impl promkit::AsAny for CsvRenderer {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Renderer for CsvRenderer {
    fn create_panes(&self, width: u16) -> Vec<promkit::pane::Pane> {
        let mut body: Vec<StyledGraphemes> = vec![];

        body.push(trim(
            width as usize,
            &StyledGraphemes::from_str(
                self.headers
                    .iter()
                    .map(|cell| {
                        let width = cell.max_width.unwrap_or(0);
                        let padding = cmp::max(width, cell.width()) - cell.width();
                        format!("{}{}", cell.value, " ".repeat(padding))
                    })
                    .collect::<Vec<String>>()
                    .join(" | "),
                ContentStyle::new(),
            ),
        ));

        body.append(
            &mut self
                .rows
                .iter()
                .map(|row| {
                    trim(
                        width as usize,
                        &StyledGraphemes::from_str(
                            row.iter()
                                .map(|cell| {
                                    let width = cell.max_width.unwrap_or(0);
                                    let padding = cmp::max(width, cell.width()) - cell.width();
                                    format!("{}{}", cell.value, " ".repeat(padding))
                                })
                                .collect::<Vec<String>>()
                                .join(" | "),
                            ContentStyle::new(),
                        ),
                    )
                })
                .collect::<Vec<StyledGraphemes>>(),
        );

        vec![promkit::pane::Pane::new(body, 0, None)]
    }
}

struct Viewer {
    query_editor_renderer: text_editor::Renderer,
    csv_renderer: CsvRenderer,
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
    };

    let mut prompt = Prompt::try_new(
        Box::new(viewer),
        Box::new(
            move |event: &Event,
                  renderer: &mut Box<dyn Renderer + 'static>|
                  -> promkit::Result<PromptSignal> {
                match event {
                    Event::Key(key_event) => {
                        if key_event == &KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL) {
                            return Ok(PromptSignal::Quit);
                        } else {
                            let viewer = renderer.as_any_mut().downcast_mut::<Viewer>().unwrap();
                            viewer.query_editor_renderer.texteditor.insert('a');
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
