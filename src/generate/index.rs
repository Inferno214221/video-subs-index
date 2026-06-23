use std::{collections::BTreeMap, ffi::OsString, fs::{self, DirEntry, File, FileType}, path::PathBuf, sync::Arc };

use crate::{generate::subtitle::{Episode, Media, SubSet}, serve::files::{self, CONTENT_ROOT}};

#[derive(Debug, Clone)]
pub struct EpisodeData {
    pub episode: Arc<Episode>,
    pub subs: SubSet,
}

#[derive(Debug, Clone, Default)]
pub struct MediaIndex {
    pub episode_data: BTreeMap<Box<str>, BTreeMap<Box<str>, EpisodeData>>,
    pub media_data: BTreeMap<Box<str>, Arc<Media>>,
}

impl MediaIndex {
    pub fn build_from_fs() -> MediaIndex {
        let media_entries: Vec<_> = fs::read_dir(CONTENT_ROOT)
            .expect("unable to read content root")
            .try_collect()
            .expect("unable to read contents of content root");

        let mut media_index = MediaIndex::default();

        for media_name in media_entries.iter().filter_map(MediaIndex::get_name_if_dir) {
            media_index.read_media_dir(media_name);
        }

        media_index
    }

    fn get_name_if_dir(entry: &DirEntry) -> Option<Box<str>> {
        if entry.file_type().as_ref().is_ok_and(FileType::is_dir) {
            entry.file_name().into_boxed_str()
        } else {
            None
        }
    }

    fn read_media_dir(&mut self, media_name: Box<str>) {
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

        let episode_index = episode_entries.iter()
            .filter_map(MediaIndex::get_name_if_dir)
            .map(|episode_name| MediaIndex::read_episode_dir(
                episode_name,
                &media_name,
                &media_path,
                &media_metadata
            )).collect();

        self.episode_data.insert(media_name.clone(), episode_index);
        self.media_data.insert(media_name, media_metadata);
    }

    fn read_episode_dir(
        episode_name: Box<str>,
        media_name: &str,
        media_path: &PathBuf,
        media_metadata: &Arc<Media>
    ) -> (Box<str>, EpisodeData) {
        let episode_path = files::episode_dir_from_media(media_path, &episode_name);
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
                files::episode_subtitles(media_name, &episode_name),
                episode_meta.clone()
            );

            (episode_name, EpisodeData {
                episode: episode_meta,
                subs: sub_index
            })
        } else {
            panic!("missing required files for episode")
        }
    }

    pub fn get_media(&self, media: &str) -> &Media {
        self.media_data.get(media).unwrap()
    }

    pub fn get_episode(&self, media: &str, episode: &str) -> &Episode {
        &self.episode_data.get(media)
            .unwrap()
            .get(episode)
            .unwrap()
            .episode
    }

    pub fn get_episodes<'a>(&'a self, media: &str) -> Option<BTreeMap<&'a str, &'a Arc<Episode>>> {
        Some(
            self.episode_data.get(media)?
                .iter()
                .map(|(key, data)| (&**key, &data.episode))
                .collect()
        )
    }

    pub fn get_subs(&self, media: &str, episode: &str) -> &SubSet {
        &self.episode_data.get(media)
            .unwrap()
            .get(episode)
            .unwrap()
            .subs
    }
}

pub trait OsStringExt {
    fn into_boxed_str(self) -> Option<Box<str>>;
}

impl OsStringExt for OsString {
    fn into_boxed_str(self) -> Option<Box<str>> {
        // I don't think this can be done without a clone. into_boxed_os_str can't be mapped without
        // returning a sized value.
        Some(Box::clone_from_ref(self.to_str()?))
    }
}