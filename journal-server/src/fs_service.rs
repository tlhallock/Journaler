
use crate::definition;
use journal_lib::errors;
use crate::events;
use crate::project;
use journal_lib::service;
use journal_lib::views;

use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;
use std::io::Write;
use uuid::Uuid;

use directories_next::ProjectDirs;
use crate::builders;

fn get_project_dirs() -> ProjectDirs {
	ProjectDirs::from("org", "Hallock",  "Journaler").unwrap()
}

fn get_data_directory() -> std::path::PathBuf {
	get_project_dirs().data_dir().to_path_buf()
}

fn get_projects_directory() -> std::path::PathBuf {
	get_data_directory().join("projects")
}


#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProjectDefinition {
	pub project_uuid: Uuid,
	pub event_templates: Vec<definition::EventTemplate>,
	pub trace_templates: Vec<definition::TraceTemplate>,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProjectData {
	pub project_uuid: Uuid,
	pub events: Vec<events::Event>,
	pub traces: Vec<events::Trace>,
}

#[derive(Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct FileSystemEventsService {
	pub projects_directory: std::path::PathBuf,
	pub event_templates: HashMap<Uuid, definition::EventTemplate>,
	pub trace_templates: HashMap<Uuid, definition::TraceTemplate>,
	pub events: HashMap<Uuid, events::Event>,
	pub traces: HashMap<Uuid, events::Trace>,
	pub projects: HashMap<Uuid, project::Project>,
}

