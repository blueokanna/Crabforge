use super::{HOUR, MONTH, Plan, WEEK, Window, duration_name, number, reset};
use anyhow::{Context, Result, ensure};
use nextjson::Value;

type Quota = (Option<String>, Vec<Window>);

pub(super) fn parse(plan: Plan, response: &Value) -> Result<Quota> {
    match plan {
        Plan::OpenCodeGo => Ok((Some("OpenCode Go".into()), opencode(response)?)),
        Plan::Zhipu => zhipu(response),
        Plan::Kimi => Ok((Some("Kimi Code".into()), kimi(response)?)),
        Plan::CommandCode => Ok(command_code(response)),
        Plan::MiniMax => Ok((None, minimax(response)?)),
        Plan::Ollama => Ok((None, ollama(response))),
        Plan::Mistral => unreachable!("Mistral requires the identified key type"),
    }
}

fn text(value: Option<&Value>) -> Option<String> {
    let text = value?.as_str()?.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

fn values<'a>(object: &'a Value, field: &str) -> impl Iterator<Item = &'a Value> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn opencode(response: &Value) -> Result<Vec<Window>> {
    let usage = response
        .get("usage")
        .context("OpenCode Go did not return usage windows")?;
    let windows: Vec<_> = [
        ("rolling", "5 hours", 5 * HOUR),
        ("weekly", "7 days", WEEK),
        ("monthly", "Month", 0),
    ]
    .into_iter()
    .filter_map(|(key, name, span)| {
        let row = usage.get(key)?;
        let percent = number(row.get("percent"))?;
        let resets = (percent > 0.0)
            .then(|| reset(row.get("resetsAt")))
            .flatten();
        Some(Window::percent(name, percent, span, resets))
    })
    .collect();
    ensure!(
        !windows.is_empty(),
        "OpenCode Go did not return usage windows"
    );
    Ok(windows)
}

fn zhipu(response: &Value) -> Result<Quota> {
    ensure!(
        response.get("success").and_then(Value::as_bool) != Some(false),
        "Zhipu rejected the quota request"
    );
    let data = response
        .get("data")
        .filter(|value| value.is_object())
        .context("Zhipu did not return plan data")?;
    let windows = values(data, "limits")
        .map(|row| {
            let aside = row
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind.eq_ignore_ascii_case("TIME_LIMIT"));
            let span = match row.get("unit").and_then(Value::as_u64) {
                Some(3) => {
                    row.get("number")
                        .and_then(Value::as_u64)
                        .filter(|n| (1..=24).contains(n))
                        .unwrap_or(5)
                        * HOUR
                }
                Some(6) => WEEK,
                _ => 0,
            };
            let mut window = Window::percent(
                if aside {
                    "MCP · Month".into()
                } else {
                    duration_name(span)
                },
                number(row.get("percentage")).unwrap_or(0.0),
                if aside { 0 } else { span },
                reset(row.get("nextResetTime")),
            );
            window.used_percent = number(row.get("percentage")).map(|n| n.clamp(0.0, 100.0));
            window.aside = aside;
            window
        })
        .collect();
    Ok((text(data.get("level")), windows))
}

