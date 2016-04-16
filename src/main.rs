extern crate bzip2;
extern crate flate2;
extern crate getopts;
extern crate tar;
extern crate xz2;

use std::env;

use getopts::Options;

mod scancsv;
use scancsv::*;

fn print_usage(progname: &str, opts: Options)
{
	println!
	(
		"{}"
		, opts.usage
		(
			&format!
			(
				"Usage: {} [options] inputfile [ files ... ]"
				, progname
			)
		)
	);
}

fn main()
{
	let args: Vec<String> = env::args().collect();
	let progname: &str = args[0].as_str();

	let mut opts = Options::new();
	opts.optopt("c", "column", "Column index. (default: 0)", "NUM");
	opts.optopt("d", "delimiter", "Separator. (default: tab)", "CHARS");
	opts.optopt
	(
		"a"
		, "array"
		, "Array of comma separated values to look for. (default: \"\")"
		, "VAL,VAL,..."
	);
	opts.optflag("t", "tar", "Input file is a tarball.");
	opts.optflag("z", "gzip", "Input file is compressed in Gzip.");
	opts.optflag("j", "bzip2", "Input file is compressed in Bzip2.");
	opts.optflag("J", "xz", "Input file is compressed in Xz.");
	opts.optopt("o", "out", "Output file. (default: stdout)", "FILE");
	opts.optflag("h", "help", "Show this usage message.");
	let matches = match opts.parse(&args[1..])
	{
		Ok(m) => m
		, Err(e) => panic!(e.to_string())
	};

	if matches.opt_present("h") || matches.free.len() < 1
	{
		print_usage(&progname, opts);
		return;
	}

	// retrieve compression algorithm from options
	let mut compression = Compression::Plain;
	if matches.opt_present("z")
	{
		compression = Compression::Gzip;
	}
	if matches.opt_present("j")
	{
		compression = Compression::Bzip2;
	}
	if matches.opt_present("J")
	{
		compression = Compression::Xz;
	}

	// retrieve whether tar or not from option
	let istar = matches.opt_present("t");

	// retrieve tarball file path from arguments
	let fpath = match matches.free.get(0)
	{
		Some(s) => s
		, None => panic!("This shouldn't happen!")
	};

	// retrieve column index from option
	let mut column: usize = 0;
	if let Some(s) = matches.opt_str("c")
	{
		column = s.parse::<usize>().unwrap();
	}

	// retrieve delimiter from option
	let delimiter = match matches.opt_str("d")
	{
		Some(s) => s
		, None => "\t".to_string()
	};

	// retrieve target values from option
	let values: Vec<String> = match matches.opt_str("a")
	{
		Some(s) => s.split(",").map(String::from).collect()
		, None => vec![]
	};

	// convert target values back to &str for easier handling
	let values: Vec<&str> = values.iter().map(|s| s.as_ref()).collect();

	let mut p = ScanCsv
	{
		writer: None
		, filepath: fpath
		, compression: compression
		, istar: istar
		, files: matches.free[1..].iter().map(|s| s.as_ref()).collect()
		, column: column
		, delimiter: delimiter.as_ref()
		, values: values
		, output: matches.opt_str("o")
	};

	if let Err(e) = p.parse()
	{
		println!("Error: {}", e);
	}
}
