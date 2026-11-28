use super::{
    Api, Config, Destination, Gateway, NativeProtocols, Round, Target, affinity,
    classifier::{Choice, Classification, Question},
};
use futures_util::{Stream, StreamExt, stream};
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
    pub effort: Option<&'a str>,
}

enum Plan<'a> {
    Model(Candidate<'a>),
    Group {
        round: Option<&'a Round>,
        branches: Vec<(usize, Plan<'a>)>,
        preferred: usize,
        classify: Option<Box<Decision<'a>>>,
        remember: Option<Box<Remember>>,
    },
}

impl Plan<'_> {
    fn has_model(
        &self,
        config: &Config,
        emitted: &HashSet<(usize, usize)>,
        require_unrestricted: bool,
    ) -> bool {
        match self {
            Self::Model(candidate) => {
                let target = candidate.target;
                !emitted.contains(&(target.provider, target.model))
                    && (!require_unrestricted || !config.providers[target.provider].no_stand_in)
            }
            Self::Group { branches, .. } => branches
                .iter()
                .any(|(_, plan)| plan.has_model(config, emitted, require_unrestricted)),
        }
    }
}

struct Decision<'a> {
    route: Route,
    settings: &'a crate::config::Classifiers,
    intents: Vec<&'a str>,
    automatic: bool,
    via: Vec<&'a str>,
    outer_limits: Limits<'a>,
}

struct Remember {
    key: affinity::Key,
    decision: affinity::Decision,
    replaced: Option<Option<usize>>,
}

#[derive(Default)]
struct Visited {
    models: HashSet<(usize, usize)>,
    groups: HashSet<usize>,
}

#[derive(Clone, Copy)]
pub(super) struct RequestRoute<'a> {
    pub api: Api<'a>,
    pub picks: &'a BTreeMap<String, usize>,
    pub facts: Option<&'a super::rules::Facts>,
    pub paused: &'a AtomicBool,
    pub classifier: bool,
}

#[derive(Clone, Copy)]
struct Limits<'a> {
    first_content: Option<i32>,
    patience: i32,
    group_effort: Option<&'a str>,
    member_effort: Option<&'a str>,
    automatic_effort: Option<&'static str>,
}