impl service::EventsService for FileSystemEventsService {
	fn list_event_templates(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::EventTemplateItemView>> {
		let last_used: HashMap<Uuid, DateTime<Utc>> = self
			.event_templates
			.iter()
			.filter(|(_, trace)| {
				project_uuid.as_ref().map_or(true, |uuid| {
					self.trace_templates
						.get(&trace.trace_template_uuid)
						.map_or(false, |trace_template| &trace_template.project_uuid == uuid)
				})
			})
			.map(|(uuid, trace)| (trace.event_template_uuid, trace.created_at))
			.collect();
		let mut ret: Vec<views::EventTemplateItemView> = self
			.event_templates
			.iter()
			.map(|(uuid, template)| template.to_item(last_used.get(&uuid).cloned()))
			.collect();
		ret.sort_by(|a, b| a.last_used.cmp(&b.last_used));
		Box::new(ret.into_iter())
	}

	fn create_event(
		&self,
		trace: &views::TraceView,
		template_uuid: Uuid,
	) -> Result<service::EventBuilder, errors::EventTemplateNotFound> {
		Ok(self
			.event_templates
			.get(&template_uuid)
			.ok_or(template_uuid.into())?
			.create_builder(trace))
	}
	
	fn create_trace(
		&self,
		trace_template_uuid: Uuid,
	) -> Result<service::TraceBuilder, errors::TraceTemplateNotFound> {
		Ok(self
			.trace_templates
			.get(&trace_template_uuid)
			.ok_or(trace_template_uuid.into())?
			.create_builder())
	}

	fn complete_trace(&mut self, trace_uuid: Uuid) {
		if let Some(trace) = self.traces.get_mut(&trace_uuid) {
			trace.completion = Some(views::TraceCompletion {
				completed_at: Utc::now(),
			});
		}
	}

	fn save_event(&mut self, event_builder: &service::EventBuilder) {
		let event = builders::build_event(event_builder);
		self.events.insert(event.event_uuid, event);
	}

	fn save_trace(&mut self, trace_builder: &service::TraceBuilder) {
		let trace = builders::build_trace(trace_builder);
		self.traces.insert(trace.trace_uuid, trace);
	}

	fn view_event(&self, event_uuid: Uuid) -> Option<views::EventView> {
		self.events
			.get(&event_uuid)
			.map(|trace| self.view_found_event(trace))
	}


	fn view_trace(&self, trace_uuid: Uuid) -> Option<views::TraceView> {
		self.traces
			.get(&trace_uuid)
			.map(|trace| self.create_trace_view(trace))
	}
	fn list_events(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::EventItemView> + '_> {
		if let Some(project_uuid) = project_uuid {
			Box::new(
				self.events
					.values()
					.filter(move |trace| {
						self.project_contains_event(&project_uuid, &trace.event_uuid)
					})
					.map(move |trace| self.view_found_event_item(trace)),
			)
		} else {
			Box::new(
				self.events
					.values()
					.map(|trace| self.view_found_event_item(trace)),
			)
		}
	}

	fn list_traces(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::TraceItemView> + '_> {
		Box::new(
			self.traces
				.values()
				.filter(|&trace| trace.completion.is_none())
				.filter(move |trace| {
					project_uuid.as_ref().map_or(true, |uuid| {
						self.trace_templates
							.get(&trace.trace_template_uuid)
							.map_or(false, |trace_template| &trace_template.project_uuid == uuid)
					})
				})
				.map(|trace| (trace, self.trace_templates.get(&trace.trace_template_uuid)))
				// .flat_map(
				// 	|(trace, trace_template)|
				// 	match trace_template {
				// 		Some(trace_template) => Some((trace, trace_template)),
				// 		None => None,
				// 	}
				// )
				// .map(|(trace, trace_template)| (trace, trace_template.name.clone()))
				.map(|(trace, maybe_trace_template)| {
					(
						trace,
						maybe_trace_template.map(|trace_template| trace_template.name.clone()),
					)
				})
				.map(|(trace, trace_template_name)| trace.to_item(trace_template_name)),
		)
	}

	fn list_trace_templates(
		&self,
		project_uuid: Option<Uuid>,
	) -> Box<dyn Iterator<Item = views::TraceTemplateItemView>> {
		// TODO filter on project uuid
		let last_used: HashMap<Uuid, DateTime<Utc>> = self
			.trace_templates
			.iter()
			.map(|(_, trace)| (trace.trace_template_uuid, trace.created_at))
			.collect();
		let mut ret: Vec<views::TraceTemplateItemView> = self
			.trace_templates
			.iter()
			.filter(|(_, trace)| {
				project_uuid
					.as_ref()
					.map_or(true, |uuid| &trace.project_uuid == uuid)
			})
			.map(|(uuid, template)| template.to_item(last_used.get(uuid).map(|x| x.clone())))
			.collect();
		ret.sort_by(|a, b| a.last_used.cmp(&b.last_used));
		Box::new(ret.into_iter())
	}

	fn import_definition(
		&mut self,
		project_name: String,
		definition_path: &std::path::PathBuf,
	) -> Result<(), errors::ParsingError> {
		let definition_file = std::fs::File::open(definition_path).map_err(|e| e.into())?;
		let definition_js: serde_json::Value =
			serde_json::from_reader(definition_file).map_err(|e| e.into())?;

		let project_uuid = Self::parse_required_uuid(&definition_js, "project-uuid")?;
		let project_definition = ProjectDefinition {
			project_uuid,
			event_templates: definition_js
				.get("event-templates")
				.map_or(Ok(vec![]), |val| {
					FileSystemEventsService::parse_event_templates(val, project_uuid)
				})?,
			trace_templates: definition_js
				.get("trace-templates")
				.map_or(Ok(vec![]), |val| {
					FileSystemEventsService::parse_trace_templates(val, project_uuid)
				})?,
		};
		self.projects.insert(
			project_definition.project_uuid,
			project::Project {
				project_uuid: project_definition.project_uuid,
				name: project_name,
				created_at: Utc::now(),
			},
		);
		self.import_project_definition(project_definition);
		Ok(())
	}

	fn save_to_disk(&self) -> Result<(), std::io::Error> {
		let projects_directory = get_projects_directory();
		println!("Writing projects to disk: {:?}", projects_directory);
		for (uuid, project) in self.projects.iter() {
			let project_path = projects_directory.join(uuid.to_string());
			std::fs::create_dir_all(&project_path)?;

			{
				let project_file = project_path.join("project.json");
				let file = std::fs::File::create(project_file)?;
				let writer = std::io::BufWriter::new(&file);
				serde_json::to_writer_pretty(writer, project)?;
			}

			{
				let project_data = self.collect_project_data(uuid);
				let project_data_file = project_path.join("data.json");
				let file = std::fs::File::create(project_data_file)?;
				let writer = std::io::BufWriter::new(&file);
				serde_json::to_writer_pretty(writer, &project_data)?;
			}

			{
				let project_definition = self.collect_project_definition(uuid);
				println!(
					"Writing project definition to disk: {:}",
					&self.event_templates.len()
				);
				println!(
					"Writing project definition to disk: {:?}",
					&project_definition.event_templates.len()
				);
				let project_definition_file = project_path.join("definition.json");

				let file = std::fs::File::create(project_definition_file)?;
				let writer = std::io::BufWriter::new(&file);
				serde_json::to_writer_pretty(writer, &project_definition)?;
			}
		}

		Ok(())
	}

	fn load_from_disk(&mut self) -> Result<(), std::io::Error> {
		let projects_directory = &get_projects_directory();
		for entry in std::fs::read_dir(projects_directory)? {
			let entry = entry?;
			let path = entry.path();
			if !path.is_dir() {
				continue;
			}

			{
				let project_path = path.join("project.json");
				if !project_path.exists() {
					continue;
				}
				let file: std::fs::File = std::fs::File::open(&project_path)?;
				// let file_string = std::fs::read_to_string(&project_path)?;
				// println!("About to read the project: {}", file_string);
				// let project: project::Project = serde_json::from_str(&file_string)?;
				let project: project::Project = serde_json::from_reader(file)?;
				// println!("Read the project");
				self.projects.insert(project.project_uuid, project);
			}

			{
				let definition_path = path.join("definition.json");
				if !definition_path.exists() {
					eprint!("Definition path does not exist: {:?}", definition_path);
					continue;
				}
				let file = std::fs::File::open(&definition_path)?;
				let project_definition: ProjectDefinition = serde_json::from_reader(file)?;
				self.import_project_definition(project_definition);
			}

			{
				let data_path = path.join("data.json");
				if !data_path.exists() {
					eprint!("Data path does not exist: {:?}", data_path);
					continue;
				}
				let file = std::fs::File::open(&data_path)?;
				let project_data: ProjectData = serde_json::from_reader(file)?;
				self.import_project_data(project_data);
			}
		}

		Ok(())
	}

}

impl FileSystemEventsService {
	fn view_event_template_item(
		&self,
		event_template_uuid: Uuid,
	) -> Option<views::EventTemplateItemView> {
		self.event_templates
			.get(&event_template_uuid)
			.map(|event_template| event_template.to_item(None))
	}

