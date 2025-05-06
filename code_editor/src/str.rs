use crate::char::CharExt;

pub trait StrExt {
    fn column_count(&self) -> usize;
    fn indent_level(&self, indent_column_count: usize) -> usize;
    fn next_indent_level(&self, indent_column_count: usize) -> usize;
    fn prev_indent_level(&self, indent_column_count: usize) -> usize;
    fn find_next_word_boundary(&self, index: usize, word_separators: &[char]) -> usize;
    fn find_prev_word_boundary(&self, index: usize, word_separators: &[char]) -> usize;
    fn indent(&self) -> Option<&str>;
    fn longest_common_prefix(&self, other: &str) -> &str;
    fn graphemes(&self) -> Graphemes<'_>;
    fn grapheme_indices(&self) -> GraphemeIndices<'_>;
    fn split_whitespace_boundaries(&self) -> SplitWhitespaceBoundaries<'_>;
}

impl StrExt for str {
    fn column_count(&self) -> usize {
        self.chars().map(|char| char.column_count()).sum()
    }

    fn indent_level(&self, indent_column_count: usize) -> usize {
        self.indent().unwrap_or("").column_count() / indent_column_count
    }

    fn next_indent_level(&self, indent_column_count: usize) -> usize {
        (self.indent().unwrap_or("").column_count() + indent_column_count) / indent_column_count
    }

    fn prev_indent_level(&self, indent_column_count: usize) -> usize {
        self.indent().unwrap_or("").column_count().saturating_sub(1) / indent_column_count
    }

    fn find_next_word_boundary(&self, index: usize, word_separators: &[char]) -> usize {
        if index == 0 {
            return index;
        }
        let start = index;
        self[index..]
            .char_indices()
            .find(|&(_, char)| word_separators.contains(&char))
            .map(|(index, _)| start + index)
            .unwrap_or_else(|| self.len())
    }

    fn find_prev_word_boundary(&self, index: usize, word_separators: &[char]) -> usize {
        if index == self.len() {
            return index;
        }
        self[..index]
            .char_indices()
            .rfind(|&(_, char)| word_separators.contains(&char))
            .map(|(char_index, char)| char_index + char.len_utf8())
            .unwrap_or(0)
    }

    fn indent(&self) -> Option<&str> {
        self.char_indices()
            .find(|(_, char)| !char.is_whitespace())
            .map(|(index, _)| &self[..index])
    }

    fn longest_common_prefix(&self, other: &str) -> &str {
        &self[..self
            .char_indices()
            .zip(other.chars())
            .find(|((_, char_0), char_1)| char_0 == char_1)
            .map(|((index, _), _)| index)
            .unwrap_or_else(|| self.len().min(other.len()))]
    }

    fn graphemes(&self) -> Graphemes<'_> {
        Graphemes { string: self }
    }

    fn grapheme_indices(&self) -> GraphemeIndices<'_> {
        GraphemeIndices {
            graphemes: self.graphemes(),
            start: self.as_ptr() as usize,
        }
    }

    fn split_whitespace_boundaries(&self) -> SplitWhitespaceBoundaries<'_> {
        SplitWhitespaceBoundaries { string: self }
    }
}

#[derive(Clone, Debug)]
pub struct Graphemes<'a> {
    string: &'a str,
}

impl<'a> Iterator for Graphemes<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            return None;
        }
        let mut end = 1;
        while !self.string.is_char_boundary(end) {
            end += 1;
        }
        let (grapheme, string) = self.string.split_at(end);
        self.string = string;
        Some(grapheme)
    }
}

impl<'a> DoubleEndedIterator for Graphemes<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            return None;
        }
        let mut start = self.string.len() - 1;
        while !self.string.is_char_boundary(start) {
            start -= 1;
        }
        let (string, grapheme) = self.string.split_at(start);
        self.string = string;
        Some(grapheme)
    }
}

#[derive(Clone, Debug)]
pub struct GraphemeIndices<'a> {
    graphemes: Graphemes<'a>,
    start: usize,
}

impl<'a> Iterator for GraphemeIndices<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = self.graphemes.next()?;
        Some((grapheme.as_ptr() as usize - self.start, grapheme))
    }
}

impl<'a> DoubleEndedIterator for GraphemeIndices<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let grapheme = self.graphemes.next_back()?;
        Some((grapheme.as_ptr() as usize - self.start, grapheme))
    }
}

#[derive(Clone, Debug)]
pub struct SplitWhitespaceBoundaries<'a> {
    string: &'a str,
}

impl<'a> Iterator for SplitWhitespaceBoundaries<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.string.is_empty() {
            return None;
        }
        let mut prev_char_is_whitespace = None;
        let index = self
            .string
            .char_indices()
            .find_map(|(index, next_char)| {
                let next_char_is_whitespace = next_char.is_whitespace();
                let is_whitespace_boundary = prev_char_is_whitespace
                    .map_or(false, |prev_char_is_whitespace| {
                        prev_char_is_whitespace != next_char_is_whitespace
                    });
                prev_char_is_whitespace = Some(next_char_is_whitespace);
                if is_whitespace_boundary {
                    Some(index)
                } else {
                    None
                }
            })
            .unwrap_or(self.string.len());
        let (string_0, string_1) = self.string.split_at(index);
        self.string = string_1;
        Some(string_0)
    }
}
