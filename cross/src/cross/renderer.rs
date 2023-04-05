use crate::egui::{Color32, Ui};
use crate::egui::plot::{Legend, MarkerShape, Plot, Points};

use super::analysis::ColorPoint;

#[derive(Default)]
pub struct ChartData {
    pub points: Vec<ColorPoint>
}

pub fn convert_to_points(point: &ColorPoint) -> Points {
    Points::new(vec![[point.x, point.y]])
        .filled(true)
        .radius(4.0)
        .color(Color32::from(point.c))
        .shape(MarkerShape::Square)
}

pub fn render_chart(ui: &mut Ui, chart_data: &ChartData) {
    let markers_plot = Plot::new("cross_pattern")
        .data_aspect(1.0)
        .legend(Legend::default());

    markers_plot.show(ui, |plot_ui| {
        for point in &chart_data.points {
            plot_ui.points(convert_to_points(point));
        }
    });
}