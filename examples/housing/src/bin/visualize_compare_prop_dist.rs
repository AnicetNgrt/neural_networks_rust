use rand;
use std::env;

use nn::datatable::DataTable;
use plotly::{
    color::Rgb,
    common::{Mode, Title},
    layout::Axis,
    ImageFormat, Layout, Plot, Scatter,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut plot = Plot::new();

    let title = "Comparative average proportional price prediction distance over epochs";

    plot.set_layout(
        Layout::new()
            .title(Title::new(title))
            .x_axis(Axis::new().title("epoch".into()))
            .y_axis(
                Axis::new()
                    .title("dist(prediction, reality) / reality".into())
                    .dtick(0.1),
            ),
    );

    for i in 1..args.len() {
        let model_name = &args[i];
        let stats = DataTable::from_file(format!("models_stats/{}.csv", model_name));
        let avg_prop_dist = stats.column_to_vecf64("avg price pred prop dist");

        let random_color = Rgb::new(
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>(),
        );
        let avg_prop_dist_trace =
            Scatter::new((0..avg_prop_dist.len()).collect(), avg_prop_dist.clone())
                .mode(Mode::Lines)
                .name(format!("{} average", model_name))
                .line(plotly::common::Line::new().color(random_color));

        plot.add_trace(avg_prop_dist_trace);
    }

    let concatenated_names = args[1..].join("_");
    plot.write_image(
        format!("visuals/{}_comp_prop_dist.png", concatenated_names),
        ImageFormat::PNG,
        1600,
        1200,
        1.0,
    );
}
