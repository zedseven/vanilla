use neon::prelude::*;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod merge_files;

const VERSION_MAX: u8 = 0;
const VERSION_MID: u8 = 1;
const VERSION_MIN: u8 = 0;

fn version(mut cx: FunctionContext) -> JsResult<JsString> {
	Ok(cx.string(format!(
		"Vanilla version: v{}.{}.{}",
		VERSION_MAX, VERSION_MID, VERSION_MIN
	)))
}

fn vanilla(mut cx: FunctionContext) -> JsResult<JsString> {
	let mergeable_files_res = merge_files::init();
	let mergeable_files;
	match mergeable_files_res {
		Err(err) => {
			eprintln!("{:?}", err);
			// TODO: return err
			return Ok(cx.string(format!("An error occurred: {:?}", err)));
		}
		_ => mergeable_files = mergeable_files_res.unwrap(),
	}

	let base_path = "testFiles/base/vehicles.meta";
	let new_path = "testFiles/new/vehicles.meta";
	let out_path = "testFiles/merged/vehicles.meta";
	let merge_res = merge_files::merge(&mergeable_files, base_path, new_path, out_path);
	if merge_res.is_err() {
		eprintln!("{:?}", merge_res.unwrap());
	}

	Ok(cx.string("lol"))
}

fn merge_files(mut cx: FunctionContext) -> JsResult<JsBoolean> {
	let base_path = cx.argument::<JsString>(0)?.value();
	let new_path = cx.argument::<JsString>(1)?.value();
	let out_path = cx.argument::<JsString>(2)?.value();

	let mergeable_files_res = merge_files::init();
	let mergeable_files;
	match mergeable_files_res {
		Err(err) => {
			eprintln!("{:?}", err);
			return Ok(cx.boolean(false));
		}
		_ => mergeable_files = mergeable_files_res.unwrap(),
	}

	let merge_res = merge_files::merge(
		&mergeable_files,
		base_path.as_str(),
		new_path.as_str(),
		out_path.as_str(),
	);
	if merge_res.is_err() {
		return Ok(cx.boolean(false));
	}
	return Ok(cx.boolean(true));
}

register_module!(mut cx, {
	cx.export_function("version", version);
	cx.export_function("mergeFiles", merge_files);
	cx.export_function("vanilla", vanilla)
});
