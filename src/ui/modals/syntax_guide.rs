//! Syntax guide modal for OmniPresenter song editor formatting & directives.

use crate::core::state::ControlPanelState;
use crate::ui::modals::editor::selection::insert_text_at_selection;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Stroke, Vec2};

/// Renders the "Guia de Sintaxe & Opções" modal dialog if opened.
pub fn render_syntax_guide_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) {
    if !state.song_editor.show_syntax_guide {
        return;
    }

    let mut open = true;
    egui::Window::new("Syntax-Leitfaden & Editor-Optionen")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(480.0)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

            ui.label(
                RichText::new("Funktionen und Markierungen in Omni Presenter")
                    .size(13.0)
                    .strong()
                    .color(tokens.accent),
            );
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    guide_section(ui, "Stilbefehle für die Projektion", tokens);
                    guide_item(ui, "//t: N (oder //size:, //s:)", "Größenbefehl", "Legt die Schriftgröße der Folie fest.", state, "\n//t: 46\n", tokens);
                    guide_item(ui, "//f: Name (oder //font:)", "Schriftbefehl", "Legt die Schrift der Strophe oder Folie fest.", state, "\n//f: Montserrat\n", tokens);
                    guide_item(ui, "//c: #HEX (oder //color:)", "Farbbefehl", "Legt die Textfarbe der Strophe oder Folie fest.", state, "\n//c: #FFD700\n", tokens);
                    guide_item(ui, "//b: #HEX (oder //shadow:)", "Schatten-/Feldbefehl", "Legt den Textschatten der Folie fest.", state, "\n//b: #00000082\n", tokens);
                    guide_item(ui, "//a: c|l|r|j (oder //align:)", "Ausrichtungsbefehl", "Legt die Ausrichtung fest (Mitte, links, rechts, Blocksatz).", state, "\n//a: c\n", tokens);
                    guide_item(ui, "//l: 1.38 (oder //line:)", "Abstandsbefehl", "Legt den Zeilenabstand der Folie fest.", state, "\n//l: 1.38\n", tokens);
                    guide_item(ui, "//ls: 1.8 (oder //letter:)", "Zeichenabstand", "Legt den Abstand zwischen Zeichen fest.", state, "\n//ls: 1.8\n", tokens);
                    guide_item(ui, "[t: N]...[/t]", "Größe der Auswahl", "Ändert nur die Größe des markierten Textes (auch [size:] oder [s:]).", state, "[t: 46]Text[/t]", tokens);
                    guide_item(ui, "[f: Name]...[/f]", "Schrift der Auswahl", "Ändert nur die Schrift des markierten Textes.", state, "[f: Montserrat]Text[/f]", tokens);
                    guide_item(ui, "[c: #HEX]...[/c]", "Farbe der Auswahl", "Ändert nur die Farbe des markierten Textes.", state, "[c: #FFD700]Text[/c]", tokens);
                    guide_item(ui, "[b: #HEX]...[/b]", "Schatten der Auswahl", "Ändert nur den Schatten des markierten Textes.", state, "[b: #00000082]Text[/b]", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Zeilenumbrüche & Folienteilung", tokens);
                    guide_item(ui, "|", "Weicher Umbruch (Standard)", "Bricht die Zeile in der Projektion weich um.", state, " | ", tokens);
                    guide_item(ui, "|16:9|, |4:3|, |9:16|, |21:9|", "Umbrüche je Format", "Bricht nur beim passenden Seitenverhältnis um.", state, " |16:9| ", tokens);
                    guide_item(ui, "---", "Strophe manuell teilen", "Erstellt sofort eine neue Folie.", state, "\n\n---\n\n", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Kommentare & Bediener-Notizen", tokens);
                    guide_item(ui, "// Text", "Kommentarzeile", "In der Projektion unsichtbar. Für Hinweise im Team.", state, "\n// Notiz: ", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Struktur & Strophennamen", tokens);
                    guide_item(ui, "[Strophe], [Refrain], [Bridge]", "Abschnittsbezeichnungen", "Erstellt automatisch Strophenbezeichnungen.", state, "\n[Refrain]\n", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Formatierung & Hervorhebung", tokens);
                    guide_item(ui, "**Wort**", "Fett", "Setzt das Wort fett.", state, "**Betonung**", tokens);
                    guide_item(ui, "*Wort*", "Kursiv", "Setzt das Wort kursiv.", state, "*Betonung*", tokens);
                    guide_item(ui, "__Wort__", "Unterstrichen", "Unterstreicht den Abschnitt.", state, "__Betonung__", tokens);
                    guide_item(ui, "~~Wort~~", "Durchgestrichen", "Streicht den Abschnitt durch.", state, "~~Betonung~~", tokens);
                });

            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("Schließen").size(11.5))
                            .fill(tokens.bg_pill)
                            .stroke(Stroke::new(1.0, tokens.border_default))
                            .corner_radius(6.0),
                    )
                    .clicked()
                {
                    state.song_editor.show_syntax_guide = false;
                }
            });
        });

    if !open {
        state.song_editor.show_syntax_guide = false;
    }
}

fn guide_section(ui: &mut egui::Ui, title: &str, tokens: &AppTheme) {
    ui.label(
        RichText::new(title)
            .size(11.5)
            .strong()
            .color(tokens.accent),
    );
}

fn guide_item(
    ui: &mut egui::Ui,
    syntax: &str,
    title: &str,
    description: &str,
    state: &mut ControlPanelState,
    insert_payload: &str,
    tokens: &AppTheme,
) {
    egui::Frame::new()
        .fill(tokens.bg_card)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(syntax)
                                .size(11.5)
                                .strong()
                                .color(tokens.text_primary),
                        );
                        ui.label(
                            RichText::new(format!("· {title}"))
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                    });
                    ui.label(
                        RichText::new(description)
                            .size(10.0)
                            .color(tokens.text_secondary),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Inserir")
                                    .size(10.5)
                                    .color(tokens.accent),
                            )
                            .fill(tokens.bg_pill)
                            .stroke(Stroke::new(1.0, tokens.border_default))
                            .corner_radius(4.0),
                        )
                        .clicked()
                    {
                        insert_text_at_selection(state, insert_payload);
                    }
                });
            });
        });
}
