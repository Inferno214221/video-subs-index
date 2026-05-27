use std::{collections::BTreeMap, fs, iter, path::Path, range::RangeInclusive, sync::Arc, time::Duration};

use ct_regex::{AnonRegex, regex};
use derive_more::{Deref, DerefMut};
use ffmpeg_light::{Time, TranscodeBuilder};
use macron_path::path;
use srt_subtitles_parser::{self as srt, Subtitle, Timestamp};

use crate::CONTENT_ROOT;

pub fn normalize_sub(text: &str) -> String {
    let mut text = text.to_lowercase();
    regex!(r"\s+").replace_all(&mut text, " ");
    regex!(r"[^a-z \-]").replace_all(&mut text, "");
    text
}

pub fn collect_words(text: &str) -> Vec<String> {
    text.split(' ')
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>()
}

#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct MediaIndex(pub BTreeMap<String, BTreeMap<String, SubData>>);

impl MediaIndex {
    pub fn build_from_fs() -> MediaIndex {
        let media_entries: Vec<_> = fs::read_dir(CONTENT_ROOT)
            .expect("unable to read content root")
            .try_collect()
            .expect("unable to read contents of content root");

        let mut media_index = MediaIndex(BTreeMap::new());

        for media in media_entries {
            let Ok(media_name) = media.file_name().into_string() else {
                panic!("invalid media name")
            };
            if !media.file_type().unwrap().is_dir() {
                panic!("content root contains non-dir")
            }

            let media_path = path!("{CONTENT_ROOT}/{media_name}");

            let episode_entries: Vec<_> = fs::read_dir(&media_path)
                .expect("unable to read media dir")
                .try_collect()
                .expect("unable to read contents of media dir");

            let episode_dirs = episode_entries.iter()
                .filter_map(
                    |entry| entry.file_type().ok().and_then(
                        |file_type| if file_type.is_dir() {
                            entry.file_name().into_string().ok()
                        } else {
                            None
                        }
                    )
                );

            let episode_index = episode_dirs.map(|episode| {
                let episode_path = media_path.join(&episode);
                if let (Ok(true), Ok(true)) = (
                    fs::exists(episode_path.with_extension("mkv")),
                    fs::exists(episode_path.with_extension("srt"))
                ) {
                    let sub_index = SubData::from_file(
                        path!("{CONTENT_ROOT}/{media_name}/{episode}.srt")
                    );
                    (episode, sub_index)
                } else {
                    panic!("missing required files for episode")
                }
            }).collect();

            media_index.insert(media_name.clone(), episode_index);
        }

        media_index
    }
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct WordMetadata(pub BTreeMap<usize, Arc<Subtitle>>);

#[derive(Debug, Clone)]
pub struct WordMap {
    pub word: String,
    pub metadata: WordMetadata,
}

impl From<(String, WordMetadata)> for WordMap {
    fn from((word, metadata): (String, WordMetadata)) -> Self {
        WordMap { word, metadata }
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct SubIndex(pub Vec<WordMap>);

impl SubIndex {
    pub fn binary_search(&self, word: &str) -> Option<usize> {
        self.binary_search_by_key(&word, |map| map.word.as_str()).ok()
    }

    pub fn find_next_word(&self, index: usize, word: &str) -> Option<usize> {
        self[index].metadata.keys()
            .find(|&&key| self[key].word == word)
            .copied()
    }

    pub fn search_subs<'w, 'i>(
        &'w self,
        search: impl Iterator<Item = &'i str>
    ) -> SubList {
        let mut search = search.peekable();
        let Some(first) = search.next() else {
            return Vec::new();
        };
        let Some(mut index) = self.binary_search(first) else {
            return Vec::new();
        };

        let mut collected_subs: Vec<_> = self[index].metadata.values().collect();

        for word in search {
            // dbg!(&word);
            index = match self.find_next_word(index, word) {
                Some(i) => i,
                None => return Vec::new(),
            };

            let new_subs: Vec<_> = self[index].metadata.values().collect();

            // dbg!(&new_subs.iter().map(|i| &i.text).collect::<Vec<_>>());

            collected_subs.retain(|sub| new_subs.contains(sub));
        }

        collected_subs.into_iter()
            .cloned()
            .collect()
    }

    pub fn search_with_query(&self, query: &str) -> SubList {
        let query_words = collect_words(&normalize_sub(query));
        self.search_subs(query_words.iter().map(String::as_str))
    }
}

pub type SubList = Vec<Arc<Subtitle>>;

#[derive(Debug, Clone)]
pub struct SubData {
    pub list: SubList,
    pub index: SubIndex,
}

impl SubData {
    pub fn from_file(sub_path: impl AsRef<Path>) -> SubData {
        let list = srt::parse_srt(&fs::read_to_string(sub_path).unwrap()).unwrap()
            .subtitles
            .into_iter()
            .map(Arc::new)
            .collect::<Vec<_>>();

        let text = list.iter().map(|sub| (
            sub,
            normalize_sub(&sub.text)
        ));

        let lines = text.map(|(sub, text)| (
            sub,
            collect_words(&text)
        )).collect::<Vec<_>>();

        let mut words = SubIndex(
            iter::once(("".to_owned(), WordMetadata::default()))
                .chain(
                    lines.iter()
                        .flat_map(|(_, line)| {
                            line.iter()
                                .cloned()
                                .zip(iter::repeat(WordMetadata::default()))
                        })
                        .collect::<BTreeMap<String, WordMetadata>>()
                ).map(WordMap::from)
                .collect::<Vec<_>>()
        );

        for (sub, line) in lines.iter() {
            println!("{:?}", line);
            line.iter()
                .map(|word| words.binary_search(word).unwrap())
                .collect::<Vec<_>>()
                .into_iter()
                .chain(iter::once(0))
                .map_windows(|&[a, b]| words[a].metadata.insert(b, (*sub).clone()))
                .last();
        }

        SubData {
            list,
            index: words,
        }
    }
}

pub fn convert_timestamp_with_offset(ts: &Timestamp, millis: i64) -> Time {
    Duration::from_millis(
        ts.to_ms().checked_add_signed(millis).unwrap()
    ).into()
}

pub fn slice_video(source: impl AsRef<Path>, target: impl AsRef<Path>, sub: &Subtitle) {
    slice_video_range(source, target, (sub..=sub).into());
}

pub fn slice_video_range(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    sub_range: RangeInclusive<&Subtitle>
) {
    let RangeInclusive { start: first, last } = sub_range;
    TranscodeBuilder::new()
        .input(source)
        .output(target)
        .overwrite(true)
        .extra_arg("-ss")
        .extra_arg(format!("{}", convert_timestamp_with_offset(&first.start, 0)))
        .extra_arg("-to")
        .extra_arg(format!("{}", convert_timestamp_with_offset(&last.end, -0)))
        .run()
        .unwrap();
}