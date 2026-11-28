use super::{infer, model_gateway, record, snapshot};
use crate::{agents::NativeAgent, config, sessions};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, TryLockError},
    io::ErrorKind,
    path::Path,
    time::Duration,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const STATE: &str = "memory-distillation.json";
const MAX_STATE: usize = 256 * 1024;
const MAX_SESSIONS: usize = 400;
const PER_HOUR: usize = 30;
const QUIET: i64 = 30 * 60;
const HOUR: i64 = 60 * 60;

#[derive(Default, NsonDeserialize, NsonSerialize)]
pub struct State {
    options: Option<Options>,
    last: Option<Report>,
    done: BTreeMap<String, String>,
}

#[derive(Clone, PartialEq, NsonDeserialize, NsonSerialize)]
struct Options {
    model: String,
    days: i16,
    enabled_at: String,
}

#[derive(Clone, NsonDeserialize, NsonSerialize)]
pub struct Report {
    started: i64,
    finished: Option<i64>,
    model: String,
    #[njson(default)]
    attempted: usize,
    read: usize,
    proposed: usize,
    unreadable: usize,
    error: Option<String>,
}

impl State {
    fn load(dir: &Path) -> Result<Self> {
        let bytes = match config::read_limited(&dir.join(STATE), MAX_STATE) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error.into()),
        };
        nextjson::from_slice(&bytes)
            .map_err(anyhow::Error::msg)
            .context("read memory distillation state")
    }

    fn save(&self, dir: &Path) -> Result<()> {
        let bytes = nextjson::to_vec(self).map_err(anyhow::Error::msg)?;
        ensure!(
            bytes.len() <= MAX_STATE,
            "memory distillation state exceeds its size limit"
        );
        config::replace_contents(&dir.join(STATE), &bytes, None)
    }
}

fn open_lock(dir: &Path, name: &str) -> Result<File> {
    fs::create_dir_all(dir)?;
    Ok(File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(name))?)
}

fn state_lock(dir: &Path) -> Result<File> {
    let guard = open_lock(dir, "memory-distillation.lock")?;
    guard.lock()?;
    Ok(guard)
}

pub fn status(dir: &Path) -> Result<State> {
    State::load(dir)
}

pub fn configure(dir: &Path, model: Option<&str>, days: i16) -> Result<State> {
    let options = if let Some(model) = model {
        let model = model.trim();
        ensure!(
            !model.is_empty() && model.len() <= 512 && !model.chars().any(char::is_control),
            "choose a valid model or group route"
        );
        ensure!(
            matches!(days, -1 | 7 | 30 | 90 | 365),
            "choose 7, 30, 90, 365, or -1 days"
        );
        Some(Options {
            model: model.into(),
            days,
            enabled_at: OffsetDateTime::now_utc().format(&Rfc3339)?,
        })
    } else {
        None
    };
    let _guard = state_lock(dir)?;
    let mut state = State::load(dir)?;
    state.options = options;
    state.last = None;
    state.save(dir)?;
    Ok(state)
}

pub(crate) async fn hourly(dir: &Path) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        if let Err(error) = run_at(dir, OffsetDateTime::now_utc().unix_timestamp(), false).await {
            eprintln!("Memory distillation: {error:#}");
        }
    }
}

pub async fn run(dir: &Path) -> Result<Report> {
    run_at(dir, OffsetDateTime::now_utc().unix_timestamp(), true)
        .await?
        .context("automatic distillation is disabled, was stopped, or is already running")
}

