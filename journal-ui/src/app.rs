use chrono::Datelike;
use chrono::Timelike;

use journal_lib::service;
use journal_server::fs_service;

use journal_lib::views;


fn format_a_time(time: &chrono::DateTime<chrono::Utc>) -> String {
	let current_time = chrono::Utc::now();
	let current_year = current_time.year();
	let time_year = time.year();
	if current_year != time_year {
		return time.format("%Y-%m-%d").to_string();
	}
	let current_month = current_time.month();
	let time_month = time.month();
	if current_month != time_month {
		return time.format("%m-%d").to_string();
	}
	let current_day = current_time.day();
	let time_day = time.day();
	if current_day != time_day {
		return time.format("%d %H:%M").to_string();
	}
	let current_hour = current_time.hour();
	let time_hour = time.hour();
	if current_hour != time_hour {
		return time.format("%H:%M").to_string();
	}
	return time.format("%H:%M:%S %3f").to_string();
}

#[derive(serde::Deserialize, serde::Serialize)]
enum Tab {
	TraceTemplates,
	Traces,
	EventTemplates,
	Events,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
	project_uuid: Option<uuid::Uuid>,

	#[serde(skip)]
	service: Box<dyn service::EventsService>,
	#[serde(skip)]
	tab: Tab,

	#[serde(skip)]
	event_we_building: Vec<service::EventBuilder>,
	#[serde(skip)]
	event_index: Option<usize>,
	#[serde(skip)]
	event_we_viewing: Option<views::EventView>,

	#[serde(skip)]
	trace_we_building: Vec<service::TraceBuilder>,
	#[serde(skip)]
	trace_index: Option<usize>,
	#[serde(skip)]
	trace_we_viewing: Option<views::TraceView>,
}

impl Default for TemplateApp {
	fn default() -> Self {
		let mut service = fs_service::FileSystemEventsService::default();

		// if let Err(err) = service.load_from_disk(&projects_directory) {
		// 	log::error!("Failed to load from disk: {}", err);
		// 	panic!();
		// }
		// println!("Loaded from disk");

		Self {
			tab: Tab::Traces,
			service: Box::new(service),
			project_uuid: None,
			event_we_building: Vec::new(),
			event_we_viewing: None,
			trace_we_building: Vec::new(),
			trace_we_viewing: None,
			event_index: None,
			trace_index: None,
		}
	}
}

