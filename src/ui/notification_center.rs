use crate::{app::MonitorController, domain::NotificationMessage};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    scroll::ScrollableElement,
    ActiveTheme, Icon, Sizable, StyledExt, Theme, TitleBar,
};
use gpui_kit::prelude::{FluentBuilder, InteractiveElement};
use gpui_kit::*;
use std::sync::Arc;

pub(crate) struct NotificationCenterView {
    controller: Entity<MonitorController>,
    messages: Vec<NotificationMessage>,
    rows: Arc<Vec<NotificationRow>>,
    selected_id: Option<String>,
    loading: bool,
    marking_id: Option<String>,
    error: Option<String>,
    scroll_handle: UniformListScrollHandle,
    focus_handle: FocusHandle,
    did_focus: bool,
}

impl NotificationCenterView {
    pub(crate) fn new(controller: Entity<MonitorController>, cx: &mut Context<Self>) -> Self {
        let mut view = Self {
            controller,
            messages: Vec::new(),
            rows: Arc::new(Vec::new()),
            selected_id: None,
            loading: false,
            marking_id: None,
            error: None,
            scroll_handle: UniformListScrollHandle::new(),
            focus_handle: cx.focus_handle(),
            did_focus: false,
        };
        view.load(cx);
        view
    }

    fn load(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        let receiver = self.controller.read(cx).fetch_notifications();
        self.loading = true;
        self.error = None;
        cx.notify();
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            let result = receiver.await.unwrap_or_else(|_| {
                Err("The authenticated API session is no longer available.".into())
            });
            let _ = entity.update(cx, |view, cx| {
                view.loading = false;
                match result {
                    Ok(messages) => {
                        if !view
                            .selected_id
                            .as_ref()
                            .is_some_and(|id| messages.iter().any(|message| &message.id == id))
                        {
                            view.selected_id = messages.first().map(|message| message.id.clone());
                        }
                        view.rows = Arc::new(messages.iter().map(NotificationRow::from).collect());
                        view.messages = messages;
                        view.error = None;
                    }
                    Err(error) => view.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn navigate(&mut self, direction: isize, cx: &mut Context<Self>) {
        if self.messages.is_empty() {
            return;
        }
        let current = self
            .selected_id
            .as_ref()
            .and_then(|id| self.messages.iter().position(|message| &message.id == id));
        let next = match current {
            Some(index) => (index as isize + direction)
                .clamp(0, self.messages.len().saturating_sub(1) as isize)
                as usize,
            None if direction > 0 => 0,
            None => self.messages.len() - 1,
        };
        self.selected_id = Some(self.messages[next].id.clone());
        self.scroll_handle
            .scroll_to_item(next, ScrollStrategy::Nearest);
        cx.notify();
    }

    fn mark_read(&mut self, id: String, notice_type: i64, cx: &mut Context<Self>) {
        if self.marking_id.is_some()
            || !self
                .messages
                .iter()
                .any(|message| message.id == id && message.status == Some(0))
        {
            return;
        }
        let receiver = self
            .controller
            .read(cx)
            .mark_notification_read(id.clone(), notice_type);
        self.marking_id = Some(id.clone());
        self.error = None;
        cx.notify();
        let entity = cx.entity().clone();
        let controller = self.controller.clone();
        cx.spawn(async move |_, cx| {
            let result = receiver.await.unwrap_or_else(|_| {
                Err("The authenticated API session is no longer available.".into())
            });
            let _ = entity.update(cx, |view, cx| {
                view.marking_id = None;
                match result {
                    Ok(()) => {
                        let was_unread = view
                            .messages
                            .iter()
                            .any(|message| message.id == id && message.status == Some(0));
                        if let Some(message) = view.messages.iter_mut().find(|m| m.id == id) {
                            message.status = Some(1);
                        }
                        view.rows =
                            Arc::new(view.messages.iter().map(NotificationRow::from).collect());
                        if was_unread {
                            let _ = controller.update(cx, |controller, cx| {
                                controller.notification_count = Some(
                                    controller
                                        .notification_count
                                        .map(|count| count.saturating_sub(1))
                                        .unwrap_or_default(),
                                );
                                cx.notify();
                            });
                        }
                        view.error = None;
                    }
                    Err(error) => view.error = Some(error),
                }
                cx.notify();
            });
        })
        .detach();
    }
}

#[derive(Clone)]
struct NotificationRow {
    id: String,
    title: String,
    description: String,
    received: String,
    unread: bool,
}

impl From<&NotificationMessage> for NotificationRow {
    fn from(message: &NotificationMessage) -> Self {
        Self {
            id: message.id.clone(),
            title: message.display_title(),
            description: message.display_description(),
            received: message.received_at(),
            unread: message.status == Some(0),
        }
    }
}

impl Render for NotificationCenterView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.did_focus {
            self.did_focus = true;
            let focus_handle = self.focus_handle.clone();
            cx.defer_in(window, move |_, window, cx| focus_handle.focus(window, cx));
        }
        let theme = cx.theme();
        let selected = self
            .messages
            .iter()
            .find(|message| Some(&message.id) == self.selected_id.as_ref());
        let mut content = div().v_flex().flex_1().min_h(px(0.)).gap_3().p_4().child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Notifications"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child("Your latest SunSynk messages"),
                        ),
                )
                .child(div().flex_1())
                .child(
                    Button::new("reload-notifications")
                        .icon(Icon::empty().path("icons/refresh-cw.svg"))
                        .accessibility_label("Refresh notifications")
                        .tooltip("Refresh notifications")
                        .ghost()
                        .xsmall()
                        .loading(self.loading)
                        .on_click(cx.listener(|view, _, _, cx| view.load(cx))),
                ),
        );

