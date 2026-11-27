use super::{Api, Config, Destination, Gateway, NativeProtocols, Round, Target};
use std::{
    collections::{BTreeMap, HashSet},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

#[derive(Clone, Copy)]
pub(super) enum Route {
    Disabled,
    Model(Target),
    Group {
        tools: bool,
        index: usize,
        protocols: NativeProtocols,
    },
}

impl Route {
    pub(super) fn fallbacks(self, config: &Config) -> &[String] {
        match self {
            Self::Model(t) => &config.providers[t.provider].fallback,
            _ => &[],
        }
    }
    pub(super) fn protocols(self) -> Option<NativeProtocols> {
        match self {
            Self::Disabled => None,
            Self::Model(target) => Some(target.protocols),
            Self::Group { protocols, .. } => Some(protocols),
        }
    }
    pub(super) fn tools(self, config: &Config) -> bool {
        match self {
            Self::Disabled => false,
            Self::Model(t) => config.providers[t.provider].web_site.is_none(),
            Self::Group { tools, .. } => tools,
        }
    }
}

#[derive(Clone)]
pub(super) struct Candidate<'a> {
    pub target: Target,
    pub destination: Destination<'a>,
    pub via: Vec<&'a str>,
    pub first_content: Option<Duration>,
    pub patience: Duration,
}

enum Plan<'a> {
    Model(Candidate<'a>),
    Group {
        round: Option<&'a Round>,
        branches: Vec<(usize, Plan<'a>)>,
        preferred: usize,
    },
}

impl Plan<'_> {
    fn has_unrestricted_model(&self, config: &Config) -> bool {
        match self {
            Self::Model(candidate) => !config.providers[candidate.target.provider].no_stand_in,
            Self::Group { branches, .. } => branches
                .iter()
                .any(|(_, plan)| plan.has_unrestricted_model(config)),
        }
    }
}

#[derive(Default)]
struct Visited {
    models: HashSet<(usize, usize)>,
    groups: HashSet<usize>,
}

struct RequestRoute<'a> {
    api: Api<'a>,
    picks: &'a BTreeMap<String, usize>,
    facts: Option<&'a super::rules::Facts>,
    paused: &'a AtomicBool,
}

#[derive(Clone, Copy)]
struct Limits {
    first_content: Option<i32>,
    patience: i32,
}

impl Limits {
    fn within(self, config: &Config, group: &str) -> Self {
        let inherit = |values: &BTreeMap<String, i32>, enclosing| {
            values
                .get(group)
                .copied()
                .filter(|&s| s != 0)
                .unwrap_or(enclosing)
        };
        Self {
            first_content: self
                .first_content
                .map(|seconds| inherit(&config.group_first_content, seconds)),
            patience: inherit(&config.group_patience, self.patience),
        }
    }
}

