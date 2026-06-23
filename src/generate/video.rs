use std::{path::Path, process::Command, time::Duration};

use ffmpeg_light::Time;
use srt_subtitles_parser as srt;

use crate::serve::{ArtifactRange, files};

const MAX_DURATION: Duration = Duration::from_secs(30);

pub struct VidRange {
    start: Time,
    end: Time,
}

impl VidRange {
    pub fn new(first: &srt::Subtitle, last: &srt::Subtitle) -> Option<VidRange> {
        let start = convert_timestamp_with_offset(&first.start, 0);
        let end = convert_timestamp_with_offset(&last.end, 0);

        if end.checked_sub(start)? <= MAX_DURATION {
            Some(VidRange {
                start: start.into(),
                end: end.into()
            })
        } else {
            None
        }
    }
}

pub fn convert_timestamp_with_offset(ts: &srt::Timestamp, millis: i64) -> Duration {
    Duration::from_millis(
        ts.to_ms().checked_add_signed(millis).unwrap()
    )
}

pub fn slice_content(
    media: impl AsRef<str>,
    episode: impl AsRef<str>,
    artifact: ArtifactRange,
    range: VidRange,
) {
    slice_video(
        files::episode_video(&media, &episode),
        files::artifact(media, episode, artifact),
        range
    );
}

pub fn slice_video(
    source: impl AsRef<Path>,
    target: impl AsRef<Path>,
    range: VidRange,
) {
    // -ss and -to before -i slices the input before decoding. This is a requirement when applying
    // palette generation, so that we only get the colors we need. It also offers a HUGE speedup.
    Command::new("ffmpeg")
        // Input start time
        .arg("-ss").arg(range.start.to_string())
        // Input end time
        .arg("-to").arg(range.end.to_string())
        // Set input source
        .arg("-i").arg(source.as_ref())
        // Generate and use palette
        .arg("-vf").arg("split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse")
        // Output to target
        .arg(target.as_ref())
        // Overwrite existing files
        .arg("-y")
        .spawn().expect("failed to spawn ffmpeg")
        .wait().expect("failed to wait for ffmpeg to finish")
        .exit_ok().expect("ffmepg task failed");

    // extra_arg are all appended at the end, which won't work for input slicing
    // TranscodeBuilder::new()
    //     .extra_arg("-ss")
    //     .extra_arg(format!("{}", convert_timestamp_with_offset(&first.start, 0)))
    //     .extra_arg("-to")
    //     .extra_arg(format!("{}", convert_timestamp_with_offset(&last.end, -0)))
    //     .input(source)
    //     .output(target)
    //     .overwrite(true)
    //     .run()
    //     .unwrap();
}