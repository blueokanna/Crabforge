use super::{Api, Config, Destination, Gateway, NativeProtocols, Round, Target};
use std::collections::{BTreeMap, HashSet};

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

pub(super) struct Candidate<'a> {
    pub target: Target,
    pub destination: Destination<'a>,
    pub via: Vec<&'a str>,
}

enum Plan<'a> {
    Model(Candidate<'a>),
    Group {
        round: Option<&'a Round>,
        branches: Vec<(usize, Plan<'a>)>,
    },
}

#[derive(Default)]
struct Visited {
    models: HashSet<(usize, usize)>,
    groups: HashSet<usize>,
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
        // A direct request keeps its origin even when the primary lacks this protocol.
        let mut unrestricted =
            matches!(route, Route::Model(t) if !self.config.providers[t.provider].no_stand_in);
        std::iter::from_fn(move || {
            loop {
                let plan = pending.pop().or_else(|| {
                    roots.find_map(|(root, group)| {
                        let mut via = group.into_iter().collect();
                        self.expand(root, api, picks, &mut via, &mut seen)
                    })
                })?;
                match plan {
                    Plan::Model(candidate) => {
                        let restricted =
                            self.config.providers[candidate.target.provider].no_stand_in;
                        if restricted && unrestricted {
                            continue;
                        }
                        unrestricted |= !restricted;
                        return Some(candidate);
                    }
                    Plan::Group {
                        round,
                        mut branches,
                    } => {
                        // A subgroup spends its turn only when forwarding reaches it.
                        if let Some(round) = round {
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
        api: Api<'a>,
        picks: &BTreeMap<String, usize>,
        via: &mut Vec<&'a str>,
        seen: &mut Visited,
    ) -> Option<Plan<'a>> {
        let index = match route {
            Route::Disabled => return None,
            Route::Group { index, .. } => index,
            Route::Model(target) => {
                if !seen.models.insert((target.provider, target.model)) {
                    return None;
                }
                let p = &self.config.providers[target.provider];
                return api
                    .destination(p, &p.models[target.model])
                    .map(|destination| Candidate {
                        target,
                        destination,
                        via: via.clone(),
                    })
                    .map(Plan::Model);
            }
        };
        if !seen.groups.insert(index) {
            return None;
        }
        let id = &self.group_ids[index];
        let members = self.live_members(&id["group/".len()..], picks);
        let mut branches = Vec::new();
        for (i, member) in members.iter().enumerate() {
            let child = self.routes[member];
            let nested = matches!(child, Route::Group { .. });
            if nested {
                via.push(member);
            }
            let choices = self.expand(child, api, picks, via, seen);
            if nested {
                via.pop();
            }
            if let Some(plan) = choices {
                branches.push((i, plan));
            }
        }
        (!branches.is_empty()).then(|| Plan::Group {
            round: self.rounds.get(id),
            branches,
        })
    }
}
