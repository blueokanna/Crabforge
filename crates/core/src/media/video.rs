use super::{Upload, ark, asset_url, dashscope, image_reference};
use crate::{
    catalog::RelayEndpoint,
    config::{Endpoints, Provider, api_base, endpoint},
};
use anyhow::{Context, Result, bail, ensure};
use bytes::Bytes;
use http::StatusCode;
use nextjson::{Map, Value, json};
use url::Url;

const XAI: &str = "/v1/videos/generations";
const MINIMAX: &str = "/v2/video_generation";
const DASHSCOPE: &str = "/api/v1/services/aigc/video-generation/video-synthesis";
const MINIMAX_MODELS: &[&str] = &[
    "MiniMax-H3",
    "MiniMax-H3-Highspeed",
    "MiniMax-H3-Lite",
    "MiniMax-H3-Max",
];
const DASHSCOPE_MODELS: &[&str] = &[
    "wan3.0-video",
    "happyhorse-1.1-t2v",
    "happyhorse-1.1-i2v",
    "happyhorse-1.1-r2v",
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    OpenAi,
    Ark,
    Xai,
    MiniMax,
    DashScope,
}

fn api_urls(bases: &Endpoints) -> impl Iterator<Item = Url> {
    [&bases.chat, &bases.responses, &bases.anthropic]
        .into_iter()
        .filter_map(|base| endpoint(base).ok())
}

fn minimax_host(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("api.minimax.io" | "api.minimax.cn" | "api.minimaxi.com")
    )
}

pub(crate) fn builtin_models(bases: &Endpoints) -> impl Iterator<Item = &'static str> {
    let mini = api_urls(bases).any(|url| minimax_host(&url));
    let dash = dashscope::base(bases).is_some();
    let xai = api_urls(bases).any(|url| url.host_str() == Some("api.x.ai"));
    [
        mini.then_some(MINIMAX_MODELS),
        dash.then_some(DASHSCOPE_MODELS),
        xai.then_some(["grok-imagine-video", "grok-imagine-video-1.5"].as_slice()),
    ]
    .into_iter()
    .flatten()
    .flatten()
    .copied()
}

pub(crate) fn native(bases: &Endpoints, model: &str) -> Option<(Kind, Url)> {
    if let Some(url) = ark::video::endpoint(bases, model) {
        return Some((Kind::Ark, url));
    }
    let (kind, mut url, path) = if model.starts_with("grok-imagine-video") {
        (
            Kind::Xai,
            api_urls(bases).find(|url| url.host_str() == Some("api.x.ai"))?,
            XAI,
        )
    } else if MINIMAX_MODELS.contains(&model) {
        (Kind::MiniMax, api_urls(bases).find(minimax_host)?, MINIMAX)
    } else if DASHSCOPE_MODELS.contains(&model) {
        (Kind::DashScope, dashscope::base(bases)?, DASHSCOPE)
    } else {
        return None;
    };
    url.set_path(path);
    Some((kind, url))
}

pub(crate) fn advertised(p: &Provider, model: &str) -> bool {
    p.model_protocols
        .get(model)
        .and_then(|hints| hints.native_endpoints.as_ref())
        .is_some_and(|endpoints| endpoints.contains(&RelayEndpoint::Videos))
}

pub(crate) fn only(p: &Provider, model: &str) -> bool {
    route(p, model).is_some_and(|(kind, url)| {
        kind != Kind::OpenAi
            || sora(&url, model)
            || p.model_protocols
                .get(model)
                .and_then(|hints| hints.native_endpoints.as_ref())
                .is_some_and(|endpoints| {
                    endpoints.len() == 1 && endpoints.contains(&RelayEndpoint::Videos)
                })
    })
}

fn sora(url: &Url, model: &str) -> bool {
    url.host_str() == Some("api.openai.com") && matches!(model, "sora-2" | "sora-2-pro")
}

pub(crate) fn route(p: &Provider, model: &str) -> Option<(Kind, Url)> {
    if p.web_site.is_some() || p.endpoints.vertex.is_some() {
        return None;
    }
    if !p.remote_gateway
        && let Some(native) = native(&p.endpoints, model)
    {
        return Some(native);
    }
    let explicit = advertised(p, model);
    let mut url = [&p.endpoints.chat, &p.endpoints.responses]
        .into_iter()
        .filter_map(|base| api_base(base, "openai").ok())
        .find(|url| explicit || sora(url, model))?;
    url.path_segments_mut().ok()?.pop_if_empty().push("videos");
    Some((Kind::OpenAi, url))
}