async fn run_at(dir: &Path, now: i64, manual: bool) -> Result<Option<Report>> {
    if State::load(dir)?.options.is_none() {
        return Ok(None);
    }
    let guard = open_lock(dir, "memory-distillation.run.lock")?;
    match guard.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Ok(None),
        Err(error) => return Err(error.into()),
    }

    let (options, mut report, done) = {
        let _guard = state_lock(dir)?;
        let mut state = State::load(dir)?;
        let Some(options) = state.options.clone() else {
            return Ok(None);
        };
        let recent = state
            .last
            .filter(|last| !manual && last.started > now.saturating_sub(HOUR));
        let report = match recent {
            Some(last) if last.finished.is_some() => return Ok(None),
            Some(last) => last,
            None => Report {
                started: now,
                finished: None,
                model: options.model.clone(),
                attempted: 0,
                read: 0,
                proposed: 0,
                unreadable: 0,
                error: None,
            },
        };
        state.last = Some(report.clone());
        state.save(dir)?;
        (options, report, state.done)
    };
    let root = dir.to_owned();
    let sessions =
        tokio::task::spawn_blocking(move || sessions::list(&root, None, MAX_SESSIONS)).await?;
    let result = match sessions {
        Ok(sessions) => process(dir, now, manual, &options, sessions, &mut report, done).await,
        Err(error) => Err(error),
    };
    if let Err(error) = result {
        report.error = Some(error.to_string());
    }
    report.finished = Some(OffsetDateTime::now_utc().unix_timestamp());
    let _guard = state_lock(dir)?;
    let mut state = State::load(dir)?;
    if state.options.as_ref() != Some(&options) {
        return Ok(None);
    }
    state.last = Some(report.clone());
    state.save(dir)?;
    Ok(Some(report))
}

async fn process(
    dir: &Path,
    now: i64,
    manual: bool,
    options: &Options,
    sessions: sessions::Listing,
    report: &mut Report,
    done: BTreeMap<String, String>,
) -> Result<()> {
    let since = if options.days == -1 {
        i64::MIN
    } else {
        OffsetDateTime::parse(&options.enabled_at, &Rfc3339)?.unix_timestamp()
            - i64::from(options.days) * 86_400
    };
    report.unreadable = sessions.unreadable;
    let latest: BTreeSet<_> = sessions.sessions.iter().map(key).collect();
    let mut gateway = None;
    for session in sessions.sessions {
        if !manual && report.attempted >= PER_HOUR {
            break;
        }
        let modified = OffsetDateTime::parse(&session.modified, &Rfc3339)?.unix_timestamp();
        let key = key(&session);
        if modified < since
            || modified > now.saturating_sub(QUIET)
            || done.get(&key) == Some(&session.modified)
        {
            continue;
        }
        let Some(agent) = NativeAgent::by_id(session.agent) else {
            continue;
        };
        let (found, preview) = match session_snapshot(dir, agent, &session).await? {
            Ok(found) => found,
            Err(_) => {
                report.unreadable += 1;
                continue;
            }
        };
        if found.modified != session.modified {
            continue;
        }
        let proposals = if preview.messages.len() >= 2 {
            let (router, token) = match &gateway {
                Some(pair) => pair,
                None => gateway.insert(model_gateway(dir)?),
            };
            {
                // Reserve the attempt before sending so interruption cannot reset the hourly limit.
                let _guard = state_lock(dir)?;
                let mut state = State::load(dir)?;
                if state.options.as_ref() != Some(options) {
                    return Ok(());
                }
                report.attempted += 1;
                state.last = Some(report.clone());
                state.save(dir)?;
            }
            let proposals = infer(router, token, &options.model, &preview).await?;
            report.read += 1;
            if session_snapshot(dir, agent, &session)
                .await?
                .is_ok_and(|(_, current)| current.revision == preview.revision)
            {
                proposals
            } else {
                continue;
            }
        } else {
            Vec::new()
        };
        let _guard = state_lock(dir)?;
        let mut state = State::load(dir)?;
        if state.options.as_ref() != Some(options) {
            return Ok(());
        }
        report.proposed += record(dir, &found, &preview, proposals)?.len();
        state.done.retain(|key, _| latest.contains(key));
        state.done.insert(key, session.modified);
        state.last = Some(report.clone());
        state.save(dir)?;
    }
    Ok(())
}

async fn session_snapshot(
    dir: &Path,
    agent: NativeAgent,
    session: &sessions::Session,
) -> Result<Result<(sessions::Session, super::Preview)>> {
    let dir = dir.to_owned();
    let profile = session.profile.clone();
    let id = session.id.clone();
    Ok(
        tokio::task::spawn_blocking(move || snapshot(&dir, agent, Some(Path::new(&profile)), &id))
            .await?,
    )
}

