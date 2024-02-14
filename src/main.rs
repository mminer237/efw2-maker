use arrayvec::ArrayString;
use clap::Parser;
use unidecode::unidecode;

#[derive(Debug, Parser)]
struct Args {
	#[arg(short, long)]
	file: String,
	#[arg(short, long)]
	ein: String,
	#[arg(short, long)]
	user_id: String,
	#[arg(short, long)]
	vendor_code: String,
}

#[derive(Debug)]
pub struct Record {
	pub data: ArrayString<512>,
}

impl Record {
	pub fn new(data: &str) -> Self {
		let data = unidecode(data).to_uppercase();
		let data = ArrayString::from(&data).unwrap();
		Record { data }
	}
}

#[derive(Debug, serde::Deserialize)]
pub struct W2Info {
	pub ssn: String,
	pub first_name: String,
	pub middle_initial: String,
	pub last_name: String,
	pub suffix: String,
	pub address_1: String,
	#[serde(default)]
	pub address_2: String,
	pub city: String,
	pub state: String,
	pub zip: String,
	#[serde(default)]
	pub email: String,
	pub wages: f64,
	pub federal_tax: f64,
	pub ss_wages: f64,
	pub ss_tax: f64,
	pub medicare_wages: f64,
	pub medicare_tax: f64,
	#[serde(default)]
	pub ss_tips: f64,
	pub taxing_state: String,
	pub state_id: String,
	pub state_wages: f64,
	pub state_tax: f64,
}

fn main() {
	let mut w2_info = Vec::new();

	let args = Args::parse();
	if args.file.ends_with(".csv") {
		let mut reader = csv::Reader::from_path(args.file).unwrap();
		for result in reader.deserialize() {
			if result.is_err() {
				eprintln!("{}", result.err().unwrap());
				std::process::exit(1);
			}
			let record: W2Info = result.unwrap();
			w2_info.push(record);
		}
	}
	let ein = str::replace(&args.ein, "-", "");
	assert!(ein.len() == 9, "EIN must be 9 digits long");

	let ra_record = Record::new("RA{ein}{args.user_id}");
	
}
