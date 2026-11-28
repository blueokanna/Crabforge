use super::{
    super::video::{Input, RATIOS, nearest, size_pair},
    base,
};
use crate::config::Endpoints;
use anyhow::{Result, bail, ensure};
use nextjson::{Value, json};
use url::Url;

pub(crate) fn endpoint(endpoints: &Endpoints, model: &str) -> Option<Url> {
    let model = model.to_ascii_lowercase();
    if !model.starts_with("doubao-seedance-") && !model.starts_with("seedance-") {
        return None;
    }
    let mut url = base(endpoints)?;
    if url.path() == "/" {
        url.set_path("/api/v3");
    }
    url.path_segments_mut()
        .ok()?
        .pop_if_empty()
        .extend(["contents", "generations", "tasks"]);
    Some(url)
}

pub(in crate::media) fn request(model: &str, input: Input) -> Result<Value> {
    let Input {
        prompt,
        size,
        seconds,
        frames,
        mut fields,
    } = input;
    ensure!(frames.len() <= 15, "too many video reference images");
    let mut content = match fields.remove("content") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(content)) => content,
        _ => bail!("Ark video content must be an array"),
    };
    if !prompt.trim().is_empty()
        && !content
            .iter()
            .any(|item| item.get("type").and_then(Value::as_str) == Some("text"))
    {
        content.insert(0, json!({"type":"text", "text":prompt}));
    }
    content.extend(frames.iter().map(|frame| frame.content()));
    ensure!(
        !content.is_empty(),
        "video requires a prompt, a reference image or native content"
    );
    if let Some(duration) = seconds {
        ensure!(
            duration == -1 || duration > 0,
            "seconds must be positive or -1"
        );
        if fields.get("duration").is_none() {
            fields.insert("duration".into(), json!(duration));
        }
    }
    let (resolution, ratio) = dimensions(&size);
    for (field, value) in [("resolution", resolution), ("ratio", ratio)] {
        if let Some(value) = value.filter(|_| fields.get(field).is_none()) {
            fields.insert(field.into(), json!(value));
        }
    }
    fields.insert("model".into(), json!(model));
    fields.insert("content".into(), json!(content));
    Ok(Value::Object(fields))
}

fn dimensions(size: &str) -> (Option<&'static str>, Option<&'static str>) {
    const RESOLUTIONS: [(&str, f64); 4] = [
        ("480p", 414720.0),
        ("720p", 921600.0),
        ("1080p", 2073600.0),
        ("4k", 8294400.0),
    ];
    let size = size.trim().to_ascii_lowercase();
    if let Some((name, _)) = RESOLUTIONS
        .iter()
        .find(|(name, _)| *name == size || (*name == "4k" && size == "2160p"))
    {
        return (Some(*name), None);
    }
    let Some((width, height)) = size_pair(&size, &['x', ':']) else {
        return (None, None);
    };
    let resolution = size
        .contains('x')
        .then(|| nearest(&RESOLUTIONS, width * height))
        .flatten();
    (resolution, nearest(&RATIOS, width / height))
}
