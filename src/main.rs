#[allow(dead_code, unused_imports)]

use std::io;
use structopt::StructOpt;


fn split_commas(string: &str) -> Vec<String> {
	let mut values: Vec<String> = Vec::new();
	values.push(String::new());

	let mut key = &mut values[0];

	for chr in string.chars() {
		if chr == ',' {
			values.push(String::new());
			let length = values.len();
			key = &mut values[length - 1];
			continue;
		}

		key.push(chr);
	}

	return values;
}


#[derive(StructOpt, Debug)]
struct Cli {
	// The pattern to parse
	pattern: String,

	// whitelist of LTSV keys
	#[structopt(short = "w", long = "whitelist", default_value = "")]
	whitelist: String,

	// blacklist of LTSV keys
	#[structopt(short = "b", long = "blacklist", default_value = "")]
	blacklist: String,
}


fn parse_ltsv(line: &String) -> Vec<(String, String)> {
	let mut decoded: Vec<(String, String)> = Vec::new();
	decoded.push((String::new(), String::new()));

	let mut escape_char = false;
	let mut parse_value = false;
	let mut index = 0 as usize;
	let mut end_of_line = false;
	let mut current_string = &mut decoded[index].0;

	for chr in line.chars() {
		if end_of_line {
			/* TODO: Handle this error */
			eprintln!("invalid LTSV - received end_of_line");
			unreachable!();
		}

		if escape_char {
			current_string.push(chr);
			escape_char = false;
			continue
		}

		if chr == '\\' {
			escape_char = true;
			continue
		}

		if parse_value && chr == '\t' {
			/* Switch to parsing the key */
			parse_value = false;

			index += 1;
			decoded.push((String::new(), String::new()));
			current_string = &mut decoded[index].0;
			continue;
		}

		if parse_value && chr == '\n' {
			end_of_line = true;
			continue;
		}

		if !parse_value && chr == '=' {
				parse_value = true;
			current_string = &mut decoded[index].1;
			continue
		}

		if "=\n\t\\".contains(chr) {
			continue;
		}

		current_string.push(chr);
	}

	return decoded;
}


fn encode_ltsv(key: &str, value: &str) -> String {
	let mut encoded = String::with_capacity(key.len() + value.len());
	for chr in key.chars() {
		if "\t\n\\".contains(chr) {
			encoded.push('\\')
		}
		encoded.push(chr)
	}

	encoded.push('=');

	for chr in value.chars() {
		if "\t\n\\".contains(chr) {
			encoded.push('\\')
		}
		encoded.push(chr)
	}
	encoded.push('\t');

	return encoded;
}


fn print_line(args: &Cli, line: &[(String, String)]) -> io::Result<()> {
	let mut whitelist: Vec<String> = Vec::new();
	let mut blacklist: Vec<String> = Vec::new();

	if args.whitelist.len() > 0 {
		whitelist = split_commas(&args.whitelist);
	}

	if args.blacklist.len() > 0 {
		blacklist = split_commas(&args.blacklist);
	}

	for (key, value) in line.iter() {
		/* If we're in whitelist mode, check for membership */
		if whitelist.len() > 0 {
			if !whitelist.contains(key) {
				continue;
			}
		}

		if blacklist.contains(key) {
			continue;
		}

		print!("{}", encode_ltsv(&key, &value));
	}
	println!();
	Ok(())
}


fn passes_grep(args: &Cli, line: &[(String, String)]) -> bool {
	let tags = parse_ltsv(&args.pattern);
	let (key, value) = (&tags[0].0, &tags[0].1);

	for (k, v) in line.iter() {
		if (key, value) == (k, 	v) {
			return true;
		}
	}

	return false;
}


fn search<R: io::BufRead>(args: Cli, reader: R) -> io::Result<()> {
	for line in reader.lines() {
		let line = parse_ltsv(&line?);
		if !passes_grep(&args, &line) {
			continue;
		}
		print_line(&args, &line)?;
	}
	Ok(())
}


fn main() -> io::Result<()> {
	let args = Cli::from_args();
	/* Open Stdin */
	let stdin = io::stdin();
	let handle = stdin.lock();
	search(args, handle)?;
	Ok(())
}
