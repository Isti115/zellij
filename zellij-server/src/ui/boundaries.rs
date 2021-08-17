use zellij_utils::pane_size::PositionAndSize;
use zellij_utils::zellij_tile;

use crate::tab::Pane;
use ansi_term::Colour::{Fixed, RGB};
use std::collections::HashMap;
use zellij_tile::data::{InputMode, Palette, PaletteColor};
use zellij_utils::shared::colors;

use std::fmt::{Display, Error, Formatter};
pub mod boundary_type {
    pub const TOP_RIGHT: &str = "┐";
    pub const VERTICAL: &str = "│";
    pub const HORIZONTAL: &str = "─";
    pub const TOP_LEFT: &str = "┌";
    pub const BOTTOM_RIGHT: &str = "┘";
    pub const BOTTOM_LEFT: &str = "└";
    pub const VERTICAL_LEFT: &str = "┤";
    pub const VERTICAL_RIGHT: &str = "├";
    pub const HORIZONTAL_DOWN: &str = "┬";
    pub const HORIZONTAL_UP: &str = "┴";
    pub const CROSS: &str = "┼";
}

pub(crate) type BoundaryType = &'static str; // easy way to refer to boundary_type above

#[derive(Clone, Copy, Debug)]
pub(crate) struct BoundarySymbol {
    boundary_type: BoundaryType,
    invisible: bool,
    color: Option<PaletteColor>,
}

impl BoundarySymbol {
    pub fn new(boundary_type: BoundaryType) -> Self {
        BoundarySymbol {
            boundary_type,
            invisible: false,
            color: Some(PaletteColor::EightBit(colors::GRAY)),
        }
    }
    pub fn invisible(mut self) -> Self {
        self.invisible = true;
        self
    }
    pub fn color(&mut self, color: Option<PaletteColor>) -> Self {
        self.color = color;
        *self
    }
}

impl Display for BoundarySymbol {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.invisible {
            true => write!(f, " "),
            false => match self.color {
                Some(color) => match color {
                    PaletteColor::Rgb((r, g, b)) => {
                        write!(f, "{}", RGB(r, g, b).paint(self.boundary_type))
                    }
                    PaletteColor::EightBit(color) => {
                        write!(f, "{}", Fixed(color).paint(self.boundary_type))
                    }
                },
                None => write!(f, "{}", self.boundary_type),
            },
        }
    }
}

