use unicode_width::UnicodeWidthStr;

pub struct TablePrint {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl TablePrint {
    pub fn print(&self) {
        let num_columns = self.headers.len();
        let mut column_widths = vec![0; num_columns];

        // Calculate width for headers
        for (i, header) in self.headers.iter().enumerate() {
            column_widths[i] = UnicodeWidthStr::width(header.as_str());
        }

        // Calculate width for rows (accounting for visible width, not ANSI codes)
        for row in &self.rows {
            for (i, item) in row.iter().enumerate().take(num_columns) {
                // Strip ANSI color codes for width calculation
                let visible_width = strip_ansi_codes(item).width();
                column_widths[i] = column_widths[i].max(visible_width);
            }
        }

        // Print headers
        for (i, header) in self.headers.iter().enumerate() {
            print!("{:<width$}   ", header, width = column_widths[i]);
        }
        println!();

        // Print rows
        for row in &self.rows {
            for (i, item) in row.iter().enumerate().take(num_columns) {
                // For colored strings, we need to pad differently
                let visible_text = strip_ansi_codes(item);
                let padding = column_widths[i].saturating_sub(visible_text.width());
                print!("{}{:padding$}   ", item, "", padding = padding);
            }
            println!();
        }
    }
}

fn strip_ansi_codes(s: &str) -> String {
    strip_ansi_escapes::strip_str(s)
}
