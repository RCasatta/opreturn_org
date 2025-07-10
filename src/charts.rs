use maud::Markup;
use maud::{html, PreEscaped};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::{Serialize, Serializer};
use std::fmt::{Display, Formatter};

#[derive(Debug, Serialize)]
pub enum Kind {
    #[serde(rename = "pie")]
    Pie,
    #[serde(rename = "line")]
    Line,
    #[serde(rename = "bar")]
    Bar,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Grey,
    Custom(u8, u8, u8, f32),
}

impl Color {
    fn components(&self) -> (u8, u8, u8, f32) {
        match self {
            Color::Custom(r, g, b, a) => (*r, *g, *b, *a),
            Color::Red => (255, 99, 132, 0.8),
            Color::Orange => (255, 159, 64, 0.8),
            Color::Yellow => (255, 205, 86, 0.8),
            Color::Green => (75, 192, 192, 0.8),
            Color::Blue => (54, 162, 235, 0.8),
            Color::Purple => (153, 102, 255, 0.8),
            Color::Grey => (201, 203, 207, 0.8),
        }
    }

    pub fn rainbow() -> Vec<Color> {
        vec![
            Color::Custom(0x63, 0x04, 0x64, 0.8),
            Color::Custom(0x33, 0x1F, 0x7A, 0.8),
            Color::Custom(0x33, 0x59, 0xAA, 0.8),
            Color::Custom(0x42, 0x8A, 0xAA, 0.8),
            Color::Custom(0x5F, 0xA8, 0x70, 0.8),
            Color::Custom(0x89, 0xB3, 0x4A, 0.8),
            Color::Custom(0xB7, 0xAF, 0x35, 0.8),
            Color::Custom(0xD8, 0x91, 0x2C, 0.8),
            Color::Custom(0xD9, 0x53, 0x22, 0.8),
            Color::Custom(0xC1, 0x06, 0x18, 0.8),
            Color::Custom(0xDF, 0xDF, 0xDF, 0.8), //final gray
        ]
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (r, g, b, a) = self.components();
        write!(f, "rgb({},{},{},{})", r, g, b, a)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chart {
    #[serde(skip)]
    id: String,
    #[serde(rename = "type")]
    kind: Kind,
    data: ChartData,
    options: Options,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    plugins: Plugins,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Plugins {
    title: Title,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Title {
    display: bool,
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChartData {
    labels: Vec<String>,
    datasets: Vec<Dataset>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    pub label: String,
    pub data: Vec<u64>,
    pub background_color: Vec<Color>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub border_color: Vec<Color>,
    pub fill: bool,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_dash: Option<[u8; 2]>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "yAxisID")]
    pub y_axis_id: Option<String>,
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Chart {
    pub fn new(title: String, kind: Kind, labels: Vec<String>) -> Chart {
        Chart {
            id: format!(
                "_{}",
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(10)
                    .map(char::from)
                    .collect::<String>()
            ),
            kind,
            data: ChartData {
                labels,
                datasets: vec![],
            },
            options: Options {
                plugins: Plugins {
                    title: Title {
                        display: true,
                        text: title,
                    },
                },
            },
        }
    }

    pub fn add_dataset(&mut self, mut dataset: Dataset, y_axis_id: Option<String>) {
        dataset.y_axis_id = y_axis_id;
        self.data.datasets.push(dataset)
    }

    pub fn to_json_dict(&self) -> String {
        let s = serde_json::to_string(&self).unwrap();
        let re = Regex::new("\"#([^#]+)#\"").unwrap();
        let result = re.replace_all(&s, "$1");
        result.to_string()
    }

    pub fn to_html(&self) -> Markup {
        let script = format!(
            "var {} = new Chart(document.getElementById('{}'), {});",
            self.id,
            self.id,
            self.to_json_dict(),
        );

        html! {
            div {
                canvas id=(self.id) {
                }
            }
            script {
                (PreEscaped(script))
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::charts::{Chart, Color, Dataset, Kind};
    use regex::Regex;
    use std::collections::BTreeMap;

    pub fn mock_pie_chart() -> Chart {
        let mut serie = BTreeMap::new();
        serie.insert("with".to_string(), 100u64);
        serie.insert("without".to_string(), 200);
        let labels: Vec<_> = serie.keys().cloned().collect();
        let mut chart = Chart::new(
            "Inputs with or without elements in witness".to_string(),
            Kind::Pie,
            labels,
        );
        let data: Vec<_> = serie.values().cloned().collect();
        chart.add_dataset(
            Dataset {
                data,
                label: "aaa".to_string(),
                background_color: vec![Color::Yellow],
                border_color: vec![],
                fill: true,
                hidden: false,
                border_dash: None,
                y_axis_id: None,
            },
            None,
        );

        chart
    }

    pub fn mock_lines_chart() -> Chart {
        let mut serie = BTreeMap::new();
        serie.insert("2019".to_string(), 50u64);
        serie.insert("2020".to_string(), 100u64);
        serie.insert("2021".to_string(), 200);
        let labels: Vec<_> = serie.keys().cloned().collect();
        let mut chart = Chart::new(
            "Inputs with or without elements in witness".to_string(),
            Kind::Line,
            labels,
        );
        let data: Vec<_> = serie.values().cloned().collect();
        chart.add_dataset(
            Dataset {
                data,
                label: "aaa".to_string(),
                background_color: vec![Color::Blue],
                border_color: vec![],
                fill: true,
                hidden: false,
                border_dash: None,
                y_axis_id: None,
            },
            None,
        );

        chart
    }

    #[test]
    fn test_regex() {
        let re = Regex::new("\"#(.+)#\"").unwrap();
        let text = "\"#azzo#\"";
        let result = re.replace_all(text, "$1");
        assert_eq!(result, "azzo");
    }

    #[ignore]
    #[test]
    fn test_chart() {
        let chart = mock_pie_chart();

        //chart.add(serie);
        assert_eq!("", chart.to_html().into_string());
    }
}
