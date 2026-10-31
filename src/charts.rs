use maud::Markup;
use maud::{html, PreEscaped};
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
        serde_json::to_string(&self).unwrap()
    }

    /// `id` must be unique in the page, it's used both as canvas id and js variable name
    pub fn to_html(&self, id: &str) -> Markup {
        let script = format!(
            "var {} = new Chart(document.getElementById('{}'), {});",
            id,
            id,
            self.to_json_dict(),
        );

        html! {
            div class="chart" {
                canvas id=(id) {
                }
            }
            script {
                (PreEscaped(script))
            }
        }
    }
}
