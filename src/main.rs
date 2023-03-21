use calamine::{open_workbook, Reader, Xlsx};
use eframe::egui;
use egui::Color32;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let data = read_xlsx_file("/home/deng/Desktop/rust/learn/four/src/9.xlsx");
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 768.0)),
        ..Default::default()
    };
    eframe::run_native(
        "deng",
        options,
        Box::new(|cc| Box::new(ExamApp::new(data, cc))),
    )
}

struct ExamApp {
    data: LinkedHashMap<String, String>,
    selected: HashMap<String, String>,
    correct: HashMap<String, String>,
    font_id: egui::FontId,
    option_chars: [char; 8],
    wrong_questions: LinkedHashMap<String, String>,
    multi_choose: HashMap<String, Vec<bool>>,
    mapping: Vec<(char, bool)>,
    multi_correct: HashMap<String, Vec<bool>>,
    visible: HashMap<String, bool>,
    page_num: usize,
    top: bool,
}

impl ExamApp {
    fn new(data: LinkedHashMap<String, String>, cc: &eframe::CreationContext) -> Self {
        let len = data.len();
        let selected = HashMap::with_capacity(len);

        let correct = HashMap::with_capacity(len);
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            data,
            selected,
            correct,
            font_id: egui::FontId::proportional(18.0),
            option_chars: ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'],
            wrong_questions: LinkedHashMap::new(),
            multi_choose: HashMap::new(),
            multi_correct: HashMap::new(),
            mapping: vec![
                ('A', true),
                ('B', true),
                ('C', true),
                ('D', true),
                ('E', true),
                ('F', true),
                ('G', true),
                ('H', true),
            ],
            visible: HashMap::new(),
            page_num: 0,
            top: false,
        }
    }

    fn check_answers(&mut self) -> u32 {
        let mut correct_count = 0;

        for (question, correct_answer) in &self.correct {
            match self.selected.get(question) {
                Some(selected_answer) if selected_answer == correct_answer => correct_count += 1,
                Some(_) => {
                    self.wrong_questions
                        .entry(question.to_owned())
                        .or_insert_with(|| self.data.get(question).unwrap().to_owned());
                }
                None => {}
            }
        }
        correct_count
    }

    fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::introspection::font_id_ui(ui, &mut self.font_id);
                egui::global_dark_light_mode_switch(ui);
                if ui.button("next").clicked() {
                    self.page_num += 50;
                    ui.scroll_to_cursor(Some(egui::Align::TOP))
                }
                self.top = ui.button("top").clicked();
            });
            egui::SidePanel::left("Test")
                .resizable(true)
                .default_width(800.0)
                .width_range(80.0..=1000.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.top {
                            ui.scroll_to_cursor(Some(egui::Align::TOP))
                        }
                        for (index, (question, option_answer)) in
                            self.data.iter().skip(self.page_num).take(50).enumerate()
                        {
                            let option_answer_vec: Vec<&str> = option_answer.split('\n').collect();
                            let index_question = format!("{}.{}", index + 1, question);
                            ui.heading(
                                egui::RichText::new(index_question).font(self.font_id.clone()),
                            );
                            let option_count = option_answer_vec.len() - 1;
                            let correct = option_answer_vec.last().expect("Last should exsit");
                            self.correct
                                .entry(question.to_owned())
                                .or_insert_with(|| correct.to_string());
                            if correct.chars().count() == 1 {
                                for (i, _) in
                                    option_answer_vec.iter().enumerate().take(option_count)
                                {
                                    let option = format!(
                                        "{}. {}",
                                        self.option_chars[i], option_answer_vec[i]
                                    );
                                    let is_selected =
                                        self.selected.get(question).map_or(false, |selected| {
                                            selected == &self.option_chars[i].to_string()
                                        });

                                    if ui
                                        .radio(
                                            is_selected,
                                            egui::RichText::new(option).font(self.font_id.clone()),
                                        )
                                        .clicked()
                                    {
                                        self.selected.insert(
                                            question.to_owned(),
                                            self.option_chars[i].to_string(),
                                        );
                                    }
                                }
                                ui.group(|ui| {
                                    let visible = !matches!(self.selected.get(question), None);
                                    ui.set_visible(visible);
                                    if visible
                                        && self.selected.get(question).unwrap()
                                            != self.correct.get(question).unwrap()
                                    {
                                        ui.label(
                                            egui::RichText::new(
                                                self.correct.get(question).unwrap(),
                                            )
                                            .color(Color32::RED),
                                        );
                                    } else {
                                        ui.label(egui::RichText::new("✔").color(Color32::GREEN));
                                    }
                                });
                            } else {
                                let result: Vec<bool> = correct
                                    .chars()
                                    .map(|c| match self.mapping.iter().find(|(x, _)| *x == c) {
                                        Some((_, b)) => *b,
                                        None => false,
                                    })
                                    .collect();
                                let len = result.len();
                                self.multi_correct.insert(question.to_owned(), result);
                                let choose = vec![false; len];
                                self.multi_choose
                                    .entry(question.to_owned())
                                    .or_insert(choose);
                                self.visible.entry(question.to_owned()).or_insert(false);

                                for (i, _) in
                                    option_answer_vec.iter().enumerate().take(option_count)
                                {
                                    let option = format!(
                                        "{}. {}",
                                        self.option_chars[i], option_answer_vec[i]
                                    );

                                    // let mut is_selected = self
                                    //     .multi_choose
                                    //     .get(question)
                                    //     .map_or(choose.clone(), |selected| selected.to_owned());

                                    if ui
                                        .checkbox(
                                            &mut self.multi_choose.get_mut(question).unwrap()[i],
                                            egui::RichText::new(option).font(self.font_id.clone()),
                                        )
                                        .clicked()
                                    {
                                        println!("hi");
                                    }
                                }
                                if ui.button("sumit").clicked() {
                                    *self.visible.get_mut(question).unwrap() =
                                        !matches!(self.multi_choose.get(question), None);
                                }
                                ui.group(|ui| {
                                    let visible = *self.visible.get(question).unwrap();
                                    ui.set_visible(visible);
                                    if visible
                                        && self.multi_choose.get(question).unwrap()
                                            != self.multi_correct.get(question).unwrap()
                                    {
                                        ui.label(
                                            egui::RichText::new(
                                                self.correct.get(question).unwrap(),
                                            )
                                            .color(Color32::RED),
                                        );
                                    } else {
                                        ui.label(egui::RichText::new("✔").color(Color32::GREEN));
                                    }
                                });
                            }

                            ui.separator();
                        }
                    })
                });
            egui::SidePanel::right("Wrong")
                .resizable(true)
                .default_width(800.0)
                .width_range(80.0..=1000.0)
                .show_inside(ui, |ui| {
                    let select = self.selected.len();
                    let scroe = self.check_answers();
                    let data_len = self.data.len();
                    ui.end_row();
                    ui.label(format!("process: {}/{}", select, self.data.len()));

                    let process_bar =
                        egui::ProgressBar::new(select as f32 / self.data.len() as f32)
                            .show_percentage();
                    ui.add(process_bar);
                    ui.label(format!("Wrong: {}", data_len as u32 - scroe));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (index, (question, option_answer)) in
                            self.wrong_questions.iter().enumerate()
                        {
                            let option_answer_vec: Vec<&str> = option_answer.split('\n').collect();
                            let index_question = format!("{}.{}", index + 1, question);
                            ui.label(
                                egui::RichText::new(index_question).font(self.font_id.clone()),
                            );
                            let option_count = option_answer_vec.len() - 1;
                            for (i, _) in option_answer_vec.iter().enumerate().take(option_count) {
                                let option =
                                    format!("{}. {}", self.option_chars[i], option_answer_vec[i]);
                                let is_selected =
                                    self.selected.get(question).map_or(false, |selected| {
                                        selected == &self.option_chars[i].to_string()
                                    });

                                if ui
                                    .radio(
                                        is_selected,
                                        egui::RichText::new(option).font(self.font_id.clone()),
                                    )
                                    .clicked()
                                {
                                    self.selected.insert(
                                        question.to_owned(),
                                        self.option_chars[i].to_string(),
                                    );
                                }
                            }

                            ui.separator();
                            ui.end_row();

                            let correct = option_answer_vec.last().expect("Last should exsit");
                            self.correct
                                .insert(question.to_owned(), correct.to_string());
                        }
                    });
                })
        });
    }
}

