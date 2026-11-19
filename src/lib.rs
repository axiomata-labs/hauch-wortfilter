//! Wortfilter, a plugin for Hauch: the user's own replacements for words its recogniser gets wrong ("Lirfel" →
//! "Lirvel"), applied to every text Hauch recognises before it shows it. The replacements live in the plugin's
//! storage, one per line as `heard<TAB>instead`, and are edited on its page in Hauch's "Plugins".

wit_bindgen::generate!({ path: "wit", world: "plugin", generate_all });

use axiomata::plugin::ui::{Action, Block, Button, Event, Field, Node, Page};
use axiomata::plugin::{log, settings, storage};
use exports::axiomata::plugin::page;
use exports::hauch::plugin::hooks;

/// Where the replacements are kept.
const KEY: &str = "replacements";
/// How many replacements, and how long each side.
const MAX_REPLACEMENTS: usize = 500;
const MAX_CHARS: usize = 100;

/// One replacement: what the recogniser writes, and what the user wants instead.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Replacement {
    heard: String,
    instead: String,
}

fn read() -> Vec<Replacement> {
    let Ok(Some(bytes)) = storage::get(KEY) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .map(|(heard, instead)| Replacement {
            heard: heard.to_owned(),
            instead: instead.to_owned(),
        })
        .collect()
}

fn write(replacements: &[Replacement]) -> Result<(), String> {
    let text: String = replacements
        .iter()
        .map(|replacement| format!("{}\t{}\n", replacement.heard, replacement.instead))
        .collect();
    storage::set(KEY, text.as_bytes())
}

