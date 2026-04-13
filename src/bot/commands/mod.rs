pub mod cw;
pub mod cw_lesson;
pub mod cw_lesson_long;
pub mod neko;
pub mod vc;

use serenity::{all::PartialChannel, model::application::ResolvedValue};

pub fn get_value_f64(v: &ResolvedValue) -> anyhow::Result<f64> {
    if let ResolvedValue::Number(n) = v {
        Ok(*n)
    } else {
        anyhow::bail!("type error. got {:?}", v)
    }
}

pub fn get_value_i64(v: &ResolvedValue) -> anyhow::Result<i64> {
    if let ResolvedValue::Integer(n) = v {
        Ok(*n)
    } else {
        anyhow::bail!("type error. got {:?}", v)
    }
}

pub fn get_value_str<'a>(v: &ResolvedValue<'a>) -> anyhow::Result<&'a str> {
    if let ResolvedValue::String(s) = v {
        Ok(s)
    } else {
        anyhow::bail!("type error. got {:?}", v)
    }
}

pub fn get_value_channel<'a>(v: &ResolvedValue<'a>) -> anyhow::Result<&'a PartialChannel> {
    if let ResolvedValue::Channel(ch) = v {
        Ok(ch)
    } else {
        anyhow::bail!("type error. got {:?}", v)
    }
}
