use std::{collections::BTreeMap, fs::{self, File}, sync::Arc, };

use crate::{generate::subtitle::{Episode, Media, SubSet}, serve::files::{self, CONTENT_ROOT}};

#[derive(Debug, Clone, Default)]
pub struct MediaIndex {
    pub episode_data: BTreeMap<String, BTreeMap<String, (Arc<Episode>, SubSet)>>,
    pub media_data: BTreeMap<String, Arc<Media>>,
}

impl MediaIndex {
    pub fn build_from_fs() -> MediaIndex {
        let media_entries: Vec<_> = fs::read_dir(CONTENT_ROOT)
            .expect("unable to read content root")
            .try_collect()
            .expect("unable to read contents of content root");

        let mut media_index = MediaIndex::default();

        for media in media_entries {
            let Ok(media_name) = media.file_name().into_string() else {
                panic!("invalid media name")
            };
            if !media.file_type().unwrap().is_dir() {
                continue;
            }

            let media_path = files::media_dir(&media_name);

            let media_metadata: Arc<Media> = Arc::new(
                serde_saphyr::from_reader(
                    File::open(files::metadata_from_media(&media_path))
                        .expect("unable to open media metadata")
                ).expect("unable to parse media metadata")
            );

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
                let episode_path = files::episode_dir_from_media(&media_path, &episode);
                if let (Ok(true), Ok(true)) = (
                    fs::exists(files::video_from_episode(&episode_path)),
                    fs::exists(files::subtitles_from_episode(&episode_path))
                ) {
                    let episode_meta = Arc::new(Episode {
                        inner: serde_saphyr::from_reader(
                            File::open(files::metadata_from_episode(&episode_path))
                                .expect("unable to open episode metadata")
                        ).expect("unable to parse episode metadata"),
                        media: media_metadata.clone(),
                    });

                    let sub_index = SubSet::parse_file(
                        files::episode_subtitles(&media_name, &episode),
                        episode_meta.clone()
                    );

                    (episode, (episode_meta, sub_index))
                } else {
                    panic!("missing required files for episode")
                }
            }).collect();

            media_index.episode_data.insert(media_name.clone(), episode_index);
            media_index.media_data.insert(media_name, media_metadata);
        }

        media_index
    }

    pub fn get_media(&self, media: &str) -> &Media {
        self.media_data.get(media).unwrap()
    }

    pub fn get_episode(&self, media: &str, episode: &str) -> &Episode {
        &self.episode_data.get(media)
            .unwrap()
            .get(episode)
            .unwrap().0
    }

    pub fn get_episodes<'a>(&'a self, media: &str) -> Option<BTreeMap<&'a String, &'a Arc<Episode>>> {
        Some(
            self.episode_data.get(media)?
                .iter()
                .map(|(key, (episode, _))| (key, episode))
                .collect()
        )
    }

    pub fn get_sub_set(&self, media: &str, episode: &str) -> &SubSet {
        &self.episode_data.get(media)
            .unwrap()
            .get(episode)
            .unwrap().1
    }
}