impl TemplateApp {
	/// Called once before the first frame.
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		cc.egui_ctx.set_pixels_per_point(1.0);
		if let Some(storage) = cc.storage {
			return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
		}
		Default::default()
	}

	fn draw_event_templates_side_panel(&mut self, ui: &mut egui::Ui) {
		ui.heading("Event Templates");

		egui::Grid::new("event_templates").show(ui, |ui| {
			for entry in self.service.list_event_templates(self.project_uuid) {
				let trace = views::TraceView {
					trace_uuid: uuid::Uuid::new_v4(),
					name: "TODO".into(),
					trace_template: None,
					tags: Vec::new(),
					completion: None,
					last_event: None,
					suggested_event_templates: Vec::new(),
					other_event_templates: Vec::new(),
				};
				ui.label(&entry.name);

				if ui.button("Create").clicked() {
					if let Ok(event) = self.service.create_event(&trace, entry.event_template_uuid)
					{
						self.event_we_building.push(event);
						self.event_index = Some(self.event_we_building.len() - 1);
					}
				}
				// ui.label(&entry.created_at.to_string());
				// ui.label(&entry.last_used.to_string());

				ui.end_row();
			}
		});
	}

	fn draw_events_side_panel(&mut self, ui: &mut egui::Ui) {
		ui.heading("Events");

		egui::Grid::new("events").show(ui, |ui| {
			self.service
				.list_events(self.project_uuid)
				.for_each(|entry: views::EventItemView| {
					if let Some(event_template) = &entry.event_template {
						ui.label(event_template.name.clone());
					} else {
						ui.label("Missing event template".to_string());
					}
					if let Some(trace_name) = &entry.trace_name {
						ui.label(trace_name.clone());
					} else {
						ui.label("Missing trace".to_string());
					}
					ui.label(format_a_time(&entry.created_at));
					if ui.button("View").clicked() {
						self.event_we_viewing = self.service.view_event(entry.event_uuid);
					}

					ui.end_row();
				});
		});
	}

	fn draw_trace_templates_side_panel(&mut self, ui: &mut egui::Ui) {
		ui.heading("Trace Templates");

		egui::Grid::new("trace_templates").show(ui, |ui| {
			for entry in self.service.list_trace_templates(self.project_uuid) {
				ui.label(&entry.name);

				if ui.button("Create").clicked() {
					if let Ok(event) = self.service.create_trace(entry.trace_template_uuid) {
						self.trace_we_building.push(event);
						self.trace_index = Some(self.trace_we_building.len() - 1);
					}
				}
				ui.end_row();
			}
		});
	}

	fn draw_traces_side_panel(&mut self, ui: &mut egui::Ui) {
		ui.heading("Traces");

		egui::Grid::new("traces").show(ui, |ui| {
			for entry in self.service.list_traces(self.project_uuid) {
				if let Some(template_name) = &entry.template_name {
					ui.label(template_name);
				} else {
					ui.label("Missing template".to_string());
				}
				ui.label(entry.name.clone());
				if ui.button("View").clicked() {
					self.trace_we_viewing = self.service.view_trace(entry.trace_uuid);
				}
				ui.end_row();
			}
		});
	}

	fn draw_side_panel(&mut self, ctx: &egui::Context) {
		egui::SidePanel::left("side_panel").show(ctx, |ui| match self.tab {
			Tab::EventTemplates => self.draw_event_templates_side_panel(ui),
			Tab::Events => self.draw_events_side_panel(ui),
			Tab::TraceTemplates => self.draw_trace_templates_side_panel(ui),
			Tab::Traces => self.draw_traces_side_panel(ui),
		});
	}

	fn draw_event_templates_main_panel(&mut self, ui: &mut egui::Ui) {
		if self.event_we_building.len() >= 2 {
			ui.horizontal(|ui| {
				self.event_we_building
					.iter()
					.enumerate()
					.for_each(|(index, event)| {
						if ui.button(&event.event_template.name).clicked() {
							self.event_index = Some(index);
						}
					});
			});
			ui.separator();
		}
		let mut remove = false;
		if let Some(event_index) = self.event_index {
			if let Some(event) = self.event_we_building.get_mut(event_index) {
				ui.heading(&event.event_template.name);
				ui.separator();
				ui.label(format_a_time(&event.began_at));

				egui::Grid::new("889de043-d0ce-4d8a-9c5c-76f952e0d3f2").show(ui, |ui| {
					for field in event.fields.iter_mut() {
						show_field(ui, field);
						ui.end_row();
					}
				});

				let traces = self
					.service
					.list_traces(self.project_uuid)
					.collect::<Vec<views::TraceItemView>>();

				if traces.len() > 0 {
					egui::ComboBox::from_label("Set trace origin")
						.selected_text(match &event.selected_trace {
							service::TraceSelection::Selected(trace) => trace.name.clone(),
							service::TraceSelection::None => "None".to_string(),
						})
						.show_ui(ui, |ui| {
							for trace in traces {
								ui.selectable_value(
									&mut event.selected_trace,
									service::TraceSelection::Selected(trace.clone()),
									&trace.name,
								);
							}
						});
				}

				ui.separator();
				ui.horizontal(|ui| {
					if ui.button("Save").clicked() {
						self.service.save_event(event);
						remove = true;
					}
					if ui.button("Cancel").clicked() {
						remove = true;
					}
				});
			} else {
				self.event_index = None;
			}
		}
		if remove {
			if let Some(event_index) = self.event_index {
				self.event_we_building.remove(event_index);
				if self.event_we_building.len() == 0 {
					self.event_index = None;
				} else if event_index >= self.event_we_building.len() {
					self.event_index = Some(self.event_we_building.len() - 1);
				}
			}
		}
	}

	fn draw_events_main_panel(&mut self, ui: &mut egui::Ui) {
		if let Some(entry) = &self.event_we_viewing {
			egui::Grid::new("event_details").show(ui, |ui| {
				ui.label("Event Template");
				if let Some(template) = &entry.event_template {
					ui.label(template.name.clone());
				} else {
					ui.label("Missing event template".to_string());
				}
				ui.end_row();

				if let Some(trace) = &entry.trace {
					ui.label("Trace template");
					if let Some(template_name) = &trace.template_name {
						ui.label(template_name.clone());
					} else {
						ui.label("Missing trace template".to_string());
					}
					ui.end_row();
				}

				ui.label("Trace");
				if let Some(trace) = &entry.trace {
					ui.label(trace.name.clone());
				} else {
					ui.label("Missing trace".to_string());
				}
				ui.end_row();

				ui.label("Uuid");
				ui.label(entry.event_uuid.to_string());
				ui.end_row();

				ui.label("Began at");
				ui.label(format_a_time(&entry.began_at));
				ui.end_row();

				ui.label("Created at");
				ui.label(format_a_time(&entry.created_at));
				ui.end_row();
			});
			ui.separator();
			ui.label("Fields");
			egui::Grid::new("07926821-a836-4d9c-abc8-b2571638ae7b").show(ui, |ui| {
				for field in &entry.fields {
					ui.label(field.label.clone());
					if let Some(value) = &field.value {
						match value {
							views::FieldValueView::Text(text) => {
								// remove option on the value? or remove the option of the value
								ui.label(&text.value.as_ref().cloned().unwrap());
							}
							views::FieldValueView::Number(number) => {
								ui.label(&number.value.unwrap().to_string());
							}
							views::FieldValueView::Bool(boolean) => {
								ui.label(&boolean.value.unwrap().to_string());
							}
							views::FieldValueView::Enumerated(enumerated) => {
								ui.label(enumerated.label.clone());
							}
						}
					}
					ui.end_row();
				}
			});

			// ui.separator();
			// if ui.button("Delete").clicked() {
			// 	// self.service.delete_event(entry.uuid);
			// }
		}
	}

	fn draw_trace_templates_main_panel(&mut self, ui: &mut egui::Ui) {
		ui.horizontal(|ui| {
			self.trace_we_building
				.iter()
				.enumerate()
				.for_each(|(index, event)| {
					if ui.button(&event.trace_template.name).clicked() {
						self.trace_index = Some(index);
					}
				});
		});
		ui.separator();
		let mut removed = false;
		if let Some(trace_index) = self.trace_index {
			if let Some(builder) = self.trace_we_building.get_mut(trace_index) {
				ui.heading(format!("Trace: {}", builder.trace_template.name));
				ui.horizontal(|ui| {
					ui.label("Name");
					ui.text_edit_singleline(&mut builder.name);
				});
				ui.separator();
				if ui.button("Create").clicked() {
					self.service.save_trace(builder);
					removed = true;
				}
				if ui.button("Cancel").clicked() {
					removed = true;
				}

				let mut traces_included = Vec::new();
				let mut traces_excluded = Vec::new();
				for trace in self.service.list_traces(self.project_uuid) {
					if builder.origin_traces.contains(&trace) {
						traces_included.push(trace);
					} else {
						// filter these to possible ones
						traces_excluded.push(trace);
					}
				}
				// TraceBuilder

				if traces_included.len() > 0 {
					ui.horizontal(|ui| {
						ui.label("Current trace origins: ");
						for trace in traces_included {
							ui.label(&trace.name);
						}
					});
				}

				if traces_excluded.len() > 0 {
					egui::ComboBox::from_label("Add trace origin")
						.selected_text(format!("{:?}", builder.selected_trace))
						.show_ui(ui, |ui| {
							for trace in traces_excluded {
								ui.selectable_value(
									&mut builder.selected_trace,
									service::TraceSelection::Selected(trace.clone()),
									&trace.name,
								);
							}
						});
					if ui.button("Add").on_hover_text("Add origin trace").clicked() {
						if let service::TraceSelection::Selected(trace_uuid) =
							&builder.selected_trace
						{
							builder.origin_traces.insert(trace_uuid.clone());
						}
						builder.selected_trace = service::TraceSelection::None;
					}
				}
			} else {
				self.trace_index = None;
			}
		}

		if removed {
			if let Some(trace_index) = self.trace_index {
				self.trace_we_building.remove(trace_index);
				if self.trace_we_building.len() == 0 {
					self.trace_index = None;
				} else if trace_index >= self.trace_we_building.len() {
					self.trace_index = Some(self.trace_we_building.len() - 1);
				}
			}
		}
	}

	fn draw_traces_main_panel(&mut self, ui: &mut egui::Ui) {
		if let Some(entry) = &self.trace_we_viewing {
			ui.heading(entry.name.to_string());
			// ui.horizontal(|ui| {
			// 	ui.label("Template");
			// 	ui.label(entry.name.clone());
			// });
			ui.horizontal(|ui| {
				ui.label("Template");
				if let Some(template) = &entry.trace_template {
					ui.label(template.name.clone());
				} else {
					ui.label("Missing trace template".to_string());
				}
			});
			for tag in &entry.tags {
				ui.label(tag);
			}
			if let Some(completion) = &entry.completion {
				ui.label(format_a_time(&completion.completed_at));
			}
			if let Some(last_event) = &entry.last_event {
				ui.label("Last event");
				ui.label(
					last_event
						.event_template
						.as_ref()
						.map(|template| template.name.clone())
						.unwrap_or("Missing event template".into()),
				);
				ui.label(format_a_time(&last_event.created_at));
			} else {
				ui.label("No events");
			}

			ui.separator();
			ui.label("Suggested events");
			egui::Grid::new("suggested_events").show(ui, |ui| {
				for event in &entry.suggested_event_templates {
					ui.label(&event.name);
					if ui.button("Create").clicked() {
						if let Ok(event) =
							self.service.create_event(&entry, event.event_template_uuid)
						{
							self.event_we_building.push(event);
							self.tab = Tab::EventTemplates;
							self.event_index = Some(self.event_we_building.len() - 1);
						}
					}
					ui.end_row();
				}
			});

			ui.separator();
			ui.label("Other events");
			egui::Grid::new("other_events").show(ui, |ui| {
				for event in &entry.other_event_templates {
					ui.label(&event.name);
					if ui.button("Create").clicked() {
						if let Ok(event) =
							self.service.create_event(&entry, event.event_template_uuid)
						{
							self.event_we_building.push(event);
							self.tab = Tab::EventTemplates;
							self.event_index = Some(self.event_we_building.len() - 1);
						}
					}
					ui.end_row();
				}
			});

			if ui.button("Complete").clicked() {
				self.service.complete_trace(entry.trace_uuid);
				self.trace_we_viewing = None;
			}
		}
	}

	fn draw_main_panel(&mut self, ctx: &egui::Context) {
		egui::CentralPanel::default().show(ctx, |ui| match self.tab {
			Tab::EventTemplates => self.draw_event_templates_main_panel(ui),
			Tab::Events => self.draw_events_main_panel(ui),
			Tab::TraceTemplates => self.draw_trace_templates_main_panel(ui),
			Tab::Traces => self.draw_traces_main_panel(ui),
		});
	}

	fn draw_top_panel(&mut self, ctx: &egui::Context) {
		egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
			egui::menu::bar(ui, |ui| {
				// NOTE: no File->Quit on web pages!
				let is_web = cfg!(target_arch = "wasm32");
				if !is_web {
					ui.menu_button("File", |ui| {
						if ui.button("Quit").clicked() {
							if let Err(err) = self.service.save_to_disk() {
								log::error!("Failed to write to disk: {}", err);
							}
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
						if ui.button("Save").clicked() {
							if let Err(err) = self.service.save_to_disk() {
								log::error!("Failed to write to disk: {}", err);
							}
						}
						if ui.button("Load").clicked() {
							if let Err(err) = self.service.load_from_disk() {
								log::error!("Failed to write to disk: {}", err);
							}
						}
						if ui.button("Import").clicked() {
							let definition = &std::path::PathBuf::from(
								"/work/projects/journal/template/eframe_template/projects/bread/definition.json");

							if let Err(err) =
								self.service.import_definition("Bread".into(), &definition)
							{
								log::error!("Failed to import definition: {:?}", err);
							}
						}
					});
					ui.add_space(16.0);
				}

				// Should reset the ui to the default state
				ui.horizontal(|ui| {
					if ui.button("Trace Templates").clicked() {
						self.tab = Tab::TraceTemplates;
					}
					if ui.button("Event Templates").clicked() {
						self.tab = Tab::EventTemplates;
					}
					if ui.button("Traces").clicked() {
						self.tab = Tab::Traces;
					}
					if ui.button("Events").clicked() {
						self.tab = Tab::Events;
					}
				});

				// egui::widgets::global_dark_light_mode_buttons(ui);
			});
		});
	}
}

