#![feature(iter_map_windows)]

use std::{collections::BTreeMap, env, fs, iter, process::{Command, Stdio}, rc::Rc, time::Duration};

use ffmpeg_light::{Time, TranscodeBuilder};
use srt_subtitles_parser::{self as srt, Subtitle, Timestamp};

pub fn start_time(ts: &Timestamp) -> Time {
    Duration::from_millis(ts.to_ms()).into()
}

pub fn end_time(ts: &Timestamp) -> Time {
    Duration::from_millis(ts.to_ms()).into()
}

fn main() {
    let args = env::args().skip(1);

    let subs = srt::parse_srt(
        &fs::read_to_string("./e01.srt").unwrap()
    ).unwrap()
    .subtitles
    .into_iter()
    .map(Rc::new)
    .collect::<Vec<_>>();

    // let target = subs.subtitles[1066].clone();
    // dbg!(&target);

    let text = subs.iter().map(|sub| (
        sub,
        sub.text.replace(['\'', '.', ',', '!', '?', '\"', '~'], "")
            .replace('\n', " ")
            .replace("  ", " ")
            .to_lowercase()
    ));

    let lines = text.map(|(sub, text)| (
        sub,
        text.split(' ')
            .filter(|word| !word.is_empty())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    )).collect::<Vec<_>>();

    let mut all_words = iter::once(("".to_owned(), BTreeMap::new())).chain(
        lines.iter()
            .flat_map(|(_, line)| {
                line.iter()
                    .cloned()
                    .zip(iter::repeat(BTreeMap::new()))
            })
            .collect::<BTreeMap<String, BTreeMap<usize, Rc<Subtitle>>>>()
    ).collect::<Vec<_>>();

    for (sub, line) in lines.iter() {
        line.iter()
            .map(|word| all_words.binary_search_by_key(&word, |(key, _)| key).unwrap())
            .collect::<Vec<_>>()
            .into_iter()
            .chain(iter::once(0))
            .map_windows(|[a, b]| all_words[*a].1.insert(*b, (*sub).clone()))
            .last();
    };

    let mut search = args.peekable();

    let mut index = all_words.binary_search_by_key(
        &&search.next().unwrap(),
        |(key, _)| key
    ).unwrap();

    let mut collected_subs = all_words[index].1.values().collect::<Vec<_>>();

    for word in search {
        index = *all_words[index].1.keys().find(|&key| all_words[*key].0 == word).unwrap();
        let new_subs = all_words[index].1.values().collect::<Vec<_>>();
        // dbg!(&new_subs.iter().map(|i| &i.text).collect::<Vec<_>>());
        // if !new_subs.is_empty() {
        //     collected_subs.retain(|sub| new_subs.contains(sub));
        // } else {
        //     break;
        // }
        collected_subs.retain(|sub| new_subs.contains(sub));
    }

    dbg!(&collected_subs);

    if let [target] = *collected_subs {
        TranscodeBuilder::new()
            .input("./e01.mkv")
            .output("./output.mkv")
            .overwrite(true)
            .extra_arg("-ss")
            .extra_arg(format!("{}", start_time(&target.start)))
            .extra_arg("-to")
            .extra_arg(format!("{}", end_time(&target.end)))
            .run()
            .unwrap();

        let _ = Command::new("vlc").arg("./output.mkv")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    } else {
        println!("ambiguous search results!")
    }

    // let status = Command::new("ffmpeg")
    //     .args([
    //         "-i", "./e01.mkv",
    //         "-ss", &format!("{}", sub_time_to_vid_time(&target.start)),
    //         "-to", &format!("{}", sub_time_to_vid_time(&target.end)),
    //         "output.mkv"
    //     ]).status();

    // dbg!(status);
}