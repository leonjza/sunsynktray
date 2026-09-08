use crate::app::{ConnectionState, MonitorController};
use gpui_kit::component::{
    scroll::ScrollableElement,
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
    ActiveTheme, Sizable, StyledExt, TitleBar,
};
use gpui_kit::*;

const DATE_COLUMN_WIDTH: Pixels = px(184.);
const ID_COLUMN_WIDTH: Pixels = px(42.);
const METHOD_COLUMN_WIDTH: Pixels = px(58.);
const STATUS_COLUMN_WIDTH: Pixels = px(66.);
const DURATION_COLUMN_WIDTH: Pixels = px(76.);

pub(crate) struct ConnectionLogView {
    pub(crate) controller: Entity<MonitorController>,
    cached_revision: u64,
    rows: Vec<LogRow>,
}

impl ConnectionLogView {
    pub(crate) fn new(controller: Entity<MonitorController>, cx: &mut Context<Self>) -> Self {
        let entity = cx.weak_entity();
        let signal = controller.read(cx).connection_log_signal.clone();
        cx.spawn(async move |_, cx| loop {
            signal.notified().await;
            if entity.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        })
        .detach();
        Self {
            controller,
            cached_revision: u64::MAX,
            rows: Vec::new(),
        }
    }
}

impl Render for ConnectionLogView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let controller = self.controller.read(cx);
        let theme = cx.theme();
        let state = match &controller.connection {
            ConnectionState::Unconfigured => "Unconfigured".to_owned(),
            ConnectionState::Connecting => "Connecting".to_owned(),
            ConnectionState::Connected => "Connected".to_owned(),
            ConnectionState::Stale => "Stale · using cached data".to_owned(),
            ConnectionState::Error(error) => format!("Error · {error}"),
        };
        let revision = controller
            .connection_log_revision
            .load(std::sync::atomic::Ordering::Acquire);
        if self.cached_revision != revision {
            let connection_log = controller
                .connection_log
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            self.rows = connection_log
                .iter()
                .rev()
                .map(|entry| parse_entry(entry))
                .collect();
            self.cached_revision = revision;
        }
        let mono = theme.mono_font_family.clone();
        let mut body = TableBody::new();
        if self.rows.is_empty() {
            body = body.child(
                TableRow::new().child(
                    TableCell::new()
                        .col_span(6)
                        .text_color(theme.muted_foreground)
                        .child("No connection events recorded yet."),
                ),
            );
        } else {
            for row in &self.rows {
                let status_color = row.status_color(theme.muted_foreground, theme.danger);
                body = body.child(
                    TableRow::new()
                        .child(
                            TableCell::new()
                                .w(DATE_COLUMN_WIDTH)
                                .text_sm()
                                .font_family(mono.clone())
                                .child(row.date.clone()),
                        )
                        .child(
                            TableCell::new()
                                .w(ID_COLUMN_WIDTH)
                                .text_sm()
                                .font_family(mono.clone())
                                .child(row.id.clone()),
                        )
                        .child(
                            TableCell::new()
                                .w(METHOD_COLUMN_WIDTH)
                                .text_sm()
                                .font_family(mono.clone())
                                .child(row.method.clone()),
                        )
                        .child(
                            TableCell::new()
                                .flex_1()
                                .text_sm()
                                .font_family(mono.clone())
                                .overflow_hidden()
                                .child(row.path.clone()),
                        )
                        .child(
                            TableCell::new()
                                .w(STATUS_COLUMN_WIDTH)
                                .text_sm()
                                .font_family(mono.clone())
                                .text_color(status_color)
                                .child(row.status.clone()),
                        )
                        .child(
                            TableCell::new()
                                .w(DURATION_COLUMN_WIDTH)
                                .text_sm()
                                .font_family(mono.clone())
                                .child(row.duration.clone()),
                        ),
                );
            }
        }
        let entries = Table::new()
            .small()
            .accessibility_label("Connection events")
            .child(
                TableHeader::new().child(
                    TableRow::new()
                        .child(TableHead::new().w(DATE_COLUMN_WIDTH).child("Date"))
                        .child(TableHead::new().w(ID_COLUMN_WIDTH).child("#"))
                        .child(TableHead::new().w(METHOD_COLUMN_WIDTH).child("Method"))
                        .child(TableHead::new().flex_1().child("Path"))
                        .child(TableHead::new().w(STATUS_COLUMN_WIDTH).child("Response"))
                        .child(TableHead::new().w(DURATION_COLUMN_WIDTH).child("Time")),
                ),
            )
            .child(body);
        div()
            .v_flex()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(TitleBar::new())
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .p_4()
                    .gap_3()
                    .child(div().text_lg().child("Connection log"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(state),
                    )
                    .child(entries),
            )
            .into_any_element()
    }
}

struct LogRow {
    date: String,
    id: String,
    method: String,
    path: String,
    status: String,
    duration: String,
    status_code: Option<u16>,
}

impl LogRow {
    fn status_color(&self, muted: Hsla, danger: Hsla) -> Hsla {
        match self.status_code {
            Some(200..=299) => rgb(0x22c55e).into(),
            Some(300..=399) => rgb(0xf59e0b).into(),
            Some(400..) => danger,
            _ => muted,
        }
    }
}

fn parse_entry(entry: &str) -> LogRow {
    let (date, message) = entry
        .find("] ")
        .map(|index| {
            (
                entry[..=index - 1].trim_start_matches('['),
                &entry[index + 2..],
            )
        })
        .unwrap_or(("", entry));
    let mut parts = message.split_whitespace();
    let id = parts
        .next()
        .filter(|part| part.starts_with('#'))
        .map(|part| part.trim_start_matches('#').to_owned())
        .unwrap_or_default();
    let method = if id.is_empty() {
        message.split_whitespace().next().unwrap_or("").to_owned()
    } else {
        parts.next().unwrap_or("").to_owned()
    };
    let path = parts.next().unwrap_or("").to_owned();
    let remainder = parts.collect::<Vec<_>>().join(" ");
    let status_code = remainder
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find_map(|window| (window[0] == "with").then_some(window[1]))
        .and_then(|value| value.parse::<u16>().ok());
    let status = status_code
        .map(|code| code.to_string())
        .unwrap_or_else(|| "—".into());
    let duration = remainder
        .split(" after ")
        .nth(1)
        .and_then(|value| value.split_whitespace().next())
        .map(|value| value.to_owned())
        .unwrap_or_else(|| "—".into());
    LogRow {
        date: date.to_owned(),
        id,
        method,
        path,
        status,
        duration,
        status_code,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_entry;

    #[test]
    fn parses_successful_request_columns() {
        let row = parse_entry(
            "[2026-09-08 14:05:43] #14 GET /api/v1/inverters succeeded with 200 OK after 1046ms",
        );
        assert_eq!(row.date, "2026-09-08 14:05:43");
        assert_eq!(row.id, "14");
        assert_eq!(row.method, "GET");
        assert_eq!(row.path, "/api/v1/inverters");
        assert_eq!(row.status, "200");
        assert_eq!(row.duration, "1046ms");
    }

    #[test]
    fn parses_failed_requests() {
        let failed = parse_entry(
            "[2026-09-08 14:05:43] #15 POST /oauth/token/new failed with 401 after 23ms",
        );
        assert_eq!(failed.status, "401");
        assert_eq!(failed.duration, "23ms");
    }
}