impl Kind {
    pub(crate) fn for_create_url(provider: &Provider, url: &Url) -> Self {
        if provider.remote_gateway {
            return Self::OpenAi;
        }
        match url.path() {
            XAI => Self::Xai,
            MINIMAX => Self::MiniMax,
            DASHSCOPE => Self::DashScope,
            path if path.ends_with("/contents/generations/tasks") => Self::Ark,
            _ => Self::OpenAi,
        }
    }

    pub(crate) fn task_url(self, mut url: Url, id: &str, content: bool) -> Url {
        match self {
            Self::Xai => url.set_path("/v1/videos"),
            Self::MiniMax => url.set_path("/v2/query/video_generation"),
            Self::DashScope => url.set_path("/api/v1/tasks"),
            _ => {}
        }
        let mut path = url.path_segments_mut().expect("validated video URL");
        path.push(id);
        if content && self == Self::OpenAi {
            path.push("content");
        }
        drop(path);
        url
    }

    pub(crate) fn request(
        self,
        model: &str,
        bytes: &[u8],
        upload: Option<&Upload>,
    ) -> Result<Option<Bytes>> {
        if self == Self::OpenAi {
            return Ok(None);
        }
        let input = Input::read(bytes, upload)?;
        let value = match self {
            Self::Ark => ark::video::request(model, input)?,
            Self::Xai => xai(model, input)?,
            Self::MiniMax => minimax(model, input)?,
            Self::DashScope => dashscope(model, input)?,
            Self::OpenAi => unreachable!(),
        };
        Ok(Some(value.to_string().into()))
    }

    pub(crate) fn created_id(self, object: &Map) -> Option<&str> {
        match self {
            Self::Xai => object.get("request_id"),
            Self::MiniMax => object.get("task_id"),
            Self::DashScope => object.get("output")?.get("task_id"),
            Self::Ark | Self::OpenAi => object.get("id"),
        }?
        .as_str()
    }

    pub(crate) fn business_status(self, object: &Map) -> Option<StatusCode> {
        let error = match self {
            Self::MiniMax => object.get("base_resp")?.get("status_code")?.as_i64()?,
            Self::DashScope => {
                return match object.get("code")?.as_str()? {
                    "" => None,
                    "Throttling.RateQuota" | "Throttling" | "TooManyRequests" => {
                        Some(StatusCode::TOO_MANY_REQUESTS)
                    }
                    "InvalidApiKey" => Some(StatusCode::UNAUTHORIZED),
                    "InvalidParameter" => Some(StatusCode::BAD_REQUEST),
                    _ => Some(StatusCode::BAD_GATEWAY),
                };
            }
            _ => return None,
        };
        match error {
            0 => None,
            1002 | 1045 => Some(StatusCode::TOO_MANY_REQUESTS),
            1004 | 2049 => Some(StatusCode::UNAUTHORIZED),
            1008 => Some(StatusCode::PAYMENT_REQUIRED),
            2057 | 2061 | 2068 => Some(StatusCode::FORBIDDEN),
            1026 | 1027 => Some(StatusCode::UNPROCESSABLE_ENTITY),
            _ => Some(StatusCode::BAD_GATEWAY),
        }
    }

    pub(crate) fn status(self, object: Map, id: &str, started: i64, created: bool) -> Option<Map> {
        if self == Self::OpenAi {
            return (created || object.get("id")?.as_str()? == id).then_some(object);
        }
        let native = Value::Object(object);
        let (task, status_field, id_field, seconds) = match self {
            Self::Ark => (&native, "status", "id", native.get("duration")),
            Self::Xai => (
                &native,
                "status",
                "request_id",
                native.pointer("/video/duration"),
            ),
            Self::MiniMax if created => (&native, "status", "task_id", None),
            Self::MiniMax => {
                let task = native.get("task")?;
                (task, "status", "id", task.get("duration"))
            }
            Self::DashScope => (
                native.get("output")?,
                "task_status",
                "task_id",
                native.pointer("/usage/duration"),
            ),
            Self::OpenAi => unreachable!(),
        };
        match task.get(id_field).or_else(|| task.get("id")) {
            Some(actual) if actual.as_str() != Some(id) => return None,
            None if self == Self::Ark => return None,
            _ => {}
        }
        let state = task.get(status_field).and_then(Value::as_str);
        let progress = task
            .get("progress")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .min(100);
        let (status, progress) = match state.unwrap_or_default().to_ascii_lowercase().as_str() {
            "completed" | "succeeded" | "done" | "success" => ("completed", 100),
            "running" | "in_progress" | "processing" => ("in_progress", progress),
            "failed" | "expired" | "cancelled" | "canceled" | "unknown" => ("failed", progress),
            "pending" | "queued" | "preparing" => ("queued", 0),
            "" if created => ("queued", 0),
            _ => return None,
        };
        let error = (status == "failed").then(|| {
            task.get("error").filter(|error| !error.is_null()).cloned().unwrap_or_else(|| {
                json!({"code":task.get("code").and_then(Value::as_str).or(state),
                    "message":task.get("message").and_then(Value::as_str).unwrap_or("Video task did not finish")})
            })
        });
        let seconds = seconds
            .and_then(Value::as_f64)
            .filter(|n| n.is_finite() && *n > 0.0);
        let Value::Object(mut object) = native else {
            unreachable!("video task metadata object")
        };
        if let Some(error) = error {
            object.insert("error".into(), error);
        }
        object.insert("status".into(), json!(status));
        object.insert("progress".into(), json!(progress));
        object.insert("object".into(), json!("video"));
        if object.get("created_at").and_then(Value::as_i64).is_none() {
            object.insert("created_at".into(), json!(started));
        }
        if let Some(seconds) = seconds {
            object.insert("seconds".into(), json!(seconds.to_string()));
        }
        Some(object)
    }