fn show_field(ui: &mut egui::Ui, field: &mut service::FieldSuggestion) {
	ui.label(&field.name);
	match &mut field.value {
		service::FieldValueSuggestion::Text(text) => {
			if let Some(value) = &mut text.value {
				ui.text_edit_singleline(value);
				if ui.button("Clear").clicked() {
					text.value = None;
				}
			} else {
				ui.label("No value");
				if ui.button("Set").clicked() {
					text.value = Some(text.default_value.as_ref().unwrap_or(&"".into()).clone());
				}
			}
		}
		service::FieldValueSuggestion::Number(number) => {
			if let Some(value) = &mut number.value {
				ui.add(egui::widgets::DragValue::new(value));
				if ui.button("Clear").clicked() {
					number.value = None;
				}
			} else {
				ui.label("No value");
				if ui.button("Set").clicked() {
					number.value = Some(number.default_value.unwrap_or(0.0));
				}
			}
		}
		service::FieldValueSuggestion::Bool(boolean) => {
			if let Some(value) = &mut boolean.value {
				ui.checkbox(value, "");
				if ui.button("Clear").clicked() {
					boolean.value = None;
				}
			} else {
				ui.label("No value");
				if ui.button("Set").clicked() {
					boolean.value = Some(boolean.default_value.unwrap_or(false));
				}
			}
		}
		service::FieldValueSuggestion::Enumerated(enumerated) => {
			egui::ComboBox::from_label(format!("Select {}", &field.name))
				.selected_text(match &enumerated.selected {
					Some(selection) => selection.label.clone(),
					None => "None".to_string(),
				})
				.show_ui(ui, |ui| {
					ui.selectable_value(&mut enumerated.selected, None, "None".to_string());

					enumerated.options.iter().for_each(|option| {
						ui.selectable_value(
							&mut enumerated.selected,
							Some(option.clone()),
							&option.label,
						);
					});
				});
		}
	}
	// match field.value {
	// 	events::FieldValue::Text(text) => {
	// 		ui.horizontal(|ui| {
	// 			ui.label(&text.name);
	// 			if let Some(value) = &mut text.value {
	// 				ui.text_edit_singleline(value);
	// 			}
	// 		});
	// 	}
	// 	events::Field::Number(number) => {
	// 		ui.horizontal(|ui| {
	// 			ui.label(&number.name);
	// 			if let Some(value) = &mut number.value {
	// 				ui.add(egui::widgets::DragValue::new(value));
	// 			}
	// 		});
	// 	}
	// 	events::Field::Bool(boolean) => {
	// 		ui.horizontal(|ui| {
	// 			// ui.label();
	// 			if let Some(value) = &mut boolean.value {
	// 				ui.checkbox(value, &boolean.name);
	// 			}
	// 		});
	// 	}
	// 	events::Field::Enumerated(enumerated) => {
	// 		ui.horizontal(|ui| {
	// 			ui.label(&enumerated.name);
	// 			ui.label("TODO");
	// 			// ui.checkbox(&mut enumerated.value.unwrap_or(false), "");
	// 		});
	// 	}
	// }
}

impl eframe::App for TemplateApp {
	fn save(&mut self, storage: &mut dyn eframe::Storage) {
		eframe::set_value(storage, eframe::APP_KEY, self);
	}

	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		// Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
		// For inspiration and more examples, go to https://emilk.github.io/egui
		self.draw_top_panel(ctx);
		self.draw_side_panel(ctx);
		self.draw_main_panel(ctx);
	}
}