        if self.loading && self.messages.is_empty() {
            content = content.child(
                div()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Loading notifications…"),
            );
        } else if let Some(error) = self.error.as_ref().filter(|_| self.messages.is_empty()) {
            content = content.child(
                div()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.danger)
                    .child(error.clone()),
            );
        } else if self.messages.is_empty() {
            content = content.child(
                div()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("No notifications"),
            );
        } else {
            if let Some(error) = &self.error {
                content = content.child(
                    div()
                        .text_sm()
                        .text_color(theme.danger)
                        .child(error.clone()),
                );
            }
            let selected_id = self.selected_id.clone();
            let rows = self.rows.clone();
            let row_entity = cx.weak_entity();
            let row_muted = theme.muted;
            let row_background = theme.background;
            let row_border = theme.border;
            let row_radius = theme.radius;
            let row_muted_foreground = theme.muted_foreground;
            let list = uniform_list(
                "notification-message-list",
                rows.len(),
                move |range, _, _| {
                    range
                        .map(|index| {
                            let row = &rows[index];
                            let row_id = row.id.clone();
                            let click_entity = row_entity.clone();
                            let message_id = row.id.clone();
                            let is_selected = selected_id.as_deref() == Some(row_id.as_str());
                            let is_unread = row.unread;
                            div()
                                .id(format!("notification-row-{row_id}"))
                                .v_flex()
                                .h(px(64.))
                                .justify_center()
                                .gap_1()
                                .w_full()
                                .px_3()
                                .py_2()
                                .rounded(row_radius)
                                .bg(if is_selected {
                                    row_muted
                                } else {
                                    row_background
                                })
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    let _ = click_entity.update(cx, |view, cx| {
                                        view.selected_id = Some(message_id.clone());
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .h_flex()
                                        .items_center()
                                        .gap_2()
                                        .child(if is_unread {
                                            div().size(px(7.)).rounded_full().bg(rgb(0x34c759))
                                        } else {
                                            div().size(px(7.))
                                        })
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_sm()
                                                .text_ellipsis()
                                                .font_weight(if is_unread {
                                                    FontWeight::SEMIBOLD
                                                } else {
                                                    FontWeight::NORMAL
                                                })
                                                .child(row.title.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(row_muted_foreground)
                                                .child(row.received.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .pl_4()
                                        .text_xs()
                                        .text_color(row_muted_foreground)
                                        .text_ellipsis()
                                        .child(row.description.clone()),
                                )
                                .border_b_1()
                                .border_color(row_border)
                        })
                        .collect::<Vec<_>>()
                },
            )
            .track_scroll(&self.scroll_handle)
            .flex_1()
            .min_h(px(0.));

            let mut detail = div()
                .v_flex()
                .flex_1()
                .min_h(px(0.))
                .h_full()
                .overflow_y_scrollbar()
                .gap_3()
                .px_4()
                .py_2();
            if let Some(message) = selected {
                let selected_id = message.id.clone();
                let selected_type = message.notice_type;
                let is_unread = message.status == Some(0);
                detail = detail
                    .child(
                        div()
                            .h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .text_xl()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(message.display_title()),
                            )
                            .when(is_unread && selected_type.is_some(), |header| {
                                header.child(
                                    Button::new("mark-notification-read")
                                        .label("Mark as read")
                                        .ghost()
                                        .xsmall()
                                        .loading(self.marking_id.as_deref() == Some(&selected_id))
                                        .on_click(cx.listener(move |view, _, _, cx| {
                                            if let Some(notice_type) = selected_type {
                                                view.mark_read(
                                                    selected_id.clone(),
                                                    notice_type,
                                                    cx,
                                                );
                                            }
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(status_chip(message.status, theme))
                            .child(type_chip(message.message_type, message.notice_type, theme)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.foreground)
                            .child(message.display_description()),
                    )
                    .child(div().h(px(1.)).w_full().bg(theme.border))
                    .child(metadata_row(
                        "Plant",
                        message.station_name.as_deref(),
                        theme,
                    ))
                    .child(metadata_row(
                        "Inverter serial",
                        nonempty(Some(&message.sn)),
                        theme,
                    ))
                    .child(metadata_row(
                        "Received",
                        Some(&message.received_at()),
                        theme,
                    ))
                    .child(metadata_row("Updated", message.update_at.as_deref(), theme));
                if let Some(soc) = message.soc {
                    detail = detail.child(metadata_row(
                        "Battery charge",
                        Some(&format!("{soc}%")),
                        theme,
                    ));
                }
            } else {
                detail = detail
                    .items_center()
                    .justify_center()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Select a notification to view its details");
            }

            content = content.child(
                div()
                    .h_flex()
                    .flex_1()
                    .min_h(px(0.))
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius)
                    .child(
                        div()
                            .v_flex()
                            .w(px(340.))
                            .flex_none()
                            .min_h(px(0.))
                            .h_full()
                            .border_r_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{} messages", self.messages.len())),
                            )
                            .child(list),
                    )
                    .child(detail),
            );
        }

        div()
            .v_flex()
            .size_full()
            .min_h(px(0.))
            .bg(theme.background)
            .text_color(theme.foreground)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "up" => view.navigate(-1, cx),
                    "down" => view.navigate(1, cx),
                    _ => {}
                }
            }))
            .child(TitleBar::new())
            .child(content)
    }
}