fn combine_symbols(
    current_symbol: BoundarySymbol,
    next_symbol: BoundarySymbol,
) -> Option<BoundarySymbol> {
    use boundary_type::*;
    let invisible = current_symbol.invisible || next_symbol.invisible;
    let color = current_symbol.color.or(next_symbol.color);
    match (current_symbol.boundary_type, next_symbol.boundary_type) {
        (CROSS, _) | (_, CROSS) => {
            // (┼, *) or (*, ┼) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_RIGHT, TOP_RIGHT) => {
            // (┐, ┐) => Some(┐)
            let boundary_type = TOP_RIGHT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_RIGHT, VERTICAL) | (TOP_RIGHT, BOTTOM_RIGHT) | (TOP_RIGHT, VERTICAL_LEFT) => {
            // (┐, │) => Some(┤)
            // (┐, ┘) => Some(┤)
            // (─, ┤) => Some(┤)
            let boundary_type = VERTICAL_LEFT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_RIGHT, HORIZONTAL) | (TOP_RIGHT, TOP_LEFT) | (TOP_RIGHT, HORIZONTAL_DOWN) => {
            // (┐, ─) => Some(┬)
            // (┐, ┌) => Some(┬)
            // (┐, ┬) => Some(┬)
            let boundary_type = HORIZONTAL_DOWN;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_RIGHT, BOTTOM_LEFT) | (TOP_RIGHT, VERTICAL_RIGHT) | (TOP_RIGHT, HORIZONTAL_UP) => {
            // (┐, └) => Some(┼)
            // (┐, ├) => Some(┼)
            // (┐, ┴) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL, HORIZONTAL) => {
            // (─, ─) => Some(─)
            let boundary_type = HORIZONTAL;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL, VERTICAL) | (HORIZONTAL, VERTICAL_LEFT) | (HORIZONTAL, VERTICAL_RIGHT) => {
            // (─, │) => Some(┼)
            // (─, ┤) => Some(┼)
            // (─, ├) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL, TOP_LEFT) | (HORIZONTAL, HORIZONTAL_DOWN) => {
            // (─, ┌) => Some(┬)
            // (─, ┬) => Some(┬)
            let boundary_type = HORIZONTAL_DOWN;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL, BOTTOM_RIGHT) | (HORIZONTAL, BOTTOM_LEFT) | (HORIZONTAL, HORIZONTAL_UP) => {
            // (─, ┘) => Some(┴)
            // (─, └) => Some(┴)
            // (─, ┴) => Some(┴)
            let boundary_type = HORIZONTAL_UP;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL, VERTICAL) => {
            // (│, │) => Some(│)
            let boundary_type = VERTICAL;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL, TOP_LEFT) | (VERTICAL, BOTTOM_LEFT) | (VERTICAL, VERTICAL_RIGHT) => {
            // (│, ┌) => Some(├)
            // (│, └) => Some(├)
            // (│, ├) => Some(├)
            let boundary_type = VERTICAL_RIGHT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL, BOTTOM_RIGHT) | (VERTICAL, VERTICAL_LEFT) => {
            // (│, ┘) => Some(┤)
            // (│, ┤) => Some(┤)
            let boundary_type = VERTICAL_LEFT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL, HORIZONTAL_DOWN) | (VERTICAL, HORIZONTAL_UP) => {
            // (│, ┬) => Some(┼)
            // (│, ┴) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_LEFT, TOP_LEFT) => {
            // (┌, ┌) => Some(┌)
            let boundary_type = TOP_LEFT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_LEFT, BOTTOM_RIGHT) | (TOP_LEFT, VERTICAL_LEFT) | (TOP_LEFT, HORIZONTAL_UP) => {
            // (┌, ┘) => Some(┼)
            // (┌, ┤) => Some(┼)
            // (┌, ┴) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_LEFT, BOTTOM_LEFT) | (TOP_LEFT, VERTICAL_RIGHT) => {
            // (┌, └) => Some(├)
            // (┌, ├) => Some(├)
            let boundary_type = VERTICAL_RIGHT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (TOP_LEFT, HORIZONTAL_DOWN) => {
            // (┌, ┬) => Some(┬)
            let boundary_type = HORIZONTAL_DOWN;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_RIGHT, BOTTOM_RIGHT) => {
            // (┘, ┘) => Some(┘)
            let boundary_type = BOTTOM_RIGHT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_RIGHT, BOTTOM_LEFT) | (BOTTOM_RIGHT, HORIZONTAL_UP) => {
            // (┘, └) => Some(┴)
            // (┘, ┴) => Some(┴)
            let boundary_type = HORIZONTAL_UP;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_RIGHT, VERTICAL_LEFT) => {
            // (┘, ┤) => Some(┤)
            let boundary_type = VERTICAL_LEFT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_RIGHT, VERTICAL_RIGHT) | (BOTTOM_RIGHT, HORIZONTAL_DOWN) => {
            // (┘, ├) => Some(┼)
            // (┘, ┬) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_LEFT, BOTTOM_LEFT) => {
            // (└, └) => Some(└)
            let boundary_type = BOTTOM_LEFT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_LEFT, VERTICAL_LEFT) | (BOTTOM_LEFT, HORIZONTAL_DOWN) => {
            // (└, ┤) => Some(┼)
            // (└, ┬) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_LEFT, VERTICAL_RIGHT) => {
            // (└, ├) => Some(├)
            let boundary_type = VERTICAL_RIGHT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (BOTTOM_LEFT, HORIZONTAL_UP) => {
            // (└, ┴) => Some(┴)
            let boundary_type = HORIZONTAL_UP;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL_LEFT, VERTICAL_LEFT) => {
            // (┤, ┤) => Some(┤)
            let boundary_type = VERTICAL_LEFT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL_LEFT, VERTICAL_RIGHT)
        | (VERTICAL_LEFT, HORIZONTAL_DOWN)
        | (VERTICAL_LEFT, HORIZONTAL_UP) => {
            // (┤, ├) => Some(┼)
            // (┤, ┬) => Some(┼)
            // (┤, ┴) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL_RIGHT, VERTICAL_RIGHT) => {
            // (├, ├) => Some(├)
            let boundary_type = VERTICAL_RIGHT;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (VERTICAL_RIGHT, HORIZONTAL_DOWN) | (VERTICAL_RIGHT, HORIZONTAL_UP) => {
            // (├, ┬) => Some(┼)
            // (├, ┴) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL_DOWN, HORIZONTAL_DOWN) => {
            // (┬, ┬) => Some(┬)
            let boundary_type = HORIZONTAL_DOWN;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL_DOWN, HORIZONTAL_UP) => {
            // (┬, ┴) => Some(┼)
            let boundary_type = CROSS;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (HORIZONTAL_UP, HORIZONTAL_UP) => {
            // (┴, ┴) => Some(┴)
            let boundary_type = HORIZONTAL_UP;
            Some(BoundarySymbol {
                boundary_type,
                invisible,
                color,
            })
        }
        (_, _) => combine_symbols(next_symbol, current_symbol),
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub(crate) struct Coordinates {
    x: usize,
    y: usize,
}

impl Coordinates {
    pub fn new(x: usize, y: usize) -> Self {
        Coordinates { x, y }
    }
}

pub struct Boundaries {
    position_and_size: PositionAndSize,
    boundary_characters: HashMap<Coordinates, BoundarySymbol>,
}

impl Boundaries {
    pub fn new(position_and_size: &PositionAndSize) -> Self {
        Boundaries {
            position_and_size: *position_and_size,
            boundary_characters: HashMap::new(),
        }
    }
    pub fn add_rect(&mut self, rect: &dyn Pane, input_mode: InputMode, palette: Option<Palette>) {
        if !self.is_fully_inside_screen(rect) {
            return;
        }
        let color = match palette.is_some() {
            true => match input_mode {
                InputMode::Normal | InputMode::Locked => Some(palette.unwrap().green),
                _ => Some(palette.unwrap().orange),
            },
            false => None,
        };
        if rect.x() > self.position_and_size.x {
            // left boundary
            let boundary_x_coords = rect.x() - 1;
            let first_row_coordinates = self.rect_right_boundary_row_start(rect);
            let last_row_coordinates = self.rect_right_boundary_row_end(rect);
            for row in first_row_coordinates..last_row_coordinates {
                let coordinates = Coordinates::new(boundary_x_coords, row);
                let mut symbol_to_add =
                    if row == first_row_coordinates && row != self.position_and_size.y {
                        BoundarySymbol::new(boundary_type::TOP_LEFT).color(color)
                    } else if row == last_row_coordinates - 1
                        && row != self.position_and_size.y + self.position_and_size.rows - 1
                    {
                        BoundarySymbol::new(boundary_type::BOTTOM_LEFT).color(color)
                    } else {
                        BoundarySymbol::new(boundary_type::VERTICAL).color(color)
                    };
                if rect.invisible_borders() {
                    symbol_to_add = symbol_to_add.invisible();
                }
                let next_symbol = self
                    .boundary_characters
                    .remove(&coordinates)
                    .and_then(|current_symbol| combine_symbols(current_symbol, symbol_to_add))
                    .unwrap_or(symbol_to_add);
                self.boundary_characters.insert(coordinates, next_symbol);
            }
        }
        if rect.y() > self.position_and_size.y {
            // top boundary
            let boundary_y_coords = rect.y() - 1;
            let first_col_coordinates = self.rect_bottom_boundary_col_start(rect);
            let last_col_coordinates = self.rect_bottom_boundary_col_end(rect);
            for col in first_col_coordinates..last_col_coordinates {
                let coordinates = Coordinates::new(col, boundary_y_coords);
                let mut symbol_to_add = if col == first_col_coordinates
                    && col != self.position_and_size.x
                {
                    BoundarySymbol::new(boundary_type::TOP_LEFT).color(color)
                } else if col == last_col_coordinates - 1 && col != self.position_and_size.cols - 1
                {
                    BoundarySymbol::new(boundary_type::TOP_RIGHT).color(color)
                } else {
                    BoundarySymbol::new(boundary_type::HORIZONTAL).color(color)
                };
                if rect.invisible_borders() {
                    symbol_to_add = symbol_to_add.invisible();
                }
                let next_symbol = self
                    .boundary_characters
                    .remove(&coordinates)
                    .and_then(|current_symbol| combine_symbols(current_symbol, symbol_to_add))
                    .unwrap_or(symbol_to_add);
                self.boundary_characters.insert(coordinates, next_symbol);
            }
        }
        if self.rect_right_boundary_is_before_screen_edge(rect) {
            // right boundary
            let boundary_x_coords = rect.right_boundary_x_coords() - 1;
            let first_row_coordinates = self.rect_right_boundary_row_start(rect);
            let last_row_coordinates = self.rect_right_boundary_row_end(rect);
            for row in first_row_coordinates..last_row_coordinates {
                let coordinates = Coordinates::new(boundary_x_coords, row);
                let mut symbol_to_add =
                    if row == first_row_coordinates && row != self.position_and_size.y {
                        BoundarySymbol::new(boundary_type::TOP_RIGHT).color(color)
                    } else if row == last_row_coordinates - 1
                        && row != self.position_and_size.y + self.position_and_size.rows - 1
                    {
                        BoundarySymbol::new(boundary_type::BOTTOM_RIGHT).color(color)
                    } else {
                        BoundarySymbol::new(boundary_type::VERTICAL).color(color)
                    };
                if rect.invisible_borders() {
                    symbol_to_add = symbol_to_add.invisible();
                }
                let next_symbol = self
                    .boundary_characters
                    .remove(&coordinates)
                    .and_then(|current_symbol| combine_symbols(current_symbol, symbol_to_add))
                    .unwrap_or(symbol_to_add);
                self.boundary_characters.insert(coordinates, next_symbol);
            }
        }
        if self.rect_bottom_boundary_is_before_screen_edge(rect) {
            // bottom boundary
            let boundary_y_coords = rect.bottom_boundary_y_coords() - 1;
            let first_col_coordinates = self.rect_bottom_boundary_col_start(rect);
            let last_col_coordinates = self.rect_bottom_boundary_col_end(rect);
            for col in first_col_coordinates..last_col_coordinates {
                let coordinates = Coordinates::new(col, boundary_y_coords);
                let mut symbol_to_add = if col == first_col_coordinates
                    && col != self.position_and_size.x
                {
                    BoundarySymbol::new(boundary_type::BOTTOM_LEFT).color(color)
                } else if col == last_col_coordinates - 1 && col != self.position_and_size.cols - 1
                {
                    BoundarySymbol::new(boundary_type::BOTTOM_RIGHT).color(color)
                } else {
                    BoundarySymbol::new(boundary_type::HORIZONTAL).color(color)
                };
                if rect.invisible_borders() {
                    symbol_to_add = symbol_to_add.invisible();
                }
                let next_symbol = self
                    .boundary_characters
                    .remove(&coordinates)
                    .and_then(|current_symbol| combine_symbols(current_symbol, symbol_to_add))
                    .unwrap_or(symbol_to_add);
                self.boundary_characters.insert(coordinates, next_symbol);
            }
        }
    }
    pub fn vte_output(&self) -> String {
        let mut vte_output = String::new();
        for (coordinates, boundary_character) in &self.boundary_characters {
            vte_output.push_str(&format!(
                "\u{1b}[{};{}H\u{1b}[m{}",
                coordinates.y + 1,
                coordinates.x + 1,
                boundary_character
            )); // goto row/col + boundary character
        }
        vte_output
    }
    fn rect_right_boundary_is_before_screen_edge(&self, rect: &dyn Pane) -> bool {
        rect.x() + rect.cols() < self.position_and_size.cols
    }
    fn rect_bottom_boundary_is_before_screen_edge(&self, rect: &dyn Pane) -> bool {
        rect.y() + rect.rows() < self.position_and_size.y + self.position_and_size.rows
    }
    fn rect_right_boundary_row_start(&self, rect: &dyn Pane) -> usize {
        if rect.y() > self.position_and_size.y {
            rect.y() - 1
        } else {
            self.position_and_size.y
        }
    }
    fn rect_right_boundary_row_end(&self, rect: &dyn Pane) -> usize {
        rect.y() + rect.rows()
    }
    fn rect_bottom_boundary_col_start(&self, rect: &dyn Pane) -> usize {
        if rect.x() == 0 {
            0
        } else {
            rect.x() - 1
        }
    }
    fn rect_bottom_boundary_col_end(&self, rect: &dyn Pane) -> usize {
        rect.x() + rect.cols()
    }
    fn is_fully_inside_screen(&self, rect: &dyn Pane) -> bool {
        rect.x() >= self.position_and_size.x
            && rect.x() + rect.cols() <= self.position_and_size.x + self.position_and_size.cols
            && rect.y() >= self.position_and_size.y
            && rect.y() + rect.rows() <= self.position_and_size.y + self.position_and_size.rows
    }
}
