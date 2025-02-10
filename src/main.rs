use std::io::Write;

use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Input contest name[ex. abc390, arc195] > ");
    std::io::stdout().flush()?;
    let contest_kind = read_line();
    let contest_kind =
        ContestKind::parse(&contest_kind).ok_or(format!("parse error on {}", contest_kind))?;
    let contest = contest_kind.get_contest_info();
    let json = serde_json::to_string(&contest)?;
    write_file("contest.json", &json)?;

    Ok(())
}
#[derive(Serialize, Deserialize, Debug, Clone)]
enum ContestKind {
    ABC(usize),
    ARC(usize),
    AGC(usize),
}
impl ContestKind {
    fn problem_list(&self) -> String {
        match self {
            Self::ABC(number) => {
                if *number <= 125 {
                    "abcd".to_string()
                } else {
                    "abcdef".to_string()
                }
            }
            Self::ARC(number) => {
                if *number <= 57 {
                    "abcd".to_string()
                } else if *number <= 103 {
                    "cdef".to_string()
                } else {
                    "abcd".to_string()
                }
            }
            Self::AGC(number) => (0..*number).map(|i| format!("AGC{}", i)).collect(),
        }
    }
    fn get_contest_info(&self) -> Contest {
        let problem_name_list = self.problem_list();
        let mut problems = vec![];
        for diff in problem_name_list.chars() {
            let url = match self {
                Self::ABC(number) => {
                    format!(
                        "https://atcoder.jp/contests/abc{number:03}/tasks/abc{number:03}_{diff}"
                    )
                }
                Self::ARC(number) => {
                    format!(
                        "https://atcoder.jp/contests/arc{number:03}/tasks/arc{number:03}_{diff}"
                    )
                }
                Self::AGC(number) => {
                    format!(
                        "https://atcoder.jp/contests/agc{number:03}/tasks/agc{number:03}_{diff}"
                    )
                }
            };
            let html = send_request(&url).unwrap();
            let io_list = extract_input_output(html);
            let problem = Problem {
                diff: diff.to_string(),
                expected_in_out: io_list,
            };
            problems.push(problem);
        }
        Contest {
            kind: self.clone(),
            problem: problems,
        }
    }

    fn parse(contest_name: &str) -> Option<Self> {
        let contest_name = contest_name.trim().to_lowercase();
        match &contest_name[..3] {
            "abc" => {
                let number = contest_name[3..].parse::<usize>().ok()?;
                Some(Self::ABC(number))
            }
            "arc" => {
                let number = contest_name[3..].parse::<usize>().ok()?;
                Some(Self::ARC(number))
            }
            "agc" => {
                let number = contest_name[3..].parse::<usize>().ok()?;
                Some(Self::AGC(number))
            }
            _ => None,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Problem {
    diff: String,
    expected_in_out: Vec<(String, String)>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Contest {
    kind: ContestKind,
    problem: Vec<Problem>,
}
fn read_line() -> String {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string()
}
fn write_file(f_name: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create(f_name)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn send_request(url: &str) -> Result<Html, Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get(url)?.text()?;
    Ok(scraper::Html::parse_document(&body))
}

fn extract_input_output(html: Html) -> Vec<(String, String)> {
    let selector = scraper::Selector::parse(r#"div[class="part"]"#).unwrap();
    let elements = html.select(&selector);
    let mut inputs = vec![None; 10];
    let mut outputs = vec![None; 10];
    for element in elements {
        let element = MyElement::new(element);
        let h3 = element.get(r#"section"#).and_then(|e| e.get(r#"h3"#));
        let pre = element.get(r#"section"#).and_then(|e| e.get(r#"pre"#));
        match (h3, pre) {
            (Some(h3), Some(pre)) => {
                if let Some(number) = get_input_number(&h3.text()) {
                    inputs[number] = Some(pre.text());
                } else if let Some(number) = get_output_number(&h3.text()) {
                    outputs[number] = Some(pre.text());
                }
            }
            _ => continue,
        }
    }
    inputs
        .iter()
        .zip(outputs.iter())
        .filter_map(|(i, o)| match (i, o) {
            (Some(i), Some(o)) => Some((i.clone(), o.clone())),
            _ => None,
        })
        .collect()
}

fn get_input_number(text: &str) -> Option<usize> {
    // 入力例 1 or Sample Input 1 -> 1
    if text.contains("入力例") || text.contains("Input") {
        let number = text.split_whitespace().last().unwrap();
        return number.parse::<usize>().ok();
    } else {
        return None;
    }
}
fn get_output_number(text: &str) -> Option<usize> {
    // 出力例 1 or Sample Output 1 -> 1
    if text.contains("出力例") || text.contains("Output") {
        let number = text.split_whitespace().last().unwrap();
        return number.parse::<usize>().ok();
    } else {
        return None;
    }
}

struct MyElement<'a> {
    element: ElementRef<'a>,
}
impl<'a> MyElement<'a> {
    #[allow(dead_code)]
    fn new(element: ElementRef<'a>) -> Self {
        Self { element }
    }
    #[allow(dead_code)]
    fn get_all(&self, selector: &str) -> Vec<MyElement<'a>> {
        self.element
            .select(&scraper::Selector::parse(selector).unwrap())
            .map(|e| MyElement::new(e))
            .collect()
    }
    #[allow(dead_code)]
    fn get(&self, selector: &str) -> Option<MyElement<'a>> {
        self.element
            .select(&scraper::Selector::parse(selector).unwrap())
            .next()
            .map(|e| MyElement::new(e))
    }
    #[allow(dead_code)]
    fn text(&self) -> String {
        self.element.text().collect()
    }
}