    pub(crate) fn content_url(self, source: &Url, object: Map) -> Option<Url> {
        let path = match self {
            Self::Ark => "/content/video_url",
            Self::Xai => "/video/url",
            Self::MiniMax => "/task/content/url",
            Self::DashScope => "/output/video_url",
            Self::OpenAi => return None,
        };
        let url = asset_url(Value::Object(object).pointer(path)?.as_str()?)?;
        if url.origin() == source.origin() {
            return Some(url);
        }
        if url.scheme() != "https" || url.port().is_some() {
            return None;
        }
        let host = url.host_str()?;
        let allowed = match self {
            Self::Ark => host
                .strip_suffix(".volces.com")
                .and_then(|name| name.rsplit('.').next())
                .is_some_and(|service| {
                    service.starts_with("tos-") && !service.contains("internal")
                }),
            Self::Xai => matches!(
                host,
                "vidgen.x.ai" | "imagine-public.x.ai" | "video-public.x.ai"
            ),
            Self::MiniMax => {
                matches!(
                    host,
                    "filecdn.minimax.chat" | "filecdn.minimaxi.com" | "cdn.hailuoai.video"
                ) || dashscope::public_oss(&url)
            }
            Self::DashScope => dashscope::public_oss(&url),
            Self::OpenAi => false,
        };
        allowed.then_some(url)
    }
}

const FRAMES: [(&str, &str); 4] = [
    ("input_reference", "first_frame"),
    ("last_frame", "last_frame"),
    ("reference_images", "reference_image"),
    ("reference_images[]", "reference_image"),
];

pub(super) struct Frame {
    role: &'static str,
    url: String,
}

impl Frame {
    pub(super) fn content(&self) -> Value {
        json!({"type":"image_url", "image_url":{"url":self.url}, "role":self.role})
    }
}

pub(super) struct Input {
    pub(super) prompt: String,
    pub(super) size: String,
    pub(super) seconds: Option<i64>,
    pub(super) frames: Vec<Frame>,
    pub(super) fields: Map,
}

