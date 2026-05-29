use std::{path::Path, range::RangeInclusive, time::Duration};

use ffmpeg_light::{Time, TranscodeBuilder};
use srt_subtitles_parser::{Subtitle, Timestamp};

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