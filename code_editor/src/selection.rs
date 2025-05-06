use {
    crate::{
        layout::Layout,
        str::StrExt,
        text::{Edit, Length, Position},
    },
    std::{ops, ops::Deref, slice::Iter},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Hash, Eq)]
pub struct Selection {
    pub cursor: Cursor,
    pub anchor: Position,
}

impl Selection {
    pub fn is_empty(self) -> bool {
        self.anchor == self.cursor.position
    }

    pub fn overlaps_with(self, other: Self) -> bool {
        if self.is_empty() || other.is_empty() {
            self.end() >= other.start()
        } else {
            self.end() > other.start()
        }
    }

    pub fn start(self) -> Position {
        self.cursor.position.min(self.anchor)
    }

    pub fn start_affinity(self) -> Affinity {
        if self.anchor < self.cursor.position {
            Affinity::After
        } else {
            self.cursor.affinity
        }
    }

    pub fn end(self) -> Position {
        self.cursor.position.max(self.anchor)
    }

    pub fn end_affinity(self) -> Affinity {
        if self.cursor.position < self.anchor {
            Affinity::Before
        } else {
            self.cursor.affinity
        }
    }

    pub fn length(self) -> Length {
        self.end() - self.start()
    }

    pub fn line_range(self) -> ops::Range<usize> {
        if self.anchor <= self.cursor.position {
            self.anchor.line_index..self.cursor.position.line_index + 1
        } else {
            self.cursor.position.line_index..if self.anchor.byte_index == 0 {
                self.anchor.line_index
            } else {
                self.anchor.line_index + 1
            }
        }
    }

    pub fn update_cursor(self, f: impl FnOnce(Cursor) -> Cursor) -> Self {
        Self {
            cursor: f(self.cursor),
            ..self
        }
    }

    pub fn reset_anchor(self) -> Self {
        Self {
            anchor: self.cursor.position,
            ..self
        }
    }

    pub fn merge_with(self, other: Self) -> Option<Self> {
        if self.overlaps_with(other) {
            Some(if self.anchor <= self.cursor.position {
                Selection {
                    anchor: self.anchor,
                    cursor: other.cursor,
                }
            } else {
                Selection {
                    anchor: other.anchor,
                    cursor: self.cursor,
                }
            })
        } else {
            None
        }
    }

    pub fn apply_edit(self, edit: &Edit) -> Self {
        Self {
            anchor: self.anchor.apply_edit(edit),
            cursor: self.cursor.apply_edit(edit),
            ..self
        }
    }
}

impl From<Cursor> for Selection {
    fn from(cursor: Cursor) -> Self {
        Self {
            cursor,
            anchor: cursor.position,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn update_selection(
        &mut self,
        index: usize,
        f: impl FnOnce(Selection) -> Selection,
    ) -> usize {
        self.selections[index] = f(self.selections[index]);
        self.normalize_selection(index)
    }

    pub fn update_all_selections(
        &mut self,
        retained_index: Option<usize>,
        mut f: impl FnMut(Selection) -> Selection,
    ) -> Option<usize> {
        for selection in &mut self.selections {
            *selection = f(*selection);
        }
        self.normalize_all_selections(retained_index)
    }

    pub fn apply_edit(&mut self, edit: &Edit, retained_index: Option<usize>) -> Option<usize> {
        self.update_all_selections(retained_index, |selection| selection.apply_edit(edit))
    }

    pub fn add_selection(&mut self, selection: Selection) -> usize {
        let index = match self
            .selections
            .binary_search_by_key(&selection.start(), |selection| selection.start())
        {
            Ok(index) => {
                self.selections[index] = selection;
                index
            }
            Err(index) => {
                self.selections.insert(index, selection);
                index
            }
        };
        self.normalize_selection(index)
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.selections.clear();
        self.selections.push(selection);
    }

    fn normalize_selection(&mut self, index: usize) -> usize {
        let mut index = index;
        while index > 0 {
            let prev_index = index - 1;
            if !self.selections[prev_index].overlaps_with(self.selections[index]) {
                break;
            }
            self.selections.remove(prev_index);
            index -= 1;
        }
        while index + 1 < self.selections.len() {
            let next_index = index + 1;
            if !self.selections[index].overlaps_with(self.selections[next_index]) {
                break;
            }
            self.selections.remove(next_index);
        }
        index
    }

    fn normalize_all_selections(&mut self, retained_index: Option<usize>) -> Option<usize> {
        let mut retained_index = retained_index;
        let mut current_index = 0;
        while current_index + 1 < self.selections.len() {
            let next_index = current_index + 1;
            let current_selection = self.selections[current_index];
            let next_selection = self.selections[next_index];
            assert!(current_selection.start() <= next_selection.start());
            if let Some(merged_selection) = current_selection.merge_with(next_selection) {
                self.selections[current_index] = merged_selection;
                self.selections.remove(next_index);
                if let Some(retained_index) = &mut retained_index {
                    if next_index <= *retained_index {
                        *retained_index -= 1;
                    }
                }
            } else {
                current_index += 1;
            }
        }
        retained_index
    }
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self {
            selections: vec![Selection::default()],
        }
    }
}

impl Deref for SelectionSet {
    type Target = [Selection];

