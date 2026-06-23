use std::{collections::BTreeMap, fs, iter, path::{Path, PathBuf}, sync::Arc};

use ct_regex::{AnonRegex, regex};
use derive_more::{Deref, DerefMut, Display};
use serde::{Deserialize, Serialize};
use srt_subtitles_parser as srt;

pub fn normalize_sub(text: &str) -> String {
    let mut text = text.to_lowercase();
    regex!(r"\s+").replace_all(&mut text, " ");
    regex!(r"[^a-z \-]").replace_all(&mut text, "");
    text
}

pub fn collect_words(text: &str) -> Vec<Box<str>> {
    text.split(' ')
        .filter(|word| !word.is_empty())
        .map(Box::clone_from_ref)
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
#[display("{title}")]
pub struct Media {
    pub name: Box<str>,
    pub title: Box<str>,
    pub icon: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeMetadataInner {
    pub name: Box<str>,
    pub title: Box<str>,
    pub season: u16,
    pub number: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deref, Display)]
#[display("S{}E{}: {}", inner.season, inner.number, inner.title)]
pub struct Episode {
    #[deref]
    pub inner: EpisodeMetadataInner,
    pub media: Arc<Media>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deref)]
pub struct Subtitle {
    #[deref]
    pub subtitle: srt::Subtitle,
    pub episode: Arc<Episode>,
}

pub type Sub = Arc<Subtitle>;

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct WordSeqBuilder(pub BTreeMap<usize, Vec<Sub>>);

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct WordSeq(pub BTreeMap<usize, Box<[Sub]>>);

impl WordSeq {
    pub fn values_flattened(&self) -> Vec<&Sub> {
        self.values()
            .flatten()
            .collect()
    }
}

impl From<WordSeqBuilder> for WordSeq {
    fn from(value: WordSeqBuilder) -> Self {
        WordSeq(
            value.0.into_iter()
                .map(|(index, list)| (index, list.into()))
                .collect()
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct WordMapBuilder {
    pub word: Box<str>,
    pub metadata: WordSeqBuilder,
}

#[derive(Debug, Clone, Default)]
pub struct WordMap {
    pub word: Box<str>,
    pub metadata: WordSeq,
}

impl From<(Box<str>, WordSeqBuilder)> for WordMapBuilder {
    fn from((word, metadata): (Box<str>, WordSeqBuilder)) -> Self {
        WordMapBuilder { word, metadata }
    }
}

impl From<WordMapBuilder> for WordMap {
    fn from(value: WordMapBuilder) -> Self {
        WordMap { word: value.word, metadata: value.metadata.into() }
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct SubIndex(pub Box<[WordMap]>);

impl SubIndex {
    pub fn binary_search(&self, word: &str) -> Option<usize> {
        self.binary_search_by_key(&word, |map| &map.word).ok()
    }

    pub fn find_next_word(&self, index: usize, word: &str) -> Option<usize> {
        self[index].metadata.keys()
            .find(|&&key| &*self[key].word == word)
            .copied()
    }

    pub fn search_subs<'w, 'i>(
        &'w self,
        search: impl Iterator<Item = &'i str>
    ) -> Vec<Sub> {
        let mut search = search.peekable();
        let Some(first) = search.next() else {
            return Vec::new();
        };
        let Some(mut index) = self.binary_search(first) else {
            return Vec::new();
        };

        let mut collected_subs = self[index].metadata.values_flattened();

        for word in search {
            // dbg!(&word);
            index = match self.find_next_word(index, word) {
                Some(i) => i,
                None => return Vec::new(),
            };

            let new_subs = self[index].metadata.values_flattened();

            // dbg!(&new_subs.iter().map(|i| &i.text).collect::<Vec<_>>());

            collected_subs.retain(|sub| new_subs.contains(sub));
        }

        collected_subs.into_iter()
            .cloned()
            .collect()
    }

    pub fn search_with_query(&self, query: &str) -> Vec<Sub> {
        let query_words = collect_words(&normalize_sub(query));
        self.search_subs(query_words.iter().map(Box::as_ref))
    }
}

#[derive(Debug, Clone)]
pub struct SubSet {
    pub list: Box<[Sub]>,
    pub index: SubIndex,
}

impl SubSet {
    pub fn parse_file(sub_path: impl AsRef<Path>, episode: Arc<Episode>) -> SubSet {
        let list = srt::parse_srt(&fs::read_to_string(sub_path).unwrap()).unwrap()
            .subtitles
            .into_iter()
            .map(|sub| Arc::new(Subtitle {
                subtitle: sub,
                episode: episode.clone()
            }))
            .collect::<Box<[_]>>();

        let text = list.iter().map(|sub| (
            sub,
            normalize_sub(&sub.text)
        ));

        let lines = text.map(|(sub, text)| (
            sub,
            collect_words(&text)
        )).collect::<Vec<_>>();

        let mut words = iter::once(WordMapBuilder::default())
            .chain(
                lines.iter()
                    .flat_map(|(_, line)| {
                        line.iter()
                            .cloned()
                            .zip(iter::repeat(WordSeqBuilder::default()))
                    })
                    .collect::<BTreeMap<Box<str>, WordSeqBuilder>>()
                    .into_iter()
                    .map(WordMapBuilder::from)
            ).collect::<Box<[_]>>();

        for (sub, line) in lines.iter() {
            // println!("{:?}", line);
            line.iter()
                .map(|word| words.binary_search_by_key(&word, |map| &map.word).unwrap())
                .collect::<Vec<_>>()
                .into_iter()
                .chain(iter::once(0))
                .map_windows(|&[a, b]| {
                    words[a].metadata
                        .entry(b)
                        .or_default()
                        .push(Arc::clone(sub))
                })
                .last();
        }

        SubSet {
            list,
            index: SubIndex(
                words.into_iter()
                    .map(WordMap::from)
                    .collect()
            ),
        }
    }
}