use crate::app::MonitorController;
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::{Input, InputState},
    scroll::ScrollableElement,
    ActiveTheme, Disableable, Sizable, StyledExt, TitleBar,
};
use gpui_kit::*;
use serde_json::Value;

struct ApiResponse {
    status: u16,
    api_success: Option<bool>,
    body: String,
}

pub(crate) struct ApiInspectorView {
    controller: Entity<MonitorController>,
    path: Entity<InputState>,
    params: Entity<InputState>,
    response: Option<ApiResponse>,
    error: Option<String>,
    loading: bool,
}

impl ApiInspectorView {
    pub(crate) fn new(
        controller: Entity<MonitorController>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let serial = {
            let controller = controller.read(cx);
            controller.selected_serial.clone()
        };
        let path = cx.new(|cx| {
            InputState::new(window, cx).default_value(match serial.as_deref() {
                Some(serial) => format!("/api/v1/inverter/grid/{serial}/realtime"),
                None => "/api/v1/".into(),
            })
        });
        let params = cx.new(|cx| {
            InputState::new(window, cx).default_value(match serial.as_deref() {
                Some(serial) => format!("sn={serial}"),
                None => String::new(),
            })
        });
        Self {
            controller,
            path,
            params,
            response: None,
            error: None,
            loading: false,
        }
    }

    fn send_request(&mut self, cx: &mut Context<Self>) {
        if self.loading {
            return;
        }
        let path = self.path.read(cx).value().trim().to_string();
        let params_text = self.params.read(cx).value().to_string();
        let params = match parse_query_params(&params_text) {
            Ok(params) => params,
            Err(error) => {
                self.error = Some(error);
                self.response = None;
                cx.notify();
                return;
            }
        };
        if let Err(error) = validate_path(&path) {
            self.error = Some(error);
            self.response = None;
            cx.notify();
            return;
        }

        let receiver = self.controller.read(cx).inspect_api_endpoint(path, params);
        self.error = None;
        self.response = None;
        self.loading = true;
        cx.notify();
        let entity = cx.entity().clone();
        cx.spawn(async move |_, cx| {
            let result = receiver.await.unwrap_or_else(|_| {
                Err("The connected API session is no longer available.".into())
            });
            let _ = entity.update(cx, |view, cx| {
                view.loading = false;
                match result {
                    Ok((status, value)) => {
                        view.response = Some(ApiResponse {
                            status,
                            api_success: value.get("success").and_then(Value::as_bool),
                            body: serde_json::to_string_pretty(&value)
                                .unwrap_or_else(|error| format!("Could not format JSON: {error}")),
                        });
                        view.error = None;
                    }
                    Err(error) => {
                        view.response = None;
                        view.error = Some(error);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for ApiInspectorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let response_height = px((window.bounds().size.height.as_f32() - 450.).max(130.));
        let response_body_height = px((response_height.as_f32() - 44.).max(80.));
        let (serial, plant_id) = {
            let controller = self.controller.read(cx);
            let plant_id = controller.selected_serial.as_ref().and_then(|serial| {
                controller
                    .inverters
                    .iter()
                    .find(|inverter| &inverter.serial == serial)
                    .and_then(|inverter| inverter.plant_id)
            });
            (controller.selected_serial.clone(), plant_id)
        };
        let grid_path = serial
            .as_ref()
            .map(|serial| format!("/api/v1/inverter/grid/{serial}/realtime"))
            .unwrap_or_else(|| "/api/v1/inverter/grid/{serial}/realtime".into());
        let grid_params = serial
            .as_ref()
            .map(|serial| format!("sn={serial}"))
            .unwrap_or_else(|| "sn={serial}".into());
        let flow_path = plant_id
            .map(|id| format!("/api/v1/plant/energy/{id}/flow"))
            .unwrap_or_else(|| "/api/v1/plant/energy/{plant_id}/flow".into());
        let flow_params = plant_id
            .map(|id| format!("date={}&id={id}", chrono::Local::now().date_naive()))
            .unwrap_or_else(|| "date=YYYY-MM-DD&id={plant_id}".into());
        let plant_path = plant_id
            .map(|id| format!("/api/v1/plant/{id}/realtime"))
            .unwrap_or_else(|| "/api/v1/plant/{plant_id}/realtime".into());
        let plant_params = plant_id
            .map(|id| format!("id={id}"))
            .unwrap_or_else(|| "id={plant_id}".into());

        let path = self.path.clone();
        let params = self.params.clone();
        let grid_path_for_button = grid_path.clone();
        let grid_params_for_button = grid_params.clone();
        let mut request_card = div()
            .v_flex()
            .gap_3()
            .p_3()
            .rounded(theme.radius)
            .bg(theme.muted)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Request"),
            )
            .child(
                div()
                    .h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(preset_button(
                        "Grid realtime",
                        "preset-grid",
                        move |window, cx| {
                            path.update(cx, |state, cx| {
                                state.set_value(grid_path_for_button.clone(), window, cx)
                            });
                            params.update(cx, |state, cx| {
                                state.set_value(grid_params_for_button.clone(), window, cx)
                            });
                        },
                    ))
                    .child({
                        let path = self.path.clone();
                        let params = self.params.clone();
                        let flow_path = flow_path.clone();
                        let flow_params = flow_params.clone();
                        preset_button("Plant flow", "preset-flow", move |window, cx| {
                            path.update(cx, |state, cx| {
                                state.set_value(flow_path.clone(), window, cx)
                            });
                            params.update(cx, |state, cx| {
                                state.set_value(flow_params.clone(), window, cx)
                            });
                        })
                    })
                    .child({
                        let path = self.path.clone();
                        let params = self.params.clone();
                        let plant_path = plant_path.clone();
                        let plant_params = plant_params.clone();
                        preset_button("Plant realtime", "preset-plant", move |window, cx| {
                            path.update(cx, |state, cx| {
                                state.set_value(plant_path.clone(), window, cx)
                            });
                            params.update(cx, |state, cx| {
                                state.set_value(plant_params.clone(), window, cx)
                            });
                        })
                    })
                    .child({
                        let path = self.path.clone();
                        let params = self.params.clone();
                        preset_button("Custom GET", "preset-custom", move |window, cx| {
                            path.update(cx, |state, cx| state.set_value("/api/v1/", window, cx));
                            params.update(cx, |state, cx| state.set_value("", window, cx));
                        })
                    }),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("GET endpoint path"),
            )
            .child(Input::new(&self.path).small())
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .child("Query parameters · key=value&key=value"),
            )
            .child(Input::new(&self.params).small())
            .child(
                div().h_flex().justify_end().child(
                    Button::new("send-api-request")
                        .label(if self.loading {
                            "Requesting…"
                        } else {
                            "Send request"
                        })
                        .primary()
                        .small()
                        .disabled(self.loading)
                        .on_click(cx.listener(|view, _, _, cx| view.send_request(cx))),
                ),
            );

        if serial.is_none() || plant_id.is_none() {
            request_card =
                request_card.child(div().text_xs().text_color(theme.muted_foreground).child(
                    "Connect an account and select an inverter to use the endpoint presets.",
                ));
        }

        let response_content = if self.loading {
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("Waiting for SunSynk…")
                .into_any_element()
        } else if let Some(error) = &self.error {
            div()
                .text_sm()
                .text_color(theme.danger)
                .child(error.clone())
                .into_any_element()
        } else if let Some(response) = &self.response {
            let api_status = match response.api_success {
                Some(true) => "API success".to_owned(),
                Some(false) => "API failure".to_owned(),
                None => "API status unavailable".to_owned(),
            };
            let body = response.body.clone();
            div()
                .v_flex()
                .h(response_height)
                .gap_2()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!("HTTP {}", response.status)),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(api_status),
                        )
                        .child(div().flex_1())
                        .child(
                            Button::new("copy-api-response")
                                .label("Copy response")
                                .ghost()
                                .small()
                                .on_click(move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(body.clone()));
                                }),
                        ),
                )
                .child(
                    div()
                        .h(response_body_height)
                        .overflow_y_scrollbar()
                        .p_3()
                        .rounded(theme.radius)
                        .bg(theme.background)
                        .font_family(theme.mono_font_family.clone())
                        .text_xs()
                        .child(response.body.clone()),
                )
                .into_any_element()
        } else {
            div()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("Choose a preset or enter a GET endpoint, then send the request.")
                .into_any_element()
        };

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
                    .min_h(px(0.))
                    .p_4()
                    .gap_3()
                    .child(div().text_lg().child("API Inspector"))
                    .child(div().text_xs().text_color(theme.muted_foreground).child("GET requests use SunTray’s active authenticated session. Responses replace the previous result and are not added to the connection log."))
                    .child(request_card)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Response"),
                    )
                    .child(
                        div()
                            .v_flex()
                            .h(response_height)
                            .child(response_content),
                    ),
            )
            .into_any_element()
    }
}