    fn deref(&self) -> &Self::Target {
        &self.selections
    }
}

impl<'a> IntoIterator for &'a SelectionSet {
    type Item = &'a Selection;
    type IntoIter = Iter<'a, Selection>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Cursor {
    pub position: Position,
    pub affinity: Affinity,
    pub preferred_column_index: Option<usize>,
}

impl Cursor {
    pub fn is_at_first_line(self) -> bool {
        self.position.line_index == 0
    }

    pub fn is_at_last_line(self, line_count: usize) -> bool {
        self.position.line_index == line_count - 1
    }

    pub fn is_at_start_of_line(self) -> bool {
        self.position.byte_index == 0
    }

    pub fn is_at_end_of_line(self, lines: &[String]) -> bool {
        self.position.byte_index == lines[self.position.line_index].len()
    }

    pub fn is_at_first_row_of_line(self, layout: &Layout<'_>) -> bool {
        let (row, _) = layout
            .line(self.position.line_index)
            .logical_to_grid_position(self.position.byte_index, self.affinity);
        row == 0
    }

    pub fn is_at_last_row_of_line(self, layout: &Layout<'_>) -> bool {
        let line = layout.line(self.position.line_index);
        let (row, _) = line.logical_to_grid_position(self.position.byte_index, self.affinity);
        row == line.row_count() - 1
    }

    pub fn move_left(self, lines: &[String]) -> Self {
        if !self.is_at_start_of_line() {
            return self.move_to_prev_grapheme(lines);
        }
        if !self.is_at_first_line() {
            return self.move_to_end_of_prev_line(lines);
        }
        self
    }

    pub fn move_right(self, lines: &[String]) -> Self {
        if !self.is_at_end_of_line(lines) {
            return self.move_to_next_grapheme(lines);
        }
        if !self.is_at_last_line(lines.len()) {
            return self.move_to_start_of_next_line();
        }
        self
    }

    pub fn move_up(self, layout: &Layout<'_>) -> Self {
        if !self.is_at_first_row_of_line(layout) {
            return self.move_to_prev_row_of_line(layout);
        }
        if !self.is_at_first_line() {
            return self.move_to_last_row_of_prev_line(layout);
        }
        self.move_to_start_of_line()
    }

    pub fn move_down(self, layout: &Layout<'_>) -> Self {
        if !self.is_at_last_row_of_line(layout) {
            return self.move_to_next_row_of_line(layout);
        }
        if !self.is_at_last_line(layout.as_text().as_lines().len()) {
            return self.move_to_first_row_of_next_line(layout);
        }
        self.move_to_end_of_line(layout.as_text().as_lines())
    }

    pub fn home(self, lines: &[String]) -> Self {
        if !self.is_at_start_of_line() {
            let indent_len = lines[self.position.line_index].indent().unwrap_or("").len();
            if self.position.byte_index <= indent_len {
                return self.move_to_start_of_line();
            } else {
                return Self {
                    position: Position {
                        line_index: self.position.line_index,
                        byte_index: indent_len,
                    },
                    affinity: Affinity::Before,
                    preferred_column_index: None,
                };
            }
        }
        self
    }

    pub fn end(self, lines: &[String]) -> Self {
        if !self.is_at_end_of_line(lines) {
            let indent_len = lines[self.position.line_index].indent().unwrap_or("").len();
            if self.position.byte_index >= indent_len {
                return self.move_to_end_of_line(lines);
            } else {
                return Self {
                    position: Position {
                        line_index: self.position.line_index,
                        byte_index: indent_len,
                    },
                    affinity: Affinity::After,
                    preferred_column_index: None,
                };
            }
        }
        self
    }

    pub fn move_to_end_of_line(self, lines: &[String]) -> Self {
        let mut me = self.clone();
        while !me.is_at_end_of_line(lines) {
            me = me.move_to_next_grapheme(lines);
        }
        me
    }

    pub fn move_to_start_of_line(self) -> Self {
        Self {
            position: Position {
                line_index: self.position.line_index,
                byte_index: 0,
            },
            affinity: Affinity::Before,
            preferred_column_index: None,
        }
    }
    
    pub fn move_to_file_start(self) -> Self {
        Self {
            position: Position {
                line_index: 0,
                byte_index: 0,
            },
            affinity: Affinity::Before,
            preferred_column_index: None,
        }
    }

