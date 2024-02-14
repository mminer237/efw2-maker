use arrayvec::ArrayString;
use clap::Parser;
use unidecode::unidecode;
use serde_yaml;

#[derive(Debug, Parser)]
struct Args {
	file: String,
	#[arg(short, long)]
	wfid: Option<ArrayString<6>>,
}

#[derive(Debug, serde::Deserialize)]
struct ConfigInfo {
	pub ein: String,
	pub user_id: String,
	pub company_name: String,
	pub address_1: String,
	pub address_2: Option<String>,
	pub city: String,
	pub state: String,
	pub zip: String,
	pub contact_name: String,
	pub phone: String,
	pub email: String,
	pub fax: Option<String>,
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

	/* Read arguments */
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

	/* Read config */
	let config_path = std::env::current_exe().unwrap().parent().unwrap().join("config.yaml");
	let config_file = std::fs::File::open(&config_path).unwrap_or_else(|e| {
		eprintln!("Error reading config file {:?}: {}", &config_path, e);
		std::process::exit(1);
	});
	let config: ConfigInfo = serde_yaml::from_reader(config_file).unwrap_or_else(|e| {
		eprintln!("Error reading config.yaml: {}", e);
		std::process::exit(1);
	});
	

	let ein = str::replace(&config.ein, "-", "");
	assert!(ein.len() == 9, "EIN must be 9 digits long");
	assert!(config.user_id.len() == 8, "User ID must be 8 characters long");
	let vendor_code = "\0\0";
	let software_code = "98";
	let company_name = format_text_field(&config.company_name, 57);
	let address_1 = format_text_field(&config.address_1, 22);
	let address_2 = format_text_field(&config.address_2.unwrap_or_default(), 22);
	let city = format_text_field(&config.city, 22);
	let state = format_text_field(&config.state, 2);
	let zip = format_text_field(&config.zip, 6);
	let zip_ext = "\0\0\0\0";
	let contact_name = format_text_field(&config.contact_name, 27);
	let phone = format_text_field(&config.phone.replace("-", ""), 15);
	let email = format_text_field(&config.email, 40);
	let fax = format_text_field(&config.fax.unwrap_or_default(), 10);

	let ra_record =
		"RA".to_owned() +
		&ein +
		&config.user_id +
		vendor_code +
		"\0\0\0\0\0" +
		(if args.wfid.is_some() { "1" } else { "0" }) +
		&args.wfid.unwrap_or_default() +
		software_code +
		&company_name +
		&address_2 +
		&address_1 +
		&city +
		&state +
		&zip + zip_ext +
		"\0\0\0\0\0" +
		&"\0".repeat(23) +
		&"\0".repeat(15) +
		"US" +
		&company_name +
		&address_2 +
		&address_1 +
		&city +
		&state +
		&zip + zip_ext +
		"\0\0\0\0\0" +
		&"\0".repeat(23) +
		&"\0".repeat(15) +
		"US" +
		&contact_name +
		&phone +
		"\0\0\0\0\0" +
		"\0\0\0" +
		&email +
		"\0\0\0" +
		&fax +
		"\0" +
		"L" +
		&"\0".repeat(12);
		
	let kind_of_employer = "N";
	
}

fn format_text_field(value: &str, length: usize) -> String {
	return format!("{:\0<width$}", unidecode(value).to_uppercase(), width = length);
}
