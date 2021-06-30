use aubio_rs::{OnsetMode, Tempo};
use nannou::prelude::*;
use nannou::ui::prelude::*;
use nannou_audio as audio;
use ringbuf::{Consumer, Producer, RingBuffer};

fn main() {
    nannou::app(model).update(update).simple_window(view).run();
}

widget_ids! {
    struct Ids {
        startstop,
        threshold,
        silence,
    }
}

struct Model {
    ui: Ui,
    ids: Ids,
    in_stream: audio::Stream<InputModel>,
    consumer: Consumer<f32>,
    tempo: Tempo,
    tempo_result: f32,
    strength: f32,
    threshold: f32,
    silence: f32,
}

struct InputModel {
    producer: Producer<f32>,
}

fn model(app: &App) -> Model {
    let mut ui = app.new_ui().build().unwrap();
    let ids = Ids::new(ui.widget_id_generator());

    let audio_host = audio::Host::new();
    let ringbuf = RingBuffer::<f32>::new(2048);
    let (producer, consumer) = ringbuf.split();
    let in_model = InputModel { producer };
    let in_stream = audio_host
        .new_input_stream::<InputModel, f32>(in_model)
        .capture(input)
        .sample_rate(44100)
        .build()
        .unwrap();

    let mut tempo = Tempo::new(OnsetMode::Complex, 1024, 512, 44100).unwrap();
    tempo.set_silence(0.1);
    tempo.set_threshold(0.3);

    Model {
        ui,
        ids,
        in_stream,
        consumer,
        tempo,
        tempo_result: 0.0,
        strength: 0.0,
        threshold: 0.3,
        silence: 0.1,
    }
}

fn input(model: &mut InputModel, buffer: &audio::Buffer) {
    for frame in buffer.frames() {
        model.producer.push(frame[0]).ok();
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    let ui = &mut model.ui.set_widgets();

    for _click in widget::Button::new()
        .top_left_with_margin(20.0)
        .w_h(200.0, 60.0)
        .label(if model.in_stream.is_playing() {
            "Stop"
        } else {
            "Start"
        })
        .set(model.ids.startstop, ui)
    {
        if model.in_stream.is_playing() {
            model.in_stream.pause().unwrap();
        } else {
            model.in_stream.play().unwrap();
        }
    }

    for value in widget::Slider::new(model.threshold, 0.0, 1.0)
        .down(10.0)
        .w_h(200.0, 30.0)
        .label("Threshold")
        .set(model.ids.threshold, ui)
    {
        model.threshold = value as f32;
        model.tempo.set_threshold(model.threshold);
    }

    for value in widget::Slider::new(model.silence, 0.0, 1.0)
        .down(10.0)
        .w_h(200.0, 30.0)
        .label("Silence")
        .set(model.ids.silence, ui)
    {
        model.silence = value as f32;
        model.tempo.set_silence(model.silence);
    }

    while model.consumer.len() >= 1024 {
        let mut samples = [0.0f32; 1024];
        model.consumer.access(|s1, s2| {
            let len_s1 = std::cmp::min(1024, s1.len());
            samples[0..len_s1].copy_from_slice(&s1[0..len_s1]);
            let len_s2 = std::cmp::min(1024 - len_s1, s2.len());
            samples[len_s1..1024].copy_from_slice(&s2[0..len_s2]);
        });
        model.consumer.discard(512);
        model.tempo_result = model.tempo.do_result(samples).unwrap();
        if model.tempo_result > 0.0 {
            model.strength = 1.0
        }
    }
    model.strength *= 0.8;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().rgb(0.2, 0.2, 0.2);
    draw.ellipse()
        .color(RED)
        .x_y(0.0, 0.0)
        .w_h(model.strength * 400.0, model.strength * 400.0);
    draw.to_frame(app, &frame).unwrap();

    model.ui.draw_to_frame(app, &frame).unwrap();
}