impl<'a> Limits<'a> {
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
            ..self
        }
    }

    fn for_member(mut self, primary: Option<&'a super::rules::Rule>, member: &str) -> Self {
        if let Some(rule) = primary.filter(|rule| !rule.use_effort.is_empty()) {
            self.group_effort.get_or_insert(&rule.use_effort);
            if rule.targets.iter().any(|target| target == member) {
                self.member_effort.get_or_insert(&rule.use_effort);
            }
        }
        self
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
        request: RequestRoute<'a>,
        classification: Classification<'a>,
    ) -> impl Stream<Item = Candidate<'a>> + Send + 'a {
        let backups = route.fallbacks(&self.config).iter().filter_map(|id| {
            let (_, backup) = self.resolve(id)?;
            Some((
                *backup,
                matches!(backup, Route::Group { .. }).then_some(id.as_str()),
            ))
        });
        let roots = std::iter::once((route, None)).chain(backups);
        let requested_group = matches!(route, Route::Group { .. });
        let limits = Limits {
            first_content: requested_group.then_some(self.config.default_group_first_content),
            patience: match route {
                Route::Model(target) => self.config.providers[target.provider].patience,
                _ => self.config.default_group_patience,
            },
            group_effort: None,
            member_effort: None,
            automatic_effort: None,
        };
        // A direct request keeps its origin even when the primary lacks this protocol.
        let unrestricted =
            matches!(route, Route::Model(t) if !self.config.providers[t.provider].no_stand_in);
        let state = (
            roots,
            Vec::new(),
            Visited::default(),
            HashSet::new(),
            unrestricted,
            classification,
            BTreeMap::new(),
        );
        stream::unfold(
            state,
            move |(
                mut roots,
                mut pending,
                mut seen,
                mut emitted,
                mut unrestricted,
                mut classification,
                mut judged,
            )| async move {
                loop {
                    let plan = pending.pop().or_else(|| {
                        roots.find_map(|(root, group)| {
                            let mut via = group.into_iter().collect();
                            self.expand(root, &request, &mut via, &mut seen, limits, None)
                        })
                    })?;
                    match plan {
                        Plan::Model(mut candidate) => {
                            let restricted =
                                self.config.providers[candidate.target.provider].no_stand_in;
                            if restricted && unrestricted {
                                continue;
                            }
                            if !emitted.insert((candidate.target.provider, candidate.target.model))
                            {
                                continue;
                            }
                            unrestricted |= !restricted;
                            // Check for a viable fallback without spending a nested group's turn.
                            if candidate.first_content.is_some()
                                && !pending
                                    .iter()
                                    .any(|p| p.has_model(&self.config, &emitted, unrestricted))
                            {
                                candidate.first_content = None;
                            }
                            return Some((
                                candidate,
                                (
                                    roots,
                                    pending,
                                    seen,
                                    emitted,
                                    unrestricted,
                                    classification,
                                    judged,
                                ),
                            ));
                        }
                        Plan::Group {
                            round,
                            mut branches,
                            preferred,
                            classify,
                            remember,
                        } => {
                            if let Some(mut decision) = classify {
                                if !branches.iter().any(|(_, plan)| {
                                    plan.has_model(&self.config, &emitted, unrestricted)
                                }) {
                                    continue;
                                }
                                let Route::Group { index, .. } = decision.route else {
                                    unreachable!("classifier route group")
                                };
                                let key = (index, decision.automatic);
                                let choice = if let Some(&choice) = judged.get(&key) {
                                    choice
                                } else {
                                    let facts = request.facts.expect("classifier request facts");
                                    let choice = classification
                                        .choose(
                                            self,
                                            &self.group_ids[index],
                                            facts,
                                            Question {
                                                settings: decision.settings,
                                                intents: &decision.intents,
                                                automatic: decision.automatic,
                                            },
                                        )
                                        .await;
                                    judged.insert(key, choice);
                                    choice
                                };
                                if let Some(choice) = choice {
                                    // Assign shared descendants in the chosen rule order. A model already
                                    // attempted through another branch still runs only once per round.
                                    let mut available = Visited {
                                        models: emitted.clone(),
                                        groups: HashSet::new(),
                                    };
                                    pending.extend(self.expand(
                                        decision.route,
                                        &request,
                                        &mut decision.via,
                                        &mut available,
                                        decision.outer_limits,
                                        Some(choice),
                                    ));
                                    continue;
                                }
                            }
                            if let Some(remember) = remember
                                && branches.iter().any(|(_, plan)| {
                                    plan.has_model(&self.config, &emitted, unrestricted)
                                })
                            {
                                let Remember {
                                    key,
                                    decision,
                                    replaced,
                                } = *remember;
                                self.turn_affinity.remember(key, decision, replaced);
                            }
                            if preferred != 0 && preferred < branches.len() {
                                pending.push(Plan::Group {
                                    round,
                                    branches: branches.split_off(preferred),
                                    preferred: 0,
                                    classify: None,
                                    remember: None,
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
            },
        )
        .fuse()
    }

    fn member_context(&self, id: &str, picks: &BTreeMap<String, usize>) -> Option<u32> {
        match *self.routes.get(id)? {
            Route::Disabled => None,
            Route::Model(target) => {
                let p = &self.config.providers[target.provider];
                p.contexts
                    .get(&p.models[target.model])
                    .or_else(|| p.contexts.get("*"))
                    .copied()
            }
            Route::Group { index, .. } => self
                .live_members(&self.group_ids[index]["group/".len()..], picks)
                .iter()
                .filter_map(|id| self.member_context(id, picks))
                .min(),
        }
    }

    fn expand<'a>(
        &'a self,
        route: Route,
        request: &RequestRoute<'a>,
        via: &mut Vec<&'a str>,
        seen: &mut Visited,
        limits: Limits<'a>,
        choice: Option<Choice<'a>>,
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
                    .filter(|destination| {
                        !request.classifier || matches!(destination, Destination::Upstream(..))
                    })
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
                        effort: limits
                            .member_effort
                            .or(limits.group_effort)
                            .or(limits.automatic_effort),
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
        let outer_limits = limits;
        let mut limits = outer_limits.within(&self.config, name);
        let facts = request.facts.filter(|_| !request.picks.contains_key(name));
        let rules = self.rules.get(name).map_or(&[][..], Vec::as_slice);
        let settings = self.config.group_classifiers.get(name);
        let key = facts
            .filter(|_| !rules.is_empty() || settings.is_some())
            .and_then(super::rules::Facts::affinity_turn)
            .map(|turn| affinity::Key::new(id, turn));
        let held = facts
            .filter(|facts| facts.continued())
            .and(key)
            .and_then(|key| self.turn_affinity.get(key));
        let intent = held
            .as_ref()
            .and_then(|held| held.intent.as_deref())
            .or_else(|| choice.and_then(|choice| choice.intent));
        let effort = held
            .as_ref()
            .and_then(|held| held.effort)
            .or_else(|| choice.and_then(|choice| choice.effort));
        limits.automatic_effort = limits.automatic_effort.or(effort);
        let classify = facts
            .filter(|_| choice.is_none() && held.is_none())
            .zip(settings)
            .and_then(|(facts, settings)| {
                let intents = super::rules::intents(rules, facts);
                let automatic = settings.automatic_effort()
                    && facts.requests_reasoning()
                    && limits.automatic_effort.is_none()
                    && limits.member_effort.is_none()
                    && limits.group_effort.is_none();
                (automatic || !intents.is_empty()).then(|| {
                    Box::new(Decision {
                        route,
                        settings,
                        intents,
                        automatic,
                        via: via.clone(),
                        outer_limits,
                    })
                })
            });
        let ordered = facts.filter(|_| !rules.is_empty()).map(|facts| {
            let ordered = super::rules::ordered(
                members,
                rules,
                facts,
                intent,
                held.as_ref().map(|held| held.rule),
                |id| self.member_context(id, request.picks),
            );
            if ordered.members.len() < members.len() {
                request.paused.store(true, Ordering::Relaxed);
            }
            ordered
        });
        let grew = ordered.as_ref().is_some_and(|ordered| ordered.grew);
        let (mut ordered, primary) = ordered.map_or((None, None), |order| {
            (Some(order.members.into_iter()), order.primary)
        });
        let remember = key
            .filter(|_| classify.is_none() && (held.is_none() || grew))
            .map(|key| {
                Box::new(Remember {
                    key,
                    decision: affinity::Decision {
                        rule: primary.map(|rule| rule.index),
                        intent: intent.map(str::to_owned),
                        effort,
                    },
                    replaced: held.as_ref().map(|held| held.rule),
                })
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
            let choices = self.expand(
                child,
                request,
                via,
                seen,
                limits.for_member(primary, member),
                None,
            );
            if nested {
                via.pop();
            }
            if let Some(plan) = choices {
                preferred_count += usize::from(preferred);
                branches.push((i, plan));
            }
        }
        (!branches.is_empty()).then(|| Plan::Group {
            round: self.rounds.get(id).map(std::sync::Arc::as_ref),
            branches,
            preferred: preferred_count,
            classify,
            remember,
        })
    }
}