	pub fn load(path: &std::path::Path) -> std::io::Result<Self> {
		Ok(Self::default())

		// EventsInterface {
		// 	templates: HashMap::new(),
		// 	events: HashMap::new(),
		// 	traces: HashMap::new(),
		// }
	}
	pub fn save(&self, path: &std::path::Path) -> std::io::Result<()> {
		Ok(())
	}

	fn get_last_event_for_trace(&self, trace_uuid: Uuid) -> Option<views::EventItemView> {
		self.events
			.values()
			.filter(|trace| trace.trace_uuid == trace_uuid)
			.max_by_key(|trace| trace.created_at)
			.map(|trace| self.view_found_event_item(trace))
	}

	fn create_trace_view(&self, trace: &events::Trace) -> views::TraceView {
		let last_event = self.get_last_event_for_trace(trace.trace_uuid);
		let last_used = last_event.as_ref().map(|x| x.created_at.clone());
		let mut suggested_event_templates = vec![];
		let mut other_event_templates = vec![];
		if let Some(trace_template) = self.trace_templates.get(&trace.trace_template_uuid) {
			let suggested = &trace_template.get_suggested_event_templates(
				last_event
					.as_ref()
					.map(|trace| {
						trace
							.event_template
							.as_ref()
							.map(|event_template| event_template.event_template_uuid)
					})
					.flatten(),
			);
			for event_template in self.event_templates.values() {
				if suggested.contains(&event_template.event_template_uuid) {
					suggested_event_templates.push(event_template.to_item(last_used));
				} else if event_template.trace_template_uuid == trace.trace_template_uuid {
					other_event_templates.push(event_template.to_item(last_used));
				}
			}
		} else {
			eprintln!("Trace template not found: {:?}", trace.trace_template_uuid);
		}
		views::TraceView {
			trace_uuid: trace.trace_uuid,
			name: trace.name.clone(),
			trace_template: self
				.trace_templates
				.get(&trace.trace_template_uuid)
				.map(|template| template.to_item(last_used)),
			tags: vec![], // TODO
			completion: trace.completion.clone(),
			last_event,
			suggested_event_templates,
			other_event_templates,
		}
	}