fn kimi(response: &Value) -> Result<Vec<Window>> {
    fn reset_at(row: &Value) -> Option<time::OffsetDateTime> {
        ["resetTime", "resetAt", "reset_at", "reset_time"]
            .into_iter()
            .find_map(|key| reset(row.get(key)))
    }
    fn counted(row: &Value, span: u64) -> Option<Window> {
        let limit = number(row.get("limit")).filter(|n| *n > 0.0)?;
        let used =
            number(row.get("used")).or_else(|| number(row.get("remaining")).map(|n| limit - n))?;
        Some(Window {
            used: Some(used),
            limit: Some(limit),
            ..Window::percent(
                duration_name(span),
                used / limit * 100.0,
                span,
                reset_at(row),
            )
        })
    }
    let mut windows: Vec<_> = values(response, "limits")
        .filter_map(|row| {
            let window = row.get("window")?;
            let span = match window
                .get("timeUnit")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "TIME_UNIT_MINUTE" => 60,
                "TIME_UNIT_HOUR" => HOUR,
                "TIME_UNIT_DAY" => 24 * HOUR,
                "TIME_UNIT_WEEK" => WEEK,
                "TIME_UNIT_MONTH" => MONTH,
                _ => 0,
            };
            let duration = number(window.get("duration")).unwrap_or(0.0) as u64;
            counted(
                row.get("detail")
                    .filter(|value| value.is_object())
                    .unwrap_or(window),
                span.saturating_mul(duration),
            )
        })
        .collect();
    if let Some(window) = response.get("usage").and_then(|row| counted(row, WEEK)) {
        windows.push(window);
    }
    if let Some(usages) = response.get("usages") {
        let mut shares = Vec::new();
        for (snake, camel, span) in [
            ("limit_5h", "limit5h", 5 * HOUR),
            ("limit_7d", "limit7d", WEEK),
            ("limit_month_total", "limitMonthTotal", MONTH),
            ("limit_month_code", "limitMonthCode", MONTH),
        ] {
            let Some(row) = usages.get(snake).or_else(|| usages.get(camel)) else {
                continue;
            };
            let Some(ratio) =
                number(row.get("used_ratio")).or_else(|| number(row.get("usedRatio")))
            else {
                continue;
            };
            if shares.contains(&span) {
                continue;
            }
            shares.push(span);
            windows.retain(|window| window.duration_seconds != Some(span));
            windows.push(Window::percent(
                duration_name(span),
                ratio * 100.0,
                span,
                reset_at(row),
            ));
        }
    }
    ensure!(
        !windows.is_empty(),
        "Kimi Code did not return usage windows"
    );
    windows.sort_by_key(|window| window.duration_seconds.unwrap_or(u64::MAX));
    if let Some(wallet) = response
        .get("booster_wallet")
        .or_else(|| response.get("boosterWallet"))
        && let Some(balance) = wallet.get("balance")
        && balance.get("type").and_then(Value::as_str) == Some("BOOSTER")
        && number(balance.get("amount")).is_some_and(|n| n > 0.0)
    {
        let left = (number(balance.get("amountLeft")).unwrap_or(0.0) / 1e6).round() / 100.0;
        let unit = wallet
            .pointer("/monthlyChargeLimit/currency")
            .or_else(|| wallet.pointer("/monthlyUsed/currency"))
            .and_then(Value::as_str)
            .unwrap_or("USD");
        if left > 0.0 {
            windows.iter_mut().for_each(|window| window.aside = true);
        }
        windows.push(Window::balance("Extra usage", left, unit));
    }
    Ok(windows)
}

fn command_code(response: &Value) -> Quota {
    let mut windows: Vec<_> = [
        ("fiveHour", "5 hours", 5 * HOUR),
        ("weekly", "7 days", WEEK),
    ]
    .into_iter()
    .filter_map(|(field, name, span)| {
        let row = response.get("windowLimits")?.get(field)?;
        let used = number(row.get("used"))?;
        let cap = number(row.get("cap")).filter(|n| *n > 0.0)?;
        Some(Window {
            used: Some(used),
            limit: Some(cap),
            ..Window::percent(name, used / cap * 100.0, span, reset(row.get("resetAt")))
        })
    })
    .collect();
    let Some(credits) = response.get("credits") else {
        return (None, windows);
    };
    for (field, name, unit) in [
        ("monthlyCredits", "Monthly credits", "credits"),
        ("purchasedCredits", "Purchased credits", "USD"),
        ("freeCredits", "Free credits", "USD"),
    ] {
        if let Some(left) = number(credits.get(field)) {
            windows.push(Window::balance(name, left, unit));
        }
    }
    (text(credits.get("planId")), windows)
}

fn minimax(response: &Value) -> Result<Vec<Window>> {
    let status = response
        .pointer("/base_resp/status_code")
        .and_then(Value::as_i64)
        .context("MiniMax did not return a plan status")?;
    ensure!(
        status == 0,
        "MiniMax rejected the quota request (code {status})"
    );
    let mut windows = Vec::new();
    for row in values(response, "model_remains") {
        let Some(bucket) = text(row.get("model_name")) else {
            continue;
        };
        for (prefix, time_prefix, fallback_span) in [
            ("current_interval", "", 0),
            ("current_weekly", "weekly_", WEEK),
        ] {
            let field = |name| row.get(&format!("{prefix}_{name}"));
            let status = field("status").and_then(Value::as_u64);
            let left = number(field("remaining_percent"));
            let percent = match (status, left) {
                (Some(3), _) => continue,
                (Some(2), _) => 100.0,
                (_, Some(left)) => 100.0 - left,
                _ => continue,
            };
            let end = reset(row.get(&format!("{time_prefix}end_time")));
            let span = end
                .zip(reset(row.get(&format!("{time_prefix}start_time"))))
                .and_then(|(end, start)| u64::try_from((end - start).whole_seconds()).ok())
                .filter(|span| *span > 0)
                .unwrap_or(fallback_span);
            let mut window = Window::percent(duration_name(span), percent, span, end);
            if !bucket.eq_ignore_ascii_case("general") {
                window.name = format!("{bucket} · {}", window.name);
                window.aside = true;
            }
            if let (Some(total), Some(count), Some(left)) = (
                number(field("total_count")),
                number(field("usage_count")),
                left,
            ) && total > 0.0
                && (0.0..=total).contains(&count)
            {
                let as_used = ((total - count) / total * 100.0 - left).abs();
                let as_left = (count / total * 100.0 - left).abs();
                if as_used.min(as_left) <= 1.0 {
                    window.limit = Some(total);
                    window.used = Some(if status == Some(2) {
                        total
                    } else if as_used < as_left {
                        count
                    } else {
                        total - count
                    });
                    if bucket.eq_ignore_ascii_case("video") {
                        window.unit = Some("videos".into());
                    }
                }
            }
            windows.push(window);
        }
    }
    Ok(windows)
}