impl Gateway {
    // Snapshot all manual choices once per request, including nested groups. Membership
    // stays with the running configuration, while CLI and desktop picks take effect immediately.
    pub(super) async fn current_picks(&self) -> anyhow::Result<BTreeMap<String, usize>> {
        if self.manual_members.is_empty() {
            return Ok(BTreeMap::new());
        }
        #[derive(nextjson::NsonDeserialize)]
        struct Picks {
            #[njson(default)]
            group_picks: BTreeMap<String, String>,
        }
        let path = self.root.join("settings.json");
        let saved = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let bytes = crate::config::read_limited(&path, crate::config::MAX_CONFIG)?;
            Ok(nextjson::from_slice::<Picks>(&bytes)
                .map_err(|e| anyhow::anyhow!("invalid group picks: {e}"))?
                .group_picks)
        })
        .await??;
        Ok(self
            .manual_members
            .iter()
            .map(|(name, members)| {
                let pick = saved.get(name).unwrap_or(&self.config.group_picks[name]);
                let index = members.iter().position(|m| m == pick).unwrap_or(0);
                (name.clone(), index)
            })
            .collect())
    }

    pub(super) fn live_members(&self, name: &str, picks: &BTreeMap<String, usize>) -> &[String] {
        let members = &self.config.groups[name];
        match picks.get(name) {
            Some(&index) => members.get(index..=index).unwrap_or_default(),
            None => members,
        }
    }

    pub(super) fn candidates<'a>(
        &'a self,
        route: Route,
        api: Api<'a>,
        picks: &'a BTreeMap<String, usize>,
        facts: Option<&'a super::rules::Facts>,
        paused: &'a AtomicBool,
    ) -> impl Iterator<Item = Candidate<'a>> + 'a {
        let mut seen = Visited::default();
        let backups = route.fallbacks(&self.config).iter().filter_map(|id| {
            let (_, backup) = self.resolve(id)?;
            Some((
                *backup,
                matches!(backup, Route::Group { .. }).then_some(id.as_str()),
            ))
        });
        let mut roots = std::iter::once((route, None)).chain(backups);
        let mut pending = Vec::new();
        let requested_group = matches!(route, Route::Group { .. });
        let limits = Limits {
            first_content: requested_group.then_some(self.config.default_group_first_content),
            patience: match route {
                Route::Model(target) => self.config.providers[target.provider].patience,
                _ => self.config.default_group_patience,
            },
        };
        // A direct request keeps its origin even when the primary lacks this protocol.
        let mut unrestricted =
            matches!(route, Route::Model(t) if !self.config.providers[t.provider].no_stand_in);
        let request = RequestRoute {
            api,
            picks,
            facts,
            paused,
        };
        std::iter::from_fn(move || {
            loop {
                let plan = pending.pop().or_else(|| {
                    roots.find_map(|(root, group)| {
                        let mut via = group.into_iter().collect();
                        self.expand(root, &request, &mut via, &mut seen, limits)
                    })
                })?;
                match plan {
                    Plan::Model(mut candidate) => {
                        let restricted =
                            self.config.providers[candidate.target.provider].no_stand_in;
                        if restricted && unrestricted {
                            continue;
                        }
                        unrestricted |= !restricted;
                        // Check for a viable fallback without spending a nested group's turn.
                        if candidate.first_content.is_some()
                            && (pending.is_empty()
                                || unrestricted
                                    && !pending
                                        .iter()
                                        .any(|p| p.has_unrestricted_model(&self.config)))
                        {
                            candidate.first_content = None;
                        }
                        return Some(candidate);
                    }
                    Plan::Group {
                        round,
                        mut branches,
                        preferred,
                    } => {
                        if preferred != 0 && preferred < branches.len() {
                            pending.push(Plan::Group {
                                round,
                                branches: branches.split_off(preferred),
                                preferred: 0,
                            });
                        } else if preferred == 0
                            && let Some(round) = round
                        {
                            // A subgroup spends its turn only when forwarding reaches it.
                            round.order(&mut branches);
                        }
                        pending.extend(branches.into_iter().rev().map(|(_, plan)| plan));
                    }
                }
            }
        })
    }

    fn expand<'a>(
        &'a self,
        route: Route,
        request: &RequestRoute<'a>,
        via: &mut Vec<&'a str>,
        seen: &mut Visited,
        limits: Limits,
    ) -> Option<Plan<'a>> {
        let index = match route {
            Route::Disabled => return None,
            Route::Group { index, .. } => index,
            Route::Model(target) => {
                if !seen.models.insert((target.provider, target.model)) {
                    return None;
                }
                let p = &self.config.providers[target.provider];
                return request
                    .api
                    .destination(p, &p.models[target.model])
                    .map(|destination| Candidate {
                        target,
                        destination,
                        via: via.clone(),
                        first_content: limits
                            .first_content
                            .filter(|&seconds| seconds > 0)
                            .map(|seconds| Duration::from_secs(seconds as u64)),
                        patience: match limits.patience {
                            0 => Duration::from_secs(60),
                            seconds => Duration::from_secs(seconds.max(0) as u64),
                        },
                    })
                    .map(Plan::Model);
            }
        };
        if !seen.groups.insert(index) {
            return None;
        }
        let id = &self.group_ids[index];
        let name = &id["group/".len()..];
        let members = self.live_members(name, request.picks);
        let limits = limits.within(&self.config, name);
        let mut ordered = request
            .facts
            .filter(|_| !request.picks.contains_key(name))
            .zip(self.rules.get(name))
            .map(|(facts, rules)| {
                let ordered = super::rules::ordered(members, rules, facts);
                if ordered.len() < members.len() {
                    request.paused.store(true, Ordering::Relaxed);
                }
                ordered.into_iter()
            });
        let mut original = members.iter().enumerate();
        let mut preferred_count = 0;
        let mut branches = Vec::new();
        let next = || match &mut ordered {
            Some(ranked) => ranked.next(),
            None => original.next().map(|(i, id)| (false, i, id.as_str())),
        };
        for (preferred, i, member) in std::iter::from_fn(next) {
            let child = self.routes[member];
            let nested = matches!(child, Route::Group { .. });
            if nested {
                via.push(member);
            }
            let choices = self.expand(child, request, via, seen, limits);
            if nested {
                via.pop();
            }
            if let Some(plan) = choices {
                preferred_count += usize::from(preferred);
                branches.push((i, plan));
            }
        }
        (!branches.is_empty()).then(|| Plan::Group {
            round: self.rounds.get(id),
            branches,
            preferred: preferred_count,
        })
    }
}