fn preset_button(
    label: &'static str,
    id: &'static str,
    update: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    Button::new(id)
        .label(label)
        .ghost()
        .xsmall()
        .on_click(move |_, window, cx| update(window, cx))
}

fn validate_path(path: &str) -> Result<(), String> {
    let path = path.trim();
    let lower_path = path.to_ascii_lowercase();
    if !path.starts_with("/api/v1/")
        || path.contains(['?', '#', '\\'])
        || path.chars().any(char::is_whitespace)
        || path.split('/').any(|part| matches!(part, "." | ".."))
        || ["%2e", "%2f", "%5c", "%25"]
            .iter()
            .any(|encoded| lower_path.contains(encoded))
    {
        return Err(
            "Use a path under /api/v1/ without a query string or encoded path separators.".into(),
        );
    }
    if path.len() > 2048 {
        return Err("Endpoint path is too long.".into());
    }
    Ok(())
}

fn parse_query_params(query: &str) -> Result<Vec<(String, String)>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut params = Vec::new();
    for item in query.split('&') {
        let Some((key, value)) = item.split_once('=') else {
            return Err("Enter query parameters as key=value pairs separated by &.".into());
        };
        let key = key.trim();
        if key.is_empty() || key.chars().any(char::is_whitespace) {
            return Err("Query parameter names cannot be empty or contain spaces.".into());
        }
        params.push((key.to_owned(), value.to_owned()));
        if params.len() > 50 {
            return Err("A request can include at most 50 query parameters.".into());
        }
    }
    Ok(params)
}
