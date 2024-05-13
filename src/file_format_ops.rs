use std::{fmt, io};

pub fn csv<R: io::BufRead>(reader: R) -> anyhow::Result<String> {
    let mut csv_reader = csv::Reader::from_reader(reader);

    let header = csv_reader.headers()?.clone();
    let records = csv_reader.records().collect::<Result<Vec<_>, _>>()?;

    let printer = OrderlyTableLayout::new(records)
        .with_header(header)
        .with_col_sep(" | ");

    Ok(printer.to_string())
}

struct OrderlyTableLayout {
    header: Option<csv::StringRecord>,
    records: Vec<csv::StringRecord>,
    col_sep: String,
}

impl OrderlyTableLayout {
    pub fn new(records: Vec<csv::StringRecord>) -> Self {
        Self {
            header: None,
            records,
            col_sep: " ".to_string(),
        }
    }

    pub fn with_header(mut self, header: csv::StringRecord) -> Self {
        self.header = Some(header);
        self
    }

    #[allow(dead_code)]
    pub fn add_record(mut self, record: csv::StringRecord) -> Self {
        self.records.push(record);
        self
    }

    pub fn with_col_sep(mut self, col_sep: impl ToString) -> Self {
        self.col_sep = col_sep.to_string();
        self
    }

    fn get_max_width_per_column(&self) -> Vec<usize> {
        let max_cols = self.get_max_cols();
        let mut max_width_per_col = vec![0; max_cols];

        for record in self.iter_header_and_records() {
            for (col, field) in record.iter().enumerate() {
                max_width_per_col[col] = max_width_per_col[col].max(field.len());
            }
        }

        max_width_per_col
    }

    // Returns the maximum number of columns in the CSV file.
    // Thefore this doesn't assume that all rows have the same number of columns,
    // which is guaranteed by the `csv` crate when option `flexible` is disabled (the default).
    fn get_max_cols(&self) -> usize {
        self.iter_header_and_records()
            .map(|r| r.len())
            .max()
            .unwrap_or(0)
    }

    fn iter_header_and_records(&'_ self) -> impl Iterator<Item = &'_ csv::StringRecord> + '_ {
        self.header.iter().chain(self.records.iter())
    }
}

impl fmt::Display for OrderlyTableLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let max_width_per_col = self.get_max_width_per_column();

        let print_record =
            |f: &mut fmt::Formatter<'_>, record: &csv::StringRecord| -> fmt::Result {
                // Printing like this in case records have different number of columns
                // (e.g. when using option `flexible` in crate `csv`).
                for (col, col_width) in max_width_per_col.iter().enumerate() {
                    // Empty field is printed if the column doesn't exist in the record.
                    let field = record.get(col).unwrap_or_default();
                    write!(f, "{field:col_width$}")?;

                    let is_not_last = col < max_width_per_col.len() - 1;

                    if is_not_last {
                        write!(f, "{}", self.col_sep)?;
                    }
                }

                Ok(())
            };

        if let Some(header) = &self.header {
            print_record(f, header)?;
            writeln!(f)?;

            let mut header_width = max_width_per_col.iter().sum::<usize>();
            header_width +=
                self.col_sep.len() * max_width_per_col.len().checked_sub(1).unwrap_or_default();

            // Yeah yeah, missing "+" to indicate intersection of columns but it's hockey night!
            writeln!(f, "{}", "-".repeat(header_width))?;
        }

        for record in &self.records {
            print_record(f, record)?;
            writeln!(f)?;
        }

        Ok(())
    }
}
