
use uuid::Uuid;

pub struct NotFoundError {
	pub message: String,
}

impl Into<ParsingError> for String {
	fn into(self) -> ParsingError {
		ParsingError { message: self }
	}
}
impl Into<ParsingError> for &str {
	fn into(self) -> ParsingError {
		ParsingError {
			message: self.to_string(),
		}
	}
}

pub struct EventTemplateNotFound {
	pub event_template_uuid: Uuid,
}
pub struct TraceTemplateNotFound {
	pub trace_template_uuid: Uuid,
}

#[derive(Debug)]
pub struct ParsingError {
	pub message: String,
}

impl Into<EventTemplateNotFound> for Uuid {
	fn into(self) -> EventTemplateNotFound {
		EventTemplateNotFound {
			event_template_uuid: self,
		}
	}
}

impl Into<TraceTemplateNotFound> for Uuid {
	fn into(self) -> TraceTemplateNotFound {
		TraceTemplateNotFound {
			trace_template_uuid: self,
		}
	}
}

impl Into<ParsingError> for std::io::Error {
	fn into(self) -> ParsingError {
		ParsingError {
			message: self.to_string(),
		}
	}
}
impl Into<ParsingError> for serde_json::Error {
	fn into(self) -> ParsingError {
		ParsingError {
			message: self.to_string(),
		}
	}
}
