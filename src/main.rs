use calamine::{open_workbook, Reader, Xlsx};
use eframe::egui;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let data = read_xlsx_file("/home/deng/Desktop/rust/learn/four/src/7.xlsx");
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1600.0, 768.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My parallel egui App",
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
}

impl ExamApp {
    fn new(data: LinkedHashMap<String, String>, cc: &eframe::CreationContext) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            data,
            selected: HashMap::new(),
            correct: HashMap::new(),
            font_id: egui::FontId::proportional(18.0),
            option_chars: ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'],
            wrong_questions: LinkedHashMap::new(),
        }
    }

    fn check_answers(&mut self) -> u32 {
        let mut correct_count = 0;

        for (question, correct_answer) in &self.correct {
            if let Some(selected_answer) = self.selected.get(question) {
                if selected_answer == correct_answer {
                    correct_count += 1;
                } else if !self.wrong_questions.contains_key(question) {
                    self.wrong_questions.insert(
                        question.to_owned(),
                        self.data.get(question).unwrap().to_owned(),
                    );
                }
            }
        }
        correct_count
    }

    fn show(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::introspection::font_id_ui(ui, &mut self.font_id);
            egui::SidePanel::left("Test")
                .resizable(true)
                .default_width(800.0)
                .width_range(80.0..=1000.0)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (index, (question, option_answer)) in self.data.iter().enumerate() {
                            let option_answer_vec: Vec<&str> = option_answer.split('\n').collect();
                            let index_question = format!("{}.{}", index + 1, question);
                            ui.label(
                                egui::RichText::new(index_question).font(self.font_id.clone()),
                            );
                            let option_count = option_answer_vec.len() - 1;
                            for i in 0..option_count {
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
                                .entry(question.to_owned())
                                .or_insert_with(|| correct.to_string());
                        }
                    });
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
                            for i in 0..option_answer_vec.len() - 1 {
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
            let mut non_empty_cells = vec![];
            for cell in row.iter() {
                if !cell.is_empty() {
                    non_empty_cells.push(cell.clone());
                }
            }

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
