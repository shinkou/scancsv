use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::io;
use std::path::Path;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use regex::Regex;
use tar::Archive;
use xz2::read::XzDecoder;

pub enum Compression
{
	Plain
	, Bzip2
	, Gzip
	, Xz
}

pub struct ScanCsv<'a>
{
	pub writer: Option<File>
	, pub filepath: &'a str
	, pub compression: Compression
	, pub istar: bool
	, pub files: Vec<&'a str>
	, pub column: usize
	, pub delimiter: &'a str
	, pub regex: Option<Regex>
	, pub values: Vec<&'a str>
	, pub output: Option<String>
}

impl<'a> ScanCsv<'a>
{
	fn println_if_matches(&self, re: &Regex, s: &str)
	{
		let cols: Vec<&str> = s.split(&self.delimiter).collect();

		if re.is_match(&cols[self.column])
		{
			println!("{}", s);
		}
	}

	fn writeln_if_matches<W: Write>(&self, mut w: W, re: &Regex, s: &str)
		-> io::Result<()>
	{
		let cols: Vec<&str> = s.split(&self.delimiter).collect();

		if re.is_match(&cols[self.column])
		{
			try!(writeln!(w, "{}", s));
		}

		Ok(())
	}

	fn println_if_equals(&self, s: &str)
	{
		let cols: Vec<&str> = s.split(&self.delimiter).collect();

		if self.values.contains(&cols[self.column])
		{
			println!("{}", s);
		}
	}

	fn writeln_if_equals<W: Write>(&self, mut w: W, s: &str)
		-> io::Result<()>
	{
		let cols: Vec<&str> = s.split(&self.delimiter).collect();

		if self.values.contains(&cols[self.column])
		{
			try!(writeln!(w, "{}", s));
		}

		Ok(())
	}

	fn parse_file<R: Read>(&self, reader: BufReader<R>) -> io::Result<()>
	{
		match self.writer
		{
			Some(ref w) =>
			{
				match self.regex
				{
					Some(ref re) =>
					{
						for line in reader.lines()
						{
							let line = try!(line);
							try!(self.writeln_if_matches(w, re, &line));
						}
					}
					, None =>
					{
						for line in reader.lines()
						{
							let line = try!(line);
							try!(self.writeln_if_equals(w, &line));
						}
					}
				}
			}
			, None =>
			{
				match self.regex
				{
					Some(ref re) =>
					{
						for line in reader.lines()
						{
							let line = try!(line);
							self.println_if_matches(re, &line);
						}
					}
					, None =>
					{
						for line in reader.lines()
						{
							let line = try!(line);
							self.println_if_equals(&line);
						}
					}
				}
			}
		};

		Ok(())
	}

	fn parse_tar<R: Read>(&self, r: R) -> io::Result<()>
	{
		let mut archive = Archive::new(r);

		let mut b_path_matches: bool;
		for entry in try!(archive.entries())
		{
			let entry = try!(entry);

			// to limit the scope where "entry" can be borrowed
			{
				let path = try!(entry.header().path());
				match path.to_str()
				{
					Some(s) =>
					{
						b_path_matches = 0 == self.files.len()
							|| self.files.contains(&s);
					}
					, None => {b_path_matches = false;}
				}
			}

			if b_path_matches
			{
				try!(self.parse_file(BufReader::new(entry)));
			}
		}

		Ok(())
	}

	fn parse_naked<R: Read>(&self, r: R) -> io::Result<()>
	{
		match self.istar
		{
			true => self.parse_tar(r)
			, _ => self.parse_file(BufReader::new(r))
		}
	}

	fn fix_flags_w_filename(&mut self)
	{
		let a: Vec<String>
			= self.filepath.split(".").map(String::from).collect();

		let mut compression = Compression::Plain;
		let mut istar = false;

		if let Some(s) = a.get(a.len() - 1)
		{
			match s.as_ref()
			{
				"gz" => compression = Compression::Gzip
				, "bz2" => compression = Compression::Bzip2
				, "xz" => compression = Compression::Xz
				, "tar" => istar = true
				, _ => return
			};
		}

		match compression
		{
			Compression::Plain => return
			, _ =>
			{
				if let Some(s) = a.get(a.len() - 2)
				{
					match s.as_ref()
					{
						"tar" => istar = true
						, _ => istar = false
					};
				}
			}
		};

		self.compression = compression;
		self.istar = istar;
	}

	pub fn parse(&mut self) -> io::Result<()>
	{
		let f = try!(File::open(Path::new(self.filepath)));

		self.writer = match self.output
		{
			Some(ref s) => Some
			(
				try!
				(
					OpenOptions::new().write(true).append(true)
						.create(true).open(s)
				)
			)
			, None => None
		};

		if false == self.istar
		{
			match self.compression
			{
				Compression::Plain => self.fix_flags_w_filename()
				, _ => {}
			};
		}

		match self.compression
		{
			Compression::Bzip2 => self.parse_naked(BzDecoder::new(f))
			, Compression::Gzip => self.parse_naked(try!(GzDecoder::new(f)))
			, Compression::Xz => self.parse_naked(XzDecoder::new(f))
			, _ => self.parse_naked(f)
		}
	}
}