impl eframe::App for ExamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show(ctx);
    }
}

fn read_xlsx_file(file_path: &str) -> LinkedHashMap<String, String> {
    let path = PathBuf::from(file_path);
    // let path = Path::new(&file_path);
    // let mut workbook: Xlsx<_> = open_workbook(path).unwrap();
    let mut workbook: Xlsx<_> = match open_workbook(path) {
        Ok(workbook) => workbook,
        Err(err) => panic!("Error opening workbook:{err}"),
    };
    let quiz_sheet = workbook.worksheet_range_at(0).unwrap().unwrap();
    let quiz_data = quiz_sheet
        .rows()
        .skip(1)
        .map(|row| {
            let non_empty_cells = row
                .iter()
                .filter(|cell| !cell.is_empty())
                .collect::<Vec<_>>();
            let options = non_empty_cells
                .iter()
                .skip(1)
                .take(non_empty_cells.len() - 2)
                .map(|cell| cell.get_string().unwrap().to_string())
                .collect::<Vec<_>>()
                .join("\n");

            (
                row[0].get_string().unwrap().to_string(),
                format!("{}\n{}", options, row.last().unwrap().get_string().unwrap()),
            )
        })
        .collect::<LinkedHashMap<_, _>>();
    quiz_data
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // Install my own font (maybe supporting non-latin characters).
    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("1.otf")),
    );

    // Put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());

    // Tell egui to use these fonts:
    ctx.set_fonts(fonts);
}
