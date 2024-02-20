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
	#[serde(default)]
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
	#[serde(default)]
	pub taxing_state: String,
	#[serde(default)]
	pub state_id: String,
	#[serde(default)]
	pub state_wages: f64,
	#[serde(default)]
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
	let vendor_code = "    ";
	let software_code = "98";
	let company_name = format_text_field(&config.company_name, 57);
	let address_1 = format_text_field(&config.address_1.replace(".", ""), 22);
	let address_2 = format_text_field(&config.address_2.unwrap_or_default().replace(".", ""), 22);
	let city = format_text_field(&config.city, 22);
	let state = format_text_field(&config.state, 2);
	let zip = format_text_field(&config.zip, 5);
	let zip_ext = "    ";
	let contact_name = format_text_field(&config.contact_name.replace(".", ""), 27);
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
		"     " +
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
		"     " +
		/* Company Foreign State/Province (177–199) */
		&" ".repeat(23) +
		/* Company Foreign Postal Code (200–214) */
		&" ".repeat(15) +
		/* Company Country Code (215–216) */
		"  " +
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
		"     " +
		/* Submitter Foreign State/Province (356–378) */
		&" ".repeat(23) +
		/* Submitter Foreign Postal Code (379–393) */
		&" ".repeat(15) +
		/* Submitter Country Code (394–395) */
		"  " +
		/* Contact Name (396–422) */
		&contact_name +
		/* Contact Phone Number (423–437) */
		&phone +
		/* Contact Phone Extension (438–442) */
		"     " +
		/* Blank (443–445) */
		"   " +
		/* Contact Email Address (446–485) */
		&email +
		/* Blank (486–488) */
		"   " +
		/* Contact Fax Number (489–498) */
		&fax +
		/* Blank (499) */
		" " +
		/* Preparer Code */
		"L" + // Self-prepared
		/* Blank (501–512) */
		&" ".repeat(12);
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
		" " +
		/* Employer EIN (8–16) */
		&ein +
		/* Agent for EIN (17–25) */
		&" ".repeat(9) +
		/* Terminating Business Indicator (26) */
		(if args.final_year.unwrap_or(false) { "1" } else { "0" }) +
		/* Establishment Number (27–30) */
		&" ".repeat(4) +
		/* Other EIN (31–39) */
		&" ".repeat(9) +
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
		&" ".repeat(4) +
		/* Foreign State/Province (179–201) */
		&" ".repeat(23) +
		/* Foreign Postal Code (202–216) */
		&" ".repeat(15) +
		/* Country Code (217–218) */
		"  " +
		/* Employment Code (219) */
		&employment_code +
		/* Tax Jurisdiction Code (220) */
		" " + // Not a US territory
		/* Third-Party Sick Pay Indicator (221) */
		"0" +
		/* Employer Contact Name (222–248) */
		&contact_name +
		/* Employer Contact Phone Number (249–263) */
		&phone +
		/* Employer Contact Phone Extension (264–268) */
		"     " +
		/* Employer Contact Fax Number (269–278) */
		&fax +
		/* Employer Contact Email Address (279–318) */
		&email +
		/* Blank (319–512) */
		&" ".repeat(194);
	assert!(re_record.len() == 512, "RA record must be 512 characters long, not {}", re_record.len());

	let mut total_wages = 0.0;
	let mut total_federal_tax = 0.0;
	let mut total_ss_wages = 0.0;
	let mut total_ss_tax = 0.0;
	let mut total_medicare_wages = 0.0;
	let mut total_medicare_tax = 0.0;
	let mut total_ss_tips = 0.0;
	let rw_records: Vec<String> = w2_info.iter().map(|line| {
		let ssn = str::replace(&line.ssn, "-", "");
		let first_name = format_text_field(&line.first_name, 15);
		let middle_initial = format_text_field(&line.middle_initial.replace(".", ""), 15);
		let last_name = format_text_field(&line.last_name, 20);
		let suffix = format_text_field(&line.suffix, 4);
		let address_1 = format_text_field(&line.address_1.replace(".", ""), 22);
		let address_2 = format_text_field(&line.address_2.replace(".", ""), 22);
		let city = format_text_field(&line.city, 22);
		let state = format_text_field(&line.state, 2);
		let zip = format_text_field(&line.zip, 5);
		let zip_ext = "    ";
		let wages = format_number_field(line.wages, 11);
		let federal_tax = format_number_field(line.federal_tax, 11);
		let ss_wages = format_number_field(line.ss_wages, 11);
		let ss_tax = format_number_field(line.ss_tax, 11);
		let medicare_wages = format_number_field(line.medicare_wages, 11);
		let medicare_tax = format_number_field(line.medicare_tax, 11);
		let ss_tips = format_number_field(line.ss_tips, 11);

		total_wages += line.wages;
		total_federal_tax += line.federal_tax;
		total_ss_wages += line.ss_wages;
		total_ss_tax += line.ss_tax;
		total_medicare_wages += line.medicare_wages;
		total_medicare_tax += line.medicare_tax;
		total_ss_tips += line.ss_tips;

		let rw_record =
			/* Record Identifier (1–2) */
			"RW".to_owned() +
			/* Employee SSN (3–11) */
			&ssn +
			/* Employee First Name (12–26) */
			&first_name +
			/* Employee Middle Initial (27–41) */
			&middle_initial +
			/* Employee Last Name (42–61) */
			&last_name +
			/* Employee Suffix (62–65) */
			&suffix +
			/* Employee Location (66–87) */
			&address_2 +
			/* Employee Delivery Address (88–109) */
			&address_1 +
			/* Employee City (110–131) */
			&city +
			/* Employee State (132–133) */
			&state +
			/* Employee ZIP Code (134–142) */
			&zip + zip_ext +
			/* Blank (143–147) */
			"     " +
			/* Employee Foreign State/Province (148–170) */
			&" ".repeat(23) +
			/* Employee Foreign Postal Code (171–185) */
			&" ".repeat(15) +
			/* Employee Country Code (186–187) */
			"  " +
			/* Employee Wages (188–198) */
			&wages +
			/* Federal Income Tax Withheld (199–209) */
			&federal_tax +
			/* Social Security Wages (210–220) */
			&ss_wages +
			/* Social Security Tax Withheld (221–231) */
			&ss_tax +
			/* Medicare Wages (232–242) */
			&medicare_wages +
			/* Medicare Tax Withheld (243–253) */
			&medicare_tax +
			/* Social Security Tips (254–264) */
			&ss_tips +
			/* Blank (265–275) */
			&" ".repeat(11) +
			/* Dependent Care Benefits (276–286) */
			&" ".repeat(11) +
			/* Deferred Compensation Contributions to 401(k) (287–297) */
			&" ".repeat(11) +
			/* Deferred Compensation Contributions to 403(b) (298–308) */
			&" ".repeat(11) +
			/* Deferred Compensation Contributions to 408(k)(6) (309–319) */
			&" ".repeat(11) +
			/* Deferred Compensation Contributions to 457(b) (320–330) */
			&" ".repeat(11) +
			/* Deferred Compensation Contributions to 501(c)(18)(D) (331–341) */
			&" ".repeat(11) +
			/* Blank (342–352) */
			&" ".repeat(11) +
			/* Nonqualified §457 Distributions or Contributions (353–363) */
			&" ".repeat(11) +
			/* Employer HSA Contributions (364–374) */
			&" ".repeat(11) +
			/* Nonqualified Non-§457 Distributions or Contributions (375–385) */
			&" ".repeat(11) +
			/* Nontaxable Combat Pay (386–396) */
			&" ".repeat(11) +
			/* Blank (397–407) */
			&" ".repeat(11) +
			/* Employer Life Insurance Premiums over $50,000 (408–418) */
			&" ".repeat(11) +
			/* Income from Exercise of Nonstatutory Stock Options (419–429) */
			&" ".repeat(11) +
			/* Deferrals under §409A Nonqualified Deferred Compensation (430–440) */
			&" ".repeat(11) +
			/* Designated 401(k) Roth Contributions (441–451) */
			&" ".repeat(11) +
			/* Designated 403(b) Roth Contributions (452–462) */
			&" ".repeat(11) +
			/* Cost of Employer-Sponsored Health Coverage (463–473) */
			&" ".repeat(11) +
			/* Permitted Benefits under a Qualified Small Employer HRA (474–484) */
			&" ".repeat(11) +
			/* Blank (485) */
			" " +
			/* Statutory Employee Indicator (486) */
			"0" +
			/* Blank (487) */
			" " +
			/* Retirement Plan Indicator (488) */
			"0" +
			/* Third-Party Sick Pay Indicator (489) */
			"0" +
			/* Blank (490–512) */
			&" ".repeat(23);

		assert!(rw_record.len() == 512, "RW record must be 512 characters long, not {}", rw_record.len());

		rw_record
	}).collect();

	let rt_record =
		/* Record Identifier (1–2) */
		"RT".to_owned() +
		/* Total Number of RW Records (3–9) */
		&format!("{:0>7}", w2_info.len()) +
		/* Total Wages (10–24) */
		&format_number_field(total_wages, 15) +
		/* Total Federal Income Tax Withheld (25–39) */
		&format_number_field(total_federal_tax, 15) +
		/* Total Social Security Wages (40–54) */
		&format_number_field(total_ss_wages, 15) +
		/* Total Social Security Tax Withheld (55–69) */
		&format_number_field(total_ss_tax, 15) +
		/* Total Medicare Wages (70–84) */
		&format_number_field(total_medicare_wages, 15) +
		/* Total Medicare Tax Withheld (85–99) */
		&format_number_field(total_medicare_tax, 15) +
		/* Total Social Security Tips (100–114) */
		&format_number_field(total_ss_tips, 15) +
		/* Blank (115–129) */
		&" ".repeat(15) +
		/* Total Dependent Care Benefits (130–144) */
		&" ".repeat(15) +
		/* Total Deferred Compensation Contributions to 401(k)s (145–159) */
		&" ".repeat(15) +
		/* Total Deferred Compensation Contributions to 403(b)s (160–174) */
		&" ".repeat(15) +
		/* Total Deferred Compensation Contributions to 408(k)(6)s (175–189) */
		&" ".repeat(15) +
		/* Total Deferred Compensation Contributions to 457(b)s (190–204) */
		&" ".repeat(15) +
		/* Total Deferred Compensation Contributions to 501(c)(18)(D)s (205–219) */
		&" ".repeat(15) +
		/* Blank (220–234) */
		&" ".repeat(15) +
		/* Total Nonqualified §457 Distributions or Contributions (235–249) */
		&" ".repeat(15) +
		/* Total Employer HSA Contributions (250–264) */
		&" ".repeat(15) +
		/* Total Nonqualified Non-§457 Distributions or Contributions (265–279) */
		&" ".repeat(15) +
		/* Total Nontaxable Combat Pay (280–294) */
		&" ".repeat(15) +
		/* Total Cost of Employer-Sponsored Health Coverage (295–309) */
		&" ".repeat(15) +
		/* Total Employer Life Insurance Premiums over $50,000 (310–324) */
		&" ".repeat(15) +
		/* Total Income Tax Withheld for Third-Party Sick Pay (325–339) */
		&" ".repeat(15) +
		/* Total Income from Exercise of Nonstatutory Stock Options (340–354) */
		&" ".repeat(15) +
		/* Total Deferrals under §409A Nonqualified Deferred Compensation (355–369) */
		&" ".repeat(15) +
		/* Total Designated 401(k) Roth Contributions (370–384) */
		&" ".repeat(15) +
		/* Total Designated 403(b) Roth Contributions (385–399) */
		&" ".repeat(15) +
		/* Total Permitted Benefits under a Qualified Small Employer HRA (400–414) */
		&" ".repeat(15) +
		/* Blank (415–512) */
		&" ".repeat(98);
	assert!(rt_record.len() == 512, "RT record must be 512 characters long, not {}", rt_record.len());

	let rf_record =
		/* Record Identifier (1–2) */
		"RF".to_owned() +
		/* Blank (3–7) */
		&" ".repeat(5) +
		/* Total Number of RW Records (8–16) */
		&format!("{:0>9}", w2_info.len()) +
		/* Blank (17–512) */
		&" ".repeat(496);
	
	print!("{}{}{}{}{}", ra_record, re_record, rw_records.join(""), rt_record, rf_record);
}

fn format_text_field(value: &str, length: usize) -> String {
	assert!(value.len() <= length, "\"{}\" too long. Must be {} or fewer characters.", value, length);
	return format!("{: <width$}", unidecode(value).to_uppercase(), width = length);
}

fn format_number_field(value: f64, length: usize) -> String {
	let value = format!("{:0>width$.0}", value * 100.0, width = length);
	assert!(value.len() <= length, "\"{}\" too long. Must be {} or fewer characters.", value, length);
	value
}