fn ollama(response: &Value) -> Vec<Window> {
    let bought = number(response.pointer("/purchased/balance_usd"));
    let aside = bought.is_some_and(|amount| amount > 0.0);
    let mut windows = Vec::new();
    if let Some(included) = response.get("included") {
        for (field, name, span) in [("session", "Session limit", 0), ("weekly", "7 days", WEEK)] {
            if let Some(row) = included.get(field)
                && let Some(left) = number(row.get("remaining_percent"))
            {
                let mut window =
                    Window::percent(name, 100.0 - left, span, reset(row.get("resets_at")));
                window.aside = aside;
                windows.push(window);
            }
        }
        if let (Some(limit), Some(left)) = (
            number(included.get("allowance_usd")),
            number(included.get("balance_usd")),
        ) && limit > 0.0
        {
            let end = reset(included.pointer("/period/until"));
            let span = end
                .zip(reset(included.pointer("/period/from")))
                .and_then(|(end, start)| u64::try_from((end - start).whole_seconds()).ok())
                .unwrap_or_default();
            let name = if (27..=32).contains(&(span / (24 * HOUR))) {
                "Month"
            } else {
                "Allowance"
            };
            let used = (limit - left).clamp(0.0, limit);
            let mut window = Window::percent(name, used / limit * 100.0, span, end);
            window.limit = Some(limit);
            window.used = Some(used);
            window.unit = Some("USD".into());
            window.aside = aside;
            windows.push(window);
        }
    }
    if let Some(left) = bought {
        windows.push(Window::balance("Purchased credits", left, "USD"));
    }
    windows
}

