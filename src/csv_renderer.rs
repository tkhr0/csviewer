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
    index: usize,
}

impl Cell {
    fn new(value: &str, index: usize) -> Self {
        let width = unicode_width::UnicodeWidthStr::width(value);
        Self {
            value: value.to_string(),
            width,
            max_width: None,
            index,
        }
    }

    fn width(&self) -> usize {
        self.width
    }
}

pub struct CsvRenderer {
    headers: Vec<Cell>,
    rows: Vec<Vec<Cell>>,
    selected_headers: Option<Vec<usize>>,
}

impl CsvRenderer {
    pub fn new(file: fs::File) -> Self {
        let mut reader = csv::Reader::from_reader(file);

        let mut headers = vec![];
        let mut header_max_width = 0;

        for (i, header) in reader.headers().unwrap().iter().enumerate() {
            let cell = Cell::new(header, i);
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
                let cell = Cell::new(cell, i);
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

        Self {
            headers,
            rows,
            selected_headers: Some(vec![0, 3, 5, 8]),
        }
    }

    pub fn select_headers(&mut self, headers: Vec<&str>) {
        let mut selected_headers = vec![];

        for (i, header) in self.headers.iter().enumerate() {
            if headers.contains(&header.value.as_str()) {
                selected_headers.push(i);
            }
        }

        self.selected_headers = Some(selected_headers);
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
                    .enumerate()
                    .map(|(i, cell)| {
                        if let Some(select_headers) = &self.selected_headers {
                            if !select_headers.contains(&i) {
                                return None;
                            }
                        }

                        let width = cell.max_width.unwrap_or(0);
                        let padding = cmp::max(width, cell.width()) - cell.width();
                        Some(format!("{}{}", cell.value, " ".repeat(padding)))
                    })
                    .filter(|cell| cell.is_some())
                    .map(|cell| cell.unwrap())
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
                                .enumerate()
                                .map(|(i, cell)| {
                                    if let Some(select_headers) = &self.selected_headers {
                                        if !select_headers.contains(&i) {
                                            return None;
                                        }
                                    }

                                    let width = cell.max_width.unwrap_or(0);
                                    let padding = cmp::max(width, cell.width()) - cell.width();
                                    Some(format!("{}{}", cell.value, " ".repeat(padding)))
                                })
                                .filter(|cell| cell.is_some())
                                .map(|cell| cell.unwrap())
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