	fn project_contains_trace(&self, project_uuid: &Uuid, trace_uuid: &Uuid) -> bool {
		self.traces.get(trace_uuid).map_or(false, |trace| {
			self.trace_templates
				.get(&trace.trace_template_uuid)
				.map_or(false, |trace_template| {
					&trace_template.project_uuid == project_uuid
				})
		})
	}

	fn project_contains_trace_template(
		&self,
		project_uuid: &Uuid,
		trace_template_uuid: &Uuid,
	) -> bool {
		self.trace_templates
			.get(&trace_template_uuid)
			.map_or(false, |trace_template| {
				&trace_template.project_uuid == project_uuid
			})
	}

	fn project_contains_event_template(
		&self,
		project_uuid: &Uuid,
		event_template_uuid: &Uuid,
	) -> bool {
		self.event_templates
			.get(&event_template_uuid)
			.map_or(false, |event_template| {
				self.project_contains_trace_template(
					project_uuid,
					&event_template.trace_template_uuid,
				)
			})
	}

	fn project_contains_event(&self, project_uuid: &Uuid, event_uuid: &Uuid) -> bool {
		self.events.get(&event_uuid).map_or(false, |trace| {
			self.project_contains_trace(project_uuid, &trace.trace_uuid)
				&& self.project_contains_event_template(project_uuid, &trace.event_template_uuid)
		})
	}

	// pub fn list_events(&self, project_uuid: Option<Uuid>) -> Box<dyn Iterator<Item = views::EventItemView>> {
	// 	let project_uuid = project_uuid.clone();  // Clone the project_uuid here
	// 	Box::new(
	// 		self.events
	// 			.values()
	// 			.filter(move |trace| project_uuid.as_ref().map_or(true, |uuid| {
	// 				self.event_templates
	// 					.get(&trace.template_uuid)
	// 					.map_or(false, |event_template| event_template.project_uuid == *uuid)
	// 			}))
	// 			.map(move |trace| self.view_found_event_item(trace))
	// 	)
	// }

	fn view_found_event_item(&self, event: &events::Event) -> views::EventItemView {
		views::EventItemView {
			event_uuid: event.event_uuid,
			event_template: self
				.event_templates
				.get(&event.event_template_uuid)
				.map(|template| template.to_item(None)), // TODO last used
			trace_name: self
				.traces
				.get(&event.trace_uuid)
				.map(|trace| trace.name.clone()),
			created_at: event.created_at,
		}
	}
	fn view_found_event(&self, event: &events::Event) -> views::EventView {
		views::EventView {
			event_uuid: event.event_uuid,
			event_template: self.view_event_template_item(event.event_template_uuid),
			fields: event.fields.iter().map(|x| x.to_view()).collect(),
			tags: event.tags.clone(),
			trace: self.traces.get(&event.trace_uuid).map(|trace| {
				trace.to_item(
					self.trace_templates
						.get(&trace.trace_template_uuid)
						.map(|trace_template| trace_template.name.clone()),
				)
			}),
			began_at: event.began_at,
			created_at: event.created_at,
		}
	}

	pub fn save_project(
		project_path: &std::path::PathBuf,
		project: &project::Project,
	) -> Result<(), std::io::Error> {
		let mut project_file = std::fs::File::create(project_path)?;
		let project_str = serde_json::to_string(&project)?;
		project_file.write_all(project_str.as_bytes())?;
		Ok(())
	}