    pub fn move_to_file_end(self, lines: &[String]) -> Self {
        Self {
            position: Position {
                line_index: lines.len() - 1,
                byte_index: lines[lines.len() - 1].len(),
            },
            affinity: Affinity::After,
            preferred_column_index: None,
        }
    }

    pub fn move_to_prev_grapheme(self, lines: &[String]) -> Self {
        Self {
            position: Position {
                line_index: self.position.line_index,
                byte_index: lines[self.position.line_index][..self.position.byte_index]
                    .grapheme_indices()
                    .next_back()
                    .map(|(index, _)| index)
                    .unwrap(),
            },
            affinity: Affinity::After,
            preferred_column_index: None,
        }
    }

    pub fn move_to_next_grapheme(self, lines: &[String]) -> Self {
        let line = &lines[self.position.line_index];
        Self {
            position: Position {
                line_index: self.position.line_index,
                byte_index: line[self.position.byte_index..]
                    .grapheme_indices()
                    .nth(1)
                    .map(|(index, _)| self.position.byte_index + index)
                    .unwrap_or(line.len()),
            },
            affinity: Affinity::Before,
            preferred_column_index: None,
        }
    }

    pub fn move_to_end_of_prev_line(self, lines: &[String]) -> Self {
        let prev_line_index = self.position.line_index - 1;
        Self {
            position: Position {
                line_index: prev_line_index,
                byte_index: lines[prev_line_index].len(),
            },
            affinity: Affinity::After,
            preferred_column_index: None,
        }
    }

    pub fn move_to_start_of_next_line(self) -> Self {
        Self {
            position: Position {
                line_index: self.position.line_index + 1,
                byte_index: 0,
            },
            affinity: Affinity::Before,
            preferred_column_index: None,
        }
    }

    pub fn move_to_prev_row_of_line(self, layout: &Layout<'_>) -> Self {
        let line = layout.line(self.position.line_index);
        let (row_index, mut column_index) =
            line.logical_to_grid_position(self.position.byte_index, self.affinity);
        if let Some(preferred_column_index) = self.preferred_column_index {
            column_index = preferred_column_index;
        }
        let (byte_index, affinity) = line.grid_to_logical_position(row_index - 1, column_index);
        Self {
            position: Position {
                line_index: self.position.line_index,
                byte_index,
            },
            affinity,
            preferred_column_index: Some(column_index),
        }
    }

    pub fn move_to_next_row_of_line(self, layout: &Layout<'_>) -> Self {
        let line = layout.line(self.position.line_index);
        let (row_index, mut column_index) =
            line.logical_to_grid_position(self.position.byte_index, self.affinity);
        if let Some(preferred_column_index) = self.preferred_column_index {
            column_index = preferred_column_index;
        }
        let (byte, affinity) = line.grid_to_logical_position(row_index + 1, column_index);
        Self {
            position: Position {
                line_index: self.position.line_index,
                byte_index: byte,
            },
            affinity,
            preferred_column_index: Some(column_index),
        }
    }

    pub fn move_to_last_row_of_prev_line(self, layout: &Layout<'_>) -> Self {
        let line = layout.line(self.position.line_index);
        let (_, mut column_index) =
            line.logical_to_grid_position(self.position.byte_index, self.affinity);
        if let Some(preferred_column_index) = self.preferred_column_index {
            column_index = preferred_column_index;
        }
        let prev_line = layout.line(self.position.line_index - 1);
        let (byte_index, affinity) =
            prev_line.grid_to_logical_position(prev_line.row_count() - 1, column_index);
        Self {
            position: Position {
                line_index: self.position.line_index - 1,
                byte_index,
            },
            affinity,
            preferred_column_index: Some(column_index),
        }
    }

    pub fn move_to_first_row_of_next_line(self, layout: &Layout<'_>) -> Self {
        let line = layout.line(self.position.line_index);
        let (_, mut column_index) =
            line.logical_to_grid_position(self.position.byte_index, self.affinity);
        if let Some(preferred_column_index) = self.preferred_column_index {
            column_index = preferred_column_index;
        }
        let next_line = layout.line(self.position.line_index + 1);
        let (byte_index, affinity) = next_line.grid_to_logical_position(0, column_index);
        Self {
            position: Position {
                line_index: self.position.line_index + 1,
                byte_index,
            },
            affinity,
            preferred_column_index: Some(column_index),
        }
    }


    pub fn apply_edit(self, edit: &Edit) -> Self {
        Self {
            position: self.position.apply_edit(edit),
            ..self
        }
    }
}

impl From<Position> for Cursor {
    fn from(position: Position) -> Self {
        Self {
            position,
            affinity: Affinity::Before,
            preferred_column_index: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Affinity {
    Before,
    After,
}

impl Default for Affinity {
    fn default() -> Self {
        Self::Before
    }
}
