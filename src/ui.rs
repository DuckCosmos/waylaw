use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, CssProvider, Entry, EventControllerKey, Image,
    Label, ListBox, ListBoxRow, Orientation, PolicyType, ScrolledWindow, Separator, gdk, glib,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::filtro;

const CSS: &str = "
window {
    background: transparent;
}

.caixa {
    background: alpha(@window_bg_color, 0.5);
    border-radius: 16px;
    border: 1px solid alpha(@window_fg_color, 0.1);
    padding: 20px;
}

entry {
    background: transparent;
    border: none;
    box-shadow: none;
    outline: none;
    color: @window_fg_color;
    padding: 4px;
}

label {
    color: @window_fg_color;
}

scrolledwindow {
    background: transparent;
}

list {
    background: transparent;
}

list row {
    background: transparent;
    padding: 6px;
    border-radius: 8px;
}

list row:hover {
    background: alpha(@window_fg_color, 0.06);
}

list row:selected,
list row:selected * {
    background-color: @accent_color;
    color: @accent_fg_color;
}
";

pub fn build_ui(app: &Application) {
    let css = CssProvider::new();
    css.load_from_string(CSS);
    gtk4::style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &css, 900);

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(400)
        .default_height(720)
        .build();
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_keyboard_mode(KeyboardMode::Exclusive);
    window.set_anchor(Edge::Top, true);
    window.set_margin(Edge::Top, 200);
    let caixa = GtkBox::new(Orientation::Vertical, 10);
    caixa.add_css_class("caixa");
    caixa.set_valign(gtk4::Align::Start);
    caixa.set_hexpand(false);
    caixa.set_halign(gtk4::Align::Fill);

    let caixa = GtkBox::new(Orientation::Vertical, 10);
    caixa.add_css_class("caixa");
    caixa.set_valign(gtk4::Align::Start);

    let entry = Entry::builder().placeholder_text("Buscar...").build();
    let separador = Separator::new(Orientation::Horizontal);
    let lista = ListBox::new();
    lista.set_selection_mode(gtk4::SelectionMode::Single);
    lista.set_focusable(false);

    let scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .propagate_natural_height(false)
        .child(&lista)
        .build();

    caixa.append(&entry);
    caixa.append(&separador);
    caixa.append(&scroll);
    window.set_child(Some(&caixa));

    let altura = Rc::new(Cell::new(0i32));
    let tick: Rc<RefCell<Option<gtk4::TickCallbackId>>> = Rc::new(RefCell::new(None));
    let lista2 = lista.clone();
    let scroll2 = scroll.clone();
    let altura2 = altura.clone();
    let tick2 = tick.clone();
    entry.connect_changed(move |e| {
        while let Some(filho) = lista2.first_child() {
            lista2.remove(&filho);
        }

        for app in filtro::filtrar(&e.text()) {
            let linha = GtkBox::new(Orientation::Horizontal, 10);

            let icone = if app.icone.is_empty() {
                Image::from_icon_name("application-x-executable")
            } else if app.icone.starts_with('/') {
                Image::from_file(&app.icone)
            } else {
                Image::from_icon_name(&app.icone)
            };
            icone.set_pixel_size(24);
            icone.set_size_request(24, 24);

            let label = Label::new(Some(&app.nome));
            label.set_halign(gtk4::Align::Start);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            label.set_max_width_chars(50);
            label.set_hexpand(true);

            linha.append(&icone);
            linha.append(&label);
            linha.set_margin_top(4);
            linha.set_margin_bottom(4);
            linha.set_margin_start(8);
            linha.set_margin_end(8);

            let row = ListBoxRow::new();
            row.set_focusable(false);
            row.set_child(Some(&linha));
            row.set_widget_name(&app.nome);
            lista2.append(&row);
        }

        if let Some(primeiro) = lista2.first_child() {
            lista2.select_row(Some(&primeiro.downcast::<ListBoxRow>().unwrap()));
        }

        let (_, natural, _, _) = lista2.measure(Orientation::Vertical, -1);
        let alvo = natural.min(480);
        let de = altura2.get();

        if let Some(id) = tick2.borrow_mut().take() {
            id.remove();
        }

        if de == 0 {
            scroll2.set_height_request(alvo);
            altura2.set(alvo);
            return;
        }

        let inicio = Cell::new(None::<i64>);
        let altura3 = altura2.clone();
        let tick3 = tick2.clone();
        let id = scroll2.add_tick_callback(move |s, clock| {
            let agora = clock.frame_time();
            let t0 = match inicio.get() {
                Some(t) => t,
                None => {
                    inicio.set(Some(agora));
                    agora
                }
            };
            let p = ((agora - t0) as f64 / 500_000.0).min(1.0);
            let suave = 1.0 - (1.0 - p).powi(3);
            let h = de + ((alvo - de) as f64 * suave) as i32;
            s.set_height_request(h);
            altura3.set(h);
            if p >= 1.0 {
                tick3.borrow_mut().take();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
        *tick2.borrow_mut() = Some(id);
    });

    entry.emit_by_name::<()>("changed", &[]);

    let w = window.clone();
    lista.connect_row_activated(move |_, row| {
        let nome = row.widget_name();
        filtro::registrar_uso(&nome);
        filtro::abrir(&nome);
        w.close();
    });

    let teclas = EventControllerKey::new();
    teclas.set_propagation_phase(gtk4::PropagationPhase::Capture);

    let w2 = window.clone();
    let lista3 = lista.clone();

    teclas.connect_key_pressed(move |_, tecla, _, _| match tecla {
        gdk::Key::Escape => {
            w2.close();
            glib::Propagation::Stop
        }
        gdk::Key::Down => {
            if let Some(atual) = lista3.selected_row() {
                if let Some(prox) = atual.next_sibling() {
                    lista3.select_row(Some(&prox.downcast::<ListBoxRow>().unwrap()));
                }
            }
            glib::Propagation::Stop
        }
        gdk::Key::Up => {
            if let Some(atual) = lista3.selected_row() {
                if let Some(ant) = atual.prev_sibling() {
                    lista3.select_row(Some(&ant.downcast::<ListBoxRow>().unwrap()));
                }
            }
            glib::Propagation::Stop
        }
        gdk::Key::Return | gdk::Key::KP_Enter => {
            if let Some(row) = lista3.selected_row() {
                lista3.emit_by_name::<()>("row-activated", &[&row]);
            }
            glib::Propagation::Stop
        }
        _ => glib::Propagation::Proceed,
    });

    window.add_controller(teclas);
    window.present();
}
