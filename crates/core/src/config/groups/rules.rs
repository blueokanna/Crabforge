use anyhow::{Context, Result, bail, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};

#[derive(Clone, Debug, Default, NsonDeserialize, NsonSerialize)]
#[njson(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct RoutingRule {
    #[njson(rename = "use")]
    pub target: String,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub more: Vec<String>,
    #[njson(skip_serializing_if = "crate::config::is_zero")]
    pub tokens: i32,
    #[njson(skip_serializing_if = "std::ops::Not::not")]
    pub images: bool,
    #[njson(skip_serializing_if = "String::is_empty")]
    pub effort: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    pub use_effort: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    pub intent: String,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<String>,
    #[njson(skip_serializing_if = "std::ops::Not::not")]
    pub compact: bool,
    #[njson(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeWindow>,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub folders: Vec<String>,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub except_folders: Vec<String>,
    #[njson(rename = "where", skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    #[njson(skip_serializing_if = "std::ops::Not::not")]
    pub pause: bool,
}

impl RoutingRule {
    pub(crate) fn targets(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.target.as_str()).chain(self.more.iter().map(String::as_str))
    }

    pub(crate) fn remap(&mut self, mut resolve: impl FnMut(&str) -> Option<String>) -> bool {
        let mut targets = self.targets().filter_map(&mut resolve).collect::<Vec<_>>();
        if targets.is_empty() {
            return false;
        }
        let mut seen = std::collections::HashSet::new();
        targets.retain(|id| seen.insert(id.clone()));
        self.target = targets.remove(0);
        self.more = targets;
        true
    }

    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(!self.target.is_empty(), "a routing rule needs a target");
        ensure!(self.tokens >= 0, "token threshold must be nonnegative");
        ensure!(
            self.intent.is_empty()
                || (!self.intent.trim().is_empty() && self.intent.chars().count() <= 200),
            "an intent must contain 1 to 200 characters"
        );
        ensure!(
            self.effort.is_empty() || self.effort == "on" || effort_rank(&self.effort).is_some(),
            "invalid reasoning effort {:?}",
            self.effort
        );
        ensure!(
            self.use_effort.is_empty()
                || matches!(
                    self.use_effort.as_str(),
                    "low" | "medium" | "high" | "xhigh" | "max"
                ),
            "invalid requested reasoning effort {:?}",
            self.use_effort
        );
        ensure!(
            self.agents.iter().all(|a| !a.trim().is_empty()),
            "empty agent ID"
        );
        ensure!(
            self.tokens > 0
                || self.images
                || !self.effort.is_empty()
                || !self.intent.is_empty()
                || self.compact
                || self.time.is_some()
                || !self.agents.is_empty()
                || !self.folders.is_empty()
                || !self.except_folders.is_empty()
                || !self.conditions.is_empty(),
            "a routing rule requires at least one condition"
        );
        if self.pause {
            ensure!(
                self.more.is_empty()
                    && self.tokens == 0
                    && !self.images
                    && self.effort.is_empty()
                    && self.use_effort.is_empty()
                    && self.intent.is_empty()
                    && !self.compact
                    && self.conditions.iter().all(|c| c.field == "agent"),
                "a pause names one member and can use hours, agents or folders"
            );
        }
        for path in self.folders.iter().chain(&self.except_folders) {
            ensure!(
                folder_key(path).is_some(),
                "invalid absolute folder {path:?}"
            );
        }
        if let Some(window) = &self.time {
            window.compile()?;
        }
        for condition in &self.conditions {
            condition.compile()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, NsonDeserialize, NsonSerialize)]
#[njson(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Classifiers {
    #[njson(skip_serializing_if = "String::is_empty")]
    pub effort: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    pub classifier: String,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    pub classifier_more: Vec<String>,
    #[njson(skip_serializing_if = "String::is_empty")]
    pub classify_prompt: String,
}

impl Classifiers {
    pub fn is_empty(&self) -> bool {
        self.effort.is_empty()
            && self.classifier.is_empty()
            && self.classifier_more.is_empty()
            && self.classify_prompt.is_empty()
    }

    pub(crate) fn automatic_effort(&self) -> bool {
        self.effort == "auto"
    }

    pub(crate) fn targets(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.classifier.as_str())
            .chain(self.classifier_more.iter().map(String::as_str))
            .skip(usize::from(self.classifier.is_empty()))
    }

    pub(crate) fn rename_targets(&mut self, rename: impl Fn(&str) -> Option<String>) {
        for id in std::iter::once(&mut self.classifier).chain(&mut self.classifier_more) {
            if let Some(next) = rename(id) {
                *id = next;
            }
        }
    }
}

#[derive(Default, NsonDeserialize, NsonSerialize)]
#[njson(default)]
pub struct RuleSet {
    pub rules: Vec<RoutingRule>,
    #[njson(flatten)]
    pub classifiers: Classifiers,
}

#[derive(Clone, Debug, NsonDeserialize, NsonSerialize)]
#[njson(deny_unknown_fields)]
pub struct TimeWindow {
    from: String,
    to: String,
    #[njson(default, skip_serializing_if = "Vec::is_empty")]
    days: Vec<String>,
    #[njson(default, skip_serializing_if = "String::is_empty")]
    zone: String,
}