fn key(session: &sessions::Session) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(
        nextjson::json!([session.agent, session.profile, session.id]).to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabforge_test_support::server::{Incoming, Json, Router, post, serve};
    use nextjson::{Value, json};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn hourly_distillation_resumes_and_keeps_verified_memories_for_review() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        let now = OffsetDateTime::now_utc().unix_timestamp() + 10 * 86_400;
        assert!(run_at(root, now, false).await?.is_none());
        let profile = root.join("agents/pi/sessions");
        fs::create_dir_all(&profile)?;
        let log = profile.join("fixture.jsonl");
        fs::write(
            &log,
            concat!(
                "{\"type\":\"session\",\"id\":\"review-fixture\",\"cwd\":\"/project\"}\n",
                "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"Always use metric units.\"}}\n",
                "{\"type\":\"message\",\"message\":{\"role\":\"assistant\",\"content\":\"Use dark mode.\"}}\n",
                "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"Please continue.\"}}\n",
            ),
        )?;
        let modified = |time: i64| -> SystemTime {
            UNIX_EPOCH + Duration::from_secs(u64::try_from(time).unwrap())
        };
        let file = File::options().write(true).open(&log)?;
        file.set_modified(modified(now - 15 * 60))?;

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let requests = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let hits = requests.clone();
        let router = Router::new().route("/v1/chat/completions", post(move |Incoming { body, .. }| {
            let hits = hits.clone();
            async move {
                use std::sync::atomic::Ordering;
                hits.fetch_add(1, Ordering::Relaxed);
                let request: Value = nextjson::from_slice(&body).unwrap();
                let messages: Value = nextjson::from_str(request.pointer("/messages/1/content").unwrap().as_str().unwrap()).unwrap();
                assert_eq!(messages, json!(["Always use metric units.", "Please continue."]));
                let proposals = json!([
                    {"text":"Use metric units.","kind":"preference","scope":"global","why":"User preference","quote":"Always use metric units."},
                    {"text":"Use dark mode.","kind":"preference","scope":"global","why":"Assistant suggestion","quote":"Use dark mode."}
                ]);
                Json(json!({"choices":[{"message":{"content":proposals.to_string()}}]}))
            }
        }));
        let server = tokio::spawn(serve(listener, router));
        let settings = json!({"providers":[{"id":"local","models":["m"],"endpoints":{"chat":format!("http://{address}/v1")}}]});
        config::save(root, &config::snapshot(root)?, &settings.to_string())?;
        let mut state = configure(root, Some("local/m"), 7)?;
        state.options.as_mut().unwrap().enabled_at =
            OffsetDateTime::from_unix_timestamp(now)?.format(&Rfc3339)?;
        state.save(root)?;

        let first = run_at(root, now, false).await?.context("initial run")?;
        assert_eq!(first.read, 0);
        assert_eq!(first.error, None);
        assert!(run_at(root, now + HOUR - 1, false).await?.is_none());
        let second = run_at(root, now + HOUR, false)
            .await?
            .context("next hourly run")?;
        assert_eq!(second.error, None);
        assert_eq!((second.read, second.proposed), (1, 1));
        let mut state = status(root)?;
        let interrupted = state.last.as_mut().unwrap();
        interrupted.finished = None;
        interrupted.attempted = PER_HOUR;
        let completed = std::mem::take(&mut state.done);
        state.save(root)?;
        let resumed = run_at(root, now + HOUR + 1, false)
            .await?
            .context("interrupted hourly run")?;
        assert_eq!(
            (resumed.attempted, resumed.read, resumed.proposed),
            (PER_HOUR, 1, 1)
        );
        assert_eq!(requests.load(std::sync::atomic::Ordering::Relaxed), 1);
        let mut state = status(root)?;
        state.done = completed;
        state.save(root)?;
        let again = run_at(root, now + HOUR * 2, false)
            .await?
            .context("resumed run")?;
        assert_eq!((again.read, again.proposed), (0, 0));
        assert_eq!(requests.load(std::sync::atomic::Ordering::Relaxed), 1);
        let saved = crate::memory::search(root, Default::default())?;
        assert_eq!(saved.entries.len(), 1);
        assert_eq!(saved.entries[0].status, "candidate");
        assert_eq!(saved.entries[0].text, "Use metric units.");
        configure(root, None, 7)?;
        assert!(run_at(root, now + HOUR * 3, false).await?.is_none());
        server.abort();
        Ok(())
    }
}
