use std::path::{Path, PathBuf};

use macron_path::path;
use rocket::fs::relative;

use crate::serve::ArtifactRange;

pub const CONTENT_ROOT: &str = relative!("content");

pub fn media_dir(media: impl AsRef<str>) -> PathBuf {
    path!(
        "{}/{}",
        CONTENT_ROOT,
        media.as_ref()
    )
}

pub fn media_metadata(media: impl AsRef<str>) -> PathBuf {
    path!(
        "{}/{}/metadata.yaml",
        CONTENT_ROOT,
        media.as_ref()
    )
}

pub fn metadata_from_media(media: impl AsRef<Path>) -> PathBuf {
    media.as_ref().join("metadata.yaml")
}

pub fn episode_dir(media: impl AsRef<str>, episode: impl AsRef<str>) -> PathBuf {
    path!(
        "{}/{}/{}",
        CONTENT_ROOT,
        media.as_ref(),
        episode.as_ref()
    )
}

pub fn episode_dir_from_media(media: impl AsRef<Path>, episode: impl AsRef<str>) -> PathBuf {
    media.as_ref().join(episode.as_ref())
}

pub fn episode_subtitles(media: impl AsRef<str>, episode: impl AsRef<str>) -> PathBuf {
    path!(
        "{}/{}/{}/subs.srt",
        CONTENT_ROOT,
        media.as_ref(),
        episode.as_ref()
    )
}

pub fn subtitles_from_episode(episode: impl AsRef<Path>) -> PathBuf {
    episode.as_ref().join("subs.srt")
}

pub fn episode_video(media: impl AsRef<str>, episode: impl AsRef<str>) -> PathBuf {
    path!(
        "{}/{}/{}/video.mkv",
        CONTENT_ROOT,
        media.as_ref(),
        episode.as_ref()
    )
}

pub fn video_from_episode(episode: impl AsRef<Path>) -> PathBuf {
    episode.as_ref().join("video.mkv")
}

pub fn episode_metadata(media: impl AsRef<str>) -> PathBuf {
    path!(
        "{}/{}/metadata.yaml",
        CONTENT_ROOT,
        media.as_ref()
    )
}

pub fn metadata_from_episode(media: impl AsRef<Path>) -> PathBuf {
    media.as_ref().join("metadata.yaml")
}

pub fn artifact(
    media: impl AsRef<str>,
    episode: impl AsRef<str>,
    artifact: ArtifactRange
) -> PathBuf {
    path!(
        "{}/{}/{}/{}.gif",
        CONTENT_ROOT,
        media.as_ref(),
        episode.as_ref(),
        artifact
    )
}