use bevy::math::{IVec2, UVec2};
use bevy_egui::egui::{
    Color32, Id, PointerButton, Pos2, Rect, Response, Sense, Stroke, StrokeKind, TextureId, Ui,
    Vec2, Widget, pos2, vec2,
};

const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 64.0;
const GRID_LINE_ZOOM: f32 = 6.0;
const SCROLL_ZOOM_SPEED: f32 = 0.0015;

#[derive(Clone, Copy)]
struct CanvasView {
    /// Offset of the texture center from the panel center, in screen points.
    pan: Vec2,
    /// Screen points per cell.
    zoom: f32,
}

pub struct ShipCanvas<'a> {
    texture: TextureId,
    origin: IVec2,
    size: UVec2,
    target: &'a mut Option<IVec2>,
}

impl<'a> ShipCanvas<'a> {
    pub fn new(
        texture: TextureId,
        origin: IVec2,
        size: UVec2,
        target: &'a mut Option<IVec2>,
    ) -> Self {
        Self {
            texture,
            origin,
            size,
            target,
        }
    }

    fn image_rect(&self, panel: Rect, view: CanvasView) -> Rect {
        let size = vec2(self.size.x as f32, self.size.y as f32) * view.zoom;
        Rect::from_center_size(panel.center() + view.pan, size)
    }

    fn cell_at(&self, image: Rect, zoom: f32, pos: Pos2) -> Option<IVec2> {
        let local = pos - image.min;
        let tx = (local.x / zoom).floor() as i32;
        let ty = (local.y / zoom).floor() as i32;
        if tx < 0 || ty < 0 || tx >= self.size.x as i32 || ty >= self.size.y as i32 {
            return None;
        }
        Some(IVec2::new(
            self.origin.x + tx,
            self.origin.y + self.size.y as i32 - 1 - ty,
        ))
    }

    fn cell_rect(&self, image: Rect, zoom: f32, cell: IVec2) -> Rect {
        let tx = (cell.x - self.origin.x) as f32;
        let ty = (self.size.y as i32 - 1 - (cell.y - self.origin.y)) as f32;
        Rect::from_min_size(image.min + vec2(tx * zoom, ty * zoom), Vec2::splat(zoom))
    }

    fn fit_zoom(&self, panel: Rect) -> f32 {
        ((panel.width() / self.size.x as f32).min(panel.height() / self.size.y as f32) * 0.9)
            .clamp(MIN_ZOOM, MAX_ZOOM)
    }
}

impl Widget for ShipCanvas<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let panel = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(panel, Sense::click_and_drag());
        let id = Id::new("ship_canvas_view");

        let mut view = ui
            .data_mut(|d| d.get_temp::<CanvasView>(id))
            .unwrap_or(CanvasView {
                pan: Vec2::ZERO,
                zoom: self.fit_zoom(panel),
            });

        if response.hovered() {
            let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
            let factor = pinch * (scroll * SCROLL_ZOOM_SPEED).exp();
            if factor != 1.0 {
                let anchor = response.hover_pos().unwrap_or(panel.center());
                let new_zoom = (view.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
                let scale = new_zoom / view.zoom;
                let center = panel.center() + view.pan;
                view.pan = (anchor + (center - anchor) * scale) - panel.center();
                view.zoom = new_zoom;
            }
        }

        if response.dragged_by(PointerButton::Middle)
            || response.dragged_by(PointerButton::Secondary)
        {
            view.pan += response.drag_delta();
        }

        let image = self.image_rect(panel, view);
        let painter = ui.painter_at(panel);
        painter.rect_filled(panel, 0.0, Color32::from_gray(18));
        painter.image(
            self.texture,
            image,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if view.zoom >= GRID_LINE_ZOOM {
            let stroke = Stroke::new(1.0, Color32::from_black_alpha(48));
            for x in 0..=self.size.x {
                let px = image.min.x + x as f32 * view.zoom;
                painter.line_segment([pos2(px, image.min.y), pos2(px, image.max.y)], stroke);
            }
            for y in 0..=self.size.y {
                let py = image.min.y + y as f32 * view.zoom;
                painter.line_segment([pos2(image.min.x, py), pos2(image.max.x, py)], stroke);
            }
        }

        if let Some(cell) = response
            .hover_pos()
            .and_then(|p| self.cell_at(image, view.zoom, p))
        {
            painter.rect_stroke(
                self.cell_rect(image, view.zoom, cell),
                0.0,
                Stroke::new(2.0, Color32::WHITE),
                StrokeKind::Inside,
            );
        }

        let painting =
            response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_down());
        if painting && let Some(pos) = response.interact_pointer_pos() {
            *self.target = self.cell_at(image, view.zoom, pos);
        }

        ui.data_mut(|d| d.insert_temp(id, view));
        response
    }
}
