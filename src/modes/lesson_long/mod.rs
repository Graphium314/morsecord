use crate::cw_audio::CWAudioPCM;
use anyhow::{Context as _, Ok};
use kanaria::string::UCSStr;
use rand::Rng;
use serenity::model::channel::Message;
use serenity::model::prelude::GuildId;
use serenity::prelude::Context;
use songbird::constants::SAMPLE_RATE_RAW;
use unicode_normalization::UnicodeNormalization as _;
use std::sync::{Arc, Mutex};
mod levenshtein;

pub struct LessonLongModeState {
    text: String,
    speed: f32,
    freq: f32,
}

pub fn get_lesson_str(probset: &str, length: usize) -> anyhow::Result<String> {
    // TODO: use braces to support nesting
    let (probset_name, probset_args_str) = probset.split_once(':').unwrap_or((probset, ""));

    use crate::modes::lesson_long;

    let prob = match probset_name {
        "alpha" => lesson_long::prob_alpha(length)?,
        "koch" | "lcwo" => {
            let args = probset_args_str.trim();
            lesson_long::prob_koch_method(length, args)?
        }
        _ => {
            anyhow::bail!(
                "unknown probset.\n".to_owned()
                    + "available selections are: alpha, koch"
            )
        }
    };
    Ok(prob)
}

impl LessonLongModeState {
    pub fn new(
        probset: &str,
        length: usize,
        speed: f32,
        freq: f32,
    ) -> anyhow::Result<Self> {
        let text = get_lesson_str(probset, length).context("failed to get lesson text")?;

        Ok(Self {
            text,
            speed,
            freq,
        })
    }
}

pub async fn start(
    ctx: &Context,
    guild: GuildId,
    state: Arc<Mutex<LessonLongModeState>>,
) -> anyhow::Result<()> {
    let man = songbird::get(ctx).await.expect("init songbird").clone();
    let call = man.get(guild).context("not in call")?;
    let (text, speed, freq) = {
        let st = state.lock().unwrap();
        (st.text.clone(), st.speed, st.freq)
    };
    let source = CWAudioPCM::new(" ".to_string() + &text, speed, freq, SAMPLE_RATE_RAW).to_input();
    let mut handler = call.lock().await;
    handler.play_only_source(source);
    Ok(())
}

pub fn end(_state: Arc<Mutex<LessonLongModeState>>) -> anyhow::Result<String> {
    Ok("Long lesson ended".to_string())
}

fn normalize_text(s: &str) -> String {
    // let s = s.replace(" ", "");
    // let s = s.replace("　", "");
    let s = s.replace(|c: char| c.is_whitespace(), "");
    let s = UCSStr::from_str(&s).upper_case().katakana().to_string();
    s.nfkd().collect::<String>()
}

pub async fn on_message(
    ctx: &Context,
    msg: &Message,
    state: Arc<Mutex<LessonLongModeState>>,
) -> anyhow::Result<()> {
    let text = {
        let st = state.lock().unwrap();
        st.text.clone()
    };
    let input = msg.content.clone();

    // normalize
    let text = normalize_text(&text);
    let input = normalize_text(&input);

    let (m_input, m_text, dist) = levenshtein::compare_str(&input, &text);
    let m_diff = m_input.chars().zip(m_text.chars())
        .map(|(a, b)| if a == b { ' ' } else { '*' })
        .collect::<String>();
    let reply = format!(
        "Difference: {dist}\n```diff\n-{m_input}\n+{m_text}\n {m_diff}\n```",
    );
    // msg.channel_id.say(&ctx.http, reply).await?;
    msg.reply_ping(&ctx.http, reply).await?;
    Ok(())
}

pub fn prob_alpha(length: usize) -> anyhow::Result<String> {

    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(length*3/2);
    for i in 0..length {
        let c = rng.gen_range(b'A'..=b'Z') as char;
        s.push(c);

        if i % 5 == 4 {
            s.push(' ');
        }
    }
    Ok(s)
}


#[rustfmt::skip]
const KOCH_CHARS: &[char] = &[
    'K', 'M', 'U', 'R', 'E', 'S', 'N', 'A', 'P', 'T',
    'L', 'W', 'I', /*'.',*/ 'J', 'Z', /*'=',*/ 'F', 'O', 'Y',
    /*',',*/ 'V', 'G', '5', '/', 'Q', '9', '2', 'H', '3',
    '8', 'B', /*'?',*/ '4', '7', 'C', '1', 'D', '6', '0',
    'X',
];

pub fn prob_koch_method(length: usize, args: &str) -> anyhow::Result<String> {
    let level = args
        .parse::<usize>()
        .context("invalid level argument")?;

    if !(2..(KOCH_CHARS.len())).contains(&(level + 1)) {
        anyhow::bail!("level must be in range 1..{}", KOCH_CHARS.len() - 1);
    }

    let allowed_chars = &KOCH_CHARS[0..level + 1];

    let mut rng = rand::thread_rng();
    let mut s = String::with_capacity(length*3/2);
    for i in 0..length {
        let c = allowed_chars[rng.gen_range(0..allowed_chars.len()) as usize];
        s.push(c);
        if i % 5 == 4 {
            s.push(' ');
        }
    }
    Ok(s)
}
