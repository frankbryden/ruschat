use radix_trie::{Trie, TrieCommon};
use std::fs::File;

use serde::{Deserialize, Serialize};
use serde_json::Result;

const EMOJI_QUERY_LIMIT: usize = 10;

#[derive(Serialize, Deserialize, Debug)]
struct EmojiEntry {
    name: String,
    is_alias: u8,
    alias_for: String,
    url: String,
}

fn load_emojis(emoji_file: &str) -> Result<Vec<EmojiEntry>> {
    let file = File::open(emoji_file).unwrap();
    let start = std::time::Instant::now();
    let emojis: Vec<EmojiEntry> = serde_json::from_reader(file)?;
    println!(
        "Loaded {} emojis in {}s",
        emojis.len(),
        start.elapsed().as_secs()
    );
    // emojis
    Ok(emojis)
}

pub struct EmojisHolder {
    emojis: Trie<String, String>,
}

impl EmojisHolder {
    pub fn new(emoji_file: &str) -> EmojisHolder {
        let emojis = load_emojis(emoji_file).unwrap();
        let mut trie = Trie::new();
        for emoji_entry in emojis {
            trie.insert(emoji_entry.name, emoji_entry.url);
        }
        EmojisHolder { emojis: trie }
    }

    pub fn query(&self, query: &str) -> Vec<(&String, &String)> {
        let result = self.emojis.subtrie(query);
        if let Some(subtrie) = result {
            subtrie
                .iter()
                .take(EMOJI_QUERY_LIMIT)
                .collect::<Vec<(&String, &String)>>()
        } else {
            vec![]
        }
    }
}