pub(super) fn mistral(budget: &Value, subscription: &Value, kind: &str) -> Quota {
    let (name, field, payg_field) = if kind == "api" {
        ("API · Month", "api_budget", "payg_enabled")
    } else {
        ("Month", "vibe_budget", "vibe_payg_enabled")
    };
    let plan =
        text(subscription.get("plan_name")).map(|plan| match plan.to_ascii_uppercase().as_str() {
            "FREE" => "Free".into(),
            "INDIVIDUAL" => "Pro".into(),
            "EDU" => "Student".into(),
            "TEAM" => "Team".into(),
            _ => plan,
        });
    let Some(row) = budget.get(field) else {
        return (plan, Vec::new());
    };
    let (Some(limit), Some(percent)) = (
        number(row.get("initial_budget")),
        number(row.get("usage_percentage")),
    ) else {
        return (plan, Vec::new());
    };
    if limit <= 0.0 {
        return (plan, Vec::new());
    }
    let mut window = Window::percent(name, percent, 0, reset(row.get("reset_at")));
    window.limit = Some(limit);
    let used = limit * (percent.max(0.0) / 100.0);
    window.used = used.is_finite().then_some(used);
    window.unit = text(row.get("currency")).map(|unit| unit.to_ascii_uppercase());
    window.aside = row
        .get("payg_enabled")
        .and_then(Value::as_bool)
        .or_else(|| subscription.get(payg_field).and_then(Value::as_bool))
        .unwrap_or(false);
    (plan, vec![window])
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::json;

    #[test]
    fn plan_windows_preserve_vendor_meaning_and_absent_counts() -> Result<()> {
        let (_, go) = parse(
            Plan::OpenCodeGo,
            &json!({"usage": {
                "rolling": {"percent": 0, "resetsAt": "2026-11-29T12:00:00Z"},
                "weekly": {"percent": 100, "resetsAt": "2026-11-30T12:00:00Z"}
            }}),
        )?;
        assert_eq!(go.len(), 2);
        assert!(go[0].resets_at.is_none());
        assert_eq!(go[1].used_percent, Some(100.0));
        assert!(go[1].resets_at.is_some());

        let (plan, glm) = parse(
            Plan::Zhipu,
            &json!({"success": true, "data": {"level": "pro", "limits": [
                {"type": "TOKENS_LIMIT", "unit": 3, "nextResetTime": 1795970000000_u64},
                {"type": "TIME_LIMIT", "percentage": 16}
            ]}}),
        )?;
        assert_eq!(plan.as_deref(), Some("pro"));
        assert_eq!(glm[0].duration_seconds, Some(5 * HOUR));
        assert!(glm[0].used_percent.is_none());
        assert!(glm[1].aside);

        let (_, kimi) = parse(
            Plan::Kimi,
            &json!({
                "usage": {"limit": "100", "used": "25"},
                "usages": {"limit7d": {"usedRatio": 0.5}, "limit_month_total": {"used_ratio": 0.25}, "limit_month_code": {"used_ratio": 0.75}},
                "boosterWallet": {"balance": {"type": "BOOSTER", "amount": 1000000000, "amountLeft": 125000000}, "monthlyUsed": {"currency": "CNY"}}
            }),
        )?;
        assert_eq!(kimi.len(), 3);
        assert_eq!(kimi[0].used_percent, Some(50.0));
        assert_eq!(kimi[1].used_percent, Some(25.0));
        assert!(kimi[0].aside);
        assert_eq!(kimi[2].balance, Some(1.25));
        assert_eq!(kimi[2].unit.as_deref(), Some("CNY"));

        let (plan, command) = parse(
            Plan::CommandCode,
            &json!({
                "credits": {"planId": "individual-goat-monthly", "monthlyCredits": "4.5", "purchasedCredits": 2},
                "windowLimits": {"fiveHour": {"used": 4, "cap": 8, "resetAt": 1795970000000_u64}}
            }),
        )?;
        assert_eq!(plan.as_deref(), Some("individual-goat-monthly"));
        assert_eq!(command[0].used_percent, Some(50.0));
        assert_eq!(
            (command[1].balance, command[1].unit.as_deref()),
            (Some(4.5), Some("credits"))
        );
        assert_eq!(
            (command[2].balance, command[2].unit.as_deref()),
            (Some(2.0), Some("USD"))
        );

        let (_, mini) = parse(
            Plan::MiniMax,
            &json!({"base_resp": {"status_code": 0}, "model_remains": [{
            "model_name": "video", "current_interval_status": 1, "current_interval_remaining_percent": 80,
            "current_interval_total_count": 5, "current_interval_usage_count": 4, "current_weekly_status": 2
        }, {"model_name":"image", "current_interval_status": 3, "current_weekly_status": 3}]}),
        )?;
        assert_eq!(mini.len(), 2);
        assert_eq!(mini[0].used, Some(1.0));
        assert_eq!(mini[0].limit, Some(5.0));
        assert_eq!(mini[1].used_percent, Some(100.0));
        assert!(mini[0].aside);

        let (_, ollama) = parse(
            Plan::Ollama,
            &json!({"included": {
            "session": {"remaining_percent": "25"}, "allowance_usd": 50, "balance_usd": 25,
            "period": {"from":"2026-10-30T00:00:00Z", "until":"2026-11-30T00:00:00Z"}
        }, "purchased": {"balance_usd": 5}}),
        )?;
        assert_eq!(ollama.len(), 3);
        assert_eq!(ollama[0].used_percent, Some(75.0));
        assert!(ollama[1].aside);
        assert_eq!(ollama[1].name, "Month");
        assert_eq!(ollama[1].used, Some(25.0));

        let (plan, mistral) = mistral(
            &json!({
                "vibe_budget": {"initial_budget": 150, "usage_percentage": 95, "currency": "eur"},
                "api_budget": {"initial_budget": 20, "usage_percentage": 10, "currency": "eur"}
            }),
            &json!({"plan_name": "INDIVIDUAL", "payg_enabled": true}),
            "api",
        );
        assert_eq!(plan.as_deref(), Some("Pro"));
        assert_eq!(mistral.len(), 1);
        assert_eq!(mistral[0].used, Some(2.0));
        assert!(mistral[0].aside);
        Ok(())
    }
}
