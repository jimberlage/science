use models::Datapoint;

fn pad_string(s: &String, len: usize) -> String {
    let mut result = s.clone();

    while result.len() < len {
        result.push(' ');
    }

    result
}

pub struct DatapointsFormatter {
    column_sizes: Vec<usize>,
    rows: Vec<Vec<String>>,
}

impl DatapointsFormatter {
    fn new() -> DatapointsFormatter {
        DatapointsFormatter {
            column_sizes: vec![0, 0, 0, 0],
            rows: vec![],
        }
    }

    fn row(dp: &Datapoint) -> Vec<String> {
        vec![
            format!("{}", dp.id),
            dp.description.clone(),
            dp.sha.clone(),
            dp.status.clone(),
        ]
    }

    fn add_datapoint(&mut self, dp: &Datapoint) {
        let row = DatapointsFormatter::row(dp);

        for i in 0..row.len() {
            let column_size = row[i].len();

            if column_size > self.column_sizes[i] {
                self.column_sizes[i] = column_size;
            }
        }

        self.rows.push(row);
    }

    pub fn from_datapoints(dps: &Vec<Datapoint>) -> DatapointsFormatter {
        let mut result = DatapointsFormatter::new();

        for dp in dps {
            result.add_datapoint(&dp);
        }

        result
    }

    pub fn format(&self) -> String {
        let mut result = vec![];

        for row in &self.rows {
            let mut row_result = vec![];

            for i in 0..row.len() {
                row_result.push(pad_string(&row[i], self.column_sizes[i]));
            }

            result.push(row_result.join(" | "));
        }

        result.join("\n")
    }
}