impl Input {
    fn read(bytes: &[u8], upload: Option<&Upload>) -> Result<Self> {
        let mut frames = Vec::new();
        let mut fields = if let Some(upload) = upload {
            let mut fields = Map::new();
            for field in &upload.fields {
                if let Some(&(_, role)) = FRAMES.iter().find(|(name, _)| *name == field.name) {
                    frames.push(Frame {
                        role,
                        url: field.image_reference()?,
                    });
                    continue;
                }
                ensure!(
                    field.filename.is_none(),
                    "unsupported video file field {}",
                    field.name
                );
                let text = std::str::from_utf8(&field.value)?;
                let value = match field.name.as_str() {
                    "model"
                    | "prompt"
                    | "size"
                    | "ratio"
                    | "aspect_ratio"
                    | "resolution"
                    | "negative_prompt"
                    | "safety_identifier"
                    | "callback_url"
                    | "service_tier"
                    | "output_format"
                    | "omni_reference_task_type" => json!(text),
                    _ => nextjson::from_str(text).unwrap_or_else(|_| json!(text)),
                };
                ensure!(
                    fields.insert(field.name.clone(), value).is_none(),
                    "duplicate video field {}",
                    field.name
                );
            }
            fields
        } else {
            let Value::Object(mut fields) =
                nextjson::from_slice(bytes).map_err(anyhow::Error::msg)?
            else {
                bail!("expected a video request object");
            };
            for (name, role) in FRAMES {
                if let Some(value) = fields.remove(name).filter(|value| !value.is_null()) {
                    let images = value
                        .as_array()
                        .map(Vec::as_slice)
                        .unwrap_or(std::slice::from_ref(&value));
                    for image in images {
                        frames.push(Frame {
                            role,
                            url: image_reference(image)?.to_owned(),
                        });
                    }
                }
            }
            fields
        };
        for role in ["first_frame", "last_frame"] {
            ensure!(
                frames.iter().filter(|frame| frame.role == role).count() <= 1,
                "duplicate {role}"
            );
        }
        fields.remove("model");
        let prompt = take_text(&mut fields, "prompt")?;
        let size = take_text(&mut fields, "size")?;
        let seconds = fields
            .remove("seconds")
            .filter(|value| !value.is_null())
            .map(|value| {
                value
                    .as_i64()
                    .or_else(|| value.as_str()?.trim().parse().ok())
                    .context("expected whole seconds")
            })
            .transpose()?;
        Ok(Self {
            prompt,
            size,
            seconds,
            frames,
            fields,
        })
    }

    fn has_content(&self) -> bool {
        !self.prompt.trim().is_empty() || !self.frames.is_empty()
    }
}

fn take_text(fields: &mut Map, key: &str) -> Result<String> {
    match fields.remove(key) {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(value)) => Ok(value),
        _ => bail!("expected a string video {key}"),
    }
}

pub(super) const RATIOS: [(&str, f64); 6] = [
    ("21:9", 21.0 / 9.0),
    ("16:9", 16.0 / 9.0),
    ("4:3", 4.0 / 3.0),
    ("1:1", 1.0),
    ("3:4", 3.0 / 4.0),
    ("9:16", 9.0 / 16.0),
];
const XAI_RATIOS: [(&str, f64); 7] = [
    RATIOS[1],
    RATIOS[2],
    RATIOS[3],
    RATIOS[4],
    RATIOS[5],
    ("3:2", 1.5),
    ("2:3", 2.0 / 3.0),
];

pub(super) fn size_pair(size: &str, separator: &[char]) -> Option<(f64, f64)> {
    let (width, height) = size.trim().split_once(separator)?;
    let (width, height) = (width.parse::<f64>().ok()?, height.parse::<f64>().ok()?);
    (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0)
        .then_some((width, height))
}

