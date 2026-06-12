use std::{path::Path, process::Command, range::RangeInclusive, time::Duration};

use ffmpeg_light::Time;
use srt_subtitles_parser::{Subtitle, Timestamp};

use crate::serve::files;

pub fn convert_timestamp_with_offset(ts: &Timestamp, millis: i64) -> Time {
    Duration::from_millis(
        ts.to_ms().checked_add_signed(millis).unwrap()
    ).into()
}

pub fn slice_content(
    media: impl AsRef<str>,
    episode: impl AsRef<str>,
    artifact: usize,
    sub: &Subtitle
) {
    slice_video(
        files::episode_video(&media, &episode),
        files::artifact(media, episode, artifact),
        sub
    );
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
    // -ss and -to before -i slices the input before decoding. This is a requirement when applying
    // palette generation, so that we only get the colors we need. It also offers a HUGE speedup.
    Command::new("ffmpeg")
        // Input start time
        .arg("-ss").arg(convert_timestamp_with_offset(&first.start, 0).to_string())
        // Input end time
        .arg("-to").arg(convert_timestamp_with_offset(&last.end, 0).to_string())
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