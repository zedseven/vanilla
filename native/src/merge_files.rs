use serde::Deserialize;
use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use thiserror::Error;
use xml::EmitterConfig;
use xmltree::{Element, XMLNode};

#[derive(Deserialize, Debug)]
pub struct MergeableFile {
	file_name: String,
	parent_tags: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct TypesVec {
	meta_file_types: Vec<MergeableFile>,
}

#[derive(Error, Debug)]
pub enum MergeError {
	#[error("Input file could not be read: {0}")]
	IOError(#[from] std::io::Error),
	#[error("TOML file could not be parsed correctly: {0}")]
	TomlParseError(#[from] toml::de::Error),
	#[error("Input file path did not have a filename or was not valid Unicode.")]
	BadFilePath,
	#[error("Files provided to merge are not the same kind of file.")]
	DifferentInputFiles,
	#[error("Files provided to merge are of a kind not in the config. (config/merge_files.toml)")]
	FileTypeNotInConfig,
	#[error("There was a parse error in one of the provided files: {0}")]
	XmlParseError(#[from] xmltree::ParseError),
	#[error("There was a write error when writing an XML document: {0}")]
	XmlWriteError(#[from] xmltree::Error),
}

const CONFIG_PATH: &str = "config/mergeFiles.toml";
const XML_WRITE_CONFIG: xmltree::EmitterConfig = EmitterConfig {
	line_separator: Cow::Borrowed("\r\n"),
	indent_string: Cow::Borrowed("  "),
	perform_indent: true,
	perform_escaping: true,
	write_document_declaration: true,
	normalize_empty_elements: false,
	cdata_to_characters: false,
	keep_element_names_stack: true,
	autopad_comments: true,
	pad_self_closing: true,
};

pub fn init() -> Result<Vec<MergeableFile>, MergeError> {
	let config = File::open(CONFIG_PATH)?;
	let mut config_buf = BufReader::new(config);
	let mut contents = String::new();
	config_buf.read_to_string(&mut contents)?;
	let config_value: TypesVec = toml::from_str(&*contents)?;

	return Ok(config_value.meta_file_types);
}

pub fn merge(
	file_types: &Vec<MergeableFile>,
	base_path: &str,
	additive_path: &str,
	out_path: &str,
) -> Result<(), MergeError> {
	// Validate base and additive paths
	let base_name = Path::new(base_path).file_name();
	if base_name.is_none() {
		return Err(MergeError::BadFilePath);
	}
	let base_name = base_name.unwrap().to_str();
	if base_name.is_none() {
		return Err(MergeError::BadFilePath);
	}
	let base_name = base_name.unwrap();
	let additive_name = Path::new(additive_path).file_name();
	if additive_name.is_none() {
		return Err(MergeError::BadFilePath);
	}
	let additive_name = additive_name.unwrap().to_str();
	if additive_name.is_none() {
		return Err(MergeError::BadFilePath);
	}
	let additive_name = additive_name.unwrap();
	if base_name != additive_name {
		return Err(MergeError::DifferentInputFiles);
	}

	// Get the file type definition
	let mut f_type = None;
	for check_file in file_types {
		if check_file.file_name == base_name {
			f_type = Some(check_file);
			break;
		}
	}
	if f_type.is_none() {
		return Err(MergeError::FileTypeNotInConfig);
	}
	let f_type = f_type.unwrap();

	// Prepare the files
	let base = File::open(base_path)?;
	let base_buf = BufReader::new(base);
	let additive = File::open(additive_path)?;
	let additive_buf = BufReader::new(additive);

	let parent_tag_count = f_type.parent_tags.len();
	let mut item_nodes: Vec<Vec<&XMLNode>> = Vec::new();
	for _ in 0..parent_tag_count {
		item_nodes.push(Vec::new())
	}

	// Build a collection of nodes from the additive file to add to the base
	let additive_top_element = Element::parse(additive_buf)?;
	for i in 0..parent_tag_count {
		let parent = additive_top_element.get_child(f_type.parent_tags[i].as_str());
		match parent {
			None => continue,
			Some(_) => {
				for child in &parent.unwrap().children {
					item_nodes[i].push(child)
				}
			}
		}
	}

	// Go through the base and add in the additive's nodes
	let mut base_top_element = Element::parse(base_buf)?;
	for i in 0..parent_tag_count {
		let mut parent = base_top_element.get_mut_child(f_type.parent_tags[i].as_str());
		match parent {
			None => continue,
			Some(ref mut parent_node) => {
				for new_node in &item_nodes[i] {
					parent_node.children.push(new_node.clone().clone())
				}
			}
		}
	}

	let buffer = File::create(out_path)?;
	base_top_element.write_with_config(buffer, XML_WRITE_CONFIG)?;

	return Ok(());
}