pub(super) fn nearest(choices: &[(&'static str, f64)], value: f64) -> Option<&'static str> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let error = |target: f64| (value / target).ln().abs();
    choices
        .iter()
        .min_by(|a, b| error(a.1).total_cmp(&error(b.1)))
        .map(|(name, _)| *name)
}

fn ratio(size: &str, choices: &[(&'static str, f64)]) -> Option<&'static str> {
    let (width, height) = size_pair(size, &['x', 'X', ':'])?;
    nearest(choices, width / height)
}

fn pixel_resolution(size: &str) -> Option<&'static str> {
    let (width, height) = size_pair(size, &['x', 'X'])?;
    Some(match width.min(height) {
        pixels if pixels >= 1080.0 => "1080p",
        pixels if pixels >= 720.0 => "720p",
        _ => "480p",
    })
}

fn xai(model: &str, input: Input) -> Result<Value> {
    ensure!(
        input.has_content(),
        "video requires a prompt or reference image"
    );
    let Input {
        prompt,
        size,
        seconds,
        frames,
        mut fields,
    } = input;
    ensure!(
        !frames.iter().any(|frame| frame.role == "last_frame"),
        "xAI does not accept a last frame"
    );
    fields.insert("model".into(), json!(model));
    fields.insert("prompt".into(), json!(prompt));
    if let Some(seconds) = seconds {
        ensure!(
            (1..=i64::from(i32::MAX)).contains(&seconds),
            "seconds must be positive"
        );
        fields.insert("duration".into(), json!(seconds));
    }
    if let Some(ratio) = ratio(&size, &XAI_RATIOS).filter(|_| fields.get("aspect_ratio").is_none())
    {
        fields.insert("aspect_ratio".into(), json!(ratio));
    }
    if let Some(mut resolution) =
        pixel_resolution(&size).filter(|_| fields.get("resolution").is_none())
    {
        if model == "grok-imagine-video" && resolution == "1080p" {
            resolution = "720p";
        }
        fields.insert("resolution".into(), json!(resolution));
    }
    if let Some(frame) = frames.iter().find(|frame| frame.role == "first_frame") {
        fields.insert("image".into(), json!({"url":frame.url}));
    }
    let refs: Vec<_> = frames
        .iter()
        .filter(|frame| frame.role == "reference_image")
        .map(|frame| json!({"url":frame.url}))
        .collect();
    if !refs.is_empty() {
        fields.insert("reference_images".into(), json!(refs));
    }
    Ok(Value::Object(fields))
}

fn minimax(model: &str, input: Input) -> Result<Value> {
    ensure!(
        input.has_content(),
        "video requires a prompt or reference image"
    );
    let Input {
        prompt,
        size,
        seconds,
        frames,
        mut fields,
    } = input;
    let refs = frames
        .iter()
        .filter(|frame| frame.role == "reference_image")
        .count();
    let has_frame = refs != frames.len();
    ensure!(
        !has_frame || refs == 0,
        "MiniMax accepts frames or reference images separately"
    );
    ensure!(refs <= 9, "MiniMax accepts up to nine reference images");
    let duration = seconds
        .unwrap_or(5)
        .clamp(if model == "MiniMax-H3-Max" { 5 } else { 4 }, 15);
    let resolution = take_text(&mut fields, "resolution")?
        .trim()
        .to_ascii_uppercase();
    let resolution = if resolution.is_empty() {
        match (model, pixel_resolution(&size)) {
            ("MiniMax-H3", Some("1080p")) => "2K",
            ("MiniMax-H3-Lite" | "MiniMax-H3-Max", Some("480p")) => "480P",
            _ => "768P",
        }
    } else {
        &resolution
    };
    let mut content = vec![json!({"type":"text", "text":prompt})];
    content.extend(frames.iter().map(Frame::content));
    let mut body = Map::new();
    body.insert("model".into(), json!(model));
    body.insert("duration".into(), json!(duration));
    body.insert("resolution".into(), json!(resolution));
    body.insert("content".into(), json!(content));
    if !has_frame {
        let explicit = take_text(&mut fields, "ratio")?;
        let size = if explicit.trim().is_empty() {
            &size
        } else {
            explicit.trim()
        };
        let ratio = if size.eq_ignore_ascii_case("adaptive") {
            Some("adaptive")
        } else {
            ratio(size, &RATIOS)
        };
        if let Some(ratio) =
            ratio.or_else(|| (refs == 0 || model == "MiniMax-H3-Lite").then_some("16:9"))
        {
            body.insert("ratio".into(), json!(ratio));
        }
    }
    if model == "MiniMax-H3-Max"
        && let Some(extra) = fields.remove("extra").filter(Value::is_object)
    {
        body.insert("extra".into(), extra);
    }
    Ok(Value::Object(body))
}

fn dashscope(model: &str, input: Input) -> Result<Value> {
    ensure!(
        input.has_content(),
        "video requires a prompt or reference image"
    );
    let Input {
        prompt,
        size,
        seconds,
        frames,
        mut fields,
    } = input;
    ensure!(
        frames
            .iter()
            .filter(|frame| frame.role == "reference_image")
            .count()
            <= 10,
        "DashScope accepts up to ten reference images"
    );
    let media: Vec<_> = frames
        .iter()
        .map(|frame| json!({"type":frame.role,"url":frame.url}))
        .collect();
    let mut input = Map::new();
    input.insert("prompt".into(), json!(prompt));
    if !media.is_empty() {
        input.insert("media".into(), json!(media));
    }
    if let Some(negative) = fields.remove("negative_prompt") {
        input.insert("negative_prompt".into(), negative);
    }
    let resolution = take_text(&mut fields, "resolution")?;
    let resolution = if resolution.trim().is_empty() {
        pixel_resolution(&size).unwrap_or_default()
    } else {
        resolution.trim()
    };
    if !resolution.is_empty() {
        fields.insert("resolution".into(), json!(resolution.to_ascii_uppercase()));
    }
    if let Some(ratio) = ratio(&size, &RATIOS).filter(|_| fields.get("ratio").is_none()) {
        fields.insert("ratio".into(), json!(ratio));
    }
    if let Some(seconds) = seconds {
        fields.insert("duration".into(), json!(seconds));
    }
    if fields.get("watermark").is_none() {
        fields.insert("watermark".into(), json!(false));
    }
    Ok(json!({"model":model, "input":Value::Object(input), "parameters":Value::Object(fields)}))
}
