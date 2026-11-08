//! 专注计时 · egui 最小桌面组件
//!
//! 这个示例想证明两件事：
//! 1. 不用 HTML/CSS，Rust 直接画界面也能做得好看
//! 2. 空闲时真的可以完全不重绘（低功耗的关键）
//!
//! 运行：cargo run --release

use std::time::{Duration, Instant};

use eframe::egui;

const BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
const TXT: egui::Color32 = egui::Color32::from_rgb(208, 208, 208);
const DIM: egui::Color32 = egui::Color32::from_rgb(138, 138, 138);
const TRACK: egui::Color32 = egui::Color32::from_rgb(52, 52, 52);
const ACCENT_WORK: egui::Color32 = egui::Color32::from_rgb(106, 169, 233);
const ACCENT_BREAK: egui::Color32 = egui::Color32::from_rgb(127, 201, 160);

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Work,
    Break,
}

struct TimerApp {
    mode: Mode,
    running: bool,
    remaining: f32,
    work_min: f32,
    break_min: f32,
    done: u32,
    frames: u32,
    fps: f32,
    last_stat: Instant,
    idle_marked: bool,
}

impl TimerApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut v = egui::Visuals::dark();
        v.panel_fill = BG;
        v.override_text_color = Some(TXT);
        v.widgets.inactive.bg_fill = egui::Color32::from_rgb(46, 46, 46);
        v.widgets.hovered.bg_fill = egui::Color32::from_rgb(58, 58, 58);
        v.widgets.active.bg_fill = egui::Color32::from_rgb(72, 72, 72);
        cc.egui_ctx.set_visuals(v);

        Self {
            mode: Mode::Work,
            running: false,
            remaining: 25.0 * 60.0,
            work_min: 25.0,
            break_min: 5.0,
            done: 0,
            frames: 0,
            fps: 0.0,
            last_stat: Instant::now(),
            idle_marked: false,
        }
    }

    fn total(&self) -> f32 {
        match self.mode {
            Mode::Work => self.work_min * 60.0,
            Mode::Break => self.break_min * 60.0,
        }
    }

    fn accent(&self) -> egui::Color32 {
        match self.mode {
            Mode::Work => ACCENT_WORK,
            Mode::Break => ACCENT_BREAK,
        }
    }

    fn reset(&mut self) {
        self.running = false;
        self.remaining = self.total();
        self.idle_marked = false;
    }

    fn finish_round(&mut self) {
        if self.mode == Mode::Work {
            self.done += 1;
            self.mode = Mode::Break;
        } else {
            self.mode = Mode::Work;
        }
        self.remaining = self.total();
        self.running = false;
    }
}

impl eframe::App for TimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt).min(0.25);

        if self.running {
            self.remaining = (self.remaining - dt).max(0.0);
            if self.remaining <= 0.0 {
                self.finish_round();
            }
            // 只有运行中才请求重绘；停下后不再调度，egui 自动进入休眠
            ctx.request_repaint();
        } else {
            self.fps = 0.0;
            if !self.idle_marked {
                // 停下来的那一刻再画一帧，把“已休眠”显示出来，然后彻底不动
                self.idle_marked = true;
                ctx.request_repaint_after(Duration::from_millis(700));
            }
        }

        self.frames += 1;
        let elapsed = self.last_stat.elapsed().as_secs_f32();
        if elapsed >= 1.0 {
            if self.running {
                self.fps = self.frames as f32 / elapsed;
            }
            self.frames = 0;
            self.last_stat = Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.add_space(4.0);

            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("低功耗组件 · egui 演示").size(11.0).color(DIM));

                let size = 232.0;
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
                let center = rect.center();
                let radius = size * 0.44;
                let painter = ui.painter();

                painter.circle_stroke(center, radius, egui::Stroke::new(10.0, TRACK));

                let total = self.total().max(1.0);
                let frac = (1.0 - self.remaining / total).clamp(0.0, 1.0);
                if frac > 0.002 {
                    let start = -std::f32::consts::FRAC_PI_2;
                    let end = start + frac * std::f32::consts::TAU;
                    draw_arc(
                        painter,
                        center,
                        radius,
                        start,
                        end,
                        egui::Stroke::new(10.0, self.accent()),
                    );
                }

                let secs = self.remaining.ceil() as u32;
                painter.text(
                    center + egui::vec2(0.0, -8.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{:02}:{:02}", secs / 60, secs % 60),
                    egui::FontId::proportional(46.0),
                    TXT,
                );

                let label = match (self.mode, self.running) {
                    (Mode::Work, true) => "专注中",
                    (Mode::Work, false) => "准备专注",
                    (Mode::Break, true) => "休息中",
                    (Mode::Break, false) => "准备休息",
                };
                painter.text(
                    center + egui::vec2(0.0, 32.0),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    DIM,
                );
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                let label = if self.running { "暂停" } else { "开始" };
                if ui
                    .add_sized([100.0, 32.0], egui::Button::new(label))
                    .clicked()
                {
                    self.running = !self.running;
                    self.idle_marked = false;
                }
                if ui
                    .add_sized([100.0, 32.0], egui::Button::new("重置"))
                    .clicked()
                {
                    self.reset();
                }
            });

            ui.add(egui::Separator::default().spacing(14.0));

            let before = self.total();
            ui.add_enabled(
                !self.running,
                egui::Slider::new(&mut self.work_min, 5.0..=60.0).text("工作分钟"),
            );
            ui.add_enabled(
                !self.running,
                egui::Slider::new(&mut self.break_min, 1.0..=20.0).text("休息分钟"),
            );
            if !self.running && (self.total() - before).abs() > 0.01 {
                self.remaining = self.total();
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let state = if self.running {
                    format!("运行中 · {:.0} 帧/秒", self.fps)
                } else {
                    "已休眠 · 不请求重绘".to_string()
                };
                ui.label(egui::RichText::new(state).size(11.0).color(DIM));
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(
                            egui::RichText::new(format!("今日完成 {}", self.done))
                                .size(11.0)
                                .color(DIM),
                        );
                    },
                );
            });
        });
    }
}

/// egui 没有内置的圆弧图元，用折线近似画一段圆弧。
fn draw_arc(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    from: f32,
    to: f32,
    stroke: egui::Stroke,
) {
    let span = to - from;
    let steps = ((span.abs() * radius / 6.0).ceil() as usize).max(2);
    let points: Vec<egui::Pos2> = (0..=steps)
        .map(|i| {
            let a = from + span * (i as f32 / steps as f32);
            center + egui::vec2(a.cos(), a.sin()) * radius
        })
        .collect();
    painter.add(egui::Shape::line(points, stroke));
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([340.0, 470.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "专注计时",
        options,
        Box::new(|cc| Ok(Box::new(TimerApp::new(cc)))),
    )
}