impl NotificationMessage {
    fn display_description(&self) -> String {
        let source = self
            .description
            .as_deref()
            .filter(|text| !text.is_empty())
            .or(self.notice_description.as_deref())
            .or(self.notice_title.as_deref())
            .unwrap_or("Notification details are unavailable.");
        let mut description = source.replace(
            "#{stationName}",
            self.station_name.as_deref().unwrap_or("your plant"),
        );
        description = description.replace(")power", ") power");
        description
    }

    fn display_title(&self) -> String {
        if let Some(title) = nonempty(self.notice_title.as_deref()) {
            return title.to_owned();
        }
        let description = self.display_description();
        let normalized = description.to_lowercase();
        if normalized.contains("power grid has been restored") {
            "Grid restored".into()
        } else if normalized.contains("disconnected from the grid") {
            "Grid disconnected".into()
        } else {
            description
        }
    }

    fn received_at(&self) -> String {
        self.create_at
            .as_deref()
            .and_then(|time| chrono::DateTime::parse_from_rfc3339(time).ok())
            .map(|time| {
                time.with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .or_else(|| {
                self.time
                    .and_then(chrono::DateTime::from_timestamp_millis)
                    .map(|time| {
                        time.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d %H:%M")
                            .to_string()
                    })
            })
            .unwrap_or_else(|| "Unknown time".into())
    }
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.trim().is_empty())
}

fn metadata_row(label: &str, value: Option<&str>, theme: &Theme) -> impl IntoElement {
    div()
        .h_flex()
        .gap_3()
        .child(
            div()
                .w(px(120.))
                .flex_none()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(label.to_owned()),
        )
        .child(div().text_sm().child(value.unwrap_or("—").to_owned()))
}

fn status_chip(status: Option<i64>, theme: &Theme) -> impl IntoElement {
    let (label, color) = match status {
        Some(0) => ("Unread", rgb(0x34c759).into()),
        Some(1) => ("Read", theme.muted_foreground),
        _ => ("Unknown status", theme.muted_foreground),
    };
    div()
        .px_2()
        .py_1()
        .rounded(theme.radius)
        .bg(theme.muted)
        .text_xs()
        .text_color(color)
        .child(label)
}

fn type_chip(
    message_type: Option<i64>,
    notice_type: Option<i64>,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .px_2()
        .py_1()
        .rounded(theme.radius)
        .bg(theme.muted)
        .text_xs()
        .text_color(theme.muted_foreground)
        .child(match (message_type, notice_type) {
            (Some(message_type), Some(notice_type)) => {
                format!("Message {message_type} · Event {notice_type}")
            }
            (Some(message_type), None) => format!("Message type {message_type}"),
            (None, Some(notice_type)) => format!("Event type {notice_type}"),
            (None, None) => "Notification".into(),
        })
}
