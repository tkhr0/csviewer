use std::cmp;
use std::fs;

use csv;
use log::info;
use promkit::crossterm::style::ContentStyle;
use promkit::grapheme::{trim, StyledGraphemes};
use promkit::Renderer;

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

pub struct CsvRenderer {
    headers: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
}

impl CsvRenderer {
    pub fn new(file: fs::File) -> Self {
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
