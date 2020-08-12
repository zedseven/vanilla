use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use neon::prelude::*;

mod merge_files;

fn lol(mut cx: FunctionContext) -> JsResult<JsString> {
	let mergeable_files_res = merge_files::init();
	let mergeable_files;
	match mergeable_files_res {
		Err(err) => {
			println!("{:?}", err);
			// TODO: return err
			return Ok(cx.string(format!("An error ocurred: {:?}", err)))
		},
		_ => mergeable_files = mergeable_files_res.unwrap()
	}
	
	let base_path = "testFiles/base/vehicles.meta";
	let new_path = "testFiles/new/vehicles.meta";
	let out_path = "testFiles/merged/vehicles.meta";
	let merge_res = merge_files::merge(&mergeable_files, base_path, new_path, out_path);
	if merge_res.is_err() {
		println!("{:?}", merge_res.unwrap());
	}
	
	Ok(cx.string("lol"))
}

register_module!(mut cx, {
	//cx.export_function("readFile", read_file)
	cx.export_function("lol", lol)
});