pub(crate) struct Clock {
    from: i16,
    to: i16,
    days: u8,
    zone: jiff::tz::TimeZone,
}

impl TimeWindow {
    pub(crate) fn compile(&self) -> Result<Clock> {
        fn minutes(text: &str) -> Option<i16> {
            let (hour, minute) = text.trim().split_once(':')?;
            if !(1..=2).contains(&hour.len())
                || minute.len() != 2
                || !hour
                    .bytes()
                    .chain(minute.bytes())
                    .all(|c| c.is_ascii_digit())
            {
                return None;
            }
            let hour = hour.parse::<i16>().ok()?;
            let minute = minute.parse::<i16>().ok()?;
            ((0..24).contains(&hour) && (0..60).contains(&minute)).then_some(hour * 60 + minute)
        }
        let from = minutes(&self.from).context("invalid routing start time")?;
        let to = minutes(&self.to).context("invalid routing end time")?;
        let mut days = if self.days.is_empty() { 127 } else { 0 };
        for day in &self.days {
            let day = day.trim().to_ascii_lowercase();
            let index = [
                "monday",
                "tuesday",
                "wednesday",
                "thursday",
                "friday",
                "saturday",
                "sunday",
            ]
            .iter()
            .position(|d| day.len() >= 3 && d.starts_with(&day))
            .with_context(|| format!("invalid routing weekday {day:?}"))?;
            days |= 1 << index;
        }
        ensure!(
            from != to || days != 127,
            "a routing window must restrict its hours or days"
        );
        let zone = if matches!(self.zone.trim(), "" | "Local") {
            jiff::tz::TimeZone::system()
        } else {
            jiff::tz::TimeZone::get(self.zone.trim()).context("invalid routing time zone")?
        };
        Ok(Clock {
            from,
            to,
            days,
            zone,
        })
    }
}

impl Clock {
    pub(crate) fn contains(&self, at: jiff::Timestamp) -> bool {
        let local = self.zone.to_datetime(at);
        let minute = i16::from(local.hour()) * 60 + i16::from(local.minute());
        let mut day = local.weekday().to_monday_zero_offset();
        if self.from > self.to && minute < self.to {
            day = (day + 6) % 7;
        }
        self.days & (1 << day) != 0
            && match self.from.cmp(&self.to) {
                std::cmp::Ordering::Equal => true,
                std::cmp::Ordering::Less => (self.from..self.to).contains(&minute),
                std::cmp::Ordering::Greater => minute >= self.from || minute < self.to,
            }
    }
}

#[derive(Clone, Debug, NsonDeserialize, NsonSerialize)]
#[njson(deny_unknown_fields)]
pub struct Condition {
    pub field: String,
    pub op: String,
    #[njson(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
}

#[derive(Clone, Copy)]
pub(crate) enum NumberOp {
    AtLeast,
    Above,
    Below,
    AtMost,
}

impl NumberOp {
    pub(crate) fn matches(self, actual: usize, threshold: usize) -> bool {
        match self {
            Self::AtLeast => actual >= threshold,
            Self::Above => actual > threshold,
            Self::Below => actual < threshold,
            Self::AtMost => actual <= threshold,
        }
    }
}

pub(crate) enum EffortTest {
    Is,
    Not,
    AtLeast,
    AtMost,
}
pub(crate) enum TextField {
    Message,
    System,
    Last,
}
pub(crate) enum TextTest {
    Contains(String),
    Excludes(String),
    Starts(String),
    Regex(regex::Regex),
}
pub(crate) enum Predicate {
    Tokens(NumberOp, usize),
    Effort(EffortTest, String),
    Agent(bool, Vec<String>),
    Subagent(bool),
    Text(TextField, TextTest),
}

pub(crate) fn effort_rank(value: &str) -> Option<usize> {
    ["minimal", "low", "medium", "high", "xhigh", "max"]
        .iter()
        .position(|level| *level == value)
}

pub(crate) fn agent_key(agent: &str) -> String {
    let agent = agent.trim();
    let canonical = |value: &str| {
        crate::agents::CLIENTS
            .iter()
            .find(|(id, name)| id.eq_ignore_ascii_case(value) || name.eq_ignore_ascii_case(value))
            .map(|(id, _)| *id)
    };
    canonical(agent)
        .or_else(|| crate::agents::client_name(agent).and_then(|name| canonical(&name)))
        .unwrap_or(agent)
        .to_ascii_lowercase()
}

