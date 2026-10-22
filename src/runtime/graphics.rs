use std::time::{Duration, Instant};

use super::{Kind, Runtime};

/// What the terminal can draw beyond text, from [`super::Scope::use_graphics`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Graphics {
    /// Supported graphics protocol, or the pending probe.
    pub support: GraphicsSupport,
    /// One cell's size in pixels, width then height, when known.
    pub cell_pixels: Option<(u16, u16)>,
}

/// The result of the terminal graphics probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphicsSupport {
    /// The probe has not finished.
    Probing,
    /// The kitty graphics protocol with Unicode placeholders.
    Kitty,
    /// No supported image protocol.
    Unsupported,
}

pub(super) struct GraphicsState {
    pub(super) value: Graphics,
    pub(super) requested: bool,
    deadline: Option<Instant>,
    kitty_seen: bool,
    driver_pixels: Option<(u16, u16)>,
    reply_pixels: Option<(u16, u16)>,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            value: Graphics {
                support: GraphicsSupport::Unsupported,
                cell_pixels: None,
            },
            requested: false,
            deadline: None,
            kitty_seen: false,
            driver_pixels: None,
            reply_pixels: None,
        }
    }
}

impl GraphicsState {
    pub(super) fn probing(&self) -> bool {
        self.requested && self.value.support == GraphicsSupport::Probing
    }
}

impl Runtime {
    pub(crate) fn configure_graphics(&mut self, enabled: bool) {
        if enabled {
            self.instance.store.graphics.borrow_mut().value.support = GraphicsSupport::Probing;
        }
    }

    pub(crate) fn graphics_probing(&self) -> bool {
        self.instance.store.graphics.borrow().probing()
    }

    pub(crate) fn start_graphics(&mut self, now: Instant) -> bool {
        if !self.graphics_probing() {
            return false;
        }
        let mut graphics = self.instance.store.graphics.borrow_mut();
        if graphics.deadline.is_some() {
            return false;
        }
        graphics.deadline = Some(now + Duration::from_secs(1));
        true
    }

    pub(crate) fn graphics_deadline(&self) -> Option<Instant> {
        self.instance.store.graphics.borrow().deadline
    }

    pub(crate) fn expire_graphics(&mut self, now: Instant) {
        if self
            .graphics_deadline()
            .is_some_and(|deadline| now >= deadline)
        {
            self.finish_graphics(false);
        }
    }

    pub(crate) fn finish_graphics(&mut self, cancelled: bool) {
        if !self.graphics_probing() {
            return;
        }
        self.update_graphics(|graphics| {
            graphics.deadline = None;
            graphics.value.support = if graphics.kitty_seen && !cancelled {
                GraphicsSupport::Kitty
            } else {
                GraphicsSupport::Unsupported
            };
        });
    }

    pub(crate) fn resize_graphics(&mut self, pixels: Option<(u16, u16)>) {
        self.update_graphics(|graphics| graphics.driver_pixels = pixels);
    }

    pub(super) fn graphics_reply(&mut self, text: &str) -> bool {
        if self.graphics_deadline().is_none() {
            return false;
        }
        if let Some(body) = text.strip_prefix("\x1b_G").and_then(|body| {
            body.strip_suffix("\x1b\\")
                .or_else(|| body.strip_suffix('\x07'))
        }) {
            if let Some((params, payload)) = body.split_once(';')
                && params.split(',').any(|param| param == "i=31")
                && payload == "OK"
            {
                self.instance.store.graphics.borrow_mut().kitty_seen = true;
            }
            return true;
        }
        if let Some(body) = text
            .strip_prefix("\x1b[6;")
            .and_then(|body| body.strip_suffix('t'))
        {
            if let Some((height, width)) = body.split_once(';')
                && let (Some(height), Some(width)) = (dimension(height), dimension(width))
            {
                self.update_graphics(|graphics| graphics.reply_pixels = Some((width, height)));
            }
            return true;
        }
        if text
            .strip_prefix("\x1b[?")
            .and_then(|body| body.strip_suffix('c'))
            .is_some_and(|body| {
                body.split(';')
                    .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
            })
        {
            self.finish_graphics(false);
            return true;
        }
        false
    }

    fn update_graphics(&mut self, update: impl FnOnce(&mut GraphicsState)) {
        let mut graphics = self.instance.store.graphics.borrow_mut();
        let previous = graphics.value;
        update(&mut graphics);
        graphics.value.cell_pixels = graphics.driver_pixels.or(graphics.reply_pixels);
        if graphics.value != previous {
            self.instance.invalidate_readers(Kind::Graphics);
        }
    }
}

fn dimension(text: &str) -> Option<u16> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok().filter(|value| *value > 0)
}