	fn parse_required_uuid(
		val: &serde_json::Value,
		field: &str,
	) -> Result<Uuid, errors::ParsingError> {
		Uuid::parse_str(
			val.as_object()
				.ok_or(errors::ParsingError {
					message: "Expected object".into(),
				})?
				.get(field)
				.ok_or_else(|| errors::ParsingError {
					message: format!("Field {} is required", field),
				})?
				.as_str()
				.ok_or_else(|| errors::ParsingError {
					message: format!("Field {} is supposed to be a string", field),
				})?,
		)
		.map_err(|e| errors::ParsingError {
			message: format!("Failed to parse UUID: {}", e),
		})
	}
	fn parse_optional_uuid(
		val: &serde_json::Value,
		field: &str,
	) -> Result<Uuid, errors::ParsingError> {
		val.as_object()
			.ok_or(errors::ParsingError {
				message: "Expected object".into(),
			})?
			.get(field)
			.map(|val| val.as_str())
			.flatten()
			.map(|uuid| Uuid::parse_str(uuid))
			.unwrap_or(Ok(Uuid::new_v4()))
			.map_err(|e| errors::ParsingError {
				message: e.to_string(),
			})
	}
	fn parse_str(
		val: &serde_json::Value,
		field: &str,
	) -> Result<Option<String>, errors::ParsingError> {
		Ok(val
			.as_object()
			.ok_or(errors::ParsingError {
				message: "Expected object".into(),
			})?
			.get(field)
			.map(|val| val.as_str())
			.flatten()
			.map(|val| val.to_string()))
	}

	fn map_name(name: &str) -> String {
		name.to_string().replace(" ", "-").to_lowercase()
	}

	fn parse_enumerated_template(
		val: &serde_json::Value,
	) -> Result<definition::EnumeratedTemplate, errors::ParsingError> {
		let obj = val.as_object().ok_or(errors::ParsingError {
			message: "Expected object".into(),
		})?;
		let options = obj
			.get("options")
			.ok_or(errors::ParsingError {
				message: "Expected 'options' field".into(),
			})?
			.as_array()
			.ok_or(errors::ParsingError {
				message: "Expected 'options' to be an array".into(),
			})?
			.iter()
			.map(|val| {
				let option = val
					.as_object()
					.ok_or("Expected the option to be an object".into())?;
				let label = option
					.get("label")
					.ok_or("Expected 'label' field".into())?
					.as_str()
					.ok_or("Expected 'label' to be a string".into())?;
				let name = option
					.get("name")
					.map(|val| val.as_str())
					.flatten()
					.map(|val| val.to_string())
					.unwrap_or(Self::map_name(label));
				Ok(views::EnumerationOption {
					name: name.to_string(),
					label: label.to_string(),
				})
			})
			.collect::<Result<Vec<views::EnumerationOption>, errors::ParsingError>>()?;

		let default_value = obj
			.get("default-value")
			.map(|val| {
				let name = val
					.as_str()
					.ok_or("Expected 'default-value' to be a string".into())
					.map(|val| val.to_string())?;

				let default_value = options
					.iter()
					.find(|option| option.name == name || option.label == name)
					.ok_or(format!("Default value {} not found in options", name).into())?;

				Ok(default_value.clone())
			})
			.transpose()?;
		Ok(definition::EnumeratedTemplate {
			default_value,
			options,
		})
	}
	
	fn parse_field_template(
		val: &serde_json::Value,
	) -> Result<definition::FieldTemplate, errors::ParsingError> {
		val.as_object().map_or(
			Err(errors::ParsingError {
				message: "Expected object".into(),
			}),
			|obj| {
				// for (key, value) in obj {
				// 	println!("field: {} -> {}", key, value);
				// }
				let field_type =
					Self::parse_str(val, "type")?.ok_or_else(|| errors::ParsingError {
						message: "Field type is required".into(),
					})?;
				let label = Self::parse_str(val, "label")?.ok_or_else(|| errors::ParsingError {
					message: "Field label is required".into(),
				})?;
				let name = Self::parse_str(val, "name")?.unwrap_or(Self::map_name(&label));

				let default_value = val.get("default-value");

				Ok(definition::FieldTemplate {
					name,
					label,
					value: match field_type.as_str() {
						"Number" => Ok(definition::FieldValueTemplate::Number(
							definition::NumberTemplate {
								default_value: default_value
									.map(|val| {
										val.as_f64().ok_or_else(|| errors::ParsingError {
											message: "Expected a number".into(),
										})
									})
									.transpose()?,
							},
						)),
						"Text" => Ok(definition::FieldValueTemplate::Text(
							definition::TextTemplate {
								default_value: default_value
									.map(|val| {
										val.as_str().map(|val| val.to_string()).ok_or_else(|| {
											errors::ParsingError {
												message: "Expected a string".into(),
											}
										})
									})
									.transpose()?,
							},
						)),
						"Boolean" => Ok(definition::FieldValueTemplate::Bool(
							definition::BoolTemplate {
								default_value: default_value
									.map(|val| {
										val.as_bool().ok_or_else(|| errors::ParsingError {
											message: "Expected a boolean".into(),
										})
									})
									.transpose()?,
							},
						)),
						"Enumerated" => Ok(definition::FieldValueTemplate::Enumerated(
							Self::parse_enumerated_template(val)?,
						)),
						_ => Err(format!("Unknown field type: {}", field_type).into()),
					}?,
				})
			},
		)
	}