/// `text` with every replacement applied: the heard words found regardless of case, only as whole words, the
/// longest first so "Lir Vel" wins over "Lir".
fn replace(text: &str, replacements: &[Replacement]) -> String {
    let mut ordered: Vec<&Replacement> = replacements
        .iter()
        .filter(|r| !r.heard.is_empty())
        .collect();
    ordered.sort_by_key(|replacement| std::cmp::Reverse(replacement.heard.chars().count()));
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    'outer: while i < chars.len() {
        let starts_word = i == 0 || !chars[i - 1].is_alphanumeric();
        if starts_word {
            for replacement in &ordered {
                if let Some(length) = matches_at(&chars, i, &replacement.heard) {
                    let ends_word = chars
                        .get(i + length)
                        .is_none_or(|next| !next.is_alphanumeric());
                    if ends_word {
                        out.push_str(&replacement.instead);
                        i += length;
                        continue 'outer;
                    }
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// How many characters of `chars` from `at` spell `word`, regardless of case.
fn matches_at(chars: &[char], at: usize, word: &str) -> Option<usize> {
    let mut length = 0;
    for wanted in word.chars() {
        let found = chars.get(at + length)?;
        if !found.to_lowercase().eq(wanted.to_lowercase()) {
            return None;
        }
        length += 1;
    }
    Some(length)
}

/// A field's text as kept: one line, no tabs, trimmed, not too long.
fn tidy(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .trim()
        .chars()
        .take(MAX_CHARS)
        .collect()
}

/// The page's words in German or English.
struct Words {
    title: &'static str,
    none: &'static str,
    remove: &'static str,
    new: &'static str,
    heard: &'static str,
    heard_example: &'static str,
    instead: &'static str,
    instead_example: &'static str,
    add: &'static str,
    both: &'static str,
    full: &'static str,
    not_kept: &'static str,
}

const DE: Words = Words {
    title: "Ersetzungen",
    none: "Noch keine. Was Hauch falsch versteht, trägst du unten ein: Es wird in allem ersetzt, was Hauch erkennt.",
    remove: "Entfernen",
    new: "Neue Ersetzung",
    heard: "Hauch versteht",
    heard_example: "Lirfel",
    instead: "Gemeint ist",
    instead_example: "Lirvel",
    add: "Hinzufügen",
    both: "Füll beide Felder aus.",
    full: "Mehr Ersetzungen passen nicht hinein.",
    not_kept: "Nicht gespeichert:",
};

const EN: Words = Words {
    title: "Replacements",
    none: "None yet. Enter below what Hauch gets wrong: it is replaced in everything Hauch recognises.",
    remove: "Remove",
    new: "New replacement",
    heard: "Hauch hears",
    heard_example: "Lirfel",
    instead: "Meant is",
    instead_example: "Lirvel",
    add: "Add",
    both: "Fill in both fields.",
    full: "No more replacements fit.",
    not_kept: "Not saved:",
};

fn words() -> &'static Words {
    if settings::language().starts_with("de") {
        &DE
    } else {
        &EN
    }
}

fn node(parent: Option<u32>, block: Block) -> Node {
    Node { parent, block }
}

/// The form's two fields as the user left them: kept on the page after anything but an addition that worked.
#[derive(Default)]
struct Typed {
    heard: String,
    instead: String,
}

/// The page, with `typed` in the form's fields and `notice` under it if there is one.
fn page_with(notice: Option<String>, typed: &Typed) -> Page {
    let w = words();
    let replacements = read();
    let mut nodes = vec![node(None, Block::Card(w.title.into()))];
    if replacements.is_empty() {
        nodes.push(node(Some(0), Block::Note(w.none.into())));
    } else {
        nodes.push(node(Some(0), Block::List));
        for (index, replacement) in replacements.iter().enumerate() {
            let row = nodes.len() as u32;
            nodes.push(node(Some(1), Block::Row));
            nodes.push(node(
                Some(row),
                Block::Text(format!(
                    "„{}“ → „{}“",
                    replacement.heard, replacement.instead
                )),
            ));
            nodes.push(node(
                Some(row),
                Block::Button(Button {
                    id: format!("remove-{index}"),
                    label: w.remove.into(),
                    primary: false,
                }),
            ));
        }
    }
    let form = nodes.len() as u32;
    nodes.push(node(None, Block::Card(w.new.into())));
    let fields = nodes.len() as u32;
    nodes.push(node(Some(form), Block::Row));
    for (id, label, example, value) in [
        ("heard", w.heard, w.heard_example, &typed.heard),
        ("instead", w.instead, w.instead_example, &typed.instead),
    ] {
        nodes.push(node(
            Some(fields),
            Block::Field(Field {
                id: id.into(),
                label: label.into(),
                value: value.clone(),
                placeholder: example.into(),
            }),
        ));
    }
    nodes.push(node(
        Some(form),
        Block::Button(Button {
            id: "add".into(),
            label: w.add.into(),
            primary: true,
        }),
    ));
    if let Some(notice) = notice {
        nodes.push(node(Some(form), Block::Note(notice)));
    }
    Page { nodes }
}

struct Wortfilter;

impl hooks::Guest for Wortfilter {
    fn filter_recognised(text: String, _language: Option<String>) -> Option<String> {
        let replaced = replace(&text, &read());
        (replaced != text).then_some(replaced)
    }
}

impl page::Guest for Wortfilter {
    fn show() -> Page {
        page_with(None, &Typed::default())
    }

    fn handle(event: Event) -> Page {
        let w = words();
        let field = |id: &str| {
            event
                .fields
                .iter()
                .find(|(field, _)| field == id)
                .map(|(_, value)| tidy(value))
                .unwrap_or_default()
        };
        let typed = Typed {
            heard: field("heard"),
            instead: field("instead"),
        };
        let Action::Clicked(id) = &event.action else {
            return page_with(None, &typed);
        };
        let mut replacements = read();
        let mut added = false;
        if id == "add" {
            if typed.heard.is_empty() || typed.instead.is_empty() {
                return page_with(Some(w.both.into()), &typed);
            }
            if replacements.len() >= MAX_REPLACEMENTS {
                return page_with(Some(w.full.into()), &typed);
            }
            replacements.retain(|replacement| {
                replacement.heard.to_lowercase() != typed.heard.to_lowercase()
            });
            replacements.push(Replacement {
                heard: typed.heard.clone(),
                instead: typed.instead.clone(),
            });
            added = true;
        } else if let Some(index) = id
            .strip_prefix("remove-")
            .and_then(|index| index.parse::<usize>().ok())
        {
            if index < replacements.len() {
                replacements.remove(index);
            }
        } else {
            return page_with(None, &typed);
        }
        match write(&replacements) {
            // An addition that worked empties the form for the next one; anything else leaves what was typed.
            Ok(()) if added => page_with(None, &Typed::default()),
            Ok(()) => page_with(None, &typed),
            Err(why) => {
                log::log(
                    log::Level::Error,
                    &format!("could not keep the replacements: {why}"),
                );
                page_with(Some(format!("{} {why}", w.not_kept)), &typed)
            }
        }
    }
}

export!(Wortfilter);

#[cfg(test)]
mod tests {
    use super::*;

    fn r(heard: &str, instead: &str) -> Replacement {
        Replacement {
            heard: heard.into(),
            instead: instead.into(),
        }
    }

    #[test]
    fn whole_words_regardless_of_case_the_longest_first() {
        let list = [r("lirfel", "Lirvel"), r("Lir", "X"), r("Lir Vel", "Lirvel")];
        assert_eq!(
            replace("Hallo Lirfel, hallo LIRFEL.", &list),
            "Hallo Lirvel, hallo Lirvel."
        );
        assert_eq!(replace("Lir Vel ist da", &list), "Lirvel ist da");
        assert_eq!(
            replace("Lirfeln bleibt", &list),
            "Lirfeln bleibt",
            "only whole words"
        );
        assert_eq!(
            replace("Grüße an Ölfel", &[r("ölfel", "Elfe")]),
            "Grüße an Elfe"
        );
        assert_eq!(replace("nichts", &[]), "nichts");
    }

    #[test]
    fn a_field_is_one_tidy_line() {
        assert_eq!(tidy("  Lir\tvel\n "), "Lir vel");
        assert_eq!(tidy(&"x".repeat(200)).chars().count(), MAX_CHARS);
    }
}
