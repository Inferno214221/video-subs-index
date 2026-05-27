use std::{collections::BTreeMap, env, fs, iter, path::Path, process::{Command, Stdio}, rc::Rc, time::Duration};

use ct_regex::{AnonRegex, regex};
use ffmpeg_light::{Time, TranscodeBuilder};
use srt_subtitles_parser::{self as srt, Subtitle, Timestamp};

type SubIndex = Vec<(String, BTreeMap<usize, Rc<Subtitle>>)>;

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

pub fn index_words(sub_path: impl AsRef<Path>) -> SubIndex {
    let subs = srt::parse_srt(&fs::read_to_string(sub_path).unwrap()).unwrap()
        .subtitles
        .into_iter()
        .map(Rc::new)
        .collect::<Vec<_>>();

    let text = subs.iter().map(|sub| (
        sub,
        normalize_sub(&sub.text)
    ));

    let lines = text.map(|(sub, text)| (
        sub,
        collect_words(&text)
    )).collect::<Vec<_>>();

    let mut words = iter::once(("".to_owned(), BTreeMap::new())).chain(
        lines.iter()
            .flat_map(|(_, line)| {
                line.iter()
                    .cloned()
                    .zip(iter::repeat(BTreeMap::new()))
            })
            .collect::<BTreeMap<String, BTreeMap<usize, Rc<Subtitle>>>>()
    ).collect::<Vec<_>>();

    for (sub, line) in lines.iter() {
        println!("{:?}", line);
        line.iter()
            .map(|word| words.binary_search_by_key(&word, |(key, _)| key).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .chain(iter::once(0))
            .map_windows(|[a, b]| words[*a].1.insert(*b, (*sub).clone()))
            .last();
    }

    words
}

pub fn search_subs<'w, 'i>(
    words: &'w SubIndex,
    search: impl Iterator<Item = &'i str>
) -> Vec<Rc<Subtitle>> {
    let mut search = search.peekable();

    let mut index = words.binary_search_by_key(
        &search.next().unwrap(),
        |(key, _)| key
    ).unwrap();

    let mut collected_subs: Vec<_> = words[index].1.values().collect();

    for word in search {
        dbg!(&word);
        index = *words[index].1.keys().find(|&key| words[*key].0 == word).unwrap();
        let new_subs: Vec<_> = words[index].1.values().collect();

        dbg!(&new_subs.iter().map(|i| &i.text).collect::<Vec<_>>());

        collected_subs.retain(|sub| new_subs.contains(sub));
    }

    collected_subs.into_iter()
        .cloned()
        .collect()
}

pub fn convert_timestamp_with_offset(ts: &Timestamp, millis: i64) -> Time {
    Duration::from_millis(
        ts.to_ms().checked_add_signed(millis).unwrap()
    ).into()
}

pub fn slice_video(video_path: impl AsRef<Path>, target: &Subtitle) {
    TranscodeBuilder::new()
        .input(video_path)
        .output("./output.mkv")
        .overwrite(true)
        .extra_arg("-ss")
        .extra_arg(format!("{}", convert_timestamp_with_offset(&target.start, 0)))
        .extra_arg("-to")
        .extra_arg(format!("{}", convert_timestamp_with_offset(&target.end, -0)))
        .run()
        .unwrap();
}

fn _main() {
    let args: Vec<_> = env::args().skip(1).collect();

    // let target = subs.subtitles[1066].clone();
    // dbg!(&target);

    let words = index_words("./e01.srt");

    let collected_subs = search_subs(&words, args.iter().map(String::as_str));

    dbg!(&collected_subs);

    if let [target] = &*collected_subs {
        slice_video("./e01.mkv", target);

        let _ = Command::new("vlc")
            .arg("./output.mkv")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    } else {
        println!("ambiguous search results!")
    }
}