impl Condition {
    pub(crate) fn compile(&self) -> Result<Predicate> {
        let op = self.op.as_str();
        let value = self.value.trim();
        Ok(match self.field.as_str() {
            "tokens" => {
                let op = match op {
                    "gte" => NumberOp::AtLeast,
                    "gt" => NumberOp::Above,
                    "lt" => NumberOp::Below,
                    "lte" => NumberOp::AtMost,
                    _ => bail!("invalid token comparison {op:?}"),
                };
                let count = value.replace('_', "").to_ascii_lowercase();
                let (count, scale) = match count.as_bytes().last() {
                    Some(b'k') => (&count[..count.len() - 1], 1_000.0),
                    Some(b'm') => (&count[..count.len() - 1], 1_000_000.0),
                    _ => (count.as_str(), 1.0),
                };
                let count = count.parse::<f64>().context("invalid token count")? * scale;
                ensure!(
                    count >= 0.0 && count < usize::MAX as f64,
                    "invalid token count"
                );
                Predicate::Tokens(op, count as usize)
            }
            "effort" => {
                let op = match op {
                    "is" => EffortTest::Is,
                    "not" => EffortTest::Not,
                    "gte" => EffortTest::AtLeast,
                    "lte" => EffortTest::AtMost,
                    _ => bail!("invalid effort comparison {op:?}"),
                };
                ensure!(
                    effort_rank(value).is_some()
                        || matches!(op, EffortTest::Is | EffortTest::Not)
                            && matches!(value, "on" | "off"),
                    "invalid reasoning effort {value:?}"
                );
                Predicate::Effort(op, value.to_owned())
            }
            "agent" => {
                ensure!(
                    matches!(op, "is" | "not"),
                    "invalid agent comparison {op:?}"
                );
                let agents = value
                    .split(|c: char| c.is_whitespace() || c == ',' || c == '|')
                    .filter(|a| !a.is_empty())
                    .map(agent_key)
                    .collect::<Vec<_>>();
                ensure!(!agents.is_empty(), "agent condition needs an agent ID");
                Predicate::Agent(op == "is", agents)
            }
            "subagent" => {
                ensure!(
                    matches!(op, "is" | "not") && value.is_empty(),
                    "invalid subagent condition"
                );
                Predicate::Subagent(op == "is")
            }
            field => {
                let field = match field {
                    "message" => TextField::Message,
                    "system" => TextField::System,
                    "last" => TextField::Last,
                    _ => bail!("unknown routing condition field {field:?}"),
                };
                ensure!(
                    !value.is_empty() && value.chars().count() <= 500,
                    "a text condition requires between 1 and 500 characters"
                );
                let test = match op {
                    "contains" => TextTest::Contains(value.to_lowercase()),
                    "excludes" => TextTest::Excludes(value.to_lowercase()),
                    "starts" => TextTest::Starts(value.to_lowercase()),
                    "matches" => TextTest::Regex(
                        regex::RegexBuilder::new(
                            value
                                .strip_prefix('/')
                                .and_then(|p| p.strip_suffix('/'))
                                .filter(|p| !p.is_empty())
                                .unwrap_or(value),
                        )
                        .case_insensitive(true)
                        .size_limit(1 << 20)
                        .build()
                        .context("invalid routing regex")?,
                    ),
                    _ => bail!("invalid text comparison {op:?}"),
                };
                Predicate::Text(field, test)
            }
        })
    }
}

pub(crate) fn folder_key(path: &str) -> Option<String> {
    let mut path = path.trim().replace('\\', "/");
    if path == "~" || path.starts_with("~/") {
        path.replace_range(..1, &std::env::home_dir()?.to_str()?.replace('\\', "/"));
    }
    let mut fold = cfg!(any(windows, target_os = "macos"));
    if let Some(share) = path.strip_prefix("//") {
        let (host, remainder) = share.split_once('/')?;
        if host.eq_ignore_ascii_case("wsl$") || host.eq_ignore_ascii_case("wsl.localhost") {
            path = remainder
                .split_once('/')
                .map_or("/".to_owned(), |(_, tail)| format!("/{tail}"));
        }
    }
    if matches!(path.as_bytes(), [b'a'..=b'z' | b'A'..=b'Z', b':'] | [b'a'..=b'z' | b'A'..=b'Z', b':', b'/', ..])
    {
        path = format!("/mnt/{}{}", &path[..1], &path[2..]);
    }
    if matches!(
        path.strip_prefix("/mnt/").map(str::as_bytes),
        Some([b'a'..=b'z' | b'A'..=b'Z'] | [b'a'..=b'z' | b'A'..=b'Z', b'/', ..])
    ) {
        fold = true;
    }
    if !path.starts_with('/') || path.chars().any(char::is_control) {
        return None;
    }
    let mut parts = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => (),
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    let key = format!("/{}", parts.join("/"));
    Some(if fold { key.to_lowercase() } else { key })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_windows_follow_the_local_start_day_and_daylight_saving() -> Result<()> {
        let window = TimeWindow {
            from: "22:00".into(),
            to: "02:00".into(),
            days: vec!["fri".into()],
            zone: "America/New_York".into(),
        }
        .compile()?;
        for (at, expected) in [
            ("2026-03-07T02:59:59Z", false),
            ("2026-03-07T03:00:00Z", true),
            ("2026-03-07T06:59:59Z", true),
            ("2026-03-07T07:00:00Z", false),
            ("2026-03-14T01:59:59Z", false),
            ("2026-03-14T02:00:00Z", true),
            ("2026-03-14T06:00:00Z", false),
        ] {
            assert_eq!(window.contains(at.parse()?), expected, "{at}");
        }
        Ok(())
    }
}