	fn parse_field_templates(
		val: &serde_json::Value,
	) -> Result<Vec<definition::FieldTemplate>, errors::ParsingError> {
		val.as_array().map_or(
			Err(errors::ParsingError {
				message: "Expected array".into(),
			}),
			|fields| {
				fields
					.iter()
					.map(|field| Self::parse_field_template(field))
					.collect()
			},
		)
	}

	fn parse_event_template(
		val: &serde_json::Value,
		project_uuid: Uuid,
	) -> Result<definition::EventTemplate, errors::ParsingError> {
		val.as_object().map_or(
			Err(errors::ParsingError {
				message: "Expected object".into(),
			}),
			|obj| {
				Ok(definition::EventTemplate {
					event_template_uuid: Self::parse_required_uuid(val, "event-template-uuid")?,
					trace_template_uuid: Self::parse_required_uuid(val, "trace-template-uuid")?,
					name: Self::parse_str(val, "name")?
						.unwrap_or("Unnamed Event Template".to_string()),
					created_at: Utc::now(), // TODO
					fields: obj
						.get("fields")
						.map_or(Ok(vec![]), |fields| Self::parse_field_templates(fields))?,
					default_tags: vec![], // TODO
				})
			},
		)
	}

	fn parse_event_templates(
		val: &serde_json::Value,
		project_uuid: Uuid,
	) -> Result<Vec<definition::EventTemplate>, errors::ParsingError> {
		val.as_array().map_or(
			Err(errors::ParsingError {
				message: "Expected array".into(),
			}),
			|event_templates| {
				event_templates
					.iter()
					.map(|event_template| Self::parse_event_template(event_template, project_uuid))
					.collect()
			},
		)
	}

	fn parse_trace_flow_entry(
		// from: Uuid,
		val: &serde_json::Value,
	) -> Result<definition::TraceFlowEntry, errors::ParsingError> {
		Ok(definition::TraceFlowEntry {
			from: Self::parse_required_uuid(val, "from")?,
			to: val
				.as_object()
				.ok_or(errors::ParsingError {
					message: "Expected flow to be an object".into(),
				})?
				.get("to")
				.ok_or(errors::ParsingError {
					message: "Expected flow to have a 'to' field".into(),
				})?
				.as_array()
				.ok_or(errors::ParsingError {
					message: "Expected flow 'to' to be an array".into(),
				})?
				.iter()
				.map(|val| {
					val.as_str().map_or(
						Err(errors::ParsingError {
							message: format!("Expected a string, found: {:?}", val),
						}),
						|uuid_str| {
							Uuid::parse_str(uuid_str).map_err(|e| errors::ParsingError {
								message: e.to_string(),
							})
						},
					)
				})
				.collect::<Result<Vec<Uuid>, errors::ParsingError>>()?,
		})
		// val.as_array().map_or(
		// 	Err(errors::ParsingError {
		// 		message: "Expected a trace flow to be an object".into(),
		// 	}),
		// 	|arr: &Vec<serde_json::Value>| {
		// 		Ok(definition::TraceFlowEntry {
		// 			from,
		// 			to: arr
		// 				.iter()
		// 				.map(|val| {
		// 					val.as_str().map_or(
		// 						Err(errors::ParsingError {
		// 							message: format!("Expected a string, found: {:?}", val),
		// 						}),
		// 						|uuid_str| {
		// 							Uuid::parse_str(uuid_str).map_err(|e| errors::ParsingError {
		// 								message: e.to_string(),
		// 							})
		// 						},
		// 					)
		// 				})
		// 				.collect::<Result<Vec<Uuid>, errors::ParsingError>>()?,
		// 		})
		// 	},
		// )
	}
	fn parse_trace_flow_entries(
		val: &serde_json::Value,
	) -> Result<Vec<definition::TraceFlowEntry>, errors::ParsingError> {
		val.as_array().map_or(
			Err(errors::ParsingError {
				message: format!("Expected an array of trace flow entries. Found: {:?}", val),
			}),
			|flow_entries| {
				flow_entries
					.iter()
					.map(|flow_entry| {
						Self::parse_trace_flow_entry(
							// Uuid::parse_str(uuid_str).map_err(|e| errors::ParsingError {
							// 	message: e.to_string(),
							// })?,
							flow_entry,
						)
					})
					.collect()
			},
		)
	}

