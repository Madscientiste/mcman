use std::cmp::Ordering;

use anyhow::Result;
use regex::Regex;

pub mod env;
pub mod maven_import;
pub mod md;

pub struct SelectItem<T>(pub T, pub String);

impl<T> ToString for SelectItem<T> {
    fn to_string(&self) -> String {
        self.1.clone()
    }
}

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub fn is_default_str(s: &str) -> bool {
    s == "latest"
}

pub fn get_latest_semver(list: &[String]) -> Option<String> {
    let mut list = list
        .iter()
        .map(|s| s.split('.').collect())
        .collect::<Vec<Vec<_>>>();

    list.sort_by(|a, b| {
        let mut ia = a.iter();
        let mut ib = b.iter();
        loop {
            break match (ia.next(), ib.next()) {
                (Some(a), Some(b)) => {
                    let a = a.parse::<i32>();
                    let b = b.parse::<i32>();

                    match (a, b) {
                        (Ok(a), Ok(b)) => match a.cmp(&b) {
                            Ordering::Equal => continue,
                            ord => ord,
                        },
                        (Err(_), Ok(_)) => Ordering::Less,
                        (Ok(_), Err(_)) => Ordering::Greater,
                        _ => Ordering::Equal,
                    }
                }
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                _ => Ordering::Equal,
            };
        }
    });

    list.last().map(|v| v.join("."))
}

/// ci.luckto.me => ci-lucko-me
pub fn url_to_folder(url: &str) -> String {
    let folder = url.replace("https://", "");
    let folder = folder.replace("http://", "");
    let folder = folder.replace('/', " ");
    let folder = folder.trim();
    folder.replace(' ', "-")
}

static SANITIZE_R1: &str = "<(?:\"[^\"]*\"['\"]*|'[^']*'['\"]*|[^'\">])+>";

pub fn sanitize(s: &str) -> Result<String> {
    let re = Regex::new(SANITIZE_R1)?;

    Ok(re
        .replace_all(
            &s.replace('\n', " ").replace('\r', "").replace("<br>", " "),
            "",
        )
        .to_string())
}
