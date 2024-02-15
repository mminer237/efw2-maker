use chrono::Datelike;
use clap::Parser;
use unidecode::unidecode;
use serde_yaml;

#[derive(Debug, Parser)]
struct Args {
	file: String,
	#[arg(short, long)]
	wfid: Option<String>,
	#[arg(short, long)]
	year: Option<i32>,
	#[arg(long)]
	final_year: Option<bool>,
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
	pub employment_code: String,
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
	let vendor_code = "\0\0\0\0";
	let software_code = "98";
	let company_name = format_text_field(&config.company_name, 57);
	let address_1 = format_text_field(&config.address_1, 22);
	let address_2 = format_text_field(&config.address_2.unwrap_or_default(), 22);
	let city = format_text_field(&config.city, 22);
	let state = format_text_field(&config.state, 2);
	let zip = format_text_field(&config.zip, 5);
	let zip_ext = "\0\0\0\0";
	let contact_name = format_text_field(&config.contact_name, 27);
	let phone = format_text_field(&config.phone.replace("-", ""), 15);
	let email = format_text_field(&config.email, 40);
	let fax = format_text_field(&config.fax.unwrap_or_default().replace("-", ""), 10);

	let ra_record =
		/* Record Identifier (1–2) */
		"RA".to_owned() +
		/* Submitter EIN (3–11) */
		&ein +
		/* User ID (12–19) */
		&config.user_id +
		/* Vendor Code (20–23) */
		vendor_code +
		/* Blank (24–28) */
		"\0\0\0\0\0" +
		/* Resubmission Indicator (29) */
		(if args.wfid.is_some() { "1" } else { "0" }) +
		/* Resubmission WFID (30–35) */
		&format_text_field(&args.wfid.unwrap_or_default(), 6) +
		/* Software Code (36–37) */
		software_code +
		/* Company Name (38–94) */
		&company_name +
		/* Company Location Address (95–116) */
		&address_2 +
		/* Company Delivery Address (117–138) */
		&address_1 +
		/* Company City (139–160) */
		&city +
		/* Company State (161–162) */
		&state +
		/* Company ZIP Code (163–171) */
		&zip + zip_ext +
		/* Blank (172–176) */
		"\0\0\0\0\0" +
		/* Company Foreign State/Province (177–199) */
		&"\0".repeat(23) +
		/* Company Foreign Postal Code (200–214) */
		&"\0".repeat(15) +
		/* Company Country Code (215–216) */
		"US" +
		/* Submitter Name (217–273) */
		&company_name +
		/* Submitter Location Address (274–295) */
		&address_2 +
		/* Submitter Delivery Address (296–317) */
		&address_1 +
		/* Submitter City (318–339) */
		&city +
		/* Submitter State (340–341) */
		&state +
		/* Submitter ZIP Code (342–350) */
		&zip + zip_ext +
		/* Blank (351–355) */
		"\0\0\0\0\0" +
		/* Submitter Foreign State/Province (356–378) */
		&"\0".repeat(23) +
		/* Submitter Foreign Postal Code (379–393) */
		&"\0".repeat(15) +
		/* Submitter Country Code (394–395) */
		"US" +
		/* Contact Name (396–422) */
		&contact_name +
		/* Contact Phone Number (423–437) */
		&phone +
		/* Contact Phone Extension (438–442) */
		"\0\0\0\0\0" +
		/* Blank (443–445) */
		"\0\0\0" +
		/* Contact Email Address (446–485) */
		&email +
		/* Blank (486–488) */
		"\0\0\0" +
		/* Contact Fax Number (489–498) */
		&fax +
		/* Blank (499) */
		"\0" +
		/* Preparer Code */
		"L" + // Self-prepared
		/* Blank (501–512) */
		&"\0".repeat(12);
	assert!(ra_record.len() == 512, "RA record must be 512 characters long, not {}", ra_record.len());
	
	let year = format!(
		"{:04}",
		args.year.unwrap_or(chrono::Local::now().year() - 1)
	);
	let kind_of_employer = "N";
	let employment_code = format_text_field(&config.employment_code, 1);

	let re_record =
		/* Record Identifier (1–2) */
		"RE".to_owned() +
		/* Tax Year (3–6) */
		&year +
		/* Agent Indicator Code (7) */
		"\0" +
		/* Employer EIN (8–16) */
		&ein +
		/* Agent for EIN (17–25) */
		&"\0".repeat(9) +
		/* Terminating Business Indicator (26) */
		(if args.final_year.unwrap_or(false) { "1" } else { "0" }) +
		/* Establishment Number (27–30) */
		&"\0".repeat(4) +
		/* Other EIN (31–39) */
		&"\0".repeat(9) +
		/* Employer Name (40–96) */
		&company_name +
		/* Employer Location Address (97–118) */
		&address_2 +
		/* Employer Delivery Address (119–140) */
		&address_1 +
		/* Employer City (141–162) */
		&city +
		/* Employer State (163–164) */
		&state +
		/* Employer ZIP Code (165–173) */
		&zip + zip_ext +
		/* Kind of Employer (174) */
		kind_of_employer +
		/* Blank (175–178) */
		&"\0".repeat(4) +
		/* Foreign State/Province (179–201) */
		&"\0".repeat(23) +
		/* Foreign Postal Code (202–216) */
		&"\0".repeat(15) +
		/* Country Code (217–218) */
		"US" +
		/* Employment Code (219) */
		&employment_code +
		/* Tax Jurisdiction Code (220) */
		"\0" + // Not a US territory
		/* Third-Party Sick Pay Indicator (221) */
		"0" +
		/* Employer Contact Name (222–248) */
		&contact_name +
		/* Employer Contact Phone Number (249–263) */
		&phone +
		/* Employer Contact Phone Extension (264–268) */
		"\0\0\0\0\0" +
		/* Employer Contact Fax Number (269–278) */
		&fax +
		/* Employer Contact Email Address (279–318) */
		&email +
		/* Blank (319–512) */
		&"\0".repeat(194);
	assert!(re_record.len() == 512, "RA record must be 512 characters long, not {}", re_record.len());
	
	print!("{}{}", ra_record, re_record);
}

fn format_text_field(value: &str, length: usize) -> String {
	assert!(value.len() <= length, "\"{}\" too long. Must be {} or fewer characters.", value, length);
	return format!("{:\0<width$}", unidecode(value).to_uppercase(), width = length);
}
