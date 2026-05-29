use std::{collections::BTreeMap, fs, };

use derive_more::{Deref, DerefMut};
use macron_path::path;

use crate::{generate::subtitle::SubData, serve::util::CONTENT_ROOT};

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