	fn parse_trace_template(
		val: &serde_json::Value,
		project_uuid: Uuid,
	) -> Result<definition::TraceTemplate, errors::ParsingError> {
		val.as_object().map_or(
			Err(errors::ParsingError {
				message: "Expected object".into(),
			}),
			|obj| {
				Ok(definition::TraceTemplate {
					trace_template_uuid: Self::parse_required_uuid(val, "trace-template-uuid")?,
					project_uuid,
					name: Self::parse_str(val, "name")?
						.unwrap_or("Unnamed Trace Template".to_string()),
					created_at: Utc::now(), // TODO
					flow: obj.get("transitions").map_or(Ok(vec![]), |transitions| {
						Self::parse_trace_flow_entries(transitions)
					})?,
				})
			},
		)
	}

	fn parse_trace_templates(
		val: &serde_json::Value,
		project_uuid: Uuid,
	) -> Result<Vec<definition::TraceTemplate>, errors::ParsingError> {
		val.as_array().map_or(
			Err(errors::ParsingError {
				message: "Expected array".into(),
			}),
			|trace_templates| {
				trace_templates
					.iter()
					.map(|trace_template| Self::parse_trace_template(trace_template, project_uuid))
					.collect()
			},
		)
	}

	fn collect_project_data(&self, project_uuid: &Uuid) -> ProjectData {
		ProjectData {
			project_uuid: project_uuid.clone(),
			events: self
				.events
				.values()
				.filter(|trace| self.project_contains_event(project_uuid, &trace.event_uuid))
				.map(|trace| trace.clone())
				.collect(),
			traces: self
				.traces
				.values()
				.filter(|trace| self.project_contains_trace(project_uuid, &trace.trace_uuid))
				.map(|trace| trace.clone())
				.collect(),
		}
	}
	fn collect_project_definition(&self, project_uuid: &Uuid) -> ProjectDefinition {
		ProjectDefinition {
			project_uuid: project_uuid.clone(),
			event_templates: self
				.event_templates
				.values()
				.filter(|event_template| {
					self.project_contains_event_template(
						project_uuid,
						&event_template.event_template_uuid,
					)
				})
				.map(|event_template| event_template.clone())
				.collect(),
			trace_templates: self
				.trace_templates
				.values()
				.filter(|trace_template| {
					self.project_contains_trace_template(
						project_uuid,
						&trace_template.trace_template_uuid,
					)
				})
				.map(|trace_template| trace_template.clone())
				.collect(),
		}
	}
	fn import_project_data(&mut self, project_data: ProjectData) {
		self.events.extend(
			project_data
				.events
				.into_iter()
				.map(|trace| (trace.event_uuid, trace)),
		);
		self.traces.extend(
			project_data
				.traces
				.into_iter()
				.map(|trace| (trace.trace_uuid, trace)),
		);
	}
	fn import_project_definition(&mut self, project_definition: ProjectDefinition) {
		self.event_templates.extend(
			project_definition
				.event_templates
				.into_iter()
				.map(|event_template| (event_template.event_template_uuid, event_template)),
		);
		self.trace_templates.extend(
			project_definition
				.trace_templates
				.into_iter()
				.map(|trace_template| (trace_template.trace_template_uuid, trace_template)),
		);
	}

}
